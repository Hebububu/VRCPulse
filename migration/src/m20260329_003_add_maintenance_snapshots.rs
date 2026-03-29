use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MaintenanceSnapshots::Table)
                    .if_not_exists()
                    .col(pk_auto(MaintenanceSnapshots::Id))
                    .col(string(MaintenanceSnapshots::MaintenanceId))
                    .col(string(MaintenanceSnapshots::Title))
                    .col(string(MaintenanceSnapshots::Status))
                    .col(timestamp(MaintenanceSnapshots::ScheduledFor))
                    .col(timestamp(MaintenanceSnapshots::ScheduledUntil))
                    .col(text(MaintenanceSnapshots::RawJson))
                    .col(timestamp(MaintenanceSnapshots::FetchedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_maintenance_snapshots_maintenance_fetched")
                    .table(MaintenanceSnapshots::Table)
                    .col(MaintenanceSnapshots::MaintenanceId)
                    .col(MaintenanceSnapshots::FetchedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MaintenanceSnapshots::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum MaintenanceSnapshots {
    Table,
    Id,
    MaintenanceId,
    Title,
    Status,
    ScheduledFor,
    ScheduledUntil,
    RawJson,
    FetchedAt,
}
