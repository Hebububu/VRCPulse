use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AiInsights::Table)
                    .if_not_exists()
                    .col(pk_auto(AiInsights::Id))
                    .col(string(AiInsights::Scope))
                    .col(string(AiInsights::TriggerType))
                    .col(string_null(AiInsights::TriggerId))
                    .col(timestamp(AiInsights::WindowStart))
                    .col(timestamp(AiInsights::WindowEnd))
                    .col(text(AiInsights::Headline))
                    .col(text(AiInsights::SummaryJson))
                    .col(double(AiInsights::Confidence))
                    .col(text(AiInsights::SignalsJson))
                    .col(string(AiInsights::ModelId))
                    .col(string(AiInsights::SourceHash))
                    .col(timestamp(AiInsights::CreatedAt))
                    .col(timestamp(AiInsights::ExpiresAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_insights_created")
                    .table(AiInsights::Table)
                    .col(AiInsights::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiInsights::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AiInsights {
    Table,
    Id,
    Scope,
    TriggerType,
    TriggerId,
    WindowStart,
    WindowEnd,
    Headline,
    SummaryJson,
    Confidence,
    SignalsJson,
    ModelId,
    SourceHash,
    CreatedAt,
    ExpiresAt,
}
