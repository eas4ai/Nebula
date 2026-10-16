# Nebula

Nebula is a starter kit for web apps on Suprnova, a Rust web framework, with an Inertia 3 and Svelte 5 frontend. A developer forks it and clones the fork to start an app (`README.md:46`). The app upgrades by merging each Nebula release tag (`UPGRADE.md:11-16`).

## The problem it solves

A new Suprnova app needs a lot wired together and tested before its own work can begin: sign-in, email verification, password reset, a profile page, sessions, CSRF protection, security headers, server-side rendering and a themed UI. Nebula ships that wiring, so the app starts from a working, tested base.

## What it is not

- Not a library. Apps copy Nebula; they do not depend on it.
- Not a content site. It has one public prose page, Home.
- Not a home for app features. What an app adds belongs in the app.

## Spec map

| File | Prefix | Covers |
|---|---|---|
| http-stack.md | STACK | the global middleware stack: security headers, session and CSRF cookies, the request timeout, unmatched paths |
| pages.md | PAGE | Home's content, the document head on first and later visits, the app's external base URL |
| twins.md | TWIN | Markdown twins of public pages, `/llms.txt`, and the alternate link that advertises a twin |

## Not yet specified

One line per area outside the current work. Each area gets a domain file when a commitment needs it.

- Sign-in: register, log in, log out and remember-me (`src/controllers/auth.rs`, `tests/auth_flows.rs`).
- Email verification: the notice, resending the link, the verify link, and the `verified` gate on `/dashboard`.
- Password reset: request a link, reset with its token, and no account enumeration.
- Profile: update details, change the password, delete the account.
- Dashboard: the signed-in, verified landing page.
- Static files: `/assets/*` and the branding files at the web root.
- UI: the shadcn-svelte layouts, theme tokens, flash toasts and the error page.
- Server-side rendering: the SSR worker and the `ssr:*` commands.
- Console: the `console` binary and its commands.
- Data: the five kit migrations and the `User` model.
