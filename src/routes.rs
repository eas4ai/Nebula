use suprnova::{StaticFiles, delete, get, group, patch, post, put, routes};

use crate::controllers;
use crate::middleware;

routes! {
    // Public routes
    get!("/", controllers::home::index),

    // Vite's manifest references bundles under /assets; branding files at
    // the web root keep their explicit whitelist below.
    get!("/assets/{*path}", StaticFiles::public().handler()),

    // Branding files browsers and the web app manifest request at the web
    // root. The public directory is not served implicitly, so the kit
    // serves its small root-level whitelist through explicit routes - the
    // same handler in dev and prod (the page origin is the Rust server in
    // both; Vite only supplies JS/CSS).
    get!("/favicon.ico", controllers::static_files::serve),
    get!("/favicon-16x16.png", controllers::static_files::serve),
    get!("/favicon-32x32.png", controllers::static_files::serve),
    get!("/android-chrome-192x192.png", controllers::static_files::serve),
    get!("/android-chrome-512x512.png", controllers::static_files::serve),
    get!("/apple-touch-icon.png", controllers::static_files::serve),
    get!("/site.webmanifest", controllers::static_files::serve),

    // The verification-token consume endpoint is self-contained: the token in
    // the query string is the proof, so no session is needed. Keep it public
    // so a link opened on a logged-out device still verifies (gating it behind
    // `auth()` would redirect to `/login` and drop the token).
    get!("/verify-email/verify", controllers::verify_email::verify),

    // Guest-only routes (redirect to dashboard if logged in)
    group!("/", {
        get!("/login", controllers::auth::show_login),
        post!("/login", controllers::auth::login),
        get!("/register", controllers::auth::show_register),
        post!("/register", controllers::auth::register),
        get!("/forgot-password", controllers::password_reset::show_request),
        post!("/forgot-password", controllers::password_reset::send_link),
        get!("/reset-password", controllers::password_reset::show_reset),
        post!("/reset-password", controllers::password_reset::reset),
    }).middleware(middleware::authenticate::guest()),

    // Authenticated, verification NOT required. An unverified-but-logged-in
    // user must be able to view the notice, resend the link, and log out - so
    // these stay off the `verified` gate. (The verify-token endpoint itself is
    // public, above - it needs no session.)
    group!("/", {
        get!("/verify-email", controllers::verify_email::show_notice),
        post!("/email/verification-notification", controllers::verify_email::resend),
        post!("/logout", controllers::auth::logout),

        // Self-service profile management. Kept off the `verified` gate so an
        // unverified-but-logged-in user can still update their details - and,
        // by changing their email, re-trigger verification.
        get!("/profile", controllers::profile::show),
        patch!("/profile", controllers::profile::update),
        put!("/profile/password", controllers::profile::update_password),
        delete!("/profile", controllers::profile::destroy),
    }).middleware(middleware::authenticate::auth()),

    // Authenticated AND email-verified. `verified()` composes after `auth()`,
    // so an unverified user is redirected to `/verify-email` and an
    // unauthenticated one to `/login`.
    group!("/", {
        get!("/dashboard", controllers::dashboard::index),
    })
        .middleware(middleware::authenticate::auth())
        .middleware(middleware::authenticate::verified()),
}
