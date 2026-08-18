//! Password-reset controller.
//!
//! The full forgot-password / reset-password flow, all on guest-only routes:
//!
//! - `GET  /forgot-password` - render the "request a reset link" form.
//! - `POST /forgot-password` - mail a reset link. Anti-enumeration: the
//!   response is identical whether or not the address exists, so a caller can't
//!   probe which emails are registered. The `PasswordReset::send_link` facade
//!   only mails when the address resolves, but always returns `Ok`.
//! - `GET  /reset-password?token=…` - render the new-password form, carrying the
//!   token through as a hidden field.
//! - `POST /reset-password` - consume the token and set the new password via
//!   `PasswordReset::complete`. An invalid or expired token re-renders the form
//!   with a `token` validation error rather than dumping a raw failure.
//!
//! Validation runs through `controllers::inertia_form`: an `X-Inertia`
//! submission that fails re-renders the originating page with a flat
//! `errors` prop, while non-Inertia clients get the Laravel-style 422
//! `{ message, errors }` envelope.
//!
//! Tokens are minted/consumed through the provider-agnostic `PasswordReset`
//! facade, which drives the address lookup + password write through the
//! `EloquentUserProvider<User>` registered in `bootstrap::register()`.

use serde::Deserialize;
use suprnova::auth_flows::PasswordReset;
use suprnova::{
    FormRequest, InertiaProps, Request, Response, Validate, ValidationErrors, handler,
    inertia_response, redirect,
};

use crate::controllers::{FormFailure, InertiaCtx, errors_json, inertia_form, validation_failure};

/// Base URL the reset token is appended to. `PasswordReset` builds
/// `{base}?token=…`, which must resolve to the `show_reset` handler below.
fn reset_base() -> String {
    format!("{}/reset-password", crate::controllers::app_url())
}

/// No per-page props: flash/status feedback rides the page object, and the
/// empty struct keeps the `inertia_response!` call shape uniform.
#[derive(InertiaProps)]
pub struct ForgotPasswordProps {}

#[derive(InertiaProps)]
pub struct ResetPasswordProps {
    /// The reset token from the emailed link, threaded into the form so the
    /// POST can present it back for consumption.
    pub token: String,
}

#[derive(Deserialize, Validate)]
pub struct SendLinkRequest {
    #[validate(email(message = "Enter a valid email address."))]
    pub email: String,
}

impl FormRequest for SendLinkRequest {}

#[derive(Deserialize, Validate)]
pub struct ResetRequest {
    pub token: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters."))]
    pub password: String,
    pub password_confirmation: String,
}

impl FormRequest for ResetRequest {
    /// Cross-field check: the password and its confirmation must match. Runs
    /// after the per-field rules pass, so the length rule is already satisfied
    /// before we compare the two values.
    fn after_validation(&self) -> Result<(), ValidationErrors> {
        if self.password != self.password_confirmation {
            let mut errs = ValidationErrors::new();
            errs.add("password", "Passwords do not match.");
            return Err(errs);
        }
        Ok(())
    }
}

/// `GET /forgot-password` - render the "request a reset link" form.
#[handler]
pub async fn show_request(req: Request) -> Response {
    inertia_response!(&req, "auth/ForgotPassword", ForgotPasswordProps {})
}

/// `POST /forgot-password` - mail a reset link.
///
/// Anti-enumeration: this always succeeds the same way regardless of whether
/// the address is registered. `send_link` mails only when the address resolves
/// but returns `Ok` either way, so the 302-back response is indistinguishable.
#[handler]
pub async fn send_link(req: Request) -> Response {
    let ctx = InertiaCtx::of(&req);
    let form = match inertia_form::<SendLinkRequest>(req).await {
        Ok(form) => form,
        Err(FormFailure::Invalid(_, errors)) => {
            return if ctx.wants_inertia() {
                inertia_response!(&ctx, "auth/ForgotPassword", {
                    "errors": errors_json(&errors),
                })
            } else {
                Err(validation_failure(errors))
            };
        }
        Err(FormFailure::Response(resp)) => return Err(*resp),
    };

    PasswordReset::send_link(&form.email, &reset_base()).await?;
    redirect!("/forgot-password").into()
}

/// `GET /reset-password?token=…` - render the new-password form.
///
/// The token is carried through to the page so the subsequent POST can present
/// it back. A missing token renders an empty field; the POST then fails the
/// token check and surfaces the standard "invalid or expired" message.
#[handler]
pub async fn show_reset(req: Request) -> Response {
    let token = req.query_param("token").unwrap_or_default();
    inertia_response!(&req, "auth/ResetPassword", ResetPasswordProps { token })
}

/// Deliver reset-password validation errors: re-render the form (with the
/// submitted token threaded back so the user can correct and resubmit) for
/// Inertia submissions, 422 envelope for everything else.
async fn render_reset(ctx: &InertiaCtx, token: &str, errors: ValidationErrors) -> Response {
    if ctx.wants_inertia() {
        inertia_response!(ctx, "auth/ResetPassword", {
            "token": token,
            "errors": errors_json(&errors),
        })
    } else {
        Err(validation_failure(errors))
    }
}

/// `POST /reset-password` - consume the token and set the new password.
///
/// On success, 302 to `/login` so the user signs in with the new credentials,
/// flashing a success message that the login landing surfaces via the page
/// object's `flash.success`. An invalid/expired/consumed token re-renders the
/// form with a `token` validation error instead of a raw failure.
#[handler]
pub async fn reset(req: Request) -> Response {
    let ctx = InertiaCtx::of(&req);
    let form = match inertia_form::<ResetRequest>(req).await {
        Ok(form) => form,
        Err(FormFailure::Invalid(form, errors)) => {
            return render_reset(&ctx, &form.token, errors).await;
        }
        Err(FormFailure::Response(resp)) => return Err(*resp),
    };

    match PasswordReset::complete(&form.token, &form.password).await {
        Ok(_user_id) => redirect!("/login")
            .with(
                "success",
                "Your password has been reset. Log in with your new password.",
            )
            .into(),
        Err(_) => {
            let mut errors = ValidationErrors::new();
            errors.add(
                "token",
                "This password reset link is invalid or has expired.",
            );
            render_reset(&ctx, &form.token, errors).await
        }
    }
}
