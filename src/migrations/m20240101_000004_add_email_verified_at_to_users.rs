use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Fresh installations receive `email_verified_at` from the canonical
        // app_users baseline. Existing installations retain this migration's
        // recorded history; `init_magnetar` performs the shape-aware,
        // data-preserving users-to-app_users upgrade at application boot.
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
