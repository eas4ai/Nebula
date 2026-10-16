//! Branding statics served at the web root.
//!
//! The framework's HTTP server has no static-file handler (its only built-in
//! path is the `/_suprnova/health` short-circuit), and the Vite pipeline only
//! covers hashed build output under `/assets/`. Browsers and the web app
//! manifest request the favicon set at the web root (`/favicon.ico`,
//! `/site.webmanifest`, ...), so the kit serves that small whitelist itself
//! from `public/` through explicit routes. Dev (`suprnova serve`) and prod
//! (the release binary) hit the exact same handler - the page origin is the
//! Rust server in both modes, with Vite only supplying JS/CSS via absolute
//! dev-server URLs.

use suprnova::{HttpResponse, Request, Response, handler, public_path};

/// Content type for a whitelisted root-level public file.
///
/// Doubles as the serve whitelist: anything not named here is refused even
/// if a matching file exists under `public/`, so the handler can never be
/// pointed at an unexpected path.
fn content_type(file: &str) -> Option<&'static str> {
    match file {
        "favicon.ico" => Some("image/x-icon"),
        "favicon-16x16.png"
        | "favicon-32x32.png"
        | "android-chrome-192x192.png"
        | "android-chrome-512x512.png"
        | "apple-touch-icon.png" => Some("image/png"),
        "site.webmanifest" => Some("application/manifest+json"),
        _ => None,
    }
}

/// Serve one whitelisted file from `public/` with its content type and a
/// day-long cache. Routes register this handler per exact filename, so the
/// request path is always a bare `/name` - the whitelist check is the
/// belt-and-braces guarantee that nothing else can be read.
#[handler]
pub async fn serve(req: Request) -> Response {
    let file = req.path().trim_start_matches('/');
    let Some(mime) = content_type(file) else {
        return Ok(HttpResponse::text("Not Found").status(404));
    };

    match tokio::fs::read(public_path(file)).await {
        Ok(bytes) => {
            Ok(HttpResponse::bytes_body(bytes, mime)
                .header("Cache-Control", "public, max-age=86400"))
        }
        Err(e) => {
            tracing::warn!(file, error = %e, "whitelisted public asset could not be read");
            Ok(HttpResponse::text("Not Found").status(404))
        }
    }
}
