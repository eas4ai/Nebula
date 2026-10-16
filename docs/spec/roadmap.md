# Roadmap

Current: app-url-required

## markdown-twins

Requirements: TWIN-001, TWIN-002, TWIN-003, TWIN-004, TWIN-005, TWIN-006, TWIN-007, TWIN-008, TWIN-009, TWIN-010, TWIN-011, TWIN-012, TWIN-013, TWIN-014

Delivery: on the `sudus` branch. Add `suprnova-markdown` at `tag = "v0.3.0"`; `Cargo.lock` gains only that package. Add `src/markdown.rs`, holding the public page list with Home, the `PublicPages` source, the `LlmsTxt` middleware, the `SUMMARY` constant and `alternate(path)`. Add `home::content()`, which both the Home handler and the twin use. Register both middlewares after `LoggingMiddleware` and before `SessionMiddleware`. The Home handler adds the alternate link through `Head::try_update`. The `twin_<nnn>_…` tests in `tests/http_flows.rs` are written before the start and fail until this work makes them pass. In `README.md`, add a "Markdown twins for crawlers" section: what twins and `/llms.txt` are, how to add a page to the list, the four safety rules, and the time limit a database-backed source must set itself, since twins run outside `TimeoutMiddleware`. In `UPGRADE.md`, add a 3.0.0 bullet for the new dependency, the middleware position and the new paths.

Done when: every requirement above has a current pass receipt. `cargo test --locked`, `cargo clippy`, `npm run check`, `npm run build` and `npm run build:ssr` pass. In a browser, Home's `<head>` has the alternate link once after a server-rendered visit and after a client-side visit, and the link is gone after navigating to `/login`. `README.md` and `UPGRADE.md` are updated.

## public-cache-cookies

Requirements: STATIC-001, STATIC-002, STATIC-003, STATIC-004

Delivery: on the `sudus` branch. Serve the branding files from a global middleware in `src/controllers/static_files.rs`, registered after `LlmsTxt` and before `SessionMiddleware`. It answers `GET` and `HEAD` for the listed paths with the file's content type and a day-long public cache, and passes every other request on. Remove the seven branding routes from `src/routes.rs`. Update the `try_register_http_stack` doc comment, the README paragraph on the branding files, and the UPGRADE.md 3.0.0 notes: the fix, the middleware position, and that apps on Nebula v0.2.1 to v2.0.1 send cookies with these files. The `static_<nnn>_…` tests in `tests/http_flows.rs` are written before the start and fail until this work makes them pass.

Done when: every requirement above has a current pass receipt. `cargo test --locked`, including the twin tests, and `cargo clippy` pass. Curl against the dev server shows the seven files with the day-long cache and no `Set-Cookie`. `README.md` and `UPGRADE.md` are updated.

## seo-kit

Requirements: SEO-001, SEO-002, SEO-003, SEO-004, SEO-005, SEO-006, SEO-007, SEO-008, SEO-009, SEO-010

Delivery: on the `sudus` branch. Add `src/seo.rs`: the site defaults (`Head::defaults`: canonical link on `APP_URL`, description `SUMMARY`, Open Graph site name, type and image, X card `summary`), the error-page metadata (`Head::errors`: `noindex`), the Home JSON-LD, and the `RobotsTxt` and `Sitemap` middlewares, registered after `BrandingFiles` and before `SessionMiddleware`. The defaults replace any registered earlier, so registering the stack twice adds nothing twice. Give the sign-in, verification, dashboard and profile route groups `noindex` with `with_head`. Home sets its description, `og:title` and JSON-LD from `home::content()`. Each `PublicPage` in `src/markdown.rs` gains its own address for the sitemap; Home's is `/`. `APP_NAME` is read in one helper beside `app_url()`. The `seo_<nnn>_…` tests in `tests/http_flows.rs` are written before the start and fail until this work makes them pass. README gets a "Search engines and social sites" section: what Nebula sends, where to change the description and the social image, and why `robots.txt` blocks nothing. UPGRADE.md gets a 3.0.0 bullet for the new tags, the two paths and the middleware position.

Done when: every requirement above has a current pass receipt. `cargo test --locked`, `cargo clippy`, `npm run check`, `npm run build` and `npm run build:ssr` pass. Curl against the dev server, with server rendering on, shows each head tag once on `/` and `/login`, and `/robots.txt` and `/sitemap.xml` without `Set-Cookie`. `README.md` and `UPGRADE.md` are updated.

## app-url-required

Requirements: BOOT-001, BOOT-002

Delivery: on the `sudus` branch. Add the `APP_URL` check at the top of `bootstrap::try_register_http_stack`: in production, an unset, empty or non-`http(s)` `APP_URL` returns an error naming it, before anything is registered. Update `register_http_stack`'s panic text to mention `APP_URL`. The `boot_<nnn>_…` tests in `tests/http_flows.rs` are written before the start and fail until this work makes them pass. README's production section and the UPGRADE.md 3.0.0 notes say the server now refuses to start without `APP_URL` in production.

Done when: every requirement above has a current pass receipt. `cargo test --locked` and `cargo clippy` pass. `README.md` and `UPGRADE.md` are updated.

## analytics-plausible

Requirements: ANALYTICS-001, ANALYTICS-002, ANALYTICS-003, ANALYTICS-004, ANALYTICS-005

Delivery: on the `sudus` branch, after app-url-required. Read `PLAUSIBLE_SRC` and `PLAUSIBLE_DOMAIN` once at boot (`src/analytics.rs`), refusing an invalid `PLAUSIBLE_SRC`. `templates/app.html` writes the script tag with the request's nonce when both are set. When analytics is on, `try_register_http_stack` registers `SecurityHeaders` with the preset's policy plus the Plausible origin in `connect-src`. The `analytics_<nnn>_…` tests in `tests/http_flows.rs` are written before the start. README gets an "Analytics" section: the two variables, that Plausible is cookieless, and that the policy change is Nebula's. UPGRADE.md gets a 3.0.0 bullet. File Suprnova issues, without starting a fix, for an API to add an origin to a preset policy and for app-wide data in the root template.

Done when: every requirement above has a current pass receipt. `cargo test --locked`, `cargo clippy`, `npm run check` and both frontend builds pass. With the two variables set against the dev server, the browser loads the script with no CSP error in the console. `README.md` and `UPGRADE.md` are updated.
