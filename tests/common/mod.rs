//! Shared support for the Nebula integration suites.
//!
//! Lives in `tests/common/` (a subdirectory, so Cargo does not compile it as
//! a test binary of its own); each suite pulls it in with `mod common;`.

use tokio::sync::Mutex;

/// Serializes tests that touch process-global state: `Mail::fake()` swaps the
/// process-global mail transport, and the DB / auth-manager bindings live in
/// the process-global `App` container (spawned server tasks resolve from
/// there). Any test that installs container bindings or fakes the mailer must
/// hold this lock for its full duration - concurrent holders would observe
/// each other's transports and connections.
pub static TEST_LOCK: Mutex<()> = Mutex::const_new(());
