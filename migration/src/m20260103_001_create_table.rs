use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Guild Configs
        manager
            .create_table(
                Table::create()
                    .table(GuildConfigs::Table)
                    .if_not_exists()
                    .col(string(GuildConfigs::GuildId).primary_key())
                    .col(string_null(GuildConfigs::ChannelId))
                    .col(integer(GuildConfigs::ReportInterval).default(60))
                    .col(integer(GuildConfigs::Threshold).default(5))
                    .col(boolean(GuildConfigs::Enabled).default(true))
                    .col(timestamp(GuildConfigs::CreatedAt))
                    .col(timestamp(GuildConfigs::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 2. User Reports
        manager
            .create_table(
                Table::create()
                    .table(UserReports::Table)
                    .if_not_exists()
                    .col(pk_auto(UserReports::Id))
                    .col(string(UserReports::GuildId))
                    .col(string(UserReports::UserId))
                    .col(string(UserReports::IncidentType))
                    .col(text_null(UserReports::Content))
                    .col(string(UserReports::Status).default("pending"))
                    .col(timestamp(UserReports::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserReports::Table, UserReports::GuildId)
                            .to(GuildConfigs::Table, GuildConfigs::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index: user_reports(guild_id, created_at)
        manager
            .create_index(
                Index::create()
                    .name("idx_user_reports_guild_created")
                    .table(UserReports::Table)
                    .col(UserReports::GuildId)
                    .col(UserReports::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // 3. Status Logs
        manager
            .create_table(
                Table::create()
                    .table(StatusLogs::Table)
                    .if_not_exists()
                    .col(pk_auto(StatusLogs::Id))
                    .col(string(StatusLogs::Indicator))
                    .col(text(StatusLogs::Description))
                    .col(timestamp(StatusLogs::SourceTimestamp).unique_key())
                    .col(timestamp(StatusLogs::CreatedAt))
                    .to_owned(),
            )
            .await?;

        // 4. Component Logs
        manager
            .create_table(
                Table::create()
                    .table(ComponentLogs::Table)
                    .if_not_exists()
                    .col(pk_auto(ComponentLogs::Id))
                    .col(string(ComponentLogs::ComponentId))
                    .col(string(ComponentLogs::Name))
                    .col(string(ComponentLogs::Status))
                    .col(timestamp(ComponentLogs::SourceTimestamp))
                    .col(timestamp(ComponentLogs::CreatedAt))
                    .to_owned(),
            )
            .await?;

        // Index: component_logs(component_id, source_timestamp)
        manager
            .create_index(
                Index::create()
                    .name("idx_component_logs_component_time")
                    .table(ComponentLogs::Table)
                    .col(ComponentLogs::ComponentId)
                    .col(ComponentLogs::SourceTimestamp)
                    .to_owned(),
            )
            .await?;

        // 5. Incidents
        manager
            .create_table(
                Table::create()
                    .table(Incidents::Table)
                    .if_not_exists()
                    .col(string(Incidents::Id).primary_key())
                    .col(string(Incidents::Title))
                    .col(string(Incidents::Impact))
                    .col(string(Incidents::Status))
                    .col(timestamp(Incidents::StartedAt))
                    .col(timestamp_null(Incidents::ResolvedAt))
                    .col(timestamp(Incidents::CreatedAt))
                    .col(timestamp(Incidents::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 6. Incident Updates
        manager
            .create_table(
                Table::create()
                    .table(IncidentUpdates::Table)
                    .if_not_exists()
                    .col(string(IncidentUpdates::Id).primary_key())
                    .col(string(IncidentUpdates::IncidentId))
                    .col(text(IncidentUpdates::Body))
                    .col(string(IncidentUpdates::Status))
                    .col(timestamp(IncidentUpdates::PublishedAt))
                    .col(timestamp(IncidentUpdates::CreatedAt))
                    .col(timestamp(IncidentUpdates::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(IncidentUpdates::Table, IncidentUpdates::IncidentId)
                            .to(Incidents::Table, Incidents::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 7. Maintenances
        manager
            .create_table(
                Table::create()
                    .table(Maintenances::Table)
                    .if_not_exists()
                    .col(string(Maintenances::Id).primary_key())
                    .col(string(Maintenances::Title))
                    .col(string(Maintenances::Status))
                    .col(timestamp(Maintenances::ScheduledFor))
                    .col(timestamp(Maintenances::ScheduledUntil))
                    .col(timestamp(Maintenances::CreatedAt))
                    .col(timestamp(Maintenances::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 8. Metric Logs
        manager
            .create_table(
                Table::create()
                    .table(MetricLogs::Table)
                    .if_not_exists()
                    .col(pk_auto(MetricLogs::Id))
                    .col(string(MetricLogs::MetricName))
                    .col(double(MetricLogs::Value))
                    .col(string(MetricLogs::Unit))
                    .col(integer(MetricLogs::IntervalSec))
                    .col(timestamp(MetricLogs::Timestamp))
                    .col(timestamp(MetricLogs::CreatedAt))
                    .to_owned(),
            )
            .await?;

        // Unique index: metric_logs(metric_name, timestamp)
        manager
            .create_index(
                Index::create()
                    .name("idx_metric_logs_name_time")
                    .table(MetricLogs::Table)
                    .col(MetricLogs::MetricName)
                    .col(MetricLogs::Timestamp)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 9. Sent Alerts
        manager
            .create_table(
                Table::create()
                    .table(SentAlerts::Table)
                    .if_not_exists()
                    .col(pk_auto(SentAlerts::Id))
                    .col(string(SentAlerts::GuildId))
                    .col(string(SentAlerts::AlertType))
                    .col(string(SentAlerts::ReferenceId))
                    .col(timestamp(SentAlerts::NotifiedAt))
                    .col(timestamp(SentAlerts::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(SentAlerts::Table, SentAlerts::GuildId)
                            .to(GuildConfigs::Table, GuildConfigs::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique index: sent_alerts(guild_id, alert_type, reference_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_sent_alerts_lookup")
                    .table(SentAlerts::Table)
                    .col(SentAlerts::GuildId)
                    .col(SentAlerts::AlertType)
                    .col(SentAlerts::ReferenceId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 10. Bot Config
        manager
            .create_table(
                Table::create()
                    .table(BotConfig::Table)
                    .if_not_exists()
                    .col(string(BotConfig::Key).primary_key())
                    .col(string(BotConfig::Value))
                    .col(timestamp(BotConfig::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop in reverse order due to foreign key constraints
        manager
            .drop_table(Table::drop().table(BotConfig::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SentAlerts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MetricLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Maintenances::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(IncidentUpdates::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Incidents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ComponentLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(StatusLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserReports::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GuildConfigs::Table).to_owned())
            .await?;

        Ok(())
    }
}

// =============================================================================
// Table Identifiers
// =============================================================================

#[derive(DeriveIden)]
enum GuildConfigs {
    Table,
    GuildId,
    ChannelId,
    ReportInterval,
    Threshold,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserReports {
    Table,
    Id,
    GuildId,
    UserId,
    IncidentType,
    Content,
    Status,
    CreatedAt,
}

#[derive(DeriveIden)]
enum StatusLogs {
    Table,
    Id,
    Indicator,
    Description,
    SourceTimestamp,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ComponentLogs {
    Table,
    Id,
    ComponentId,
    Name,
    Status,
    SourceTimestamp,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Incidents {
    Table,
    Id,
    Title,
    Impact,
    Status,
    StartedAt,
    ResolvedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum IncidentUpdates {
    Table,
    Id,
    IncidentId,
    Body,
    Status,
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Maintenances {
    Table,
    Id,
    Title,
    Status,
    ScheduledFor,
    ScheduledUntil,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum MetricLogs {
    Table,
    Id,
    MetricName,
    Value,
    Unit,
    IntervalSec,
    Timestamp,
    CreatedAt,
}

#[derive(DeriveIden)]
enum SentAlerts {
    Table,
    Id,
    GuildId,
    AlertType,
    ReferenceId,
    NotifiedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum BotConfig {
    Table,
    Key,
    Value,
    UpdatedAt,
}
