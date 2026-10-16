# Pages

Prefix: PAGE

Every block here describes the code as read on 2026-10-16.

[PAGE-001] The Home handler answers `GET /` with the Inertia page `Home` and the props `title` "Welcome to Suprnova!" and `message` "Your Inertia app is ready.".
Falsifier: `GET /` renders another component, or a `title` or `message` with other text.
Rationale: `content()` and `index` in `src/controllers/home.rs`, and `get!("/", …)` in `src/routes.rs`. The strings are hard-coded for an app to replace.
Status: Observed

[PAGE-002] When a request sets a head tag, a first visit writes it into the HTML `<head>`, and an Inertia response carries the page's head tags as the `head` prop, which the client applies on later visits.
Falsifier: A request sets a head tag, and the first visit's `<head>` lacks it, or the Inertia response's `head` prop lacks it.
Rationale: `templates/app.html:35`, `src/bootstrap.rs:44-46`, `serverHead: true` in `frontend/src/main.ts:161` and `frontend/src/ssr.ts:18`, Suprnova v4.1.0 `framework/src/view/head.rs:11-15,479-499`. When nothing sets a head, the page has no `head` prop: an Inertia visit to `/login` has none, and one to `/` had none before Home linked its twin.
Status: Observed

[PAGE-003] `app_url()` returns `APP_URL`, or `http://localhost:8765` when `APP_URL` is unset.
Falsifier: With `APP_URL` set, `app_url()` returns another value; with it unset, it returns another address.
Rationale: `src/controllers/mod.rs:15-17`. Port 8765 is Suprnova's own default listen port (v4.1.0 `framework/src/config/providers/server.rs:12`). `.env.example` sets `SERVER_PORT` and `APP_URL` to port 8080 together.
Status: Observed
