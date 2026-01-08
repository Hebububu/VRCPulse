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
                    .col(boolean(GuildConfigs::Enabled).default(true))
                    .col(timestamp(GuildConfigs::CreatedAt))
                    .col(timestamp(GuildConfigs::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 2. User Configs (for user-install DM alerts)
        manager
            .create_table(
                Table::create()
                    .table(UserConfigs::Table)
                    .if_not_exists()
                    .col(string(UserConfigs::UserId).primary_key())
                    .col(boolean(UserConfigs::Enabled).default(true))
                    .col(timestamp(UserConfigs::CreatedAt))
                    .col(timestamp(UserConfigs::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 3. User Reports (guild_id nullable for user-install reports)
        manager
            .create_table(
                Table::create()
                    .table(UserReports::Table)
                    .if_not_exists()
                    .col(pk_auto(UserReports::Id))
                    .col(string_null(UserReports::GuildId))
                    .col(string(UserReports::UserId))
                    .col(string(UserReports::IncidentType))
                    .col(text_null(UserReports::Content))
                    .col(string(UserReports::Status).default("active"))
                    .col(timestamp(UserReports::CreatedAt))
                    .to_owned(),
            )
            .await?;

        // Index: user_reports(incident_type, created_at) for threshold queries
        manager
            .create_index(
                Index::create()
                    .name("idx_user_reports_type_created")
                    .table(UserReports::Table)
                    .col(UserReports::IncidentType)
                    .col(UserReports::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Index: user_reports(user_id, incident_type, created_at) for duplicate check
        manager
            .create_index(
                Index::create()
                    .name("idx_user_reports_user_type_created")
                    .table(UserReports::Table)
                    .col(UserReports::UserId)
                    .col(UserReports::IncidentType)
                    .col(UserReports::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // 4. Status Logs
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

        // 5. Component Logs
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

        // 6. Incidents
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

        // 7. Incident Updates
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

        // 8. Maintenances
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

        // 9. Metric Logs
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

        // 10. Sent Alerts (guild_id and user_id both nullable, one must be set)
        manager
            .create_table(
                Table::create()
                    .table(SentAlerts::Table)
                    .if_not_exists()
                    .col(pk_auto(SentAlerts::Id))
                    .col(string_null(SentAlerts::GuildId))
                    .col(string_null(SentAlerts::UserId))
                    .col(string(SentAlerts::AlertType))
                    .col(string(SentAlerts::ReferenceId))
                    .col(timestamp(SentAlerts::NotifiedAt))
                    .col(timestamp(SentAlerts::CreatedAt))
                    .to_owned(),
            )
            .await?;

        // Unique index: sent_alerts(guild_id, user_id, alert_type, reference_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_sent_alerts_lookup")
                    .table(SentAlerts::Table)
                    .col(SentAlerts::GuildId)
                    .col(SentAlerts::UserId)
                    .col(SentAlerts::AlertType)
                    .col(SentAlerts::ReferenceId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 11. Command Logs (audit trail)
        manager
            .create_table(
                Table::create()
                    .table(CommandLogs::Table)
                    .if_not_exists()
                    .col(pk_auto(CommandLogs::Id))
                    .col(string(CommandLogs::CommandName))
                    .col(string_null(CommandLogs::Subcommand))
                    .col(string(CommandLogs::UserId))
                    .col(string_null(CommandLogs::GuildId))
                    .col(string_null(CommandLogs::ChannelId))
                    .col(timestamp(CommandLogs::ExecutedAt))
                    .to_owned(),
            )
            .await?;

        // Index: command_logs(user_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_command_logs_user_id")
                    .table(CommandLogs::Table)
                    .col(CommandLogs::UserId)
                    .to_owned(),
            )
            .await?;

        // Index: command_logs(guild_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_command_logs_guild_id")
                    .table(CommandLogs::Table)
                    .col(CommandLogs::GuildId)
                    .to_owned(),
            )
            .await?;

        // 12. Bot Config
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

        // Seed default config values
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            INSERT INTO bot_config (key, value, updated_at) VALUES
                ('polling.status', '60', datetime('now')),
                ('polling.incident', '60', datetime('now')),
                ('polling.maintenance', '60', datetime('now')),
                ('polling.metrics', '60', datetime('now')),
                ('report_threshold', '1', datetime('now')),
                ('report_interval', '60', datetime('now'))
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop in reverse order
        manager
            .drop_table(Table::drop().table(BotConfig::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CommandLogs::Table).to_owned())
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
            .drop_table(Table::drop().table(UserConfigs::Table).to_owned())
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
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserConfigs {
    Table,
    UserId,
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
    UserId,
    AlertType,
    ReferenceId,
    NotifiedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum CommandLogs {
    Table,
    Id,
    CommandName,
    Subcommand,
    UserId,
    GuildId,
    ChannelId,
    ExecutedAt,
}

#[derive(DeriveIden)]
enum BotConfig {
    Table,
    Key,
    Value,
    UpdatedAt,
}
