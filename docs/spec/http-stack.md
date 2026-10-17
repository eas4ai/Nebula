# HTTP stack

Prefix: STACK

The global middleware that `try_register_http_stack` in `src/bootstrap.rs` registers. Its doc comment gives the order. Every block here describes the code as read on 2026-10-16, and cites code by name rather than by line.

[STACK-001] Nebula adds the security headers, including `X-Content-Type-Options: nosniff`, to every response that does not set them itself.
Falsifier: A response that sets no `X-Content-Type-Options` header of its own reaches the client without `X-Content-Type-Options: nosniff`.
Rationale: Suprnova always runs a typed `SecurityHeaders` registration outside the other middleware, whatever the registration order (v4.1.0 `framework/src/middleware/chain.rs:89-90`), and adds only the headers a response lacks (`framework/src/security_headers/mod.rs:304-308`). Nebula also registers it first, in `try_register_http_stack`.
Status: Observed

[STACK-002] A response that passes through `SessionMiddleware` and `CsrfMiddleware` sets the session cookie and the `XSRF-TOKEN` cookie.
Falsifier: A first `GET /` with no cookies answers without a `Set-Cookie` for the session or for `XSRF-TOKEN`.
Rationale: `SessionMiddleware` and `CsrfMiddleware` in `try_register_http_stack`. The test client in `tests/http_flows.rs` (`Client::request`) reads `XSRF-TOKEN` from responses to send it back. Live suprnova.app `/llms.txt`, a routed response, carried both cookies on 2026-10-16.
Status: Observed

[STACK-003] Nebula answers 503 to a request whose handler runs longer than 30 seconds.
Falsifier: A handler that runs for more than 30 seconds gets its own response through, or the request answers a status other than 503.
Rationale: `TimeoutMiddleware::default()` sits after the Inertia layer and before CSRF and the handlers, in `try_register_http_stack`. It bounds the handlers, not middleware registered before it.
Status: Observed

[STACK-004] Nebula answers 404 for a path that no route matches.
Falsifier: A GET of a path that no route matches answers a status other than 404.
Rationale: `src/routes.rs` has no fallback route; the stray-path check in `branding_statics_resolve_at_web_root` (`tests/http_flows.rs`).
Status: Observed
