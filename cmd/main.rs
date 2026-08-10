//! Suprnova Application Entry Point

use suprnova::Application;

use nebula::{bootstrap, config, migrations, routes};

// `#[suprnova::main]`, not `#[tokio::main]`: the framework loads `.env`
// while the process is still single-threaded, then builds the runtime.
// `#[tokio::main]` wraps the runtime around the whole of `main`, so its
// worker threads already exist when the environment is written - an
// unsound `set_var` race. Since v0.8.0 `Application::run` refuses to
// start at all unless the environment was loaded pre-runtime, so this
// is a hard requirement, not a style preference.
#[suprnova::main]
async fn main() {
    Application::new()
        .config(config::register_all)
        .bootstrap(bootstrap::register)
        .routes(routes::register)
        .migrations::<migrations::Migrator>()
        .run()
        .await;
}
