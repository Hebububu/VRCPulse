//! Add language column to guild_configs and user_configs tables
//!
//! This migration adds i18n support by allowing guilds and users to set
//! their preferred language. NULL means "use Discord auto-detect".

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add language column to guild_configs
        manager
            .alter_table(
                Table::alter()
                    .table(GuildConfigs::Table)
                    .add_column(string_null(GuildConfigs::Language))
                    .to_owned(),
            )
            .await?;

        // Add language column to user_configs
        manager
            .alter_table(
                Table::alter()
                    .table(UserConfigs::Table)
                    .add_column(string_null(UserConfigs::Language))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove language column from user_configs
        manager
            .alter_table(
                Table::alter()
                    .table(UserConfigs::Table)
                    .drop_column(UserConfigs::Language)
                    .to_owned(),
            )
            .await?;

        // Remove language column from guild_configs
        manager
            .alter_table(
                Table::alter()
                    .table(GuildConfigs::Table)
                    .drop_column(GuildConfigs::Language)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum GuildConfigs {
    Table,
    Language,
}

#[derive(DeriveIden)]
enum UserConfigs {
    Table,
    Language,
}
