use std::process::Command;

use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

#[tokio::test]
async fn console_initializes_encryption_only_for_real_commands() {
    let working =
        std::env::temp_dir().join(format!("nebula-console-bootstrap-{}", std::process::id()));
    std::fs::create_dir_all(&working).unwrap();
    let run = |args: &[&str], environment: &str, key: Option<&str>| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_console"));
        command
            .args(args)
            .current_dir(&working)
            .env("APP_ENV", environment)
            .env("APP_DEBUG", "false")
            .env(
                "DATABASE_URL",
                format!("sqlite://{}", working.join("test.db").display()),
            )
            .env_remove("APP_KEY")
            .env_remove("APP_KEY_PREVIOUS")
            .env_remove("APP_PREVIOUS_KEYS");
        if let Some(key) = key {
            command.env("APP_KEY", key);
        }
        command.output().unwrap()
    };

    // Informational paths must remain usable without production credentials.
    for argument in ["--help", "--version"] {
        let output = run(&[argument], "production", None);
        assert!(
            output.status.success(),
            "{argument}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(!working.join("test.db").exists());

    // A real bootstrap must reject missing and malformed keys before opening DB.
    for key in [None, Some("invalid-key")] {
        let output = run(&["db:seed"], "production", key);
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("framework encryption init failed")
        );
        assert!(!working.join("test.db").exists());
    }

    // The real console bootstrap must initialize Crypt before Magnetar.
    let output = run(
        &["db:seed"],
        "test",
        Some("AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE"),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(working.join("test.db").exists());
    // The kit migration must define defaults before Magnetar creates app_users.
    let db = Database::connect(format!("sqlite://{}", working.join("test.db").display()))
        .await
        .unwrap();
    db.execute_unprepared("INSERT INTO app_users (email) VALUES ('bootstrap@example.test')")
        .await
        .expect("fresh user schema must supply authentication and timestamp defaults");
    let row = db.query_one_raw(Statement::from_string(
        DbBackend::Sqlite,
        "SELECT auth_epoch, session_version FROM app_users WHERE email = 'bootstrap@example.test'",
    )).await.unwrap().unwrap();
    assert_eq!(row.try_get::<i64>("", "auth_epoch").unwrap(), 0);
    assert_eq!(row.try_get::<i64>("", "session_version").unwrap(), 0);
    db.close().await.unwrap();
    std::fs::remove_dir_all(working).unwrap();
}
