# Upgrade Guide

## Nebula 2.0.0 → 2.0.1

Release `v2.0.1` upgrades Suprnova from 2.0.0 to 3.2.1. Rust, frontend and npm lockfile package versions are 2.0.1. The framework tag resolves to `2bd4bd53d04fa4581fdbb152ebd70cfb682b342c` in Cargo.lock.

## Prepare and merge

Commit local changes. Preserve `.env`, the existing `APP_KEY`, database, custom brand files and previous deployment artifacts. Take a consistent SQLite backup or PostgreSQL `pg_dump`. A live SQLite file copy can miss WAL data.

Configure a fork's `upstream` once to `https://github.com/eas4ai/Nebula.git`, then:

```bash
git fetch upstream refs/tags/v2.0.1
git switch -c upgrade/suprnova-3.2.1
git merge --no-ff FETCH_HEAD
```

Retain customizations and the release lockfiles. Fresh-install instructions to copy `.env.example` and generate a key do not apply to an existing deployment.

## Build and migrate

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

## Production configuration

Preserve `APP_KEY`; set `APP_ENV=production`, `APP_DEBUG=false` and the correct HTTPS `APP_URL`. Use `RATE_LIMIT_DRIVER=redis` and `RATE_LIMIT_REDIS_URL`. Exactly one process may explicitly retain memory with `RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION=true`; quotas reset on restart and do not span replicas.

Replace development `MAIL_DRIVER=log` with a delivering transport. For SMTP, rename `MAIL_HOST`, `MAIL_PORT`, `MAIL_USERNAME` and `MAIL_PASSWORD` to `MAIL_SMTP_HOST`, `MAIL_SMTP_PORT`, `MAIL_SMTP_USER` and `MAIL_SMTP_PASS`. Configure `MAIL_FROM` and `MAIL_SMTP_ENCRYPTION=starttls` (normally 587) or `tls` (normally 465), according to the provider. Both require credentials. Plaintext `none` is for a local development mail catcher.

Upgrade the developer CLI separately if used:

```bash
cargo install --git https://github.com/eas4ai/suprnova.git --tag v3.2.1 suprnova-cli --locked
```

## Verify and roll back

Check actual JS/CSS loading, branding, registration, login/logout, reset and verification mail, and the protected dashboard. Unlisted public files must remain inaccessible. The release passed 14 Rust tests, frontend checks/builds and an isolated production asset smoke check; verify your production mail, Redis and database separately.

If validation fails, stop writers and restore the previous binaries, assets and configuration. Restore the matching database/key backup if migrations changed state. Keep the prior deployment until recovery is verified.
