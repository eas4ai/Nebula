# Nebula recon

Date: 2026-10-16. Path A: no `docs/spec/` existed. Read at local `main` `185f74e` (17 commits ahead of `origin/main` `bfe01d3`, not pushed).

Status words: **Exists** (seen in code or by running it), **Documented** (a doc says so; not observed here), **Contradicted** (two sources disagree), **Unverified** (not checked).

## Manifests and build

| Claim | Status | Citation |
|---|---|---|
| Crate `nebula` 3.0.0, Rust edition 2024, `rust-version` 1.94.0, binaries `nebula` (`cmd/main.rs`) and `console` (`src/bin/console.rs`). | Exists | `Cargo.toml:1-24` |
| Suprnova is pinned by `tag = "v4.1.0"` with default features off and eleven listed; the `testing` feature is on only for dev builds. | Exists | `Cargo.toml:32`, `Cargo.toml:47` |
| Cargo builds into `/home/shawn/workspace2/scratchpads/targets/Nebula/main`; `.cargo/` is not tracked. | Exists | `.cargo/config.toml:4-5`; `git ls-files .cargo` is empty |
| Frontend `nebula-frontend` 3.0.0: Inertia 3.8, Svelte 5, shadcn-svelte (bits-ui), Tailwind v4, Vite; scripts `check`, `build` (svelte-check + vite), `build:ssr`. | Exists | `frontend/package.json:1-30` |
| `cargo test --locked`, `npm run check`, `npm run build`, `npm run build:ssr` all pass on this tree. | Exists | Run 2026-10-16 after the merge: Rust 5+1+8 passed, svelte-check 0 errors 0 warnings, both builds succeeded |
| README asks for Node 20+. | Documented | `README.md:37`; `package.json` has no `engines` field (Unverified that 20 is the real floor) |

## Entry points and request path

| Claim | Status | Citation |
|---|---|---|
| The server boots with `#[suprnova::main]`: config, `bootstrap::register` (all processes), `bootstrap::register_http_stack` (server only), routes, migrations. | Exists | `cmd/main.rs:14-25` |
| Global middleware order: SecurityHeaders, Logging, Session, Inertia layer, Timeout (30 s), CSRF, Include. | Exists | `src/bootstrap.rs:125,128,134,156,176,183,190` |
| Every Inertia response shares `root` and a lazy `auth` prop. | Exists | `src/bootstrap.rs:194,199` |
| Routes: `/` public; `/assets/*` and seven branding files at the web root; `/verify-email/verify` public; guest group (login, register, forgot/reset password); auth group (verify notice, resend, logout, profile); auth + verified group (`/dashboard`). No fallback route. | Exists | `src/routes.rs:8-67` |
| Home props are two hard-coded strings, `title` and `message`. | Exists | `src/controllers/home.rs:5-6,15-16` |
| `app_url()` falls back to `http://localhost:8765` when `APP_URL` is unset. That is Suprnova's own default listen port; `.env.example` sets both `SERVER_PORT=8080` and `APP_URL=http://localhost:8080`, which the README follows. (An earlier reading marked this Contradicted.) | Exists | `src/controllers/mod.rs:16`; Suprnova v4.1.0 `framework/src/config/providers/server.rs:12` (`DEFAULT_SERVER_PORT: u16 = 8765`); `.env.example:4,13`; `README.md:52-55` |
| First visits render through `templates/app.html`, which writes the resolved head with `{{ head }}`. | Exists | `templates/app.html:35`, `src/bootstrap.rs:44-46` |
| The client keeps the document head in step with the server's `head` prop (`serverHead: true`) in both entries. | Exists | `frontend/src/main.ts:161`, `frontend/src/ssr.ts:18` |
| Server-side rendering is on by default; without a running SSR worker pages render in the browser. | Documented | `README.md:24-25,190-195`; `UPGRADE.md` "Server-side rendering" |

## Data

| Claim | Status | Citation |
|---|---|---|
| Five kit migrations (users, sessions, remember tokens, `email_verified_at`, auth-flow tokens), then the framework's `CreateSessionsTable`. Migrations run on boot. | Exists | `src/migrations/mod.rs:15-24`, `src/bootstrap.rs:61-70` |
| One model, `User`. | Exists | `src/models/user.rs` |

## Tests

| Claim | Status | Citation |
|---|---|---|
| `tests/auth_flows.rs` (5 tests, flow logic, mail faked), `tests/http_flows.rs` (8 tests, real routes + `try_register_http_stack` over a loopback socket), `tests/console_bootstrap.rs` (1 test). No unit tests in `src/`. | Exists | `grep -c` of test attributes; `tests/http_flows.rs:119-126` |
| README says two suites cover the flows; a third suite, `console_bootstrap.rs`, is not mentioned. | Contradicted (minor) | `README.md:214` vs `tests/console_bootstrap.rs` |
| HTTP tests set `APP_ENV=test`, `APP_URL=http://nebula.test` and hold `common::TEST_LOCK`. | Exists | `tests/http_flows.rs:85-94` |

## CI, scripts, docs

| Claim | Status | Citation |
|---|---|---|
| No CI: there is no `.github/` directory. No `scripts/` directory. | Exists | `ls -a` at the root |
| README (setup, customizing, production, testing) and UPGRADE (3.0.0, 2.0.1 sections) are the user docs; `docs/header.jpg` is the README image. | Exists | `README.md`, `UPGRADE.md`, `docs/` |
| Apps are made by cloning the kit, so every tracked file ships to every new app. | Documented | `README.md:46` ("git clone <your-fork-of-this-kit> my-app") |

## Sudus fit

| Claim | Status | Citation |
|---|---|---|
| Sudus initialized 2026-10-16 with authority remote `origin` (`github.com/eas4ai/Nebula`); `.sudus/settings.json` is untracked. | Exists | init record `f24066f`; `git status` |
| `.gitignore` ignores `AGENTS.md`, which Sudus requires to be committed as the working agreement. | Contradicted | `.gitignore:40` vs the Sudus working agreement ("protected; changes only between commitments") |
| `docs/spec/`, `docs/recon.md` and `docs/decisions.jsonl` are not ignored; only `/docs/superpowers/` is. | Exists | `.gitignore:43`; `git check-ignore` |
| Checks run in the working tree, so `.cargo/config.toml` applies to `cargo test` run by a mechanism. | Exists | Sudus `lib/check.mjs:227` (`runCwd = def.cwd ? join(cwd, def.cwd) : cwd`) |

## Framework facts the planned work depends on

| Claim | Status | Citation |
|---|---|---|
| The CSRF middleware sets an `XSRF-TOKEN` cookie on every response. | Documented | Suprnova v4.1.0 `framework/src/csrf/middleware.rs:169-180` |
| A route cannot leave out global middleware. | Documented | Suprnova v4.1.0 `framework/src/routing/router.rs:2735-2739` |
| `Head::try_update` and `LinkTag` set a page's head at run time; Inertia writes the resolved head into the first visit's `<head>` and shares it as the `head` prop. | Documented | Suprnova v4.1.0 `framework/src/view/head.rs:11-15,146-160`, `framework/src/view/head/builder.rs:587-622,1349-1360` |
| A routed `public, max-age=3600` text file gets both cookies in practice: live suprnova.app `/llms.txt` answered with `suprnova_session` and `XSRF-TOKEN` `Set-Cookie`. | Exists | `curl -D - https://suprnova.app/llms.txt`, 2026-10-16 |
| `suprnova-markdown` v0.3.0: `MarkdownSource::markdown -> Result<Option<String>, FrameworkError>`; only GET/HEAD reach the source; failed lookups answer a no-store 500; twins are `text/markdown; charset=utf-8`, `noindex`, `public, max-age=3600`. | Exists | `~/workspace2/suprnova-markdown/src/lib.rs:66-68,117-148`; tag `v0.3.0` on `github.com/eas4ai/suprnova-markdown` |
| Whether the dev SSR render writes the alternate link once or twice in the first visit's HTML. | Unverified | To check in the work (plan Task 5, Step 5) |

## Recent history

- 2026-10-16: Nebula 3.0.0 on Suprnova 4.1.0 (`7e3bc66`), the shadcn-svelte UI and review fixes (16 commits to `185f74e`); merged into local `main` without a merge commit, not pushed, not tagged.
- 2026-10-05: Nebula 2.0.1 on Suprnova 3.2.1 (`70765c2`), the last state on `origin/main` (`bfe01d3`, a docs commit).
- 2026-08-24: branch `upgrade/seaorm-2` (local and on `origin`), the SeaORM 2 upgrade; fully merged into `main` (0 commits ahead).
- 68 commits in all; no earlier Sudus or Cairn records.

## Unresolved findings carried

None: this is the first recon.
