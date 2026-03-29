use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AiInsights::Table)
                    .add_column(
                        ColumnDef::new(AiInsights::Language)
                            .string()
                            .not_null()
                            .default("ko"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AiInsights::Table)
                    .add_column(
                        ColumnDef::new(AiInsights::CycleId)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_insights_cycle_id")
                    .table(AiInsights::Table)
                    .col(AiInsights::CycleId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_ai_insights_cycle_id")
                    .table(AiInsights::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AiInsights::Table)
                    .drop_column(AiInsights::Language)
                    .drop_column(AiInsights::CycleId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AiInsights {
    Table,
    Language,
    CycleId,
}
