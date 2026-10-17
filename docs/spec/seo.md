# Search and social metadata

Prefix: SEO

What search engines and social sites read from Nebula besides the Markdown twins: the document head's canonical link, description, robots rule, Open Graph and X card tags, JSON-LD on Home, `/robots.txt` and `/sitemap.xml`. The head tags come from Suprnova's `Head` API (`src/seo.rs`): site defaults, error-page metadata, route-group metadata and Home's own tags. Titles stay in the Svelte pages. `/robots.txt` and `/sitemap.xml` are answered before the session middleware, like `/llms.txt`, so they carry no cookie. Ported on 2026-10-16 from the kit-level SEO features of suprnova.app; analytics went to its own item.

The mechanism `seo` runs the tests named `seo_<nnn>_…` in `tests/http_flows.rs` and reports each test's result for `SEO-<nnn>`.

[SEO-001] Every page Nebula renders through Inertia MUST carry a canonical link built from `APP_URL` and the page's path, without the query string, whatever `Host` the request names.
Falsifier: With `APP_URL=http://nebula.test`, the first-visit HTML of `GET /login?utm_source=x` sent with `Host: evil.example` lacks `<link rel="canonical" href="http://nebula.test/login">`, or that of `GET /` lacks one to `http://nebula.test/`.
Mechanism: seo
Status: Agreed 2026-10-16

[SEO-002] Every page Nebula renders through Inertia MUST carry a meta description: Home's is Home's message, and every other page's is `SUMMARY`.
Falsifier: The first-visit HTML of `GET /` lacks a `description` meta tag holding the `message` an Inertia visit to `/` returns, or that of `GET /login` lacks one holding `SUMMARY`.
Mechanism: seo
Status: Agreed 2026-10-16

[SEO-003] Every page Nebula renders through Inertia MUST carry `og:site_name` set to `APP_NAME`, `og:type` set to `website`, `og:image` set to the absolute URL of `/android-chrome-512x512.png` on `APP_URL`, and `twitter:card` set to `summary`; Home also carries `og:title` set to its title.
Falsifier: With `APP_NAME=Nebula` and `APP_URL=http://nebula.test`, the first-visit HTML of `GET /` or `GET /login` lacks one of those four tags or has another value, or that of `GET /` lacks `og:title` holding the `title` an Inertia visit to `/` returns.
Mechanism: seo
Status: Agreed 2026-10-16

[SEO-004] Home MUST be open to indexing, and the sign-in, registration, password-reset, email-verification, dashboard and profile pages MUST each carry a robots meta tag that forbids indexing.
Falsifier: The first-visit HTML of `GET /` has a robots meta tag holding `noindex` or `none`, or that of `/login`, `/register`, `/forgot-password`, `/reset-password`, `/verify-email`, `/dashboard` or `/profile` has no robots meta tag holding `noindex` or `none`.
Mechanism: seo
Status: Agreed 2026-10-16

[SEO-005] An error page MUST carry a robots meta tag that forbids indexing.
Falsifier: The first-visit HTML of a `GET` of a path no route matches has no robots meta tag holding `noindex` or `none`.
Mechanism: seo
Status: Agreed 2026-10-16

[SEO-006] Nebula MUST answer `GET` and `HEAD /robots.txt` with status 200, `Content-Type: text/plain; charset=utf-8` and `Cache-Control: public, max-age=3600`, and a body that lets every crawler fetch every path and names the sitemap on `APP_URL`.
Falsifier: With `APP_URL=http://nebula.test`, `GET` or `HEAD /robots.txt` answers another status, `Content-Type` or `Cache-Control`, or the `GET` body lacks the line `User-agent: *` or the line `Sitemap: http://nebula.test/sitemap.xml`, or has a `Disallow:` line naming a path.
Mechanism: seo
Status: Agreed 2026-10-16

[SEO-007] Nebula MUST answer `GET` and `HEAD /sitemap.xml` with status 200, `Content-Type: application/xml; charset=utf-8` and `Cache-Control: public, max-age=3600`, and a body that is a sitemaps.org `urlset` with one `<loc>` per public page, built from `APP_URL`.
Falsifier: With `APP_URL=http://nebula.test`, `GET` or `HEAD /sitemap.xml` answers another status, `Content-Type` or `Cache-Control`, or the `GET` body lacks the namespace `http://www.sitemaps.org/schemas/sitemap/0.9` or the element `<loc>http://nebula.test/</loc>`.
Mechanism: seo
Rationale: The public pages are the list in `src/markdown.rs`, the same list as the twins and `/llms.txt`.
Status: Agreed 2026-10-16

[SEO-008] A response to `/robots.txt` or `/sitemap.xml` MUST carry no `Set-Cookie` header, whether or not the request carries a session cookie.
Falsifier: A `GET` or `HEAD` of `/robots.txt` or `/sitemap.xml`, with or without a session cookie, answers with a `Set-Cookie` header.
Mechanism: seo
Rationale: Both are public for an hour, so a shared cache must never store a visitor's cookie with them.
Status: Agreed 2026-10-16

[SEO-009] Home's first-visit HTML MUST carry JSON-LD describing the site: a `WebSite` and an `Organization`, each with `name` set to `APP_NAME` and `url` set to `APP_URL` plus `/`, and the `Organization` also with `logo` set to the absolute URL of `/android-chrome-512x512.png`.
Falsifier: With `APP_NAME=Nebula` and `APP_URL=http://nebula.test`, the `GET /` HTML has no `application/ld+json` script holding a `WebSite` with those values, or none holding an `Organization` with those values and that logo.
Mechanism: seo
Status: Agreed 2026-10-16

[SEO-010] Nebula MUST serve `/robots.txt` and `/sitemap.xml` only for `GET` and `HEAD`.
Falsifier: `POST /robots.txt` answers 200 `text/plain` with the robots body, or `POST /sitemap.xml` answers 200 `application/xml` with the sitemap body.
Mechanism: seo
Status: Agreed 2026-10-16
