//! Threshold-based alert system
//!
//! Monitors user reports and sends alerts when the count exceeds the configured threshold.

use chrono::{Duration, Utc};
use rust_i18n::t;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serenity::all::{ChannelId, Colour, Context, CreateEmbed, CreateEmbedFooter, CreateMessage};
use tracing::{error, info, warn};

use crate::discord::shared::{colors, incident_types};
use crate::entity::{bot_config, guild_configs, sent_alerts, user_configs, user_reports};
use crate::i18n::{resolve_guild_locale_by_id, resolve_user_locale_by_id};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of recent report timestamps to show in alert
const MAX_RECENT_REPORTS: u64 = 5;

// =============================================================================
// Public API
// =============================================================================

/// Check if threshold is reached and send alerts to all registered recipients
///
/// Called after a new report is inserted. Checks the global report count
/// for the given incident type and sends alerts if threshold is exceeded.
pub async fn check_and_send_alerts(ctx: &Context, db: &DatabaseConnection, incident_type: &str) {
    // Get config values (required - seeded in migration)
    let Some(threshold) = get_config_value(db, "report_threshold").await else {
        error!("Missing required config: report_threshold");
        return;
    };
    let Some(interval) = get_config_value(db, "report_interval").await else {
        error!("Missing required config: report_interval");
        return;
    };

    // Count active reports for this incident type within the interval
    let count = count_active_reports(db, incident_type, interval).await;

    info!(
        incident_type = incident_type,
        count = count,
        threshold = threshold,
        "Checking alert threshold"
    );

    if count < threshold {
        return;
    }

    // Threshold reached - get recent report timestamps for the alert message
    let recent_reports = get_recent_reports(db, incident_type, interval, MAX_RECENT_REPORTS).await;

    // Generate reference ID for deduplication (15-minute blocks)
    let reference_id = generate_reference_id(incident_type);

    // Get all registered guilds
    let guilds = get_registered_guilds(db).await;
    for guild in guilds {
        send_guild_alert(
            ctx,
            db,
            &guild,
            incident_type,
            count,
            interval,
            &recent_reports,
            &reference_id,
        )
        .await;
    }

    // Get all registered users (for DM alerts)
    let users = get_registered_users(db).await;
    for user in users {
        send_user_alert(
            ctx,
            db,
            &user,
            incident_type,
            count,
            interval,
            &recent_reports,
            &reference_id,
        )
        .await;
    }
}

// =============================================================================
// Database Queries
// =============================================================================

async fn get_config_value(db: &DatabaseConnection, key: &str) -> Option<i64> {
    bot_config::Entity::find_by_id(key)
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|c| c.value.parse().ok())
}

/// Count unique users who reported this incident type within the interval
async fn count_active_reports(db: &DatabaseConnection, incident_type: &str, interval: i64) -> i64 {
    use sea_orm::{QuerySelect, sea_query::Expr};

    let cutoff = Utc::now() - Duration::minutes(interval);

    // Count distinct users, not total reports
    let result = user_reports::Entity::find()
        .filter(user_reports::Column::IncidentType.eq(incident_type))
        .filter(user_reports::Column::Status.eq("active"))
        .filter(user_reports::Column::CreatedAt.gt(cutoff))
        .select_only()
        .column_as(
            Expr::col(user_reports::Column::UserId).count_distinct(),
            "count",
        )
        .into_tuple::<i64>()
        .one(db)
        .await;

    result.ok().flatten().unwrap_or(0)
}

async fn get_recent_reports(
    db: &DatabaseConnection,
    incident_type: &str,
    interval: i64,
    limit: u64,
) -> Vec<chrono::DateTime<Utc>> {
    use sea_orm::QuerySelect;

    let cutoff = Utc::now() - Duration::minutes(interval);

    let reports = user_reports::Entity::find()
        .filter(user_reports::Column::IncidentType.eq(incident_type))
        .filter(user_reports::Column::Status.eq("active"))
        .filter(user_reports::Column::CreatedAt.gt(cutoff))
        .order_by_desc(user_reports::Column::CreatedAt)
        .limit(limit)
        .all(db)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to fetch recent reports");
            vec![]
        });

    reports.into_iter().map(|r| r.created_at).collect()
}

async fn get_registered_guilds(db: &DatabaseConnection) -> Vec<guild_configs::Model> {
    guild_configs::Entity::find()
        .filter(guild_configs::Column::Enabled.eq(true))
        .filter(guild_configs::Column::ChannelId.is_not_null())
        .all(db)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to fetch registered guilds for alerts");
            vec![]
        })
}

async fn get_registered_users(db: &DatabaseConnection) -> Vec<user_configs::Model> {
    user_configs::Entity::find()
        .filter(user_configs::Column::Enabled.eq(true))
        .all(db)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to fetch registered users for alerts");
            vec![]
        })
}

/// Result of attempting to record a sent alert
enum RecordAlertResult {
    /// Alert was recorded, contains the record ID for potential rollback
    Recorded(i64),
    /// Alert was already sent (duplicate)
    AlreadySent,
    /// Database error occurred
    Error,
}

/// Try to record a sent alert. Returns the record ID if successful, or indicates duplicate/error.
/// Uses INSERT with unique constraint to prevent race conditions (TOCTOU).
async fn try_record_sent_alert(
    db: &DatabaseConnection,
    guild_id: Option<String>,
    user_id: Option<String>,
    reference_id: &str,
) -> RecordAlertResult {
    let now = Utc::now();
    let alert = sent_alerts::ActiveModel {
        guild_id: Set(guild_id),
        user_id: Set(user_id),
        alert_type: Set("threshold".to_string()),
        reference_id: Set(reference_id.to_string()),
        notified_at: Set(now),
        created_at: Set(now),
        ..Default::default()
    };

    match alert.insert(db).await {
        Ok(record) => RecordAlertResult::Recorded(record.id), // Successfully inserted
        Err(e) => {
            // Check if it's a unique constraint violation (already sent)
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("unique") || err_str.contains("duplicate") {
                RecordAlertResult::AlreadySent // Dedup working correctly
            } else {
                error!(error = %e, "Failed to record sent alert");
                RecordAlertResult::Error // Don't send alert if we can't record it
            }
        }
    }
}

/// Delete a sent alert record (used for rollback on send failure)
async fn delete_sent_alert(db: &DatabaseConnection, record_id: i64) {
    if let Err(e) = sent_alerts::Entity::delete_by_id(record_id).exec(db).await {
        error!(record_id = record_id, error = %e, "Failed to delete sent_alert record for retry");
    }
}

// =============================================================================
// Alert Sending
// =============================================================================

#[allow(clippy::too_many_arguments)]
async fn send_guild_alert(
    ctx: &Context,
    db: &DatabaseConnection,
    guild: &guild_configs::Model,
    incident_type: &str,
    count: i64,
    interval: i64,
    recent_reports: &[chrono::DateTime<Utc>],
    reference_id: &str,
) {
    // Get channel ID
    let Some(channel_id_str) = &guild.channel_id else {
        return;
    };

    let Ok(channel_id) = channel_id_str.parse::<u64>() else {
        warn!(guild_id = %guild.guild_id, "Invalid channel ID");
        return;
    };

    // Try to record first (atomic deduplication via unique constraint)
    // If this fails due to duplicate, we skip sending
    let record_id =
        match try_record_sent_alert(db, Some(guild.guild_id.clone()), None, reference_id).await {
            RecordAlertResult::Recorded(id) => id,
            RecordAlertResult::AlreadySent => return, // Already sent - skip
            RecordAlertResult::Error => return,       // Can't record - don't send
        };

    // Resolve locale for this guild
    let locale = resolve_guild_locale_by_id(db, &guild.guild_id).await;

    // Build and send embed
    let embed = build_alert_embed(incident_type, count, interval, recent_reports, &locale);
    let message = CreateMessage::new().embed(embed);

    let channel = ChannelId::new(channel_id);
    match channel.send_message(&ctx.http, message).await {
        Ok(_) => {
            info!(
                guild_id = %guild.guild_id,
                incident_type = incident_type,
                count = count,
                "Sent threshold alert to guild"
            );
        }
        Err(e) => {
            error!(
                guild_id = %guild.guild_id,
                error = %e,
                "Failed to send alert to guild channel, will retry on next trigger"
            );
            // Delete the record so we can retry on the next report
            delete_sent_alert(db, record_id).await;
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn send_user_alert(
    ctx: &Context,
    db: &DatabaseConnection,
    user: &user_configs::Model,
    incident_type: &str,
    count: i64,
    interval: i64,
    recent_reports: &[chrono::DateTime<Utc>],
    reference_id: &str,
) {
    // Parse user ID
    let Ok(user_id) = user.user_id.parse::<u64>() else {
        warn!(user_id = %user.user_id, "Invalid user ID");
        return;
    };

    // Try to record first (atomic deduplication via unique constraint)
    // If this fails due to duplicate, we skip sending
    let record_id =
        match try_record_sent_alert(db, None, Some(user.user_id.clone()), reference_id).await {
            RecordAlertResult::Recorded(id) => id,
            RecordAlertResult::AlreadySent => return, // Already sent - skip
            RecordAlertResult::Error => return,       // Can't record - don't send
        };

    // Get user and create DM channel
    let user_obj = match serenity::all::UserId::new(user_id).to_user(&ctx.http).await {
        Ok(u) => u,
        Err(e) => {
            error!(user_id = %user.user_id, error = %e, "Failed to get user, will retry on next trigger");
            delete_sent_alert(db, record_id).await;
            return;
        }
    };

    let dm_channel = match user_obj.create_dm_channel(&ctx.http).await {
        Ok(c) => c,
        Err(e) => {
            error!(user_id = %user.user_id, error = %e, "Failed to create DM channel, will retry on next trigger");
            delete_sent_alert(db, record_id).await;
            return;
        }
    };

    // Resolve locale for this user
    let locale = resolve_user_locale_by_id(db, &user.user_id).await;

    // Build and send embed
    let embed = build_alert_embed(incident_type, count, interval, recent_reports, &locale);
    let message = CreateMessage::new().embed(embed);

    match dm_channel.send_message(&ctx.http, message).await {
        Ok(_) => {
            info!(
                user_id = %user.user_id,
                incident_type = incident_type,
                count = count,
                "Sent threshold alert to user DM"
            );
        }
        Err(e) => {
            error!(
                user_id = %user.user_id,
                error = %e,
                "Failed to send alert to user DM, will retry on next trigger"
            );
            // Delete the record so we can retry on the next report
            delete_sent_alert(db, record_id).await;
        }
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn generate_reference_id(incident_type: &str) -> String {
    let now = Utc::now();
    // Round down to 15-minute block
    let minutes = now.format("%M").to_string().parse::<i32>().unwrap_or(0);
    let block = (minutes / 15) * 15;
    let timestamp = now.format("%Y-%m-%dT%H").to_string();
    format!("threshold_{}_{timestamp}:{block:02}", incident_type)
}

fn build_alert_embed(
    incident_type: &str,
    count: i64,
    interval: i64,
    recent_reports: &[chrono::DateTime<Utc>],
    locale: &str,
) -> CreateEmbed {
    let display_name = incident_types::display_name_localized(incident_type, locale);
    let now = Utc::now();

    // Format recent reports as relative timestamps
    let recent_text = if recent_reports.is_empty() {
        t!("embeds.alerts.threshold.no_recent_reports", locale = locale).to_string()
    } else {
        recent_reports
            .iter()
            .map(|ts| {
                let diff = now.signed_duration_since(*ts);
                let mins = diff.num_minutes();
                if mins < 1 {
                    format!("- {}", t!("time.just_now", locale = locale))
                } else if mins == 1 {
                    format!("- {}", t!("time.min_ago_one", locale = locale))
                } else {
                    format!("- {}", t!("time.min_ago_many", n = mins, locale = locale))
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let title = t!("embeds.alerts.threshold.title", locale = locale);
    let description = t!(
        "embeds.alerts.threshold.description",
        count = count,
        incident_type = display_name,
        interval = interval,
        locale = locale
    );
    let field_name = t!(
        "embeds.alerts.threshold.field_recent_reports",
        locale = locale
    );
    let footer = t!("embeds.alerts.threshold.footer", locale = locale);

    CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::new(colors::MAJOR))
        .field(field_name, recent_text, false)
        .footer(CreateEmbedFooter::new(footer))
        .timestamp(serenity::all::Timestamp::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::MigratorTrait;
    use sea_orm::Database;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory SQLite");
        migration::Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");
        db
    }

    async fn insert_report(
        db: &DatabaseConnection,
        user_id: &str,
        incident_type: &str,
        created_at: chrono::DateTime<Utc>,
    ) {
        let report = user_reports::ActiveModel {
            guild_id: Set(None),
            user_id: Set(user_id.to_string()),
            incident_type: Set(incident_type.to_string()),
            content: Set(None),
            status: Set("active".to_string()),
            created_at: Set(created_at),
            ..Default::default()
        };
        report.insert(db).await.unwrap();
    }

    async fn insert_guild_config(db: &DatabaseConnection, guild_id: &str, channel_id: &str) {
        let now = Utc::now();
        let model = guild_configs::ActiveModel {
            guild_id: Set(guild_id.to_string()),
            channel_id: Set(Some(channel_id.to_string())),
            enabled: Set(true),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(db).await.unwrap();
    }

    async fn insert_user_config(db: &DatabaseConnection, user_id: &str) {
        let now = Utc::now();
        let model = user_configs::ActiveModel {
            user_id: Set(user_id.to_string()),
            enabled: Set(true),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(db).await.unwrap();
    }

    // =========================================================================
    // Config Values
    // =========================================================================

    #[tokio::test]
    async fn config_values_from_migration() {
        let db = setup_test_db().await;

        let threshold = get_config_value(&db, "report_threshold").await;
        assert_eq!(threshold, Some(1));

        let interval = get_config_value(&db, "report_interval").await;
        assert_eq!(interval, Some(60));
    }

    #[tokio::test]
    async fn config_value_missing_returns_none() {
        let db = setup_test_db().await;

        let result = get_config_value(&db, "nonexistent_key").await;
        assert!(result.is_none());
    }

    // =========================================================================
    // Active Report Counting
    // =========================================================================

    #[tokio::test]
    async fn count_active_reports_distinct_users() {
        let db = setup_test_db().await;
        let now = Utc::now();

        insert_report(&db, "100", "login", now).await;
        insert_report(&db, "200", "login", now).await;
        insert_report(&db, "300", "login", now).await;

        let count = count_active_reports(&db, "login", 60).await;
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn count_active_reports_same_user_once() {
        let db = setup_test_db().await;
        let now = Utc::now();

        // Same user, two reports -- should count as 1 distinct user
        insert_report(&db, "100", "login", now).await;
        insert_report(&db, "100", "login", now - Duration::minutes(1)).await;

        let count = count_active_reports(&db, "login", 60).await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn count_active_reports_respects_interval() {
        let db = setup_test_db().await;
        let now = Utc::now();

        insert_report(&db, "100", "login", now).await;
        // Old report outside the 60-min interval
        insert_report(&db, "200", "login", now - Duration::minutes(90)).await;

        let count = count_active_reports(&db, "login", 60).await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn count_active_reports_respects_type() {
        let db = setup_test_db().await;
        let now = Utc::now();

        insert_report(&db, "100", "login", now).await;
        insert_report(&db, "200", "api", now).await;

        assert_eq!(count_active_reports(&db, "login", 60).await, 1);
        assert_eq!(count_active_reports(&db, "api", 60).await, 1);
    }

    #[tokio::test]
    async fn count_active_reports_empty() {
        let db = setup_test_db().await;

        let count = count_active_reports(&db, "login", 60).await;
        assert_eq!(count, 0);
    }

    // =========================================================================
    // Recent Reports
    // =========================================================================

    #[tokio::test]
    async fn recent_reports_returns_timestamps() {
        let db = setup_test_db().await;
        let now = Utc::now();

        insert_report(&db, "100", "login", now - Duration::minutes(1)).await;
        insert_report(&db, "200", "login", now - Duration::minutes(2)).await;
        insert_report(&db, "300", "login", now - Duration::minutes(3)).await;

        let recent = get_recent_reports(&db, "login", 60, 5).await;
        assert_eq!(recent.len(), 3);
        // Should be in descending order (most recent first)
        assert!(recent[0] > recent[1]);
        assert!(recent[1] > recent[2]);
    }

    #[tokio::test]
    async fn recent_reports_respects_limit() {
        let db = setup_test_db().await;
        let now = Utc::now();

        for i in 0..10 {
            insert_report(&db, &format!("{}", i), "login", now - Duration::minutes(i)).await;
        }

        let recent = get_recent_reports(&db, "login", 60, 5).await;
        assert_eq!(recent.len(), 5);
    }

    // =========================================================================
    // Registered Guilds / Users
    // =========================================================================

    #[tokio::test]
    async fn get_registered_guilds_only_enabled_with_channel() {
        let db = setup_test_db().await;

        insert_guild_config(&db, "111", "999").await;
        insert_guild_config(&db, "222", "888").await;

        // Insert disabled guild
        let now = Utc::now();
        let model = guild_configs::ActiveModel {
            guild_id: Set("333".to_string()),
            channel_id: Set(Some("777".to_string())),
            enabled: Set(false),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&db).await.unwrap();

        // Insert guild without channel
        let model = guild_configs::ActiveModel {
            guild_id: Set("444".to_string()),
            channel_id: Set(None),
            enabled: Set(true),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&db).await.unwrap();

        let guilds = get_registered_guilds(&db).await;
        assert_eq!(guilds.len(), 2);
    }

    #[tokio::test]
    async fn get_registered_users_only_enabled() {
        let db = setup_test_db().await;

        insert_user_config(&db, "100").await;
        insert_user_config(&db, "200").await;

        // Insert disabled user
        let now = Utc::now();
        let model = user_configs::ActiveModel {
            user_id: Set("300".to_string()),
            enabled: Set(false),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&db).await.unwrap();

        let users = get_registered_users(&db).await;
        assert_eq!(users.len(), 2);
    }

    // =========================================================================
    // Alert Deduplication
    // =========================================================================

    #[tokio::test]
    async fn record_sent_alert_success() {
        let db = setup_test_db().await;

        let result = try_record_sent_alert(&db, Some("111".to_string()), None, "ref_001").await;
        assert!(matches!(result, RecordAlertResult::Recorded(_)));
    }

    #[tokio::test]
    async fn record_sent_alert_duplicate_blocked() {
        let db = setup_test_db().await;

        // Use explicit user_id to avoid SQLite NULL != NULL in unique index
        let result = try_record_sent_alert(
            &db,
            Some("111".to_string()),
            Some("user1".to_string()),
            "ref_001",
        )
        .await;
        assert!(matches!(result, RecordAlertResult::Recorded(_)));

        // Same guild + user + reference_id should be blocked
        let result = try_record_sent_alert(
            &db,
            Some("111".to_string()),
            Some("user1".to_string()),
            "ref_001",
        )
        .await;
        assert!(matches!(result, RecordAlertResult::AlreadySent));
    }

    #[tokio::test]
    async fn record_sent_alert_different_guild_allowed() {
        let db = setup_test_db().await;

        let result = try_record_sent_alert(&db, Some("111".to_string()), None, "ref_001").await;
        assert!(matches!(result, RecordAlertResult::Recorded(_)));

        // Different guild, same reference -- should succeed
        let result = try_record_sent_alert(&db, Some("222".to_string()), None, "ref_001").await;
        assert!(matches!(result, RecordAlertResult::Recorded(_)));
    }

    #[tokio::test]
    async fn record_sent_alert_different_reference_allowed() {
        let db = setup_test_db().await;

        let result = try_record_sent_alert(&db, Some("111".to_string()), None, "ref_001").await;
        assert!(matches!(result, RecordAlertResult::Recorded(_)));

        // Same guild, different reference -- should succeed (new time window)
        let result = try_record_sent_alert(&db, Some("111".to_string()), None, "ref_002").await;
        assert!(matches!(result, RecordAlertResult::Recorded(_)));
    }

    #[tokio::test]
    async fn delete_sent_alert_rollback() {
        let db = setup_test_db().await;

        let record_id =
            match try_record_sent_alert(&db, Some("111".to_string()), None, "ref_001").await {
                RecordAlertResult::Recorded(id) => id,
                _ => panic!("Expected Recorded"),
            };

        // Delete it (simulating send failure rollback)
        delete_sent_alert(&db, record_id).await;

        // Should be able to record again (retry on next trigger)
        let result = try_record_sent_alert(&db, Some("111".to_string()), None, "ref_001").await;
        assert!(matches!(result, RecordAlertResult::Recorded(_)));
    }

    // =========================================================================
    // Reference ID Generation
    // =========================================================================

    #[tokio::test]
    async fn reference_id_format() {
        let ref_id = generate_reference_id("login");
        assert!(ref_id.starts_with("threshold_login_"));
        // Should contain date-time and block
        assert!(ref_id.contains('T'));
        assert!(ref_id.contains(':'));
    }

    #[tokio::test]
    async fn reference_id_same_within_15min_block() {
        // Two calls in the same instant should produce the same reference
        let ref1 = generate_reference_id("login");
        let ref2 = generate_reference_id("login");
        assert_eq!(ref1, ref2);
    }

    #[tokio::test]
    async fn reference_id_different_types() {
        let ref1 = generate_reference_id("login");
        let ref2 = generate_reference_id("api");
        assert_ne!(ref1, ref2);
    }
}
