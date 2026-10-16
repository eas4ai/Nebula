//! Email-verification controller.
//!
//! Three handlers that sit *behind* `AuthMiddleware` (an authenticated but
//! possibly-unverified user):
//!
//! - `GET  /verify-email` - render the "please verify" notice. Already-verified
//!   users are bounced straight to `/dashboard`.
//! - `POST /email/verification-notification` - resend a fresh verification link
//!   to the current user (no-op if already verified).
//! - `GET  /verify-email/verify?token=…` - consume the token via the
//!   `EmailVerification` facade and 302 to `/dashboard`; an invalid/expired
//!   token re-renders the notice with a `status` flag the page can surface.
//!
//! The user model implements `MustVerifyEmail` (see `models::user`), so the
//! provider-agnostic `EmailVerification` facade drives the address lookup +
//! timestamp write through the `EloquentUserProvider<User>` registered in
//! `bootstrap::register()`.

use suprnova::auth_flows::EmailVerification;
use suprnova::{
    Auth, FrameworkError, InertiaProps, MustVerifyEmail, Request, Response, handler,
    inertia_response, redirect,
};

use crate::models::user::User;

/// Base URL the verification token is appended to. `EmailVerification`
/// builds `{base}?token=…`, which must resolve to the `verify` handler below.
fn verify_base() -> String {
    format!("{}/verify-email/verify", crate::controllers::app_url())
}

#[derive(InertiaProps)]
pub struct VerifyEmailProps {
    /// `None` on the plain notice; `Some("invalid-or-expired")` when a bad
    /// token landed on the `verify` handler so the page can show an error.
    pub status: Option<String>,
}

/// `GET /verify-email` - render the verification notice.
///
/// An already-verified user has nothing to do here, so skip to the dashboard.
#[handler]
pub async fn show_notice(req: Request) -> Response {
    if let Some(user) = Auth::user_as::<User>().await?
        && user.is_email_verified()
    {
        return redirect!("/dashboard").into();
    }
    inertia_response!(&req, "auth/VerifyEmail", VerifyEmailProps { status: None })
}

/// `POST /email/verification-notification` - resend the verification link.
///
/// Mails a fresh token to the currently authenticated user. Already-verified
/// users are a silent no-op (still a 302 back to the notice). The route is
/// behind `AuthMiddleware`, so an unauthenticated caller never reaches here;
/// the explicit `Unauthorized` guard is belt-and-suspenders.
#[handler]
pub async fn resend(_req: Request) -> Response {
    let user = Auth::user_as::<User>()
        .await?
        .ok_or(FrameworkError::Unauthorized)?;
    if !user.is_email_verified() {
        EmailVerification::send_link(&user, &verify_base()).await?;
    }
    redirect!("/verify-email").into()
}

/// `GET /verify-email/verify?token=…` - consume a verification token.
///
/// Delegates the mutation (and the `EmailVerified` event) to
/// `EmailVerification::verify`. On success, 302 to `/dashboard`. A missing,
/// invalid, or expired token re-renders the notice with `status` set so the
/// page can explain the failure instead of dumping a raw error.
#[handler]
pub async fn verify(req: Request) -> Response {
    let token = req.query_param("token").unwrap_or_default();
    match EmailVerification::verify(&token).await {
        Ok(_user_id) => redirect!("/dashboard").into(),
        Err(_) => inertia_response!(
            &req,
            "auth/VerifyEmail",
            VerifyEmailProps {
                status: Some("invalid-or-expired".into()),
            }
        ),
    }
}
