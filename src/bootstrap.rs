//! Application Bootstrap
//!
//! This is where you register global middleware and services that need runtime configuration.
//! Services that don't need runtime config can use `#[service(ConcreteType)]` instead.
//!
//! # Example
//!
//! ```rust,ignore
//! // For services with no runtime config, use the macro:
//! #[service(RedisCache)]
//! pub trait CacheStore { ... }
//!
//! // For services needing runtime config, register here:
//! pub async fn register() {
//!     // Initialize database
//!     DB::init().await.expect("Failed to connect to database");
//!
//!     // Global middleware
//!     global_middleware!(middleware::LoggingMiddleware);
//!
//!     // Services
//!     bind!(dyn Database, PostgresDB::new());
//! }
//! ```

use std::sync::Arc;

use sea_orm_migration::MigratorTrait;

#[allow(unused_imports)]
use suprnova::{
    App, Auth, AuthConfig, AuthManager, CsrfMiddleware, DB, DevToolsConfig, EloquentUserProvider,
    FrameworkError, Frontend, IncludeMiddleware, Inertia, InertiaConfig, InertiaRootTemplate,
    RootShare, SecurityHeaders, SessionConfig, SessionMiddleware, bind, global_middleware,
    singleton,
};

use crate::middleware;
use crate::models::user::User;

/// The document every Inertia first visit renders into: `templates/app.html`.
///
/// The framework hands the template its parts - `title`, `head` (the CSRF
/// tag and the Vite tags), `body` (the page data and the mount element),
/// `lang`, `csrf_token`, `nonce`, `ssr` and the response's `view` data - and
/// Askama checks at compile time that it names nothing else. Put the
/// application's own head content there: a favicon, fonts, meta tags.
#[suprnova::inertia_root(path = "app.html")]
pub struct AppDocument;

/// Register process-wide services.
///
/// Called from `cmd/main.rs` via `.bootstrap(bootstrap::register)` and from
/// the console binary. Every subcommand runs it, not only `serve`, so it
/// holds only what every process needs: encryption, the database and its
/// schema, Magnetar, the rate limiter, and the auth provider. The HTTP stack
/// (global middleware and `Inertia::install`) lives in
/// [`register_http_stack`], wired separately via `.http_bootstrap(...)` in
/// `cmd/main.rs`, so a console or worker process that ships no built
/// frontend assets still boots.
pub async fn register() {
    // Console and server bootstrap both need Crypt before Magnetar is installed.
    suprnova::boot::initialize_crypt_or_exit();

    // Initialize database connection
    DB::init().await.expect("Failed to connect to database");
    let db = DB::connection().expect("DB not initialized");
    // Apply the kit schema before Magnetar can create its shared user table.
    crate::migrations::Migrator::up(db.inner(), None)
        .await
        .expect("Failed to migrate application database");
    let magnetar = suprnova::MagnetarConfig::from_sea_orm(db.inner().clone()).passkey_config(
        suprnova::PasskeyConfig {
            rp_id: std::env::var("PASSKEY_RP_ID").unwrap_or_else(|_| "localhost".to_string()),
            rp_origin: std::env::var("PASSKEY_RP_ORIGIN")
                .unwrap_or_else(|_| "http://localhost".to_string()),
        },
    );
    suprnova::init_magnetar(magnetar)
        .await
        .expect("Failed to initialize Magnetar");
    suprnova::rate_limit::bootstrap_default().await;

    // Authentication: register the AuthManager (the config/auth.php analogue)
    // and a user provider so `Auth::attempt` and `Auth::user_as::<User>()`
    // resolve users. `EloquentUserProvider<User>` queries the typed model; the
    // session middleware persists the authenticated id across requests.
    App::singleton(AuthManager::new(AuthConfig::from_env()));
    Auth::register_provider("users", Arc::new(EloquentUserProvider::<User>::new()))
        .expect("register users provider");

    // Example: Register a trait binding with runtime config
    // bind!(dyn Database, PostgresDB::new());

    // Example: Register a concrete singleton
    // singleton!(CacheService::new());

    // Add your service registrations here
}

/// Register the global middleware chain and the Inertia layer.
///
/// Called on the server path, via
/// `.http_bootstrap(|| async { bootstrap::register_http_stack() })` in
/// `cmd/main.rs`, after [`register`], and by the `ssr:*` commands, which read
/// the Inertia configuration installed below. Never on the console binary:
/// `Inertia::install` fails closed in production when the built frontend
/// manifest is missing.
pub fn register_http_stack() {
    try_register_http_stack()
        .expect("HTTP stack: check the SECURITY_HEADERS settings; production also needs a built frontend manifest");
}

/// The HTTP stack, returning its errors. `tests/http_flows.rs` calls this to
/// serve the same stack production runs.
///
/// Order matters: security headers outermost, so every response carries
/// them; the Markdown twins, `/llms.txt` and the branding files before the
/// session, so those publicly cacheable responses never carry a cookie; the session before
/// Inertia (the version middleware re-flashes the session before it bounces
/// a stale client); the request timeout between the Inertia layer and CSRF;
/// CSRF after the session it reads.
pub fn try_register_http_stack() -> Result<(), FrameworkError> {
    // Response security headers and the per-request CSP nonce, outermost so
    // every response is covered. `SECURITY_HEADERS` picks the preset
    // (`balanced` when unset).
    global_middleware!(SecurityHeaders::from_env()?);

    // Global middleware (runs on every request in registration order)
    global_middleware!(middleware::LoggingMiddleware);

    // Markdown twins of the public pages (`/index.html.md`), for crawlers
    // that do not run JavaScript; the list is `src/markdown.rs`. Then
    // `/llms.txt`, the index of those twins, from the same list. Both before
    // the session: their responses are publicly cacheable, so they must never
    // start a session or carry a cookie.
    global_middleware!(suprnova_markdown::MarkdownMiddleware::new(
        crate::markdown::PublicPages
    ));
    global_middleware!(crate::markdown::LlmsTxt);

    // The favicon set and the web app manifest at the web root, publicly
    // cached for a day, so before the session for the same reason.
    global_middleware!(crate::controllers::static_files::BrandingFiles);

    // Session middleware (required for authentication). The config is
    // cloned because the CSRF middleware below reads the same values, so
    // the session cookie and the XSRF cookie cannot drift apart.
    let session_config = SessionConfig::from_env();
    global_middleware!(SessionMiddleware::new(session_config.clone()));

    // Inertia protocol layer: the headers middleware (`Vary: X-Inertia`, and
    // an empty 200 on an Inertia visit turned into a 303 back), the version
    // middleware (409 + `X-Inertia-Location` when the client's asset version
    // no longer matches), the 303 middleware (302 -> 303 on non-GET
    // redirects, so a browser doesn't replay a PATCH/PUT/DELETE), the
    // validation-redirect middleware, and, because `.error_page` is set, the
    // error-page middleware. It sits after the session middleware because
    // the version middleware re-flashes the session before it bounces a
    // stale client.
    //
    // No asset-version override: the default hashes the Vite manifest, so
    // each frontend build changes the version and an open tab reloads onto
    // the new assets. The frontend is pinned so the HTML shell does not
    // depend on `SUPRNOVA_FRONTEND` at runtime. `Inertia::install` retains
    // this config as the default every `InertiaResponse` starts from.
    //
    // `.root_template` renders every first visit through `AppDocument`
    // (`templates/app.html`). `.error_page` renders framework error
    // responses (403, 404, 419, 500, 503) through `pages/Error.svelte`
    // instead of the Inertia client's plain-JSON modal.
    Inertia::install(
        &InertiaConfig::new()
            .frontend(Frontend::Svelte)
            .default_title("Nebula")
            .root_template(InertiaRootTemplate::of::<AppDocument>())
            // Every message per field, as an array, the shape the kit's own
            // `errors_json` sends and the pages read (`form.errors.x?.[0]`).
            // `suprnova generate-types` reads this call and types
            // `errorValueType` as `string[]`.
            .with_all_errors(true)
            // Record in local development for the Inertia DevTools browser
            // extension. INERTIA_DEVTOOLS_ENABLED=false turns it off.
            .devtools(DevToolsConfig::new())
            .error_page("Error"),
    )?;

    // Request timeout: a 30-second ceiling on the time to a response; past
    // it the request answers 503. After `Inertia::install`, so the 503
    // renders as the error page inside the visitor's session, and before
    // CSRF and the handlers, so it bounds their work.
    global_middleware!(suprnova::TimeoutMiddleware::default());

    // CSRF protection (validates tokens on POST/PUT/PATCH/DELETE).
    // `with_session_config` copies the session cookie's `Secure`,
    // `SameSite`, `Domain`, `Path` and lifetime onto the JS-readable
    // `XSRF-TOKEN` cookie, so it works over `http://localhost` when
    // `SESSION_SECURE=false`.
    global_middleware!(CsrfMiddleware::new().with_session_config(&session_config));

    // Parse `?include=`/`?exclude=`/`?only=`/`?except=` and `?fields[...]=`
    // into the per-request task-local so `#[derive(Data)]` responses,
    // `Resource::single`, and lazy `Prop` resolution honour the client's
    // requested shape out of the box. Without this, Data DTOs silently
    // ignore include/fieldset query parameters.
    global_middleware!(IncludeMiddleware);

    // The `root` prop on every Inertia response: the public root pages build
    // their URLs from, so the same build runs at `/` and under a path prefix.
    App::register_inertia_shared(Arc::new(RootShare::new()));

    // The signed-in user, or none, on every page under `auth`, read from the
    // request's session each time a response sends it. `SharedData` in
    // `src/props/shared.rs` types it for `suprnova generate-types`.
    App::inertia_share_lazy("auth", crate::props::shared::Auth::current);
    Ok(())
}
