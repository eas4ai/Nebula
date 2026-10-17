//! Shared support for the Nebula integration suites.
//!
//! Lives in `tests/common/` (a subdirectory, so Cargo does not compile it as
//! a test binary of its own); each suite pulls it in with `mod common;`.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use sea_orm_migration::MigratorTrait;
use tokio::sync::{Mutex, OnceCell};

use nebula::migrations::Migrator;

/// Serializes tests that touch process-global state: `Mail::fake()` swaps the
/// process-global mail transport, and the DB / auth-manager bindings live in
/// the process-global `App` container (spawned server tasks resolve from
/// there). Any test that installs container bindings or fakes the mailer must
/// hold this lock for its full duration - concurrent holders would observe
/// each other's transports and connections.
pub static TEST_LOCK: Mutex<()> = Mutex::const_new(());

static TEST_DATABASE: OnceCell<DatabaseConnection> = OnceCell::const_new();
static MAGNETAR: OnceCell<()> = OnceCell::const_new();

async fn clear_test_rows(connection: &DatabaseConnection) {
    let tables = connection
        .query_all_raw(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' \
             AND name NOT LIKE 'sqlite_%' \
             AND name <> 'seaql_migrations'",
        ))
        .await
        .expect("list SQLite test tables");
    let triggers = connection
        .query_all_raw(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type = 'trigger'",
        ))
        .await
        .expect("list SQLite test triggers");
    connection
        .execute_unprepared("PRAGMA foreign_keys = OFF")
        .await
        .expect("disable SQLite foreign keys for test reset");
    for trigger in triggers {
        let name: String = trigger.try_get("", "name").expect("SQLite trigger name");
        let quoted = name.replace('"', "\"\"");
        connection
            .execute_unprepared(&format!("DROP TRIGGER IF EXISTS \"{quoted}\""))
            .await
            .unwrap_or_else(|error| panic!("drop SQLite test trigger {name}: {error}"));
    }
    for table in tables {
        let name: String = table.try_get("", "name").expect("SQLite table name");
        let quoted = name.replace('"', "\"\"");
        connection
            .execute_unprepared(&format!("DELETE FROM \"{quoted}\""))
            .await
            .unwrap_or_else(|error| panic!("clear SQLite test table {name}: {error}"));
    }
    let has_sequence = connection
        .query_one_raw(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT 1 AS present FROM sqlite_master \
             WHERE type = 'table' AND name = 'sqlite_sequence'",
        ))
        .await
        .expect("check SQLite sequence table")
        .is_some();
    if has_sequence {
        connection
            .execute_unprepared("DELETE FROM sqlite_sequence")
            .await
            .expect("reset SQLite autoincrement sequences");
    }
    connection
        .execute_unprepared("PRAGMA foreign_keys = ON")
        .await
        .expect("restore SQLite foreign keys after test reset");
}

/// Reset the one database connection captured by the process-global Magnetar
/// engine, then rebind that connection for the next serialized test.
pub async fn fresh_magnetar_database() {
    suprnova::Crypt::init(suprnova::EncryptionKey::generate());

    let initialize = TEST_DATABASE.get().is_none();
    let connection = TEST_DATABASE
        .get_or_init(|| async {
            let path = std::env::current_dir()
                .expect("current directory")
                .join("target")
                .join(format!("nebula-auth-tests-{}.sqlite", std::process::id()));
            std::fs::create_dir_all(path.parent().expect("test database parent"))
                .expect("create test database parent");
            let _ = std::fs::remove_file(&path);
            let mut options =
                sea_orm::ConnectOptions::new(format!("sqlite://{}?mode=rwc", path.display()));
            options.min_connections(1).max_connections(1);
            sea_orm::Database::connect(options)
                .await
                .expect("connect test SQLite database")
        })
        .await
        .clone();

    if initialize {
        Migrator::up(&connection, None)
            .await
            .expect("run Nebula migrations against sqlite::memory:");
    } else {
        clear_test_rows(&connection).await;
    }
    suprnova::App::singleton(suprnova::DbConnection::from_raw(connection.clone()));

    MAGNETAR
        .get_or_init(|| async {
            let config = suprnova::MagnetarConfig::from_sea_orm(connection).passkey_config(
                suprnova::PasskeyConfig {
                    rp_id: std::env::var("PASSKEY_RP_ID")
                        .unwrap_or_else(|_| "localhost".to_string()),
                    rp_origin: std::env::var("PASSKEY_RP_ORIGIN")
                        .unwrap_or_else(|_| "http://localhost".to_string()),
                },
            );
            suprnova::init_magnetar(config)
                .await
                .expect("Failed to initialize Magnetar");
        })
        .await;

    suprnova::rate_limit::bootstrap_default().await;
}
