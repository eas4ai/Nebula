//! nebula console — runtime command dispatch.
//!
//! Per-project entry point for `db:seed`, your own `#[command]`s, and
//! other one-shot CLI tasks. Calls `nebula::bootstrap::register()`
//! lazily (only when a real subcommand matches), then routes argv to
//! a registered console command.
//!
//! ```text
//! cargo run --bin console -- db:seed
//! cargo run --bin console -- --version
//! cargo run --bin console -- help
//! ./target/debug/console <your-command>
//! ```
//!
//! Tokio flavor is `current_thread` — console commands are one-shot,
//! so the multi-threaded worker pool would buy nothing. Bootstrap
//! runs only when a real subcommand is matched, so `console --help`
//! and `console --version` work without DATABASE_URL set.
//!
//! `#[suprnova::main]`, not `#[tokio::main]`: loading `.env` writes the
//! process environment, which is only sound while the process is
//! single-threaded. The macro loads it before building the runtime.

use std::process::ExitCode;

#[suprnova::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    // The server binary inherits this check from `Application::run`;
    // the console never reaches that path, so make it here. Reverting
    // to `#[tokio::main]` + `dotenvy::dotenv()` would otherwise keep
    // working - unsoundly, writing the process environment after the
    // runtime's threads exist - with nothing to say so.
    if !suprnova::boot::env_loaded_pre_runtime() {
        eprintln!(
            "console: the environment was loaded after the Tokio runtime started, \
             which is an unsound write to the process environment. Put \
             #[suprnova::main(flavor = \"current_thread\")] on `fn main`, not \
             #[tokio::main]."
        );
        return ExitCode::FAILURE;
    }

    // Surface this project's package version via `--version` and
    // `--help`. `env!("CARGO_PKG_VERSION")` reflects nebula,
    // not the framework.
    suprnova::console::set_version(env!("CARGO_PKG_VERSION"));

    let argv: Vec<String> = std::env::args().collect();
    // dispatch_argv_with_init owns all user-facing stderr (both clap
    // parse errors and handler-returned errors); main is pure
    // Result → ExitCode translation. The bootstrap closure runs only
    // when clap matches a real registered subcommand — help, version,
    // and parse-error paths skip it entirely.
    let result = suprnova::console::dispatch_argv_with_init(argv, || async {
        nebula::config::register_all();
        nebula::bootstrap::register().await;
    })
    .await;

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
