# Upgrade Guide

## Nebula 2.0.1 → 3.0.0

Release `v3.0.0` upgrades Suprnova from 3.2.1 to 4.1.0 and brings the kit to the shape `suprnova new` generates at 4.1.0. Rust, frontend and npm lockfile package versions are 3.0.0. The framework tag resolves to `3aa40bd7c266d41d86a21c0eebbbb1d5a2d09d1d` in Cargo.lock.

### Prepare and merge

Commit local changes. Preserve `.env`, the existing `APP_KEY`, the database and custom brand files. Take a consistent SQLite backup or PostgreSQL `pg_dump`; a live SQLite file copy can miss WAL data.

With `upstream` set to `https://github.com/eas4ai/Nebula.git`:

```bash
git fetch upstream refs/tags/v3.0.0
git switch -c upgrade/suprnova-4.1.0
git merge --no-ff FETCH_HEAD
```

### What changes

- **Sessions table.** Suprnova 4 keeps database sessions in Laravel 13's layout. The migrator now ends with the framework's `CreateSessionsTable`, which reshapes the table `m20240101_000002` created and keeps every row: its id, user, data and CSRF token. Signed-in users stay signed in. Without this migration every request answers `500 Internal Server Error: session persistence failed`. If you keep your own migration list, add `CreateSessionsTable::new(SessionUserKey::Integer)` after your last migration.
- **`APP_ENV`.** An unset `APP_ENV` now means production. Set `APP_ENV=local` in development. Test harnesses that drive the HTTP stack set `APP_ENV=test`, as the kit's suites do.
- **Security headers.** The HTTP stack starts with `SecurityHeaders::from_env()`. An unset `SECURITY_HEADERS` means `balanced`: an enforced Content-Security-Policy with a fresh nonce on each HTML response. An inline `<script>` runs only with that nonce, so the kit moved mode-watcher's early theme script into `templates/app.html`, which receives it. To see violations without blocking, set `SECURITY_HEADERS_CSP_REPORT_ONLY=true`.
- **HTTP stack.** `bootstrap::register()` now holds only process-wide services. The global middleware and `Inertia::install` moved to `bootstrap::try_register_http_stack()`, wired with `.http_bootstrap(...)` in `cmd/main.rs`. Move middleware you added to `register()` there. The order is: security headers, logging, Markdown twins, `/llms.txt`, branding files, `/robots.txt`, `/sitemap.xml`, canonical link, session, Inertia, request timeout (30 seconds, then `503`), CSRF, include. The console no longer installs the HTTP stack, so it runs without built assets, and it ends with a command's own exit code.
- **Root template.** `frontend/index.html` is gone. First visits render through `templates/app.html`. Move head content you added to `index.html` there.
- **Error page.** Framework error responses (403, 404, 419, 429, 500, 503) render `frontend/src/pages/Error.svelte` instead of the Inertia client's JSON dialog.
- **Asset version.** `INERTIA_VERSION` is gone. The version is the hash of the Vite manifest, so each frontend build makes open tabs reload onto the new assets.
- **Shared and flash props.** `bootstrap::AuthShare` is replaced by `src/props/shared.rs`, shared lazily as `auth`, and `src/props/flash.rs`. `suprnova generate-types` types both, so `frontend/src/types/auth.ts` and `frontend/src/types/inertia.d.ts` are removed. `with_all_errors(true)` keeps every validation message per field, as `string[]`.
- **Validation.** `inertia_form` runs the field rules and both cross-field hooks and returns all their errors together, as `FormRequest` does since 4.0.
- **Password reset.** Only a `400` from `PasswordReset::complete` (an invalid, used or expired token) shows the invalid-link message. Other errors are returned as errors.
- **Frontend packages.** `@inertiajs/core`, `@inertiajs/svelte` and `@inertiajs/vite` at `^3.8.0`, and `@sentry/svelte`. Declaring `@inertiajs/vite` makes `suprnova serve` write `public/hot`, so development pages load from Vite. Without it, the output of an earlier `npm run build` shadows the dev server.
- **UI library.** sv5ui is replaced by shadcn-svelte. Components live in `frontend/src/lib/components/ui/`, the theme tokens in `frontend/src/app.css` (`:root` and `.dark`), and Nebula's own pieces in `frontend/src/lib/layouts/` and `frontend/src/lib/components/`. Sign-in pages use a guest layout and the rest an app layout. Flash messages show as toasts. The unused `@tailwindcss/forms` and `@tailwindcss/typography` plugins are removed; add them back if your own pages use their classes. Pages that you built on sv5ui components or its `--color-on-*` / `surface` tokens must be ported to shadcn-svelte components and tokens; the sv5ui `$app/state` stub alias is gone. `frontend/src/lib/Layout.svelte` is deleted: its top bar is now `lib/layouts/AppLayout.svelte`, its flash alerts are toasts (`lib/flash.ts`), and its icon and manifest links are in `templates/app.html`. `frontend/src/lib/AuthEmblem.svelte` is deleted too; the guest layout shows the mark. If you changed either file, carry your changes into the new places before you merge, or git reports a modify/delete conflict.
- **Server-side rendering.** `frontend/src/ssr.ts` and `npm run build:ssr` build the SSR worker to `frontend/bootstrap/ssr/ssr.js`. In development, `suprnova serve` renders first visits through Vite. In production, run the worker next to the server (see below). Without a worker, pages render in the browser.
- **Markdown twins.** New dependency `suprnova-markdown` (v0.3.0). The Home page has a Markdown copy at `/index.html.md` for crawlers that do not run JavaScript, `/llms.txt` lists it, and Home's `<head>` links to it. `src/markdown.rs` holds the list of pages with twins. Both are global middleware placed before the session, so their responses carry no cookie. If you added a route at `/llms.txt`, the new middleware answers it first. A route of your own that ends in `.md` keeps working unless `PAGES` lists its page. See "Markdown twins for crawlers" in the README.
- **Branding files (security fix).** Since Nebula 0.2.1 the favicon set and `site.webmanifest` were routes. Each response told caches to keep it for a day (`Cache-Control: public, max-age=86400`) and also set the session and `XSRF-TOKEN` cookies, so a shared cache such as a CDN could store one visitor's session cookie and hand it to others. `BrandingFiles` in `src/controllers/static_files.rs` now serves these files as global middleware before the session, so they carry no cookie. The seven `static_files::serve` routes are gone from `src/routes.rs`. If you added a root-level file there, add it to the whitelist in `static_files.rs` instead. If you run 2.x behind a shared cache, also check that the cache does not store responses that carry `Set-Cookie`. After you deploy the fix, purge these seven files from any shared cache, which may hold copies with a visitor's cookie for up to a day, and if such a cache sat in front of the app, consider ending existing sessions. `templates/app.html` no longer asks for the manifest with `crossorigin="use-credentials"`: that worked around the cookie the manifest response used to set.
- **Search and social metadata.** New `src/seo.rs`. Every page now carries a canonical link built from `APP_URL`, a `description` (`SUMMARY`, or a page's own), Open Graph tags (site name `APP_NAME`, type `website`, image `/android-chrome-512x512.png`) and an X card. The sign-in, account and error pages carry `noindex`, through `with_head` on their route groups and on the public `/verify-email/verify` route, and `Head::errors`. `seo::CanonicalLink`, a global middleware, builds each page's canonical link from `APP_URL` and the request path, because Suprnova's own `canonical()` keeps a path starting with `//` as a link to another host. Home adds JSON-LD. `/robots.txt` and `/sitemap.xml` are new, served as global middleware before the session; a route of your own at either path is no longer reached for `GET` or `HEAD`. Each `PublicPage` in `src/markdown.rs` gains an `href` field, the page's own address. If your pages set meta tags of their own in `<svelte:head>`, remove the ones the server now sends, or they appear twice. See "Search engines and social sites" in the README.
- **Unchanged.** Nebula's password reset runs through Magnetar, so two 4.0 changes do not apply. The reset mail greets the reader as it did in 2.0.1. The reset revokes sessions and remember-me tokens in the same step, so no queue worker is needed for revocation retries.

### Build and migrate

Use Rust 1.94+ and the Node/npm versions the locked Vite dependencies require. From the repository root:

```bash
(cd frontend && npm ci && npm run check && npm run build && npm run build:ssr)
cargo build --locked --release --bins
cargo test --locked
```

Install the CLI at the same tag:

```bash
cargo install --force --locked --git https://github.com/eas4ai/suprnova.git --tag v4.1.0 suprnova-cli
```

Stop writers, back up, then migrate once with your deployment configuration:

```bash
./target/release/nebula migrate
./target/release/nebula serve --no-migrate
```

### Production configuration

Deploy `public/assets/` and `frontend/bootstrap/ssr/` with the binaries. The SSR worker imports packages from `frontend/node_modules` when it runs, so install the frontend dependencies on the host as well. Run the worker as a second supervised process, from the `frontend/` directory:

```bash
node bootstrap/ssr/ssr.js    # listens on 127.0.0.1:13714
```

Keep `APP_ENV=production`, `APP_DEBUG=false`, the existing `APP_KEY`, the HTTPS `APP_URL`, the mail and rate-limit settings from 2.0.1. Add:

- `SESSION_SECURE=true`.
- `SECURITY_HEADERS` (`balanced` when unset). Check the browser console for policy violations before you enforce a stricter preset.
- `REPORTING_DSN` and the other `REPORTING_*` settings to send server errors to Sentry, GlitchTip or Rustrak. Set `VITE_REPORTING_DSN` before `npm run build` to report browser errors. Without a DSN nothing is sent.

### Verify and roll back

Check that JS/CSS load, that first visits are server-rendered (the page source has `data-server-rendered`), and that the browser console shows no Content-Security-Policy violations. Then check registration, login and logout, reset and verification mail, the protected dashboard, and an unknown URL (the Error page). A user who was signed in before the upgrade should still be signed in.

If validation fails, stop writers and restore the previous binaries, assets and configuration together with the database backup: the 2.0.1 binaries expect the old `sessions` layout, with its `csrf_token` column.

## Nebula 2.0.0 → 2.0.1

Release `v2.0.1` upgrades Suprnova from 2.0.0 to 3.2.1. Rust, frontend and npm lockfile package versions are 2.0.1. The framework tag resolves to `2bd4bd53d04fa4581fdbb152ebd70cfb682b342c` in Cargo.lock.

### Prepare and merge

Commit local changes. Preserve `.env`, the existing `APP_KEY`, database, custom brand files and previous deployment artifacts. Take a consistent SQLite backup or PostgreSQL `pg_dump`. A live SQLite file copy can miss WAL data.

Configure a fork's `upstream` once to `https://github.com/eas4ai/Nebula.git`, then:

```bash
git fetch upstream refs/tags/v2.0.1
git switch -c upgrade/suprnova-3.2.1
git merge --no-ff FETCH_HEAD
```

Retain customizations and the release lockfiles. Fresh-install instructions to copy `.env.example` and generate a key do not apply to an existing deployment.

### Build and migrate

Use Rust 1.94+ and the Node/npm versions required by the locked Vite dependencies. From the repository root:

```bash
(cd frontend && npm ci && npm run check && npm run build)
cargo build --locked --release --bins
cargo test --locked
```

Deploy `public/assets/` with the binaries and root brand files. This release adds `/assets/{*path}` using Suprnova's StaticFiles handler for built JS/CSS. Retain this route in customized route tables; other root assets still use the explicit whitelist in `src/controllers/static_files.rs`.

Stop writers and back up before running migrations once with your deployment configuration:

```bash
./target/release/nebula migrate
./target/release/nebula serve --no-migrate
```

Use your supervisor and the application working directory. This release adds no Nebula migration file; check migrations from your own changes.

### Production configuration

Preserve `APP_KEY`; set `APP_ENV=production`, `APP_DEBUG=false` and the correct HTTPS `APP_URL`. Use `RATE_LIMIT_DRIVER=redis` and `RATE_LIMIT_REDIS_URL`. Exactly one process may explicitly retain memory with `RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION=true`; quotas reset on restart and do not span replicas.

Replace development `MAIL_DRIVER=log` with a delivering transport. For SMTP, rename `MAIL_HOST`, `MAIL_PORT`, `MAIL_USERNAME` and `MAIL_PASSWORD` to `MAIL_SMTP_HOST`, `MAIL_SMTP_PORT`, `MAIL_SMTP_USER` and `MAIL_SMTP_PASS`. Configure `MAIL_FROM` and `MAIL_SMTP_ENCRYPTION=starttls` (normally 587) or `tls` (normally 465), according to the provider. Both require credentials. Plaintext `none` is for a local development mail catcher.

Upgrade the developer CLI separately if used:

```bash
cargo install --git https://github.com/eas4ai/suprnova.git --tag v3.2.1 suprnova-cli --locked
```

### Verify and roll back

Check actual JS/CSS loading, branding, registration, login/logout, reset and verification mail, and the protected dashboard. Unlisted public files must remain inaccessible. The release passed 14 Rust tests, frontend checks/builds and an isolated production asset smoke check; verify your production mail, Redis and database separately.

If validation fails, stop writers and restore the previous binaries, assets and configuration. Restore the matching database/key backup if migrations changed state. Keep the prior deployment until recovery is verified.
