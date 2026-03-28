use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IncidentSnapshots::Table)
                    .if_not_exists()
                    .col(pk_auto(IncidentSnapshots::Id))
                    .col(string(IncidentSnapshots::IncidentId))
                    .col(string(IncidentSnapshots::Title))
                    .col(string(IncidentSnapshots::Impact))
                    .col(string(IncidentSnapshots::Status))
                    .col(timestamp(IncidentSnapshots::StartedAt))
                    .col(timestamp_null(IncidentSnapshots::ResolvedAt))
                    .col(integer(IncidentSnapshots::UpdateCount))
                    .col(text(IncidentSnapshots::RawJson))
                    .col(timestamp(IncidentSnapshots::FetchedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_incident_snapshots_incident_fetched")
                    .table(IncidentSnapshots::Table)
                    .col(IncidentSnapshots::IncidentId)
                    .col(IncidentSnapshots::FetchedAt)
                    .to_owned(),
            )
            .await?;

        // Add history polling interval to bot_config
        let db = manager.get_connection();
        db.execute_unprepared(
            "INSERT OR IGNORE INTO bot_config (key, value, updated_at) VALUES ('polling.history', '300', datetime('now'))",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IncidentSnapshots::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum IncidentSnapshots {
    Table,
    Id,
    IncidentId,
    Title,
    Impact,
    Status,
    StartedAt,
    ResolvedAt,
    UpdateCount,
    RawJson,
    FetchedAt,
}
