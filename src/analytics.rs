//! Optional, cookieless page analytics through Plausible.
//!
//! Off unless both `PLAUSIBLE_SRC` (the script URL on a Plausible instance)
//! and `PLAUSIBLE_DOMAIN` (the site name registered there) are set.
//! `templates/app.html` writes the script tag through [`plausible`], with the
//! request's CSP nonce.
//!
//! Plausible's script posts its events to its own origin, which the preset
//! Content-Security-Policy blocks (`connect-src 'self'`), so
//! [`security_headers`] adds that origin to `connect-src`. Suprnova has no
//! API to add an origin to a preset policy (eas4ai/suprnova#156), so the two
//! preset policies are copied below; `analytics_003_…` in
//! `tests/http_flows.rs` compares the result with the framework's own.
//!
//! Plausible stores nothing on the visitor's device. An app that swaps in a
//! tool that sets cookies takes on the consent question.

use std::sync::{PoisonError, RwLock};

use suprnova::{ContentSecurityPolicy, FrameworkError, SecurityHeaders};

/// Suprnova v4.1.0's strict policy (`framework/src/security_headers/policy.rs`).
const STRICT: &str = "default-src 'self'; script-src 'nonce-{nonce}' 'strict-dynamic'; style-src 'self' 'nonce-{nonce}'; img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'";

/// Suprnova v4.1.0's balanced policy, the default.
const BALANCED: &str = "default-src 'self'; script-src 'self' 'nonce-{nonce}'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; font-src 'self' data: https:; connect-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'self'";

/// The Plausible settings the root template writes into every page.
#[derive(Clone, Debug)]
pub struct Plausible {
    /// The script URL on the Plausible instance.
    pub src: String,
    /// The site name registered there.
    pub domain: String,
    /// The script's origin, which the policy must let the page post to.
    origin: String,
}

/// The settings [`configure`] read, replaced each time it runs.
static CURRENT: RwLock<Option<Plausible>> = RwLock::new(None);

/// The Plausible settings, or `None` when analytics is off.
pub fn plausible() -> Option<Plausible> {
    CURRENT
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// Read `PLAUSIBLE_SRC` and `PLAUSIBLE_DOMAIN`. Analytics is on only when
/// both are set. Surrounding spaces are trimmed, so a value of only spaces
/// counts as empty, and an empty value as unset. A set `PLAUSIBLE_SRC` must be
/// an absolute `http://` or `https://` URL on a host name or IPv4 address, and
/// neither value may be invalid Unicode, or the boot stops, as
/// `SecurityHeaders` does for its own settings.
pub fn configure() -> Result<(), FrameworkError> {
    let read = |name: &str| match std::env::var(name) {
        Ok(value) => Ok(Some(value.trim().to_string()).filter(|value| !value.is_empty())),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(FrameworkError::internal(format!(
            "{name}: expected Unicode text"
        ))),
    };
    let src = read("PLAUSIBLE_SRC")?;
    let domain = read("PLAUSIBLE_DOMAIN")?;
    let origin = match &src {
        Some(src) => Some(script_origin(src).ok_or_else(|| {
            FrameworkError::internal(
                "PLAUSIBLE_SRC: expected the script's absolute http:// or https:// URL on a host name or IPv4 address, such as https://plausible.example/js/script.js",
            )
        })?),
        None => None,
    };
    let config = match (src, domain, origin) {
        (Some(src), Some(domain), Some(origin)) => Some(Plausible {
            src,
            domain,
            origin,
        }),
        _ => None,
    };
    *CURRENT.write().unwrap_or_else(PoisonError::into_inner) = config;
    Ok(())
}

/// The origin of an absolute `http(s)` URL whose host can stand in a CSP
/// source: a host name of letters, digits and hyphens between dots, or an
/// IPv4 address. The URL parser lets `;`, `,`, `'` and braces through in a
/// host, and each of them would change the policy the origin is written
/// into. A bracketed IPv6 address is not a CSP source.
fn script_origin(src: &str) -> Option<String> {
    let url = url::Url::parse(src).ok()?;
    let host_fits = match url.host()? {
        url::Host::Domain(name) => name.split('.').all(|label| {
            !label.is_empty()
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        }),
        url::Host::Ipv4(_) => true,
        url::Host::Ipv6(_) => false,
    };
    (matches!(url.scheme(), "http" | "https") && host_fits)
        .then(|| url.origin().ascii_serialization())
}

/// The security headers `SECURITY_HEADERS` selects. With analytics on and a
/// preset that sends a policy (strict or balanced), the policy is that
/// preset's with the Plausible origin added to `connect-src`.
pub fn security_headers() -> Result<SecurityHeaders, FrameworkError> {
    let headers = SecurityHeaders::from_env()?;
    let Some(plausible) = plausible() else {
        return Ok(headers);
    };
    let template = match std::env::var("SECURITY_HEADERS").ok().as_deref() {
        None | Some("balanced") => BALANCED,
        Some("strict") => STRICT,
        // `relaxed` and `off` send no policy.
        _ => return Ok(headers),
    };
    let policy = ContentSecurityPolicy::new(with_connect_source(template, &plausible.origin))?;
    Ok(headers.content_security_policy(Some(policy)))
}

/// `template` with `origin` added to its `connect-src` directive.
fn with_connect_source(template: &str, origin: &str) -> String {
    template
        .split(';')
        .map(str::trim)
        .map(|directive| {
            if directive.split_whitespace().next() == Some("connect-src") {
                format!("{directive} {origin}")
            } else {
                directive.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_the_origin_to_connect_src_only() {
        let policy = with_connect_source(BALANCED, "https://plausible.example");
        assert_eq!(
            policy,
            BALANCED.replace(
                "connect-src 'self'",
                "connect-src 'self' https://plausible.example"
            )
        );
    }

    #[test]
    fn only_http_urls_with_a_host_have_an_origin() {
        assert_eq!(
            script_origin("https://plausible.example/js/script.js").as_deref(),
            Some("https://plausible.example")
        );
        assert_eq!(
            script_origin("http://10.0.0.5:8000/js/script.js").as_deref(),
            Some("http://10.0.0.5:8000")
        );
        for bad in [
            "javascript:alert(1)",
            "https://:8443/x",
            "plausible.example/js",
            "https://plausible.example;sandbox/js/script.js",
            "https://plausible.example,evil/js/script.js",
            "https://plausible'example/js/script.js",
            "https://plausible{nonce}/js/script.js",
            "https://[::1]/js/script.js",
            "https://plausible.example./js/script.js",
        ] {
            assert_eq!(script_origin(bad), None, "{bad}");
        }
    }
}
