# Glossary

Terms as the code names them. A term in `code` is an identifier in the source.

- **Public page**: a page anyone may read without signing in. Nebula has one, Home at `/`.
- **Markdown twin** (twin): a Markdown copy of a public page, served at the page's path plus `.md`. The site root's twin is `/index.html.md`.
- **Public page list** (`PublicPages`): the list in `src/markdown.rs` of the public pages that have twins. The twin middleware asks it for Markdown, and `/llms.txt` is built from it.
- **`MarkdownSource`**: the trait from `suprnova-markdown` that returns a page's Markdown, or none for a path with no twin.
- **`/llms.txt`**: a plain-text index of the twins in the llmstxt.org format, for crawlers.
- **Alternate link**: `<link rel="alternate" type="text/markdown" href="…">` in a page's `<head>`, naming the page's twin.
- **Global middleware**: middleware that runs for every request, in registration order (`global_middleware!` in `src/bootstrap.rs`). A route cannot leave it out.
- **Security headers** (`SecurityHeaders`): response headers such as `X-Content-Type-Options: nosniff`, added to every response.
- **Session cookie**: the cookie `SessionMiddleware` sets to carry a visitor's session.
- **`XSRF-TOKEN`**: the cookie `CsrfMiddleware` sets so the client can send the CSRF token back with unsafe requests.
- **First visit**: a request without the `X-Inertia` header. It answers a full HTML document through `templates/app.html`.
- **Inertia visit**: a request with `X-Inertia: true`, which the client sends when it navigates. It answers the page as JSON.
- **Head** (`Head`, the `head` prop): the resolved document head. A first visit writes it into `<head>`. When the request sets a head tag, the Inertia response carries the page's tags as the `head` prop, and `serverHead` makes the client apply it.
- **`APP_URL`**: the app's external base URL, read by `app_url()`.
- **`APP_NAME`**: the app's name. `/llms.txt` uses it as its heading.
- **Canonical link**: `<link rel="canonical">`, the one address search engines should index a page under. Nebula builds it from `APP_URL` and the page's path.
- **Robots meta tag**: `<meta name="robots">` in a page's head. `noindex` or `none` keeps the page out of search results.
- **Sitemap**: `/sitemap.xml`, the sitemaps.org list of the public pages' addresses.
- **CSP nonce**: the per-request random value in the `Content-Security-Policy` (`'nonce-…'`). A `<script>` that carries it may run; `SecurityHeaders` makes a new one for every request.
