//! Account-management flow tests for the Nebula starter kit - request path.
//!
//! These drive the kit's **real** HTTP surface: `nebula::routes::register()`
//! (the actual `routes!` table, root-prefix `group!("/")` groups, guest /
//! auth / verified middleware) plus the same global middleware stack
//! `bootstrap::register()` installs (logging → session → CSRF → include),
//! served through `suprnova::handle_request` - the framework's in-process
//! request surface. Because `hyper::body::Incoming` cannot be built
//! synthetically, requests travel over an ephemeral loopback socket whose
//! service fn is `handle_request`, exactly like the framework's own
//! integration harnesses (`framework/tests/root_group_redirect.rs`,
//! `auth_http_middleware.rs`).
//!
//! The facade-level suite (`tests/auth_flows.rs`) proves the flow *logic*;
//! this suite proves the *wiring*: route matching through root-prefix groups,
//! session-cookie continuity, CSRF token round-trips, the guest/auth/verified
//! gates, real PATCH/PUT/DELETE verbs (no method spoofing), and the
//! `redirect!("/literal")` → `302 Location: /literal` contract.
//!
//! ## History
//!
//! This surface was broken at framework rev `06b9447f`
//! (`group!("/")` registered unmatchable `//login` patterns; `redirect!`
//! resolved literal paths as route names). Both are fixed upstream as of
//! `95777465` (canonical `join_paths` for group prefixes; literal-shape
//! dispatch in `redirect!`) - this suite is the consumer-side pin on those
//! fixes.
//!
//! ## Serial execution
//!
//! `Mail::fake()` swaps the process-global mail transport and the DB /
//! auth-manager bindings live in the process-global container (the server
//! tasks must see them), so the file is serialized behind the shared
//! `common::TEST_LOCK` (also held by `tests/auth_flows.rs`).

mod common;

use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use chrono::Utc;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use serde_json::{Value, json};

use suprnova::auth::AuthConfig;
use suprnova::mail::{Mail, MailFake};
use suprnova::{
    App, Auth, AuthManager, CsrfMiddleware, EloquentUserProvider, IncludeMiddleware,
    Inertia303Middleware, MiddlewareRegistry, MustVerifyEmail, SessionConfig, SessionMiddleware,
    handle_request,
};

use nebula::middleware::LoggingMiddleware;
use nebula::models::user::User;

/// Held-for-the-test guard: keeps the SeaORM connection + auth wiring
/// registered in the global container for the duration of the test, and
/// aborts the test's accept loop on drop so the server dies with its harness.
struct Harness {
    _lock: tokio::sync::MutexGuard<'static, ()>,
    mail: MailFake,
    server: Option<tokio::task::AbortHandle>,
}

impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(server) = self.server.take() {
            server.abort();
        }
    }
}

/// Fresh in-memory DB with Nebula's full migration set (app_users, sessions,
/// remember_tokens, auth_flow_tokens), the `EloquentUserProvider::<User>`
/// registered as the active "users" provider (mirroring
/// `bootstrap::register()`), and the load-bearing `MAIL_FROM` / `APP_URL` env
/// set. Bindings go into the **global** container - the spawned server tasks
/// resolve `DB::connection()` / `AuthManager` from there.
async fn setup() -> Harness {
    let lock = common::TEST_LOCK.lock().await;

    // SAFETY: every test in this file is serialized behind `common::TEST_LOCK`.
    unsafe {
        std::env::set_var("MAIL_FROM", "test@nebula.test");
        std::env::set_var("APP_URL", "http://nebula.test");
    }

    // The session middleware fails closed (500) without an encryption key -
    // `Server::from_config` installs one at boot; this harness drives
    // `handle_request` directly, so install a process-wide test key here.
    let mail = Mail::fake();
    common::fresh_magnetar_database().await;

    App::singleton(AuthManager::new(AuthConfig::default()));
    Auth::register_provider("users", Arc::new(EloquentUserProvider::<User>::new()))
        .expect("register users provider");

    // Mirror bootstrap's `App::register_inertia_shared(Arc::new(AuthShare))`
    // so authenticated page renders carry the shared `auth.user` prop here
    // too. Re-registering across tests is fine - the provider slot is a
    // replace-on-write.
    App::register_inertia_shared(Arc::new(nebula::bootstrap::AuthShare));

    Harness {
        _lock: lock,
        mail,
        server: None,
    }
}

impl Harness {
    /// Spawn the kit's real app - `nebula::routes::register()` behind the same
    /// global middleware stack `bootstrap::register()` installs - on an
    /// ephemeral loopback listener whose service fn is
    /// `suprnova::handle_request`. The accept loop's abort handle is stored on
    /// the harness and aborted on drop, so the server's lifetime is the test's.
    async fn spawn_app(&mut self) -> SocketAddr {
        let router = Arc::new(nebula::routes::register());
        let registry = Arc::new(
            MiddlewareRegistry::new()
                .append(LoggingMiddleware)
                .append(SessionMiddleware::new(SessionConfig::from_env()))
                .append(CsrfMiddleware::new())
                .append(IncludeMiddleware)
                // Mirrors bootstrap's `Inertia::install`: upgrade Inertia
                // PUT/PATCH/DELETE 302s to 303 so browsers don't replay the
                // verb against the redirect target. (The version middleware
                // is omitted - these tests never send `X-Inertia-Version`.)
                .append(Inertia303Middleware::new()),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral listener");
        let addr = listener.local_addr().expect("local_addr");

        let accept_loop = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    return;
                };
                let io = TokioIo::new(stream);
                let router = router.clone();
                let registry = registry.clone();
                tokio::spawn(async move {
                    let svc = service_fn(move |req: hyper::Request<Incoming>| {
                        let router = router.clone();
                        let registry = registry.clone();
                        async move { Ok::<_, Infallible>(handle_request(router, registry, req).await) }
                    });
                    let _ = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, svc)
                        .await;
                });
            }
        });
        self.server = Some(accept_loop.abort_handle());

        addr
    }
}

/// One captured HTTP exchange.
struct Resp {
    status: u16,
    /// Lowercased header names, last value wins (`set-cookie` is handled
    /// separately by the client's jar).
    headers: HashMap<String, String>,
    body: String,
}

impl Resp {
    fn location(&self) -> &str {
        self.headers
            .get("location")
            .map(String::as_str)
            .unwrap_or_else(|| panic!("expected a Location header, got: {:?}", self.headers))
    }
}

/// A minimal browser: cookie jar + CSRF echo. Every request opens a fresh
/// HTTP/1.1 connection (matching how the accept loop serves), carries the
/// jar as `Cookie`, and echoes the `XSRF-TOKEN` cookie back as
/// `X-XSRF-TOKEN` on state-changing verbs - exactly what the Inertia client
/// does in production.
struct Client {
    addr: SocketAddr,
    cookies: HashMap<String, String>,
}

impl Client {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            cookies: HashMap::new(),
        }
    }

    async fn get(&mut self, path: &str) -> Resp {
        self.request("GET", path, None, false).await
    }

    async fn post_json(&mut self, path: &str, body: Value) -> Resp {
        self.request("POST", path, Some(body), false).await
    }

    /// Same as [`post_json`](Self::post_json) but flagged as an Inertia
    /// visit (`X-Inertia: true`) - what `useForm().post` sends from the
    /// browser. Validation failures answer these with a page re-render
    /// carrying an `errors` prop instead of the 422 envelope.
    async fn inertia_post_json(&mut self, path: &str, body: Value) -> Resp {
        self.request("POST", path, Some(body), true).await
    }

    async fn patch_json(&mut self, path: &str, body: Value) -> Resp {
        self.request("PATCH", path, Some(body), false).await
    }

    async fn put_json(&mut self, path: &str, body: Value) -> Resp {
        self.request("PUT", path, Some(body), false).await
    }

    /// Inertia-flagged PUT - see [`inertia_post_json`](Self::inertia_post_json).
    async fn inertia_put_json(&mut self, path: &str, body: Value) -> Resp {
        self.request("PUT", path, Some(body), true).await
    }

    async fn delete_json(&mut self, path: &str, body: Value) -> Resp {
        self.request("DELETE", path, Some(body), false).await
    }

    async fn request(
        &mut self,
        method: &str,
        path: &str,
        body: Option<Value>,
        inertia: bool,
    ) -> Resp {
        let stream = tokio::net::TcpStream::connect(self.addr)
            .await
            .expect("connect to test server");
        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake::<_, Full<Bytes>>(io)
            .await
            .expect("client handshake");
        tokio::spawn(async move {
            let _ = conn.await;
        });

        let payload = body.map(|v| Bytes::from(v.to_string())).unwrap_or_default();

        let mut builder = hyper::Request::builder()
            .method(method)
            .uri(path)
            .header("Host", "nebula.test")
            .header("Content-Length", payload.len().to_string());
        if !payload.is_empty() {
            builder = builder.header("Content-Type", "application/json");
        }
        if inertia {
            builder = builder.header("X-Inertia", "true");
        }
        if !self.cookies.is_empty() {
            let jar = self
                .cookies
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("; ");
            builder = builder.header("Cookie", jar);
        }
        // Echo the CSRF token on state-changing verbs, as the SPA client does.
        if matches!(method, "POST" | "PUT" | "PATCH" | "DELETE")
            && let Some(token) = self.cookies.get("XSRF-TOKEN")
        {
            builder = builder.header("X-XSRF-TOKEN", token.as_str());
        }

        let req = builder.body(Full::new(payload)).expect("build request");
        let resp = tokio::time::timeout(Duration::from_secs(10), sender.send_request(req))
            .await
            .expect("send_request timeout")
            .expect("hyper send_request");

        let (parts, body) = resp.into_parts();

        // Fold every Set-Cookie into the jar (a Max-Age=0 tombstone evicts).
        for value in parts.headers.get_all("set-cookie") {
            let Ok(raw) = value.to_str() else { continue };
            let mut segments = raw.split(';');
            let Some((name, val)) = segments.next().and_then(|nv| nv.split_once('=')) else {
                continue;
            };
            let expired = segments.any(|attr| {
                let attr = attr.trim().to_ascii_lowercase();
                attr == "max-age=0"
            });
            if expired || val.is_empty() {
                self.cookies.remove(name.trim());
            } else {
                self.cookies
                    .insert(name.trim().to_string(), val.to_string());
            }
        }

        let headers = parts
            .headers
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_lowercase(),
                    v.to_str().unwrap_or("").to_string(),
                )
            })
            .collect();
        let bytes = body.collect().await.expect("collect body").to_bytes();
        Resp {
            status: parts.status.as_u16(),
            headers,
            body: String::from_utf8_lossy(&bytes).to_string(),
        }
    }
}

/// Reload a user from the DB by email via the same model surface the kit uses.
async fn reload(email: &str) -> User {
    User::find_by_email(email)
        .await
        .expect("lookup")
        .unwrap_or_else(|| panic!("user {email} exists"))
}

/// Stamp a user verified (seeds the password-reset / profile fixtures in the
/// same already-verified state a click-through produces).
async fn mark_verified(user: &mut User) {
    user.email_verified_at = Some(Utc::now());
    suprnova::eloquent::Model::save(user)
        .await
        .expect("stamp email_verified_at");
}

/// Pull the plaintext token out of the first captured mail whose text body
/// carries a `token=` link.
fn token_from_fake(fake: &MailFake) -> String {
    let captured = fake.captured();
    let msg = captured
        .iter()
        .find(|m| {
            m.text
                .as_deref()
                .is_some_and(|t| t.lines().any(|l| l.contains("token=")))
        })
        .expect("a captured mail with a token link");
    let text = msg.text.as_deref().expect("token mail has a text body");
    let link = text
        .lines()
        .find(|l| l.contains("token="))
        .expect("a line with the token link");
    link.split_once("token=")
        .map(|(_, tail)| tail.trim().to_string())
        .expect("verification link should carry token=")
}

// ============================================================================
// 1. Email verification over HTTP: register → gated → verify → dashboard
// ============================================================================

#[tokio::test]
async fn register_then_verify_email_over_http() {
    let mut harness = setup().await;
    let addr = harness.spawn_app().await;
    let mut client = Client::new(addr);

    // Acquire a session + CSRF cookie through the guest-gated register page -
    // this also proves the root-prefix `group!("/")` routes match over HTTP.
    let resp = client.get("/register").await;
    assert_eq!(resp.status, 200, "GET /register must render: {}", resp.body);

    // Register. The success path is `redirect!("/dashboard")` - the literal
    // Location pins the framework's literal-redirect dispatch fix.
    let fake = &harness.mail;
    let before = fake.count();
    let resp = client
        .post_json(
            "/register",
            json!({
                "name": "Grace Hopper",
                "email": "grace@nebula.test",
                "password": "supersecret",
                "password_confirmation": "supersecret",
            }),
        )
        .await;
    assert_eq!(resp.status, 302, "register must redirect: {}", resp.body);
    assert_eq!(resp.location(), "/dashboard");

    // A verification mail was captured for the new address.
    fake.assert_sent_to("grace@nebula.test");
    assert_eq!(fake.count(), before + 1, "exactly one verification mail");
    let token = token_from_fake(fake);

    // Logged in but unverified: the `verified` gate bounces to the notice.
    let resp = client.get("/dashboard").await;
    assert_eq!(resp.status, 302, "unverified user must not see /dashboard");
    assert_eq!(resp.location(), "/verify-email");

    // The notice itself renders (auth-but-not-verified group).
    let resp = client.get("/verify-email").await;
    assert_eq!(resp.status, 200, "the verify notice must render");

    // Consume the emailed token (public route - the token is the proof).
    let resp = client
        .get(&format!("/verify-email/verify?token={token}"))
        .await;
    assert_eq!(resp.status, 302, "verify must redirect: {}", resp.body);
    assert_eq!(resp.location(), "/dashboard");

    // The stamp persisted, and the gate now opens.
    assert!(
        reload("grace@nebula.test").await.is_email_verified(),
        "verification must persist email_verified_at"
    );
    let resp = client.get("/dashboard").await;
    assert_eq!(resp.status, 200, "verified user reaches /dashboard");
}

// ============================================================================
// 2. Resend over HTTP: a logged-in unverified user gets a fresh link
// ============================================================================

#[tokio::test]
async fn resend_verification_notification_over_http() {
    let mut harness = setup().await;
    let addr = harness.spawn_app().await;
    let mut client = Client::new(addr);
    let fake = &harness.mail;

    // Register (logged in, unverified) while keeping the shared fake alive so
    // the resend assertions can compare count deltas on the same transport.
    let resp = client.get("/register").await;
    assert_eq!(resp.status, 200);
    let initial_before = fake.count();
    let resp = client
        .post_json(
            "/register",
            json!({
                "name": "Ada Lovelace",
                "email": "ada@nebula.test",
                "password": "oldpass1!",
                "password_confirmation": "oldpass1!",
            }),
        )
        .await;
    assert_eq!(resp.status, 302, "register: {}", resp.body);
    assert_eq!(
        fake.count(),
        initial_before + 1,
        "register must capture the initial verification mail"
    );

    // Resend: a fresh link is mailed to the logged-in unverified user.
    let resend_before = fake.count();
    let resp = client
        .post_json("/email/verification-notification", json!({}))
        .await;
    assert_eq!(resp.status, 302, "resend must redirect: {}", resp.body);
    assert_eq!(resp.location(), "/verify-email");
    assert_eq!(
        fake.count(),
        resend_before + 1,
        "resend must capture a fresh mail"
    );
    fake.assert_sent_to("ada@nebula.test");
    assert!(
        !token_from_fake(fake).is_empty(),
        "the resent link carries a token"
    );
}

// ============================================================================
// 3. Password reset over HTTP (incl. anti-enumeration)
// ============================================================================

#[tokio::test]
async fn password_reset_over_http_with_anti_enumeration() {
    let mut harness = setup().await;
    let addr = harness.spawn_app().await;
    let mut client = Client::new(addr);
    let fake = &harness.mail;

    // Seed a verified user with a known password.
    let mut ada = User::create("Ada Reset", "ada@x.com", "oldpass1!")
        .await
        .expect("create user");
    mark_verified(&mut ada).await;

    // Acquire session + CSRF on the guest-gated request form.
    let resp = client.get("/forgot-password").await;
    assert_eq!(resp.status, 200, "GET /forgot-password must render");

    // Known email → a reset mail is captured.
    let known_before = fake.count();
    let resp = client
        .post_json("/forgot-password", json!({ "email": "ada@x.com" }))
        .await;
    assert_eq!(resp.status, 302, "send-link must redirect: {}", resp.body);
    assert_eq!(resp.location(), "/forgot-password");
    fake.assert_sent_to("ada@x.com");
    assert_eq!(
        fake.count(),
        known_before + 1,
        "known email must capture a reset mail"
    );
    let token = token_from_fake(fake);

    // Unknown email → NO new mail, and the *same* neutral redirect
    // (anti-enumeration: the wire response is indistinguishable).
    let unknown_before = fake.count();
    let resp = client
        .post_json("/forgot-password", json!({ "email": "nobody@x.com" }))
        .await;
    assert_eq!(resp.status, 302, "unknown email gets the same 302");
    assert_eq!(resp.location(), "/forgot-password");
    assert_eq!(
        fake.count(),
        unknown_before,
        "unknown email must send nothing"
    );

    // The reset form renders with the token threaded through.
    let resp = client.get(&format!("/reset-password?token={token}")).await;
    assert_eq!(resp.status, 200, "GET /reset-password must render");

    // Complete the reset → 302 /login.
    let resp = client
        .post_json(
            "/reset-password",
            json!({
                "token": token,
                "password": "newpass1!",
                "password_confirmation": "newpass1!",
            }),
        )
        .await;
    assert_eq!(resp.status, 302, "reset must redirect: {}", resp.body);
    assert_eq!(resp.location(), "/login");

    // The success flash survives the redirect: the /login landing's page
    // object (embedded as JSON in the data-page script tag) carries it
    // under `flash.success`.
    let resp = client.get("/login").await;
    assert_eq!(resp.status, 200, "GET /login must render after reset");
    assert!(
        resp.body.contains(
            r#""flash":{"success":"Your password has been reset. Log in with your new password."}"#
        ),
        "login landing must carry the success flash in its page object: {}",
        resp.body
    );

    // Flash is one-shot: a second visit no longer carries it.
    let resp = client.get("/login").await;
    assert_eq!(resp.status, 200, "second GET /login must render");
    assert!(
        !resp.body.contains("Your password has been reset."),
        "success flash must not survive a second request: {}",
        resp.body
    );

    // The old password no longer logs in…
    let resp = client
        .post_json(
            "/login",
            json!({ "email": "ada@x.com", "password": "oldpass1!" }),
        )
        .await;
    assert_eq!(
        resp.status, 422,
        "old password must be rejected after reset"
    );
    assert!(
        resp.body
            .contains("These credentials do not match our records."),
        "rejection rides the validation envelope: {}",
        resp.body
    );

    // …and the new one does (literal redirect pin: Location is /dashboard).
    let resp = client
        .post_json(
            "/login",
            json!({ "email": "ada@x.com", "password": "newpass1!" }),
        )
        .await;
    assert_eq!(resp.status, 302, "new password must log in: {}", resp.body);
    assert_eq!(resp.location(), "/dashboard");
}

// ============================================================================
// 4. Profile over HTTP: PATCH / PUT / DELETE with real verbs
// ============================================================================

#[tokio::test]
async fn profile_update_password_and_delete_over_http() {
    let mut harness = setup().await;
    let addr = harness.spawn_app().await;
    let mut client = Client::new(addr);

    // Seed a verified user and log in through the real login flow.
    let mut user = User::create("Edsger", "edsger@x.com", "oldpass1!")
        .await
        .expect("create user");
    mark_verified(&mut user).await;

    let resp = client.get("/login").await;
    assert_eq!(resp.status, 200, "GET /login must render");
    let resp = client
        .post_json(
            "/login",
            json!({ "email": "edsger@x.com", "password": "oldpass1!" }),
        )
        .await;
    assert_eq!(resp.status, 302, "login: {}", resp.body);
    assert_eq!(resp.location(), "/dashboard");

    // The guest gate now bounces the authenticated user off /login.
    let resp = client.get("/login").await;
    assert_eq!(resp.status, 302, "guest gate must bounce a logged-in user");
    assert_eq!(resp.location(), "/dashboard");

    // The profile page renders for the authenticated user.
    let resp = client.get("/profile").await;
    assert_eq!(resp.status, 200, "GET /profile must render: {}", resp.body);

    // --- PATCH /profile: change name + email. The email change nulls the
    //     verification stamp and re-sends the link to the NEW address.
    let fake = &harness.mail;
    let before = fake.count();
    let resp = client
        .patch_json(
            "/profile",
            json!({ "name": "Edsger Dijkstra", "email": "edsger.new@x.com" }),
        )
        .await;
    assert_eq!(resp.status, 302, "profile update: {}", resp.body);
    assert_eq!(resp.location(), "/profile");

    let updated = reload("edsger.new@x.com").await;
    assert_eq!(
        updated.name.as_deref(),
        Some("Edsger Dijkstra"),
        "name saved"
    );
    assert!(
        !updated.is_email_verified(),
        "changing the email must null email_verified_at"
    );
    assert_eq!(
        fake.count(),
        before + 1,
        "email change re-sends a verification link"
    );
    fake.assert_sent_to("edsger.new@x.com");

    // The verified gate kicks back in after the email change.
    let resp = client.get("/dashboard").await;
    assert_eq!(resp.status, 302, "re-unverified user is gated again");
    assert_eq!(resp.location(), "/verify-email");

    // --- PUT /profile/password: wrong current password → 422 pinned to the
    //     field; correct current password → rotated.
    let resp = client
        .put_json(
            "/profile/password",
            json!({
                "current_password": "not-the-password",
                "password": "brandnew1!",
                "password_confirmation": "brandnew1!",
            }),
        )
        .await;
    assert_eq!(resp.status, 422, "wrong current password must 422");
    assert!(
        resp.body.contains("current_password"),
        "the 422 pins the current_password field: {}",
        resp.body
    );

    let resp = client
        .put_json(
            "/profile/password",
            json!({
                "current_password": "oldpass1!",
                "password": "brandnew1!",
                "password_confirmation": "brandnew1!",
            }),
        )
        .await;
    assert_eq!(resp.status, 302, "password rotation: {}", resp.body);
    assert_eq!(resp.location(), "/profile");
    let after = reload("edsger.new@x.com").await;
    assert!(
        after.verify_password("brandnew1!").expect("verify rotated"),
        "the rotated password verifies"
    );
    assert!(
        !after.verify_password("oldpass1!").expect("verify old"),
        "the old password no longer verifies"
    );

    // --- DELETE /profile: wrong password → 422 and the account survives;
    //     correct password → row gone + logged out.
    let resp = client
        .delete_json("/profile", json!({ "password": "wrong" }))
        .await;
    assert_eq!(resp.status, 422, "wrong delete password must 422");
    assert!(
        User::find_by_email("edsger.new@x.com")
            .await
            .expect("lookup")
            .is_some(),
        "a rejected delete must not remove the user"
    );

    let resp = client
        .delete_json("/profile", json!({ "password": "brandnew1!" }))
        .await;
    assert_eq!(resp.status, 302, "confirmed delete: {}", resp.body);
    assert_eq!(resp.location(), "/");
    assert!(
        User::find_by_email("edsger.new@x.com")
            .await
            .expect("lookup")
            .is_none(),
        "a confirmed delete removes the user row"
    );

    // The session is gone with the account: /profile now bounces to /login.
    let resp = client.get("/profile").await;
    assert_eq!(resp.status, 302, "deleted account must be logged out");
    assert_eq!(resp.location(), "/login");
}

// ============================================================================
// 5. Literal-redirect pin: `redirect!("/dashboard")` answers a literal 302
// ============================================================================

#[tokio::test]
async fn login_success_redirects_to_literal_dashboard() {
    let mut harness = setup().await;
    let addr = harness.spawn_app().await;
    let mut client = Client::new(addr);

    let mut user = User::create("Pin User", "pin@x.com", "supersecret")
        .await
        .expect("create user");
    mark_verified(&mut user).await;

    let resp = client.get("/login").await;
    assert_eq!(resp.status, 200);

    // The login controller's success arm is `redirect!("/dashboard")` - a
    // string literal with a leading `/`. At framework rev 06b9447f this
    // resolved as a route *name* and 500'd (`Route '/dashboard' not found`);
    // since 95777465 the macro dispatches literal shapes to `Redirect::to`.
    let resp = client
        .post_json(
            "/login",
            json!({ "email": "pin@x.com", "password": "supersecret" }),
        )
        .await;
    assert_eq!(
        resp.status, 302,
        "redirect!(\"/dashboard\") must produce a 302, not a named-route 500: {}",
        resp.body
    );
    assert_eq!(
        resp.location(),
        "/dashboard",
        "the Location header carries the literal path"
    );
}

// ============================================================================
// 6. Inertia form contract: validation failures re-render the page with a
//    flat `errors` prop (the 422 envelope is API-client-only), unsafe-verb
//    success redirects are upgraded to 303, and the shared `auth.user` prop
//    rides authenticated page renders.
// ============================================================================

#[tokio::test]
async fn inertia_submissions_render_errors_and_303_redirects() {
    let mut harness = setup().await;
    let addr = harness.spawn_app().await;
    let mut client = Client::new(addr);

    let mut user = User::create("Inertia User", "inertia@x.com", "supersecret")
        .await
        .expect("create user");
    mark_verified(&mut user).await;

    let resp = client.get("/login").await;
    assert_eq!(resp.status, 200);

    // Inertia submission + bad credentials → NOT the 422 envelope (which the
    // Inertia client can only display as a raw-JSON error modal) but a
    // re-render of the login page whose flat `errors` prop carries the
    // field message for `useForm().errors`.
    let resp = client
        .inertia_post_json(
            "/login",
            json!({ "email": "inertia@x.com", "password": "wrong" }),
        )
        .await;
    assert_eq!(
        resp.status, 200,
        "an Inertia validation failure re-renders the page: {}",
        resp.body
    );
    let page: Value = serde_json::from_str(&resp.body).expect("an Inertia page object");
    assert_eq!(page["component"], "auth/Login", "page: {}", resp.body);
    assert_eq!(
        page["props"]["errors"]["email"][0], "These credentials do not match our records.",
        "flat field->messages errors prop: {}",
        resp.body
    );

    // The same submission without the X-Inertia flag keeps the API envelope.
    let resp = client
        .post_json(
            "/login",
            json!({ "email": "inertia@x.com", "password": "wrong" }),
        )
        .await;
    assert_eq!(
        resp.status, 422,
        "non-Inertia clients keep the 422 envelope"
    );

    // A successful Inertia POST keeps the browser-default 302 (the 303
    // upgrade is scoped to the verbs a browser would otherwise replay).
    let resp = client
        .inertia_post_json(
            "/login",
            json!({ "email": "inertia@x.com", "password": "supersecret" }),
        )
        .await;
    assert_eq!(resp.status, 302, "Inertia POST login: {}", resp.body);
    assert_eq!(resp.location(), "/dashboard");

    // The shared `auth.user` prop (AuthShare in bootstrap.rs) rides every
    // page render once authenticated - it is what the layout's user menu
    // and Dashboard nav link key off.
    let resp = client.get("/dashboard").await;
    assert_eq!(resp.status, 200);
    assert!(
        resp.body.contains("inertia@x.com"),
        "the shared auth.user prop must ride the dashboard render: {}",
        resp.body
    );

    // An Inertia PUT that redirects is upgraded 302 → 303 so the browser
    // follows with GET instead of replaying the PUT against the target
    // (which loops until the 20-redirect cap without the middleware).
    let resp = client
        .inertia_put_json(
            "/profile/password",
            json!({
                "current_password": "supersecret",
                "password": "evenmoresecret1!",
                "password_confirmation": "evenmoresecret1!",
            }),
        )
        .await;
    assert_eq!(
        resp.status, 303,
        "Inertia PUT redirects must be 303: {}",
        resp.body
    );
    assert_eq!(resp.location(), "/profile");

    // And a failing Inertia PUT re-renders the Profile page with the field
    // error - wrong current password pins `current_password`.
    let resp = client
        .inertia_put_json(
            "/profile/password",
            json!({
                "current_password": "not-the-password",
                "password": "anothernew1!",
                "password_confirmation": "anothernew1!",
            }),
        )
        .await;
    assert_eq!(
        resp.status, 200,
        "Inertia failure re-renders: {}",
        resp.body
    );
    let page: Value = serde_json::from_str(&resp.body).expect("an Inertia page object");
    assert_eq!(page["component"], "Profile", "page: {}", resp.body);
    assert_eq!(
        page["props"]["errors"]["current_password"][0], "The current password is incorrect.",
        "errors prop pins the field: {}",
        resp.body
    );
}

// ============================================================================
// 7. Branding statics: the favicon set + webmanifest resolve at the web root
// ============================================================================

/// The framework server has no static-file handler, so the kit serves its
/// `public/` branding whitelist through explicit routes
/// (`controllers::static_files::serve`). This drives those routes through the
/// real router + global middleware stack, exactly as a browser requests them
/// - and since dev and prod are the same Rust server (Vite only supplies
/// JS/CSS via absolute dev-server URLs), one pass covers both modes.
#[tokio::test]
async fn branding_statics_resolve_at_web_root() {
    let mut harness = setup().await;
    let addr = harness.spawn_app().await;
    let mut client = Client::new(addr);

    let icon = client.get("/favicon.ico").await;
    assert_eq!(
        icon.status, 200,
        "GET /favicon.ico must serve: {}",
        icon.body
    );
    assert_eq!(
        icon.headers.get("content-type").map(String::as_str),
        Some("image/x-icon")
    );
    assert!(!icon.body.is_empty(), "favicon body must not be empty");
    let icon_len: usize = icon
        .headers
        .get("content-length")
        .expect("favicon response carries Content-Length")
        .parse()
        .expect("favicon Content-Length parses as usize");
    assert!(icon_len > 0, "favicon Content-Length must be non-zero");

    let png = client.get("/favicon-32x32.png").await;
    assert_eq!(png.status, 200, "GET /favicon-32x32.png must serve");
    assert_eq!(
        png.headers.get("content-type").map(String::as_str),
        Some("image/png")
    );
    assert_eq!(
        png.headers.get("cache-control").map(String::as_str),
        Some("public, max-age=86400"),
        "statics carry a day-long cache"
    );
    let png_len: usize = png
        .headers
        .get("content-length")
        .expect("png response carries Content-Length")
        .parse()
        .expect("png Content-Length parses as usize");
    assert!(png_len > 0, "png Content-Length must be non-zero");

    let manifest = client.get("/site.webmanifest").await;
    assert_eq!(manifest.status, 200, "GET /site.webmanifest must serve");
    assert_eq!(
        manifest.headers.get("content-type").map(String::as_str),
        Some("application/manifest+json")
    );
    assert!(
        manifest.body.contains("\"name\": \"Nebula\""),
        "manifest names the app: {}",
        manifest.body
    );

    // The whitelist is the route table: an unrouted public path stays a 404
    // (nothing falls through to a directory listing or arbitrary file read).
    let stray = client.get("/site.webmanifest.bak").await;
    assert_eq!(stray.status, 404);
}

#[tokio::test]
async fn frontend_assets_stay_within_public_route_boundary() {
    let mut harness = setup().await;
    let addr = harness.spawn_app().await;
    let mut client = Client::new(addr);

    let name = format!("nebula-assets-test-{}", std::process::id());
    let directory = suprnova::public_path(format!("assets/{name}"));
    std::fs::create_dir_all(directory.parent().unwrap()).unwrap();
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("app.js"), "console.log('Nebula');").unwrap();
    std::fs::write(directory.join("app.css"), "body { color: white; }").unwrap();
    let unlisted = suprnova::public_path(format!("{name}.txt"));
    std::fs::write(&unlisted, "unlisted public fixture").unwrap();

    let script = client.get(&format!("/assets/{name}/app.js")).await;
    let style = client.get(&format!("/assets/{name}/app.css")).await;
    let root_file = client.get(&format!("/{name}.txt")).await;
    let traversal = client.get(&format!("/assets/%2e%2e/{name}.txt")).await;
    let missing = client.get(&format!("/assets/{name}/missing.js")).await;

    // Remove fixtures before assertions, including the expected failing run.
    std::fs::remove_dir_all(&directory).unwrap();
    std::fs::remove_file(&unlisted).unwrap();

    assert_eq!(script.status, 200, "built JavaScript must be served");
    assert_eq!(script.body, "console.log('Nebula');");
    assert!(script.headers["content-type"].contains("javascript"));
    assert_eq!(style.status, 200, "built CSS must be served");
    assert_eq!(style.body, "body { color: white; }");
    assert!(style.headers["content-type"].starts_with("text/css"));
    assert_eq!(root_file.status, 404, "unlisted root files stay private");
    assert_eq!(
        traversal.status, 404,
        "asset URLs cannot escape their prefix"
    );
    assert_eq!(missing.status, 404, "missing assets stay a 404");
}
