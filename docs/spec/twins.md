# Markdown twins

Prefix: TWIN

Crawlers that do not run JavaScript, such as GPTBot, ClaudeBot, PerplexityBot and `curl`, read Nebula's public pages through Markdown twins. `MarkdownMiddleware` from `suprnova-markdown` v0.3.0 serves the twins from the public page list in `src/markdown.rs`. A small `LlmsTxt` middleware serves `/llms.txt` from the same list. Both are global middleware, registered after `LoggingMiddleware` and before `SessionMiddleware`. Server-side rendering stays on for search engines. Design approved by the developer on 2026-10-16.

The four safety rules for every `MarkdownSource`, in Nebula and in apps built on it: (1) return Markdown only for pages anyone may read, decided on every request; (2) use `max_age(0)` for a page that can stop being public; (3) serve only `GET` and `HEAD`, and answer a failed lookup with a 500 that hides the error text, which plugin v0.3.0 does; (4) never set a cookie on a twin or on `/llms.txt`.

The mechanism `twins` runs the tests named `twin_<nnn>_…` in `tests/http_flows.rs` and in `src/markdown.rs`, and reports each test's result for `TWIN-<nnn>`.

[TWIN-001] Nebula MUST answer `GET /index.html.md` with status 200 and the headers `Content-Type: text/markdown; charset=utf-8`, `X-Robots-Tag: noindex` and `Cache-Control: public, max-age=3600`.
Falsifier: `GET /index.html.md` answers a status other than 200, or one of those three headers is missing or has another value.
Mechanism: twins
Status: Agreed 2026-10-16

[TWIN-002] Nebula MUST answer `HEAD /index.html.md` with the same status and the same `Content-Type`, `X-Robots-Tag` and `Cache-Control` values as `GET /index.html.md`.
Falsifier: `HEAD /index.html.md` differs from `GET /index.html.md` in status or in one of those three headers.
Mechanism: twins
Status: Agreed 2026-10-16

[TWIN-003] Nebula MUST answer `/index.html.md` with a query string, such as `?utm_source=test`, with the same status and body as without one.
Falsifier: `GET /index.html.md?utm_source=test` answers another status or another body than `GET /index.html.md`.
Mechanism: twins
Status: Agreed 2026-10-16

[TWIN-004] The Home twin's body MUST contain the `title` and the `message` that the Home page renders.
Falsifier: The `GET /index.html.md` body lacks the `title` or the `message` that an Inertia visit to `/` returns.
Mechanism: twins
Rationale: The Home handler and the twin both take their text from `home::content()`, so the page and its twin cannot drift apart.
Status: Agreed 2026-10-16

[TWIN-005] Nebula MUST answer 404, never a Markdown body, for a path not in the public page list, including `/index.md`, `/index.html`, `/index.html.md/`, `/login.md`, `/dashboard.md` and `/nope.md`.
Falsifier: One of those paths answers a status other than 404, or a `text/markdown` content type.
Mechanism: twins
Rationale: Safety rule 1. This guard also passes before the twins exist; it catches a later source that answers for too much.
Status: Agreed 2026-10-16

[TWIN-006] Nebula MUST serve a twin or `/llms.txt` only for `GET` and `HEAD`; a `POST` to either path gets neither the twin nor the index.
Falsifier: `POST /index.html.md` answers `text/markdown`, or `POST /llms.txt` answers 200 `text/plain` with the index body.
Mechanism: twins
Status: Agreed 2026-10-16

[TWIN-007] A response to a twin or to `/llms.txt` MUST carry no `Set-Cookie` header, whether or not the request carries a session cookie.
Falsifier: A `GET` or `HEAD` of `/index.html.md` or `/llms.txt`, with or without a session cookie, answers with a `Set-Cookie` header.
Mechanism: twins
Rationale: Safety rule 4. These responses are public for an hour, so a shared cache must never store a visitor's session or `XSRF-TOKEN` cookie with them.
Status: Agreed 2026-10-16

[TWIN-008] A response to a twin or to `/llms.txt` MUST carry `X-Content-Type-Options: nosniff`.
Falsifier: `GET /index.html.md` or `GET /llms.txt` answers without `X-Content-Type-Options: nosniff`.
Mechanism: twins
Rationale: Both middlewares run inside `SecurityHeaders` (STACK-001).
Status: Agreed 2026-10-16

[TWIN-009] Nebula MUST answer `GET /llms.txt` and `HEAD /llms.txt` with status 200, `Content-Type: text/plain; charset=utf-8`, `Cache-Control: public, max-age=3600` and no `X-Robots-Tag` header.
Falsifier: `GET` or `HEAD /llms.txt` answers another status, `Content-Type` or `Cache-Control`, or carries an `X-Robots-Tag` header.
Mechanism: twins
Status: Agreed 2026-10-16

[TWIN-010] The `/llms.txt` body MUST follow llmstxt.org: a `# ` heading holding `APP_NAME`, a `> ` summary line, a `## Pages` heading, and one Markdown link per public page to its twin's absolute URL built from `APP_URL`.
Falsifier: With `APP_NAME=Nebula` and `APP_URL=http://nebula.test`, the body lacks the line `# Nebula`, a line starting `> `, the line `## Pages`, or the line `- [Home](http://nebula.test/index.html.md)`.
Mechanism: twins
Status: Agreed 2026-10-16

[TWIN-011] An `APP_URL` that ends in `/` MUST NOT put a double slash into the `/llms.txt` links.
Falsifier: With `APP_URL=http://nebula.test/`, the `/llms.txt` body contains `//index.html.md`.
Mechanism: twins
Status: Agreed 2026-10-16

[TWIN-012] A page's first-visit HTML MUST carry a `text/markdown` alternate link to its twin in `<head>` exactly when the page has a twin: `GET /` carries `<link rel="alternate" type="text/markdown" href="/index.html.md">`, and `GET /login` carries none.
Falsifier: The `GET /` HTML lacks that link in `<head>`, or the `GET /login` HTML has a `text/markdown` alternate link.
Mechanism: twins
Status: Agreed 2026-10-16

[TWIN-013] The `head` prop of an Inertia visit MUST carry the alternate link exactly when the page has a twin: an Inertia visit to `/` carries it, and an Inertia visit to `/login` does not.
Falsifier: The `head` prop of an Inertia visit to `/` lacks the alternate link, or the `head` prop of an Inertia visit to `/login` has a `text/markdown` alternate link.
Mechanism: twins
Rationale: With `serverHead` (PAGE-002), the client applies this prop, so the link appears and disappears on client-side navigation.
Status: Agreed 2026-10-16

[TWIN-014] Adding `suprnova-markdown` MUST leave the normal build with exactly one `suprnova` package and without suprnova's `testing` feature.
Falsifier: `cargo tree --edges normal` lists two `suprnova` packages, or lists suprnova's `testing` feature when features are shown.
Mechanism: deps
Rationale: The plugin must name the framework with the same `tag = "v4.1.0"` and turn off the framework's default features.
Status: Agreed 2026-10-16
