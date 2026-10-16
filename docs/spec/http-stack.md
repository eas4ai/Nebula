# HTTP stack

Prefix: STACK

The global middleware that `try_register_http_stack` in `src/bootstrap.rs` registers, in order: `SecurityHeaders`, `LoggingMiddleware`, `SessionMiddleware`, the Inertia layer, `TimeoutMiddleware`, `CsrfMiddleware`, `IncludeMiddleware`. Every block here describes the code as read on 2026-10-16.

[STACK-001] Nebula adds the security headers, including `X-Content-Type-Options: nosniff`, to every response.
Falsifier: A response from Nebula's HTTP stack lacks `X-Content-Type-Options: nosniff`.
Rationale: `SecurityHeaders` is registered first, so it wraps every later layer (`src/bootstrap.rs:121-125`).
Status: Observed

[STACK-002] A response that passes through `SessionMiddleware` and `CsrfMiddleware` sets the session cookie and the `XSRF-TOKEN` cookie.
Falsifier: A first `GET /` with no cookies answers without a `Set-Cookie` for the session or for `XSRF-TOKEN`.
Rationale: `src/bootstrap.rs:131-134,183`. The test client in `tests/http_flows.rs:271-285` reads `XSRF-TOKEN` from responses to send it back. Live suprnova.app `/llms.txt`, a routed response, carried both cookies on 2026-10-16.
Status: Observed

[STACK-003] Nebula answers 503 to a request whose handler runs longer than 30 seconds.
Falsifier: A handler that runs for more than 30 seconds gets its own response through, or the request answers a status other than 503.
Rationale: `TimeoutMiddleware::default()` sits after the Inertia layer and before CSRF and the handlers (`src/bootstrap.rs:172-176`). It bounds the handlers, not middleware registered before it.
Status: Observed

[STACK-004] Nebula answers 404 for a path that no route matches.
Falsifier: A GET of a path that no route matches answers a status other than 404.
Rationale: `src/routes.rs` has no fallback route; `tests/http_flows.rs:971-974`.
Status: Observed
