//! Authentication controller.
//!
//! Renders the login/register Inertia pages on GET, validates and
//! persists credentials on POST, redirects to `/dashboard` on success.
//! Form bodies are validated through `controllers::inertia_form`: an
//! `X-Inertia` submission that fails validation re-renders the
//! originating page with a flat `errors` prop (which `useForm` merges
//! into `form.errors`), while non-Inertia clients get the Laravel-style
//! 422 `{ message, errors }` envelope.

use std::sync::Arc;

use serde::Deserialize;
use suprnova::{
    Auth, Credentials, FormRequest, InertiaProps, Request, Response, Validate, ValidationErrors,
    handler, inertia_response, redirect,
};

use crate::controllers::{FormFailure, InertiaCtx, errors_json, inertia_form, validation_failure};
use crate::models::user::User;

// ============================================================================
// Login
// ============================================================================

/// No per-page props: validation errors ride Inertia's own `errors` prop
/// (re-rendered by `render_login` on failure), and the empty struct keeps
/// the `inertia_response!` call shape uniform.
#[derive(InertiaProps)]
pub struct LoginProps {}

#[handler]
pub async fn show_login(req: Request) -> Response {
    inertia_response!(&req, "auth/Login", LoginProps {})
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    #[serde(default)]
    pub remember: bool,
}

impl FormRequest for LoginRequest {}

/// Deliver login validation errors: re-render the page for Inertia
/// submissions, 422 envelope for everything else.
async fn render_login(ctx: &InertiaCtx, errors: ValidationErrors) -> Response {
    if ctx.wants_inertia() {
        inertia_response!(ctx, "auth/Login", { "errors": errors_json(&errors) })
    } else {
        Err(validation_failure(errors))
    }
}

#[handler]
pub async fn login(req: Request) -> Response {
    let ctx = InertiaCtx::of(&req);
    let form = match inertia_form::<LoginRequest>(req).await {
        Ok(form) => form,
        Err(FormFailure::Invalid(_, errors)) => return render_login(&ctx, errors).await,
        Err(FormFailure::Response(resp)) => return Err(*resp),
    };

    // `Auth::attempt` verifies the password through the registered user
    // provider, logs the user into the session on success, and issues a
    // remember-me token when requested - all via the named-guard system
    // wired in bootstrap.rs.
    match Auth::attempt(
        &Credentials::password(&form.email, &form.password),
        form.remember,
    )
    .await?
    {
        Some(_user) => redirect!("/dashboard").into(),
        None => {
            let mut errors = ValidationErrors::new();
            errors.add("email", "These credentials do not match our records.");
            render_login(&ctx, errors).await
        }
    }
}

// ============================================================================
// Registration
// ============================================================================

/// No per-page props - see [`LoginProps`].
#[derive(InertiaProps)]
pub struct RegisterProps {}

#[handler]
pub async fn show_register(req: Request) -> Response {
    inertia_response!(&req, "auth/Register", RegisterProps {})
}

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, message = "Name must be at least 2 characters"))]
    pub name: String,
    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub password_confirmation: String,
}

impl FormRequest for RegisterRequest {
    /// Cross-field check: confirm the password and its confirmation
    /// match. Runs after the per-field rules pass, so we know each
    /// individual value is well-formed before comparing them.
    fn after_validation(&self) -> Result<(), ValidationErrors> {
        if self.password != self.password_confirmation {
            let mut errs = ValidationErrors::new();
            errs.add("password_confirmation", "Passwords do not match.");
            return Err(errs);
        }
        Ok(())
    }
}

/// Deliver registration validation errors - same split as [`render_login`].
async fn render_register(ctx: &InertiaCtx, errors: ValidationErrors) -> Response {
    if ctx.wants_inertia() {
        inertia_response!(ctx, "auth/Register", { "errors": errors_json(&errors) })
    } else {
        Err(validation_failure(errors))
    }
}

#[handler]
pub async fn register(req: Request) -> Response {
    let ctx = InertiaCtx::of(&req);
    let form = match inertia_form::<RegisterRequest>(req).await {
        Ok(form) => form,
        Err(FormFailure::Invalid(_, errors)) => return render_register(&ctx, errors).await,
        Err(FormFailure::Response(resp)) => return Err(*resp),
    };

    if User::find_by_email(&form.email).await?.is_some() {
        let mut errors = ValidationErrors::new();
        errors.add("email", "This email is already registered.");
        return render_register(&ctx, errors).await;
    }

    let user = User::create(&form.name, &form.email, &form.password).await?;
    let user = Arc::new(user);
    // Log the freshly-created user into the session (fires the Login event).
    Auth::login(user.clone(), false).await?;

    // Send the verification link to the new account. The user implements
    // `MustVerifyEmail`, so the provider-agnostic facade mints a token and
    // mails `{APP_URL}/verify-email/verify?token=…`. Registration leaves the
    // user logged-in-but-unverified; the `verified` gate on `/dashboard`
    // routes them to `/verify-email` until they click the link.
    //
    // This send is best-effort: the account is already created and the session
    // logged in, so a mail failure (e.g. a misconfigured `MAIL_FROM`) must not
    // 500 registration. We log and continue - the user lands on the verified
    // gate at `/verify-email` and can resend from there.
    let base = format!("{}/verify-email/verify", crate::controllers::app_url());
    if let Err(err) = suprnova::auth_flows::EmailVerification::send_link(user.as_ref(), &base).await
    {
        tracing::warn!(error = %err, "failed to send verification email on registration");
    }

    redirect!("/dashboard").into()
}

// ============================================================================
// Logout
// ============================================================================

#[handler]
pub async fn logout(_req: Request) -> Response {
    Auth::logout().await?;
    redirect!("/").into()
}
