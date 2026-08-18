pub mod auth;
pub mod dashboard;
pub mod home;
pub mod password_reset;
pub mod profile;
pub mod static_files;
pub mod verify_email;

use suprnova::{
    FormRequest, FrameworkError, HttpResponse, InertiaRequestExt, Request, ValidationErrors,
    serde_json,
};

/// The app's external base URL (for building emailed links). `APP_URL` or the dev default.
pub(crate) fn app_url() -> String {
    std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8765".into())
}

// ============================================================================
// Inertia-aware form validation
//
// `FormRequest` extraction surfaces validation failures as the Laravel-style
// 422 `{ message, errors }` envelope. That is right for API clients, but the
// Inertia client cannot render a plain-JSON error response - it shows the
// raw payload in its error modal instead of putting messages on the fields.
// The browser-facing controllers therefore validate through `inertia_form`
// and, on failure of an `X-Inertia` submission, re-render the originating
// page with a flat `errors` prop ({ field: [messages] }) that `useForm`
// merges into `form.errors`. Non-Inertia submissions keep the 422 envelope.
// ============================================================================

/// Owned snapshot of the request bits an Inertia re-render consults (path +
/// headers), captured before body extraction consumes the [`Request`]. Passed
/// to `inertia_response!` in validation-failure paths.
pub(crate) struct InertiaCtx {
    path: String,
    headers: Vec<(String, String)>,
}

impl InertiaCtx {
    pub(crate) fn of(req: &Request) -> Self {
        Self {
            path: req.path().to_string(),
            headers: req
                .headers()
                .iter()
                .filter_map(|(k, v)| {
                    v.to_str()
                        .ok()
                        .map(|val| (k.as_str().to_string(), val.to_string()))
                })
                .collect(),
        }
    }

    /// Whether the snapshotted request came from the Inertia client.
    pub(crate) fn wants_inertia(&self) -> bool {
        InertiaRequestExt::is_inertia(self)
    }
}

impl InertiaRequestExt for InertiaCtx {
    fn path(&self) -> &str {
        &self.path
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

/// Failure outcome of [`inertia_form`].
pub(crate) enum FormFailure<T> {
    /// The body parsed but a validation stage rejected it. Carries the
    /// parsed form so failure re-renders can reuse submitted fields (e.g.
    /// the password-reset token), plus the per-field error bag.
    Invalid(Box<T>, ValidationErrors),
    /// Authorization / parse / transport failure, already converted to the
    /// framework's standard error response - return it as `Err(*resp)`.
    Response(Box<HttpResponse>),
}

/// Parse + validate a form body through the same staged pipeline as
/// `FormRequest::extract` (authorize → parse → per-field rules → sync
/// cross-field hook → async cross-field hook), but hand validation
/// failures back to the caller together with the parsed form so Inertia
/// submissions can re-render the originating page instead of receiving
/// the API-style 422 envelope.
pub(crate) async fn inertia_form<T: FormRequest>(req: Request) -> Result<T, FormFailure<T>> {
    if !T::authorize(&req) {
        return Err(FormFailure::Response(Box::new(
            FrameworkError::Unauthorized.into(),
        )));
    }

    // `input()` dispatches on Content-Type (JSON or form-urlencoded),
    // matching the body shapes the Inertia client sends.
    let form: T = match req.input().await {
        Ok(form) => form,
        Err(err) => return Err(FormFailure::Response(Box::new(err.into()))),
    };

    let errors = match form.validate() {
        Err(errors) => ValidationErrors::from_validator(errors),
        Ok(()) => match form.after_validation() {
            Err(errors) => errors,
            Ok(()) => match form.after_validation_async().await {
                Err(errors) => errors,
                Ok(()) => return Ok(form),
            },
        },
    };
    Err(FormFailure::Invalid(Box::new(form), errors))
}

/// Flatten a [`ValidationErrors`] bag into the `{ field: [messages] }` JSON
/// the Inertia client merges into `useForm().errors`. Messages render
/// through `messages_for`, so keyed messages become locale-aware text -
/// the same rendering the framework's own 422 envelope applies.
pub(crate) fn errors_json(errors: &ValidationErrors) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = errors
        .errors
        .keys()
        .map(|field| (field.clone(), serde_json::json!(errors.messages_for(field))))
        .collect();
    serde_json::Value::Object(map)
}

/// The non-Inertia delivery for a validation failure: the standard 422
/// `{ message, errors }` envelope.
pub(crate) fn validation_failure(errors: ValidationErrors) -> HttpResponse {
    FrameworkError::Validation(errors).into()
}
