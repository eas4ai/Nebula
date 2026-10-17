//! Authentication middleware helpers

pub use suprnova::{AuthMiddleware, EnsureEmailVerifiedMiddleware, GuestMiddleware};

/// Create auth middleware that redirects unauthenticated users to login
pub fn auth() -> AuthMiddleware {
    AuthMiddleware::redirect_to("/login")
}

/// Create guest middleware that redirects authenticated users to dashboard
pub fn guest() -> GuestMiddleware {
    GuestMiddleware::redirect_to("/dashboard")
}

/// Create email-verification middleware that redirects unverified users to the
/// verification notice. Compose it *after* `auth()` so it runs on an already
/// authenticated request.
pub fn verified() -> EnsureEmailVerifiedMiddleware {
    EnsureEmailVerifiedMiddleware::redirect_to("/verify-email")
}
