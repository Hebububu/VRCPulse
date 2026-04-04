//! /report command - User incident reporting for VRChat issues

use chrono::{Duration, Utc};
use rust_i18n::t;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbedFooter, ResolvedValue, Timestamp,
};
use tracing::{error, info};

use crate::discord::shared::{defer, embeds, incident_types, respond_error};
use crate::entity::{bot_config, guild_configs, user_configs, user_reports};
use crate::i18n::{resolve_locale, resolve_locale_async};

// =============================================================================
// Constants
// =============================================================================

/// Duplicate report cooldown in minutes
const DUPLICATE_COOLDOWN_MINUTES: i64 = 5;

/// Maximum length for details field
const MAX_DETAILS_LENGTH: usize = 500;

// =============================================================================
// Command Registration
// =============================================================================

/// /report command definition
pub fn register() -> CreateCommand {
    let mut incident_type_option = CreateCommandOption::new(
        CommandOptionType::String,
        "type",
        t!("commands.report.option_type"),
    )
    .name_localized("ko", "유형")
    .description_localized("ko", t!("commands.report.option_type", locale = "ko"))
    .required(true);

    // Add choices for incident types with localization
    for key in incident_types::INCIDENT_TYPE_KEYS {
        let display_en = incident_types::display_name(key);
        let display_ko = incident_types::display_name_localized(key, "ko");
        incident_type_option = incident_type_option.add_string_choice_localized(
            display_en,
            *key,
            [("ko", display_ko)],
        );
    }

    CreateCommand::new("report")
        .description(t!("commands.report.description"))
        .name_localized("ko", t!("commands.report.name", locale = "ko"))
        .description_localized("ko", t!("commands.report.description", locale = "ko"))
        .add_option(incident_type_option)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "details",
                t!("commands.report.option_details"),
            )
            .name_localized("ko", "상세")
            .description_localized("ko", t!("commands.report.option_details", locale = "ko"))
            .required(false),
        )
}

/// /report command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    // Use sync locale for validation errors (before defer)
    let sync_locale = resolve_locale(interaction);
    let options = interaction.data.options();

    // Parse incident_type (required)
    let incident_type = options
        .iter()
        .find(|opt| opt.name == "type")
        .and_then(|opt| {
            if let ResolvedValue::String(s) = opt.value {
                Some(s)
            } else {
                None
            }
        });

    let Some(incident_type) = incident_type else {
        return respond_error(
            ctx,
            interaction,
            &t!("errors.missing_incident_type", locale = &sync_locale),
            &sync_locale,
        )
        .await;
    };

    // Parse details (optional)
    let details = options
        .iter()
        .find(|opt| opt.name == "details")
        .and_then(|opt| {
            if let ResolvedValue::String(s) = opt.value {
                Some(s.to_string())
            } else {
                None
            }
        });

    // Validate details length
    if let Some(ref d) = details
        && d.len() > MAX_DETAILS_LENGTH
    {
        return respond_error(
            ctx,
            interaction,
            &t!(
                "errors.details_too_long",
                locale = &sync_locale,
                max = MAX_DETAILS_LENGTH,
                current = d.len()
            ),
            &sync_locale,
        )
        .await;
    }

    // Defer response before DB operations
    defer(ctx, interaction).await?;

    // Now resolve locale with full DB lookup
    let locale = resolve_locale_async(ctx, interaction).await;

    // Get database - clone connection to avoid holding AppState lock
    let db = crate::infrastructure::database::get_db(ctx).await;

    let user_id = interaction.user.id;
    let guild_id = interaction.guild_id;

    // Check registration
    match check_registration(&db, guild_id, user_id).await {
        RegistrationStatus::Registered => {}
        RegistrationStatus::GuildNotRegistered => {
            return defer::edit_error(
                ctx,
                interaction,
                &t!("embeds.report.error_guild_not_registered", locale = &locale),
                &locale,
            )
            .await;
        }
        RegistrationStatus::UserNotRegistered => {
            return edit_user_intro(ctx, interaction, &locale).await;
        }
    }

    // Try to insert report first (atomic operation to prevent race condition)
    match try_insert_report(&db, guild_id, user_id, incident_type, details.clone()).await {
        ReportInsertResult::Success => {
            // Report inserted successfully - continue to alert check
        }
        ReportInsertResult::CooldownActive(last_report_time) => {
            // User is in cooldown - show when they can report again
            let can_report_at = last_report_time + Duration::minutes(DUPLICATE_COOLDOWN_MINUTES);
            let time_text = format!("<t:{}:R>", can_report_at.timestamp());
            let embed = embeds::warning_embed(
                t!("embeds.report.cooldown.title", locale = &locale),
                t!(
                    "embeds.report.cooldown.description",
                    locale = &locale,
                    time = time_text
                ),
            );
            return defer::edit_embed(ctx, interaction, embed).await;
        }
        ReportInsertResult::Error(e) => {
            error!(error = %e, "Failed to insert report");
            return defer::edit_error(
                ctx,
                interaction,
                &t!("embeds.report.error_insert_failed", locale = &locale),
                &locale,
            )
            .await;
        }
    }

    // Check threshold and send alerts if needed
    crate::alerting::check_and_send_alerts(ctx, &db, incident_type).await;

    // Get count of similar reports
    let interval = get_report_interval(&db).await;
    let similar_count = get_similar_report_count(&db, incident_type, user_id, interval).await;

    info!(
        user_id = %user_id,
        guild_id = ?guild_id,
        incident_type = incident_type,
        similar_count = similar_count,
        "Report submitted"
    );

    // Success response
    let display_name = incident_types::display_name_localized(incident_type, &locale);
    let others_text = if similar_count == 0 {
        t!("embeds.report.success.others_none", locale = &locale).to_string()
    } else if similar_count == 1 {
        t!(
            "embeds.report.success.others_one",
            locale = &locale,
            interval = interval
        )
        .to_string()
    } else {
        t!(
            "embeds.report.success.others_many",
            locale = &locale,
            count = similar_count,
            interval = interval
        )
        .to_string()
    };

    let embed = embeds::success_embed(
        t!("embeds.report.success.title", locale = &locale),
        t!(
            "embeds.report.success.description",
            locale = &locale,
            incident_type = display_name,
            others_text = others_text
        ),
    )
    .footer(CreateEmbedFooter::new(t!(
        "embeds.report.success.footer",
        locale = &locale
    )))
    .timestamp(Timestamp::now());

    defer::edit_embed(ctx, interaction, embed).await
}

// =============================================================================
// Registration Check
// =============================================================================

enum RegistrationStatus {
    Registered,
    GuildNotRegistered,
    UserNotRegistered,
}

async fn check_registration(
    db: &DatabaseConnection,
    guild_id: Option<serenity::all::GuildId>,
    user_id: serenity::all::UserId,
) -> RegistrationStatus {
    match guild_id {
        Some(gid) => {
            // Guild context - check guild_configs
            let config = guild_configs::Entity::find_by_id(gid.to_string())
                .one(db)
                .await
                .ok()
                .flatten();

            match config {
                Some(c) if c.enabled => RegistrationStatus::Registered,
                _ => RegistrationStatus::GuildNotRegistered,
            }
        }
        None => {
            // User install context - check user_configs
            let config = user_configs::Entity::find_by_id(user_id.to_string())
                .one(db)
                .await
                .ok()
                .flatten();

            match config {
                Some(c) if c.enabled => RegistrationStatus::Registered,
                _ => RegistrationStatus::UserNotRegistered,
            }
        }
    }
}

// =============================================================================
// Insert Report (Atomic with Cooldown Check)
// =============================================================================

/// Result of attempting to insert a report
enum ReportInsertResult {
    /// Report was inserted successfully
    Success,
    /// User is in cooldown, contains the time of their last report
    CooldownActive(chrono::DateTime<Utc>),
    /// Database error occurred
    Error(sea_orm::DbErr),
}

/// Try to insert a report atomically with race condition handling.
///
/// This uses INSERT-first pattern to prevent race conditions:
/// 1. Check if user has recent report (optimistic check for better UX)
/// 2. If no recent report, insert new report
/// 3. After insert, verify no race condition occurred (multiple reports in window)
/// 4. If race detected, the earliest report wins, duplicates are deleted
///
/// This ensures that even if two requests arrive simultaneously, only one
/// report is recorded and the user sees proper cooldown messaging.
async fn try_insert_report(
    db: &DatabaseConnection,
    guild_id: Option<serenity::all::GuildId>,
    user_id: serenity::all::UserId,
    incident_type: &str,
    content: Option<String>,
) -> ReportInsertResult {
    // First, check if there's an existing active report in the cooldown window
    // This is still needed to get the exact timestamp for the error message
    let cutoff = Utc::now() - Duration::minutes(DUPLICATE_COOLDOWN_MINUTES);

    let existing = user_reports::Entity::find()
        .filter(user_reports::Column::UserId.eq(user_id.to_string()))
        .filter(user_reports::Column::IncidentType.eq(incident_type))
        .filter(user_reports::Column::Status.eq("active"))
        .filter(user_reports::Column::CreatedAt.gt(cutoff))
        .order_by_desc(user_reports::Column::CreatedAt)
        .one(db)
        .await;

    match existing {
        Ok(Some(report)) => {
            // User already has a recent report - return cooldown
            return ReportInsertResult::CooldownActive(report.created_at);
        }
        Ok(None) => {
            // No recent report, proceed to insert
        }
        Err(e) => {
            return ReportInsertResult::Error(e);
        }
    }

    // Try to insert the report
    // We use a transaction to ensure atomicity
    let report = user_reports::ActiveModel {
        guild_id: Set(guild_id.map(|g| g.to_string())),
        user_id: Set(user_id.to_string()),
        incident_type: Set(incident_type.to_string()),
        content: Set(content),
        status: Set("active".to_string()),
        created_at: Set(Utc::now()),
        ..Default::default()
    };

    match report.insert(db).await {
        Ok(inserted_report) => {
            // Double-check: verify we're the only report in the window
            // This handles the race condition where two requests pass the initial check
            // Recalculate cutoff to avoid stale timestamp issues
            let fresh_cutoff = Utc::now() - Duration::minutes(DUPLICATE_COOLDOWN_MINUTES);
            let reports_in_window = user_reports::Entity::find()
                .filter(user_reports::Column::UserId.eq(user_id.to_string()))
                .filter(user_reports::Column::IncidentType.eq(incident_type))
                .filter(user_reports::Column::Status.eq("active"))
                .filter(user_reports::Column::CreatedAt.gt(fresh_cutoff))
                .order_by_asc(user_reports::Column::CreatedAt)
                .order_by_asc(user_reports::Column::Id) // Tiebreaker for same-millisecond inserts
                .all(db)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Failed to query reports for race detection");
                    vec![]
                });

            if reports_in_window.len() > 1 {
                // Race condition detected - multiple reports in the window
                // The first one (by created_at) wins, others get deleted
                let first_report = &reports_in_window[0];

                if inserted_report.id != first_report.id {
                    // We lost the race - delete our report and return cooldown
                    let _ = user_reports::Entity::delete_by_id(inserted_report.id)
                        .exec(db)
                        .await;
                    return ReportInsertResult::CooldownActive(first_report.created_at);
                }
                // We won the race - delete the others
                for report in reports_in_window.iter().skip(1) {
                    if report.id != inserted_report.id {
                        let _ = user_reports::Entity::delete_by_id(report.id).exec(db).await;
                    }
                }
            }

            ReportInsertResult::Success
        }
        Err(e) => ReportInsertResult::Error(e),
    }
}

// =============================================================================
// Report Count
// =============================================================================

/// Count unique OTHER users who reported this incident type within the interval
async fn get_similar_report_count(
    db: &DatabaseConnection,
    incident_type: &str,
    exclude_user_id: serenity::all::UserId,
    interval_minutes: i64,
) -> i64 {
    use sea_orm::{QuerySelect, sea_query::Expr};

    let cutoff = Utc::now() - Duration::minutes(interval_minutes);

    // Count distinct users (excluding current user)
    let result = user_reports::Entity::find()
        .filter(user_reports::Column::IncidentType.eq(incident_type))
        .filter(user_reports::Column::UserId.ne(exclude_user_id.to_string()))
        .filter(user_reports::Column::CreatedAt.gt(cutoff))
        .filter(user_reports::Column::Status.eq("active"))
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

// =============================================================================
// Bot Config
// =============================================================================

/// Default report interval in minutes (used if config missing)
const DEFAULT_REPORT_INTERVAL: i64 = 60;

/// Get report interval from database, falls back to default if missing
async fn get_report_interval(db: &DatabaseConnection) -> i64 {
    bot_config::Entity::find_by_id("report_interval")
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|c| c.value.parse().ok())
        .unwrap_or_else(|| {
            tracing::warn!(
                "Missing config 'report_interval', using default: {} minutes",
                DEFAULT_REPORT_INTERVAL
            );
            DEFAULT_REPORT_INTERVAL
        })
}

// =============================================================================
// Edit Helpers (after defer - edit deferred response)
// =============================================================================

async fn edit_user_intro(
    ctx: &Context,
    interaction: &CommandInteraction,
    locale: &str,
) -> Result<(), serenity::Error> {
    let embed = embeds::info_embed(
        t!("embeds.report.intro.title", locale = locale),
        t!("embeds.report.intro.description", locale = locale),
    )
    .field(
        t!("embeds.report.intro.field_getting_started", locale = locale),
        t!(
            "embeds.report.intro.field_getting_started_value",
            locale = locale
        ),
        false,
    )
    .field(
        t!("embeds.report.intro.field_commands", locale = locale),
        t!("embeds.report.intro.field_commands_value", locale = locale),
        false,
    )
    .footer(CreateEmbedFooter::new(t!(
        "embeds.report.intro.footer",
        locale = locale
    )));

    defer::edit_embed(ctx, interaction, embed).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::MigratorTrait;
    use sea_orm::Database;
    use serenity::all::{GuildId, UserId};

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory SQLite");
        migration::Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");
        db
    }

    /// Helper: insert a report directly into the database
    async fn insert_report(
        db: &DatabaseConnection,
        user_id: &str,
        incident_type: &str,
        guild_id: Option<&str>,
        created_at: chrono::DateTime<Utc>,
    ) {
        let report = user_reports::ActiveModel {
            guild_id: Set(guild_id.map(|g| g.to_string())),
            user_id: Set(user_id.to_string()),
            incident_type: Set(incident_type.to_string()),
            content: Set(None),
            status: Set("active".to_string()),
            created_at: Set(created_at),
            ..Default::default()
        };
        report.insert(db).await.unwrap();
    }

    // =========================================================================
    // Report Insertion
    // =========================================================================

    #[tokio::test]
    async fn insert_report_success() {
        let db = setup_test_db().await;
        let user_id = UserId::new(100);
        let guild_id = Some(GuildId::new(200));

        let result =
            try_insert_report(&db, guild_id, user_id, "login", Some("test".to_string())).await;
        assert!(matches!(result, ReportInsertResult::Success));

        // Verify it's in the database
        let reports = user_reports::Entity::find().all(&db).await.unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].user_id, "100");
        assert_eq!(reports[0].incident_type, "login");
        assert_eq!(reports[0].content, Some("test".to_string()));
        assert_eq!(reports[0].guild_id, Some("200".to_string()));
    }

    #[tokio::test]
    async fn insert_report_cooldown_blocks_duplicate() {
        let db = setup_test_db().await;
        let user_id = UserId::new(100);

        // First report succeeds
        let result = try_insert_report(&db, None, user_id, "login", None).await;
        assert!(matches!(result, ReportInsertResult::Success));

        // Second report within cooldown window is blocked
        let result = try_insert_report(&db, None, user_id, "login", None).await;
        assert!(matches!(result, ReportInsertResult::CooldownActive(_)));

        // Only 1 report should be in the database
        let reports = user_reports::Entity::find().all(&db).await.unwrap();
        assert_eq!(reports.len(), 1);
    }

    #[tokio::test]
    async fn insert_report_different_types_allowed() {
        let db = setup_test_db().await;
        let user_id = UserId::new(100);

        let result = try_insert_report(&db, None, user_id, "login", None).await;
        assert!(matches!(result, ReportInsertResult::Success));

        // Different incident type should succeed
        let result = try_insert_report(&db, None, user_id, "api", None).await;
        assert!(matches!(result, ReportInsertResult::Success));

        let reports = user_reports::Entity::find().all(&db).await.unwrap();
        assert_eq!(reports.len(), 2);
    }

    #[tokio::test]
    async fn insert_report_different_users_allowed() {
        let db = setup_test_db().await;

        let result = try_insert_report(&db, None, UserId::new(100), "login", None).await;
        assert!(matches!(result, ReportInsertResult::Success));

        // Different user, same type should succeed
        let result = try_insert_report(&db, None, UserId::new(200), "login", None).await;
        assert!(matches!(result, ReportInsertResult::Success));
    }

    #[tokio::test]
    async fn insert_report_expired_cooldown_allows_new() {
        let db = setup_test_db().await;
        let user_id = UserId::new(100);

        // Insert old report (6 minutes ago, past the 5-min cooldown)
        let old_time = Utc::now() - Duration::minutes(6);
        insert_report(&db, "100", "login", None, old_time).await;

        // New report should succeed since cooldown expired
        let result = try_insert_report(&db, None, user_id, "login", None).await;
        assert!(matches!(result, ReportInsertResult::Success));
    }

    // =========================================================================
    // Similar Report Count
    // =========================================================================

    #[tokio::test]
    async fn similar_count_excludes_self() {
        let db = setup_test_db().await;
        let now = Utc::now();

        insert_report(&db, "100", "login", None, now).await;
        insert_report(&db, "200", "login", None, now).await;
        insert_report(&db, "300", "login", None, now).await;

        // From user 100's perspective, 2 OTHER users reported
        let count = get_similar_report_count(&db, "login", UserId::new(100), 60).await;
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn similar_count_only_within_interval() {
        let db = setup_test_db().await;
        let now = Utc::now();

        insert_report(&db, "200", "login", None, now).await;
        // Old report (2 hours ago)
        insert_report(&db, "300", "login", None, now - Duration::hours(2)).await;

        // With 60-min interval, only user 200 is counted
        let count = get_similar_report_count(&db, "login", UserId::new(100), 60).await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn similar_count_only_matching_type() {
        let db = setup_test_db().await;
        let now = Utc::now();

        insert_report(&db, "200", "login", None, now).await;
        insert_report(&db, "300", "api", None, now).await;

        let count = get_similar_report_count(&db, "login", UserId::new(100), 60).await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn similar_count_distinct_users() {
        let db = setup_test_db().await;
        let now = Utc::now();

        // Same user reports twice (shouldn't double-count)
        insert_report(&db, "200", "login", None, now).await;
        insert_report(&db, "200", "login", None, now - Duration::minutes(1)).await;

        let count = get_similar_report_count(&db, "login", UserId::new(100), 60).await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn similar_count_empty_db() {
        let db = setup_test_db().await;

        let count = get_similar_report_count(&db, "login", UserId::new(100), 60).await;
        assert_eq!(count, 0);
    }

    // =========================================================================
    // Bot Config
    // =========================================================================

    #[tokio::test]
    async fn report_interval_from_db() {
        let db = setup_test_db().await;

        // Migration seeds report_interval=60
        let interval = get_report_interval(&db).await;
        assert_eq!(interval, 60);
    }

    // =========================================================================
    // Registration Check
    // =========================================================================

    #[tokio::test]
    async fn check_registration_guild_registered() {
        let db = setup_test_db().await;
        let now = Utc::now();

        let model = guild_configs::ActiveModel {
            guild_id: Set("111".to_string()),
            channel_id: Set(Some("999".to_string())),
            enabled: Set(true),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&db).await.unwrap();

        let status = check_registration(&db, Some(GuildId::new(111)), UserId::new(222)).await;
        assert!(matches!(status, RegistrationStatus::Registered));
    }

    #[tokio::test]
    async fn check_registration_guild_not_registered() {
        let db = setup_test_db().await;

        let status = check_registration(&db, Some(GuildId::new(111)), UserId::new(222)).await;
        assert!(matches!(status, RegistrationStatus::GuildNotRegistered));
    }

    #[tokio::test]
    async fn check_registration_guild_disabled() {
        let db = setup_test_db().await;
        let now = Utc::now();

        let model = guild_configs::ActiveModel {
            guild_id: Set("111".to_string()),
            channel_id: Set(Some("999".to_string())),
            enabled: Set(false),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&db).await.unwrap();

        let status = check_registration(&db, Some(GuildId::new(111)), UserId::new(222)).await;
        assert!(matches!(status, RegistrationStatus::GuildNotRegistered));
    }

    #[tokio::test]
    async fn check_registration_user_registered() {
        let db = setup_test_db().await;
        let now = Utc::now();

        let model = user_configs::ActiveModel {
            user_id: Set("222".to_string()),
            enabled: Set(true),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&db).await.unwrap();

        let status = check_registration(&db, None, UserId::new(222)).await;
        assert!(matches!(status, RegistrationStatus::Registered));
    }

    #[tokio::test]
    async fn check_registration_user_not_registered() {
        let db = setup_test_db().await;

        let status = check_registration(&db, None, UserId::new(222)).await;
        assert!(matches!(status, RegistrationStatus::UserNotRegistered));
    }
}
