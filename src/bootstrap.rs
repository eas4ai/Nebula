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

#[allow(unused_imports)]
use suprnova::{
    App, Auth, AuthConfig, AuthManager, CsrfMiddleware, DB, EloquentUserProvider, FrameworkError,
    Frontend, IncludeMiddleware, Inertia, InertiaConfig, InertiaRequestExt, InertiaSharedData,
    Prop, SessionConfig, SessionMiddleware, bind, global_middleware, indexmap::IndexMap,
    serde_json, singleton,
};

use crate::middleware;
use crate::models::user::User;

/// Shares the authenticated user on every Inertia response as `auth.user` -
/// the shape `frontend/src/types/auth.ts` declares and `Layout.svelte`
/// renders the user menu / Dashboard nav link from. Guests share
/// `auth.user: null` so pages can branch on it without optional-chaining
/// surprises.
///
/// Public so the integration-test harness (`tests/http_flows.rs`) can mirror
/// this registration against its own server stack.
pub struct AuthShare;

#[suprnova::__async_trait]
impl InertiaSharedData for AuthShare {
    async fn share(
        &self,
        _req: &dyn InertiaRequestExt,
        _component: &str,
    ) -> Result<IndexMap<String, Prop>, FrameworkError> {
        let user = Auth::user_as::<User>().await?;
        let mut out = IndexMap::new();
        out.insert(
            "auth".to_string(),
            Prop::eager(serde_json::json!({
                "user": user.map(|u| serde_json::json!({
                    "id": u.id,
                    "name": u.name.unwrap_or_else(|| "Account".to_string()),
                    "email": u.email,
                })),
            })),
        );
        Ok(out)
    }
}

/// Register global middleware and services
///
/// Called from cmd/main.rs before `Server::from_config()`.
/// Middleware and services registered here can use environment variables, config files, etc.
/// Inertia asset version: the version middleware bounces clients whose
/// cached bundle no longer matches. Bump when you ship new frontend assets
/// (or replace with a build hash) - it reaches both the middleware and every
/// page object, because `Inertia::install` retains its config.
pub const INERTIA_VERSION: &str = "1.0";

pub async fn register() {
    // Initialize database connection
    DB::init().await.expect("Failed to connect to database");
    let db = DB::connection().expect("DB not initialized");
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

    // Global middleware (runs on every request in registration order)
    global_middleware!(middleware::LoggingMiddleware);

    // Session middleware (required for authentication)
    let session_config = SessionConfig::from_env();
    global_middleware!(SessionMiddleware::new(session_config));

    // CSRF protection (validates tokens on POST/PUT/PATCH/DELETE)
    global_middleware!(CsrfMiddleware::new());

    // Parse `?include=`/`?exclude=`/`?only=`/`?except=` and `?fields[...]=`
    // into the per-request task-local so `#[derive(Data)]` responses,
    // `Resource::single`, and `Prop::Lazy` resolution honour the client's
    // requested shape out of the box. Without this, Data DTOs silently
    // ignore include/fieldset query parameters.
    global_middleware!(IncludeMiddleware);

    // Authentication: register the AuthManager (the config/auth.php analogue)
    // and a user provider so `Auth::attempt` and `Auth::user_as::<User>()`
    // resolve users. `EloquentUserProvider<User>` queries the typed model; the
    // SessionMiddleware above persists the authenticated id across requests.
    App::singleton(AuthManager::new(AuthConfig::from_env()));
    Auth::register_provider("users", Arc::new(EloquentUserProvider::<User>::new()))
        .expect("register users provider");

    // Inertia protocol layer, three middlewares in one call: the headers
    // middleware (`Vary: X-Inertia` on every response, and an empty 200 on
    // an Inertia visit substituted with a 303 back), the version middleware
    // (409 + `X-Inertia-Location` when the client's asset version doesn't
    // match INERTIA_VERSION, reflashing the session so errors survive the
    // reload), and the 303 middleware (302 -> 303 on non-GET redirects, so
    // a browser doesn't replay a PATCH/PUT/DELETE against the redirect
    // target until its 20-redirect cap kills the visit).
    //
    // The config passed here is the default every `InertiaResponse` starts
    // from (framework v1.2.4), so INERTIA_VERSION is the one named place to
    // bump when you ship new assets, and the pinned frontend reaches the
    // rendered HTML shell without `SUPRNOVA_FRONTEND` being set at runtime.
    Inertia::install(&InertiaConfig::new().version(INERTIA_VERSION).frontend(Frontend::Svelte))
        .expect("Inertia install failed - in production this fails closed when the frontend manifest has not been built");

    // Share the authenticated user (`auth.user`) on every Inertia
    // response so the layout can render the user menu without each
    // handler threading the user through its own props.
    App::register_inertia_shared(Arc::new(AuthShare));

    // Example: Register a trait binding with runtime config
    // bind!(dyn Database, PostgresDB::new());

    // Example: Register a concrete singleton
    // singleton!(CacheService::new());

    // Add your middleware and service registrations here
}
