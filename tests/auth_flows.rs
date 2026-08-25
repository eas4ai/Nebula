//! Account-management flow tests for the Nebula starter kit - facade level.
//!
//! These exercise the three account flows the kit ships - email verification,
//! password reset (with anti-enumeration), and the profile update / password /
//! delete surface - end-to-end against a **real** in-memory database (Nebula's
//! own `Migrator`) with the mail transport **faked** (`Mail::fake()`). No mocks:
//! the assertions read the persisted `app_users` row back through the same
//! `EloquentUserProvider<User>` the kit registers, and the tokens are extracted
//! from the captured mail bodies.
//!
//! ## Facade level vs request path
//!
//! This suite tests *below* the router on purpose: it isolates the flow
//! **logic** (token mint/consume, password rotation, verification-stamp
//! write, anti-enumeration) from the HTTP wiring. The request-path
//! counterpart lives in `tests/http_flows.rs`, which drives the kit's real
//! router + global middleware stack through `suprnova::handle_request`.
//!
//! The HTTP wiring was broken at framework rev `06b9447f` - `group!("/")`
//! registered unmatchable `//login` patterns, and `redirect!("/path")`
//! resolved literal paths as route *names* (500 in any app with no named
//! routes). Both are fixed as of `95777465` (canonical `join_paths` prefix
//! joining; literal-shape dispatch in `redirect!`), pinned framework-side by
//! `framework/tests/root_group_redirect.rs` and consumer-side by
//! `tests/http_flows.rs`. Because this suite sits below the router, wiring
//! defects of that kind cannot fail it - a failure here points at the flow
//! logic itself.
//!
//! ## Coverage note
//!
//! These prove: verification send + single-use consume + stamp persistence;
//! resend; password reset rotation + revocation + single-use;
//! anti-enumeration; and the profile email-change re-verification and
//! password-rotation *logic* against the real `User` model. Route gates,
//! session continuity, CSRF, and controller status codes are covered over
//! HTTP in `tests/http_flows.rs`.
//!
//! ## Serial execution
//!
//! `Mail::fake()` swaps the process-global mail transport and the active user
//! provider is process-global, so the file is serialized behind the shared
//! `common::TEST_LOCK` (also held by `tests/http_flows.rs`).

mod common;

use std::sync::Arc;

use chrono::Utc;
use sea_orm::{ConnectionTrait, Statement};

use suprnova::auth::AuthConfig;
use suprnova::auth_flows::{EmailVerification, PasswordReset};
use suprnova::mail::{Mail, MailFake};
use suprnova::{App, Auth, AuthManager, Authenticatable, EloquentUserProvider, MustVerifyEmail};

use nebula::models::user::User;

/// Held-for-the-test guard: keeps the SeaORM connection registered for the
/// duration of the test so the provider + facades resolve `DB::connection()`.
struct Harness {
    _lock: tokio::sync::MutexGuard<'static, ()>,
    mail: MailFake,
}

/// Fresh in-memory DB with Nebula's full migration set, the
/// `EloquentUserProvider::<User>` registered as the active "users" provider
/// (mirroring `bootstrap::register()`), and the load-bearing `MAIL_FROM` /
/// `APP_URL` env set (the verify/reset send paths fail closed without
/// `MAIL_FROM`; `APP_URL` pins the emitted link base).
async fn setup() -> Harness {
    let lock = common::TEST_LOCK.lock().await;

    // SAFETY: every test in this file is serialized behind `common::TEST_LOCK`.
    unsafe {
        std::env::set_var("MAIL_FROM", "test@nebula.test");
        std::env::set_var("APP_URL", "http://nebula.test");
    }
    let mail = Mail::fake();
    common::fresh_magnetar_database().await;

    // Auth wiring - mirror `bootstrap::register()` exactly. `AuthConfig::default()`'s
    // "web" guard points at the "users" provider.
    App::singleton(AuthManager::new(AuthConfig::default()));
    Auth::register_provider("users", Arc::new(EloquentUserProvider::<User>::new()))
        .expect("register users provider");

    Harness { _lock: lock, mail }
}

async fn run_in_request<F, T>(fut: F) -> T
where
    F: Future<Output = T>,
{
    let session_slot = suprnova::session::new_session_slot_for_test();
    let pending_slot = suprnova::session::new_pending_cookies_slot_for_test();
    suprnova::session::session_scope_for_test(
        session_slot,
        suprnova::session::pending_cookies_scope_for_test(
            pending_slot,
            suprnova::auth::request_state::request_state_scope_for_test(fut),
        ),
    )
    .await
}

/// Reload a user from the DB by email via the same model surface the kit uses.
async fn reload(email: &str) -> User {
    User::find_by_email(email)
        .await
        .expect("lookup")
        .unwrap_or_else(|| panic!("user {email} exists"))
}

/// Stamp a user verified (used to seed the password-reset / profile fixtures in
/// the same already-verified state the kit produces after a click-through).
async fn mark_verified(user: &mut User) {
    user.email_verified_at = Some(Utc::now());
    suprnova::eloquent::Model::save(user)
        .await
        .expect("stamp email_verified_at");
}

/// Pull the plaintext token out of the first captured mail whose text body
/// carries a `token=` link (the text body renders the URL verbatim; the HTML
/// body HTML-escapes slashes) - the same extraction the framework facade tests
/// use.
fn token_from_fake(fake: &MailFake) -> String {
    let captured = fake.captured();
    let msg = captured
        .iter()
        .find(|m| {
            m.text
                .as_deref()
                .is_some_and(|t| t.lines().any(|l| l.contains("token=")))
        })
        .expect("a captured mail with a token link");
    let text = msg.text.as_deref().expect("token mail has a text body");
    let link = text
        .lines()
        .find(|l| l.contains("token="))
        .expect("a line with the token link");
    link.split_once("token=")
        .map(|(_, tail)| tail.trim().to_string())
        .expect("verification link should carry token=")
}

// ============================================================================
// 1. Verification: send link → single-use consume → stamp persists
// ============================================================================
//
// The kit's `register` controller sends this link; the public
// `/verify-email/verify` handler consumes it through `EmailVerification::verify`
// and the `verified` gate keys off `is_email_verified()`. This proves the mint
// → consume → persistence chain the gate depends on.

#[tokio::test]
async fn verification_sends_link_consumes_once_and_persists_stamp() {
    let h = setup().await;
    let fake = &h.mail;

    // A freshly-registered, unverified user (created via the kit's own helper).
    let user = User::create("Grace Hopper", "grace@nebula.test", "supersecret")
        .await
        .expect("create user");
    assert!(
        !user.is_email_verified(),
        "a freshly created user is unverified"
    );

    // Send the verification link (the base the kit appends `?token=` to).
    let baseline = fake.count();
    EmailVerification::send_link(&user, "http://nebula.test/verify-email/verify")
        .await
        .expect("send verification link");
    fake.assert_sent_to("grace@nebula.test");
    assert_eq!(fake.count(), baseline + 1, "exactly one verification mail");
    let token = token_from_fake(fake);

    // Not yet verified in the DB.
    assert!(!reload("grace@nebula.test").await.is_email_verified());

    // Consume the token inside a request-scoped authenticated session - the
    // framework requires the caller to be the token owner.
    let user_id = user.get_auth_identifier();
    let token_for_scope = token.clone();
    run_in_request(async move {
        Auth::password()
            .authenticate("grace@nebula.test", "supersecret", None, None)
            .await
            .expect("authenticate matching user into request");
        let id = EmailVerification::verify(&token_for_scope)
            .await
            .expect("verify");
        assert_eq!(id, user_id);
    })
    .await;

    assert!(
        reload("grace@nebula.test").await.is_email_verified(),
        "verify must persist email_verified_at through the provider"
    );

    // Single-use: a second consume of the same token fails.
    assert!(
        EmailVerification::verify(&token).await.is_err(),
        "a consumed verification token must not verify again"
    );
}

// ============================================================================
// 2. Resend: a fresh link for an unverified user; silent for unknown
// ============================================================================
//
// Mirrors `POST /email/verification-notification` (resend) + the kit's
// anti-enumeration posture.

#[tokio::test]
async fn resend_sends_a_fresh_link_and_is_silent_for_unknown() {
    let h = setup().await;
    let fake = &h.mail;
    User::create("Ada Lovelace", "ada@nebula.test", "oldpass1!")
        .await
        .expect("create user");

    // Known, unverified email → a fresh link is mailed.
    let known_before = fake.count();
    EmailVerification::resend("ada@nebula.test", "http://nebula.test/verify-email/verify")
        .await
        .expect("resend known");
    assert_eq!(
        fake.count(),
        known_before + 1,
        "known email must trigger a fresh link"
    );
    fake.assert_sent_to("ada@nebula.test");
    assert!(
        !token_from_fake(fake).is_empty(),
        "the resent link carries a token"
    );

    // Unknown email → anti-enumeration: nothing sent, still Ok.
    let unknown_before = fake.count();
    EmailVerification::resend(
        "nobody@nebula.test",
        "http://nebula.test/verify-email/verify",
    )
    .await
    .expect("resend unknown returns Ok (no leak)");
    assert_eq!(
        fake.count(),
        unknown_before,
        "unknown email must send nothing (anti-enumeration)"
    );
}

// ============================================================================
// 3. Password reset: rotate + revoke + single-use, and anti-enumeration
// ============================================================================
//
// Mirrors `POST /forgot-password` (send_link, anti-enumeration) and `POST
// /reset-password` (complete). Asserts the new password verifies, the old one
// does not, a live session is revoked, the token is single-use, and an unknown
// email leaks nothing.

#[tokio::test]
async fn password_reset_rotates_password_revokes_sessions_and_is_anti_enumerating() {
    let h = setup().await;
    let fake = &h.mail;

    let mut ada = User::create("Ada Reset", "ada@x.com", "oldpass1!")
        .await
        .expect("create user");
    mark_verified(&mut ada).await;
    let id = ada.get_auth_identifier();

    // Seed a live Magnetar session so the reset completion's atomic credential
    // rotation has a persisted session to revoke.
    let db = suprnova::DB::connection().expect("DB initialized");
    db.inner()
        .execute_unprepared(&format!(
            "INSERT INTO auth_sessions \
             (id, user_id, auth_epoch, token_digest, token_hash, user_agent, ip_address, expires_at, revoked_at) \
             VALUES ('ada-auth-session-1', {}, 0, '{}', NULL, NULL, NULL, '2099-01-01T00:00:00Z', NULL)",
            ada.id,
            "a".repeat(64),
        ))
        .await
        .expect("seed Magnetar session");

    // Known email → a reset mail is sent.
    let known_before = fake.count();
    PasswordReset::send_link("ada@x.com", "http://nebula.test/reset-password")
        .await
        .expect("send_link");
    assert_eq!(
        fake.count(),
        known_before + 1,
        "known email must trigger a reset mail"
    );
    fake.assert_sent_to("ada@x.com");
    let token = token_from_fake(fake);

    // Unknown email → anti-enumeration: nothing sent, still Ok.
    let unknown_before = fake.count();
    PasswordReset::send_link("nobody@x.com", "http://nebula.test/reset-password")
        .await
        .expect("send_link unknown returns Ok (no leak)");
    assert_eq!(
        fake.count(),
        unknown_before,
        "unknown email must send nothing (anti-enumeration)"
    );

    // Complete the reset. Magnetar rotates the password and revokes active
    // sessions in one commit.
    let outcome = PasswordReset::complete_with_outcome(&token, "newpass1!")
        .await
        .expect("complete");
    assert_eq!(outcome.user_id, id);
    assert_eq!(
        outcome.sessions_revoked.expect("session revocation"),
        1,
        "the reset must revoke the seeded Magnetar session"
    );

    // New password verifies; old one no longer does.
    let ada = reload("ada@x.com").await;
    assert!(
        ada.verify_password("newpass1!").expect("verify new"),
        "the new password must verify after reset"
    );
    assert!(
        !ada.verify_password("oldpass1!").expect("verify old"),
        "the old password must no longer verify"
    );

    // The persisted session row is tombstoned.
    let row = db
        .inner()
        .query_one_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT COUNT(*) AS revoked FROM auth_sessions \
             WHERE id = 'ada-auth-session-1' AND revoked_at IS NOT NULL",
        ))
        .await
        .expect("query revoked session")
        .expect("revocation count row");
    let revoked: i64 = row.try_get("", "revoked").expect("revocation count");
    assert_eq!(revoked, 1);

    // Single-use: a second complete on the same token fails.
    assert!(
        PasswordReset::complete(&token, "again1!").await.is_err(),
        "a consumed reset token must not complete again"
    );
}

#[tokio::test]
async fn profile_update_does_not_restore_stale_authentication_state() {
    let _h = setup().await;
    let user = User::create("Stale Snapshot", "stale@nebula.test", "oldpass1!")
        .await
        .expect("create user");
    let stale = user.clone();
    let user_id = user.id;
    let replacement_hash = suprnova::hashing::hash("newpass1!").expect("hash replacement");
    user.update_password_hash(replacement_hash.clone())
        .await
        .expect("rotate stored hash");

    suprnova::DB::connection()
        .expect("DB initialized")
        .inner()
        .execute_unprepared(&format!(
            "UPDATE app_users SET auth_epoch = 7 WHERE id = {}",
            user_id
        ))
        .await
        .expect("advance auth epoch");

    stale
        .update_profile(
            "Fresh Profile".to_owned(),
            "stale-updated@nebula.test".to_owned(),
            None,
        )
        .await
        .expect("save profile columns");

    let updated = reload("stale-updated@nebula.test").await;
    assert_eq!(
        updated.password_hash.as_deref(),
        Some(replacement_hash.as_str())
    );
    assert_eq!(updated.auth_epoch, 7);
}

// ============================================================================
// 4. Profile: email change re-verifies; password rotation is gated
// ============================================================================
//
// Mirrors the kit's `PATCH /profile` (email change → null the stamp + re-send
// verification) and `PUT /profile/password` (gate on the current password) +
// `DELETE /profile` (delete removes the row) logic, exercised against the real
// `User` model exactly as the controllers do.

#[tokio::test]
async fn profile_email_change_reverifies_and_password_and_delete_logic() {
    let h = setup().await;
    let fake = &h.mail;

    let mut user = User::create("Edsger", "edsger@x.com", "oldpass1!")
        .await
        .expect("create user");
    mark_verified(&mut user).await;
    assert!(
        reload("edsger@x.com").await.is_email_verified(),
        "seed user starts verified"
    );

    // --- Email change: null the verification stamp, save, re-send the link.
    //     This mirrors `profile::update` exactly.
    let before_send = fake.count();
    let user = reload("edsger@x.com")
        .await
        .update_profile(
            "Edsger Dijkstra".to_owned(),
            "edsger.new@x.com".to_owned(),
            None,
        )
        .await
        .expect("save profile update");
    EmailVerification::send_link(&user, "http://nebula.test/verify-email/verify")
        .await
        .expect("re-send verification after email change");

    let updated = reload("edsger.new@x.com").await;
    assert_eq!(
        updated.name.as_deref(),
        Some("Edsger Dijkstra"),
        "name saved"
    );
    assert_eq!(updated.email, "edsger.new@x.com", "email saved");
    assert!(
        !updated.is_email_verified(),
        "changing the email nulls email_verified_at"
    );
    assert_eq!(
        fake.count(),
        before_send + 1,
        "email change re-sends a verification link"
    );
    fake.assert_sent_to("edsger.new@x.com");

    // --- Password rotation is gated on the current password. The controller
    //     rejects a wrong current password with a 422; the gate is
    //     `verify_password(current)`. Prove both arms of that gate.
    let user = reload("edsger.new@x.com").await;
    assert!(
        !user
            .verify_password("not-the-password")
            .expect("verify wrong current"),
        "a wrong current password must NOT verify (controller → 422)"
    );
    assert!(
        user.verify_password("oldpass1!")
            .expect("verify right current"),
        "the correct current password verifies (controller → rotate)"
    );

    // Rotate: hash + save the new password, exactly as `profile::update_password`.
    let new_hash = suprnova::hashing::hash("brandnew1!").expect("hash");
    user.update_password_hash(new_hash)
        .await
        .expect("save rotated password");
    let after = reload("edsger.new@x.com").await;
    assert!(
        after.verify_password("brandnew1!").expect("verify rotated"),
        "the rotated password verifies"
    );
    assert!(
        !after.verify_password("oldpass1!").expect("verify old"),
        "the old password no longer verifies"
    );

    // --- Delete removes the row (mirrors `profile::destroy` after the password
    //     gate passes). Wrong-password gate proven above via verify_password.
    suprnova::eloquent::Model::delete(after)
        .await
        .expect("delete user");
    assert!(
        User::find_by_email("edsger.new@x.com")
            .await
            .expect("lookup")
            .is_none(),
        "a confirmed delete removes the user row"
    );
}
