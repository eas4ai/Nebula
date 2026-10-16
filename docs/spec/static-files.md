# Static files

Prefix: STATIC

The branding files at the web root, read from `public/`: `/favicon.ico`, `/favicon-16x16.png`, `/favicon-32x32.png`, `/android-chrome-192x192.png`, `/android-chrome-512x512.png`, `/apple-touch-icon.png` and `/site.webmanifest`. Browsers and the web app manifest ask for them at the root, and shared caches may keep them for a day. Found on 2026-10-16: they were routes, so every response also set the session and `XSRF-TOKEN` cookies, from Nebula v0.2.1 through v2.0.1. Like the Markdown twins, they are now answered before the session middleware.

The mechanism `statics` runs the tests named `static_<nnn>_…` in `tests/http_flows.rs` and reports each test's result for `STATIC-<nnn>`.

[STATIC-001] Nebula MUST answer `GET` and `HEAD` of each branding file with status 200, the file's content type and `Cache-Control: public, max-age=86400`, and a `GET` with the file's bytes from `public/`.
Falsifier: A `GET` or `HEAD` of a branding file answers another status, content type or `Cache-Control`, or a `GET` body differs from the file in `public/`.
Mechanism: statics
Status: Draft

[STATIC-002] A response to a branding file MUST carry no `Set-Cookie` header, whether or not the request carries a session cookie.
Falsifier: A `GET` or `HEAD` of a branding file, with or without a session cookie, answers with a `Set-Cookie` header.
Mechanism: statics
Rationale: The fourth safety rule of the twins holds for every publicly cacheable response: a shared cache must never store a visitor's session or `XSRF-TOKEN` cookie.
Status: Draft

[STATIC-003] A response to a branding file MUST carry `X-Content-Type-Options: nosniff`.
Falsifier: A `GET` of a branding file answers without `X-Content-Type-Options: nosniff`.
Mechanism: statics
Status: Draft

[STATIC-004] Nebula MUST serve a branding file only for a listed path and only for `GET` and `HEAD`: an unlisted root path answers 404, and a `POST` gets no file.
Falsifier: `/site.webmanifest.bak` or `/Cargo.toml` answers a status other than 404, or `POST /favicon.ico` answers 200 with `Content-Type: image/x-icon`.
Mechanism: statics
Status: Draft
