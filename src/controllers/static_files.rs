//! Branding statics served at the web root.
//!
//! The framework's HTTP server has no static-file handler (its only built-in
//! path is the `/_suprnova/health` short-circuit), and the Vite pipeline only
//! covers hashed build output under `/assets/`. Browsers and the web app
//! manifest request the favicon set at the web root (`/favicon.ico`,
//! `/site.webmanifest`, ...), so the kit serves that small whitelist itself
//! from `public/`. Dev (`suprnova serve`) and prod (the release binary) run
//! the same middleware - the page origin is the Rust server in both modes,
//! with Vite only supplying JS/CSS via absolute dev-server URLs.
//!
//! [`BrandingFiles`] is a global middleware placed before the session
//! middleware (see `bootstrap::try_register_http_stack`). Shared caches may
//! keep these files for a day, so they must never carry a session or
//! `XSRF-TOKEN` cookie, which every routed response does.

use suprnova::{HttpResponse, Middleware, Next, Request, Response, async_trait, public_path};

/// Content type for a whitelisted root-level public file.
///
/// Doubles as the serve whitelist: anything not named here passes through to
/// the router even if a matching file exists under `public/`, so the
/// middleware can never be pointed at an unexpected path.
fn content_type(file: &str) -> Option<&'static str> {
    match file {
        "favicon.ico" => Some("image/x-icon"),
        "favicon-16x16.png"
        | "favicon-32x32.png"
        | "android-chrome-192x192.png"
        | "android-chrome-512x512.png"
        | "apple-touch-icon.png" => Some("image/png"),
        "site.webmanifest" => Some("application/manifest+json"),
        "android-chrome-512x512.webp" => Some("image/webp"),
        _ => None,
    }
}

/// Answers `GET` and `HEAD` for each whitelisted file with its content type
/// and a day-long public cache; every other request passes through.
pub struct BrandingFiles;

#[async_trait]
impl Middleware for BrandingFiles {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let whitelisted = request
            .path()
            .strip_prefix('/')
            .and_then(|file| content_type(file).map(|mime| (file.to_string(), mime)));
        let Some((file, mime)) = whitelisted else {
            return next(request).await;
        };
        if !matches!(request.method().as_str(), "GET" | "HEAD") {
            return next(request).await;
        }

        // `Err` is how a middleware answers without calling the router.
        match tokio::fs::read(public_path(&file)).await {
            Ok(bytes) => Err(HttpResponse::bytes_body(bytes, mime)
                .header("Cache-Control", "public, max-age=86400")),
            Err(e) => {
                tracing::warn!(file, error = %e, "whitelisted public asset could not be read");
                Err(HttpResponse::text("Not Found").status(404))
            }
        }
    }
}
