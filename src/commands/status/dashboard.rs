//! /status dashboard command

use rust_i18n::t;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serenity::all::{
    Colour, CommandInteraction, Context, CreateAttachment, CreateCommand, CreateEmbed,
    CreateEmbedFooter, Timestamp,
};
use tracing::error;

use crate::commands::shared::{colors, defer, embeds};
use crate::entity::{component_logs, status_logs};
use crate::i18n::resolve_locale_async;
use crate::state::AppStateKey;
use crate::visualization::generate_dashboard;

/// /status command definition
pub fn register() -> CreateCommand {
    CreateCommand::new("status")
        .description(t!("commands.status.description"))
        .name_localized("ko", t!("commands.status.name", locale = "ko"))
        .description_localized("ko", t!("commands.status.description", locale = "ko"))
}

/// /status command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    // Defer response since dashboard generation takes time
    defer::defer(ctx, interaction).await?;

    let locale = resolve_locale_async(ctx, interaction).await;

    // Get database from AppState
    let data = ctx.data.read().await;
    let state = data
        .get::<AppStateKey>()
        .expect("AppState not found in TypeMap");
    let state = state.read().await;
    let db = state.database.as_ref();

    // Fetch system status
    let system_status = status_logs::Entity::find()
        .order_by_desc(status_logs::Column::SourceTimestamp)
        .one(db)
        .await
        .ok()
        .flatten();

    // Fetch latest component statuses (limit to recent data to avoid loading entire history)
    // We only need the most recent status for each component
    use chrono::{Duration, Utc};

    let recent_cutoff = Utc::now() - Duration::hours(24);
    let components = component_logs::Entity::find()
        .filter(component_logs::Column::SourceTimestamp.gt(recent_cutoff))
        .order_by_desc(component_logs::Column::SourceTimestamp)
        .all(db)
        .await
        .unwrap_or_default();

    // Get unique latest components (first occurrence due to DESC order)
    let mut seen_components = std::collections::HashSet::new();
    let latest_components: Vec<_> = components
        .into_iter()
        .filter(|c| seen_components.insert(c.component_id.clone()))
        .collect();

    // Generate dashboard
    let result = generate_dashboard(db).await;

    match result {
        Ok((png_bytes, stats)) => {
            // Format system status
            let (status_emoji, status_text, embed_color) = match system_status {
                Some(ref s) => {
                    let (emoji, color) = match s.indicator.as_str() {
                        "none" => ("🟢", colors::SUCCESS),
                        "minor" => ("🟡", colors::WARNING),
                        "major" => ("🟠", colors::MAJOR),
                        "critical" => ("🔴", colors::ERROR),
                        _ => ("⚪", colors::BRAND),
                    };
                    (emoji, s.description.clone(), color)
                }
                None => (
                    "⚪",
                    t!("status.unknown", locale = &locale).to_string(),
                    colors::BRAND,
                ),
            };

            // Format component statuses
            let component_fields = format_component_groups(&latest_components, &locale);

            // Format stats for embed
            let online_users = if stats.online_users_avg >= 1000.0 {
                format!(
                    "{:.0}k (avg) / {:.0}k (max)",
                    stats.online_users_avg / 1000.0,
                    stats.online_users_max / 1000.0
                )
            } else {
                format!(
                    "{:.0} (avg) / {:.0} (max)",
                    stats.online_users_avg, stats.online_users_max
                )
            };

            let mut embed = CreateEmbed::default()
                .title(t!("embeds.dashboard.title", locale = &locale))
                .color(Colour::new(embed_color))
                .image("attachment://dashboard.png")
                .field(
                    t!("embeds.dashboard.system_status", locale = &locale),
                    format!("{} {}", status_emoji, status_text),
                    false,
                )
                .field(
                    t!("embeds.dashboard.online_users", locale = &locale),
                    &online_users,
                    true,
                )
                .field(
                    t!("embeds.dashboard.api_error_rate", locale = &locale),
                    format!("{:.4}%", stats.api_error_rate_avg),
                    true,
                )
                .field("\u{200B}", "\u{200B}", true)
                .field(
                    t!("embeds.dashboard.steam_auth", locale = &locale),
                    format!("{:.1}%", stats.steam_success_avg),
                    true,
                )
                .field(
                    t!("embeds.dashboard.meta_auth", locale = &locale),
                    format!("{:.1}%", stats.meta_success_avg),
                    true,
                )
                .field("\u{200B}", "\u{200B}", true);

            // Add component group fields
            for (name, value, inline) in component_fields {
                embed = embed.field(name, value, inline);
            }

            let embed = embed
                .footer(CreateEmbedFooter::new(t!(
                    "embeds.dashboard.footer_timeframe",
                    locale = &locale
                )))
                .timestamp(Timestamp::now());

            let attachment = CreateAttachment::bytes(png_bytes, "dashboard.png");

            let response = serenity::builder::EditInteractionResponse::new()
                .embed(embed)
                .new_attachment(attachment);

            interaction.edit_response(&ctx.http, response).await?;
        }
        Err(e) => {
            error!(error = %e, "Failed to generate dashboard");

            let embed = embeds::error_embed(
                t!("embeds.dashboard.error_title", locale = &locale),
                t!("embeds.dashboard.error_description", locale = &locale),
            );

            defer::edit_embed(ctx, interaction, embed).await?;
        }
    }

    Ok(())
}

// Component group IDs (hardcoded from VRChat status API)
const GROUP_API_WEBSITE: &str = "64b3rr3cxgk5";
const GROUP_REALTIME_NETWORKING: &str = "t1jm7fqqq43h";

// Child component IDs for each group
const API_WEBSITE_CHILDREN: &[&str] = &[
    "ll3syftt0xwm", // Authentication / Login
    "fcb1zgxm9b3s", // Social / Friends List
    "6yydlg6mdf01", // SDK Asset Uploads
    "ftp7mrsh0fwm", // Realtime Player State Changes
];

const REALTIME_NETWORKING_CHILDREN: &[&str] = &[
    "sc8glkrd3yr4", // USA, West (San José)
    "76vv54mp1zfz", // USA, East (Washington D.C.)
    "yxhq0fcg5lkj", // Europe (Amsterdam)
    "3rv208r2qv7z", // Japan (Tokyo)
];

/// Format component statuses into grouped embed fields
fn format_component_groups(
    components: &[component_logs::Model],
    locale: &str,
) -> Vec<(String, String, bool)> {
    if components.is_empty() {
        return vec![(
            t!("embeds.dashboard.components", locale = locale).to_string(),
            t!("embeds.dashboard.no_data", locale = locale).to_string(),
            false,
        )];
    }

    // Build a map of component_id -> (name, status)
    let component_map: std::collections::HashMap<&str, (&str, &str)> = components
        .iter()
        .map(|c| {
            (
                c.component_id.as_str(),
                (c.name.as_str(), c.status.as_str()),
            )
        })
        .collect();

    let format_status = |status: &str| -> &str {
        match status {
            "operational" => "🟢",
            "degraded_performance" => "🟡",
            "partial_outage" => "🟠",
            "major_outage" => "🔴",
            "under_maintenance" => "🔵",
            _ => "⚪",
        }
    };

    // Translate component name
    let translate_component = |name: &str| -> String {
        // Try to get localized name, fall back to original
        let key = format!("components.{}", name);
        let translated = t!(&key, locale = locale);
        // If translation key doesn't exist, rust-i18n returns the key itself
        if translated.contains("components.") {
            name.to_string()
        } else {
            translated.to_string()
        }
    };

    let format_group = |children: &[&str]| -> String {
        children
            .iter()
            .filter_map(|id| component_map.get(id))
            .map(|(name, status)| {
                format!("{} {}", format_status(status), translate_component(name))
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Get group statuses
    let api_status = component_map
        .get(GROUP_API_WEBSITE)
        .map(|(_, s)| format_status(s))
        .unwrap_or("⚪");
    let network_status = component_map
        .get(GROUP_REALTIME_NETWORKING)
        .map(|(_, s)| format_status(s))
        .unwrap_or("⚪");

    vec![
        (
            format!(
                "{} {}",
                api_status,
                t!("embeds.dashboard.group_api_website", locale = locale)
            ),
            format_group(API_WEBSITE_CHILDREN),
            false,
        ),
        (
            format!(
                "{} {}",
                network_status,
                t!(
                    "embeds.dashboard.group_realtime_networking",
                    locale = locale
                )
            ),
            format_group(REALTIME_NETWORKING_CHILDREN),
            false,
        ),
    ]
}
