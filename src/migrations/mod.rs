pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users_table;
mod m20240101_000002_create_sessions_table;
mod m20240101_000003_create_remember_tokens_table;
mod m20240101_000004_add_email_verified_at_to_users;
mod m20240101_000005_create_auth_flow_tokens;

mod m20261023_000001_create_notes_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations: Vec<Box<dyn MigrationTrait>> = vec![
            Box::new(m20240101_000001_create_users_table::Migration),
            Box::new(m20240101_000002_create_sessions_table::Migration),
            Box::new(m20240101_000003_create_remember_tokens_table::Migration),
            Box::new(m20240101_000004_add_email_verified_at_to_users::Migration),
            Box::new(m20240101_000005_create_auth_flow_tokens::Migration),
            // Suprnova 4's database session driver reads and writes Laravel
            // 13's `sessions` layout. This framework migration moves the
            // table `m20240101_000002` created into that layout, keeping
            // every session; it does nothing to a table already in it.
            Box::new(suprnova::session::migrations::CreateSessionsTable::new(
                suprnova::session::migrations::SessionUserKey::Integer,
            )),
            Box::new(suprnova::auth::migrations::UniqueEmailIgnoringCase::on(
                "app_users",
                "email",
            )),
            Box::new(m20261023_000001_create_notes_table::Migration),
            Box::new(suprnova::render_cache::migration::Migration),
        ];
        migrations.extend(suprnova::search::migrations());
        migrations.extend(suprnova::recommendations::migrations());
        migrations.extend(suprnova::agents::migrations());
        migrations
    }
}
