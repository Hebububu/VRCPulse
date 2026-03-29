use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop old incident_translations table
        manager
            .drop_table(
                Table::drop()
                    .table(OldIncidentTranslations::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        // Create new translations table
        manager
            .create_table(
                Table::create()
                    .table(Translations::Table)
                    .if_not_exists()
                    .col(pk_auto(Translations::Id))
                    .col(string(Translations::ItemType))
                    .col(string(Translations::ItemId))
                    .col(string(Translations::Locale))
                    .col(string(Translations::ContentHash))
                    .col(text(Translations::TranslatedName))
                    .col(text(Translations::TranslatedBody))
                    .col(text(Translations::TranslatedUpdates))
                    .col(timestamp(Translations::CreatedAt))
                    .col(timestamp(Translations::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_translations_item_locale")
                    .table(Translations::Table)
                    .col(Translations::ItemType)
                    .col(Translations::ItemId)
                    .col(Translations::Locale)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Translations::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum OldIncidentTranslations {
    #[sea_orm(iden = "incident_translations")]
    Table,
}

#[derive(DeriveIden)]
enum Translations {
    Table,
    Id,
    ItemType,
    ItemId,
    Locale,
    ContentHash,
    TranslatedName,
    TranslatedBody,
    TranslatedUpdates,
    CreatedAt,
    UpdatedAt,
}
