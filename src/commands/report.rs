//! /report command - User incident reporting for VRChat issues

use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    ResolvedValue, Timestamp,
};
use tracing::{error, info};

use crate::entity::{bot_config, guild_configs, user_configs, user_reports};
use crate::state::AppStateKey;

// =============================================================================
// Constants
// =============================================================================

/// Duplicate report cooldown in minutes
const DUPLICATE_COOLDOWN_MINUTES: i64 = 5;

/// Maximum length for details field
const MAX_DETAILS_LENGTH: usize = 500;

/// Brand color for embeds
const COLOR_BRAND: u32 = 0x00b0f4;
/// Success color for embeds
const COLOR_SUCCESS: u32 = 0x57f287;
/// Error color for embeds
const COLOR_ERROR: u32 = 0xed4245;
/// Warning color for embeds
const COLOR_WARNING: u32 = 0xfee75c;

// =============================================================================
// Incident Types
// =============================================================================

/// Available incident types for reporting
const INCIDENT_TYPES: &[(&str, &str)] = &[
    ("login", "Login Issues"),
    ("instance", "Instance/World Loading"),
    ("api", "API/Website Issues"),
    ("auth", "Authentication Issues"),
    ("download", "Content Download Issues"),
    ("other", "Other Issues"),
];

/// Get display name for incident type
fn get_incident_display_name(incident_type: &str) -> &str {
    INCIDENT_TYPES
        .iter()
        .find(|(value, _)| *value == incident_type)
        .map(|(_, display)| *display)
        .unwrap_or(incident_type)
}

// =============================================================================
// Command Registration
// =============================================================================

/// /report command definition
pub fn register() -> CreateCommand {
    let mut incident_type_option = CreateCommandOption::new(
        CommandOptionType::String,
        "type",
        "Type of issue you're experiencing",
    )
    .required(true);

    // Add choices for incident types
    for (value, display) in INCIDENT_TYPES {
        incident_type_option = incident_type_option.add_string_choice(*display, *value);
    }

    CreateCommand::new("report")
        .description("Report a VRChat issue")
        .add_option(incident_type_option)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "details",
                "Additional details about the issue (max 500 chars)",
            )
            .required(false),
        )
}

/// /report command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
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
        return respond_error(ctx, interaction, "Missing incident type").await;
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
    if let Some(ref d) = details {
        if d.len() > MAX_DETAILS_LENGTH {
            return respond_error(
                ctx,
                interaction,
                &format!(
                    "Details must be under {} characters.\nYou provided {} characters.",
                    MAX_DETAILS_LENGTH,
                    d.len()
                ),
            )
            .await;
        }
    }

    // Get database
    let data = ctx.data.read().await;
    let state = data
        .get::<AppStateKey>()
        .expect("AppState not found in TypeMap");
    let state = state.read().await;
    let db = state.database.as_ref();

    let user_id = interaction.user.id;
    let guild_id = interaction.guild_id;

    // Check registration
    match check_registration(db, guild_id, user_id).await {
        RegistrationStatus::Registered => {}
        RegistrationStatus::GuildNotRegistered => {
            return respond_error(
                ctx,
                interaction,
                "An administrator must run `/config setup #channel` first.",
            )
            .await;
        }
        RegistrationStatus::UserNotRegistered => {
            return respond_user_intro(ctx, interaction).await;
        }
    }

    // Try to insert report first (atomic operation to prevent race condition)
    match try_insert_report(db, guild_id, user_id, incident_type, details.clone()).await {
        ReportInsertResult::Success => {
            // Report inserted successfully - continue to alert check
        }
        ReportInsertResult::CooldownActive(last_report_time) => {
            // User is in cooldown - show when they can report again
            let can_report_at = last_report_time + Duration::minutes(DUPLICATE_COOLDOWN_MINUTES);
            return respond_warning(
                ctx,
                interaction,
                "Report Cooldown",
                &format!(
                    "You recently submitted a report.\nYou can report again <t:{}:R>.",
                    can_report_at.timestamp()
                ),
            )
            .await;
        }
        ReportInsertResult::Error(e) => {
            error!(error = %e, "Failed to insert report");
            return respond_error(
                ctx,
                interaction,
                "Failed to submit report. Please try again.",
            )
            .await;
        }
    }

    // Check threshold and send alerts if needed
    crate::alerts::check_and_send_alerts(ctx, db, incident_type).await;

    // Get count of similar reports
    let interval = get_report_interval(db).await;
    let similar_count = get_similar_report_count(db, incident_type, user_id, interval).await;

    info!(
        user_id = %user_id,
        guild_id = ?guild_id,
        incident_type = incident_type,
        similar_count = similar_count,
        "Report submitted"
    );

    // Success response
    let display_name = get_incident_display_name(incident_type);
    let others_text = if similar_count == 0 {
        "You're the first to report this issue recently.".to_string()
    } else if similar_count == 1 {
        format!(
            "1 other user reported this issue in the last {} minutes.",
            interval
        )
    } else {
        format!(
            "{} others reported this issue in the last {} minutes.",
            similar_count, interval
        )
    };

    let embed = CreateEmbed::default()
        .title("Report Submitted")
        .description(format!(
            "Thank you for reporting **{}**.\n\n{}",
            display_name, others_text
        ))
        .color(Colour::new(COLOR_SUCCESS))
        .footer(CreateEmbedFooter::new(
            "Your report helps us detect widespread issues.",
        ))
        .timestamp(Timestamp::now());

    let response = CreateInteractionResponseMessage::new().embed(embed);
    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
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
// Response Helpers
// =============================================================================

async fn respond_error(
    ctx: &Context,
    interaction: &CommandInteraction,
    message: &str,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title("Error")
        .description(message)
        .color(Colour::new(COLOR_ERROR));

    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

async fn respond_warning(
    ctx: &Context,
    interaction: &CommandInteraction,
    title: &str,
    message: &str,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title(title)
        .description(message)
        .color(Colour::new(COLOR_WARNING));

    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}

async fn respond_user_intro(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), serenity::Error> {
    let embed = CreateEmbed::default()
        .title("Welcome to VRCPulse!")
        .description(
            "VRCPulse monitors VRChat server status and alerts you when issues occur.",
        )
        .color(Colour::new(COLOR_BRAND))
        .field(
            "Getting Started",
            "1. Run `/config setup` to register for DM alerts\n2. Check current VRChat status with `/status`",
            false,
        )
        .field(
            "Commands",
            "- `/config setup` - Register for DM alerts\n- `/config show` - View current settings\n- `/status` - View VRChat status dashboard",
            false,
        )
        .footer(CreateEmbedFooter::new(
            "Run /config setup to start receiving alerts and submit reports!",
        ));

    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}
