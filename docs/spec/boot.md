# Boot configuration

Prefix: BOOT

What the server checks before it starts. The canonical link, the sitemap, the social image, the JSON-LD, `/llms.txt` and the verification and reset emails all build their URLs from `APP_URL`. Unset, it falls back to `http://localhost:8765`, which is right for local development and wrong anywhere else. The check runs first in `bootstrap::try_register_http_stack`, so it covers the server and not the console binary. Production is Suprnova's `Config::is_production()`: an unset `APP_ENV` counts as production.

The mechanism `boot` runs the tests named `boot_<nnn>_…` in `tests/http_flows.rs` and reports each test's result for `BOOT-<nnn>`.

[BOOT-001] In production, Nebula's server MUST refuse to start when `APP_URL` is unset, empty, or not an absolute `http://` or `https://` URL, with an error that names `APP_URL`; a valid `APP_URL` passes the check.
Falsifier: With `APP_ENV=production`, `try_register_http_stack` returns `Ok`, or an error that does not name `APP_URL`, when `APP_URL` is unset, empty or `localhost:8080`; or it returns an error that names `APP_URL` when `APP_URL=https://nebula.test`.
Mechanism: boot
Status: Agreed 2026-10-16

[BOOT-002] Outside production, Nebula's server MUST start without `APP_URL` and use `http://localhost:8765`.
Falsifier: With `APP_ENV=test` and `APP_URL` unset, `try_register_http_stack` returns an error, or the `/llms.txt` link to Home's twin is not `http://localhost:8765/index.html.md`.
Mechanism: boot
Status: Agreed 2026-10-16
