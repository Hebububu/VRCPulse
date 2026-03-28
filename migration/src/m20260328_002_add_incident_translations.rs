use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IncidentTranslations::Table)
                    .if_not_exists()
                    .col(pk_auto(IncidentTranslations::Id))
                    .col(string(IncidentTranslations::IncidentId))
                    .col(string(IncidentTranslations::Locale))
                    .col(text(IncidentTranslations::TranslatedTitle))
                    .col(text(IncidentTranslations::TranslatedUpdates))
                    .col(timestamp(IncidentTranslations::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_incident_translations_incident_locale")
                    .table(IncidentTranslations::Table)
                    .col(IncidentTranslations::IncidentId)
                    .col(IncidentTranslations::Locale)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IncidentTranslations::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum IncidentTranslations {
    Table,
    Id,
    IncidentId,
    Locale,
    TranslatedTitle,
    TranslatedUpdates,
    CreatedAt,
}
