//! /status dashboard command

use sea_orm::{EntityTrait, QueryOrder};
use serenity::all::{
    Colour, CommandInteraction, Context, CreateAttachment, CreateCommand, CreateEmbed,
    CreateEmbedFooter, Timestamp,
};
use tracing::error;

use crate::entity::{component_logs, status_logs};
use crate::state::AppStateKey;
use crate::visualization::generate_dashboard;

/// /status command definition
pub fn register() -> CreateCommand {
    CreateCommand::new("status")
        .description("View VRChat status dashboard with metrics visualization")
}

/// /status command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    // Defer response since dashboard generation takes time
    interaction.defer(&ctx.http).await?;

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

    // Fetch component statuses (latest for each component)
    let components = component_logs::Entity::find()
        .order_by_desc(component_logs::Column::SourceTimestamp)
        .all(db)
        .await
        .unwrap_or_default();

    // Get unique latest components
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
                        "none" => ("🟢", 0x57f287),
                        "minor" => ("🟡", 0xfee75c),
                        "major" => ("🟠", 0xf0b132),
                        "critical" => ("🔴", 0xed4245),
                        _ => ("⚪", 0x00b0f4),
                    };
                    (emoji, s.description.clone(), color)
                }
                None => ("⚪", "Unknown".to_string(), 0x00b0f4),
            };

            // Format component statuses
            let component_fields = format_component_groups(&latest_components);

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
                .title("VRChat Status Dashboard")
                .color(Colour::new(embed_color))
                .image("attachment://dashboard.png")
                .field(
                    "System Status",
                    format!("{} {}", status_emoji, status_text),
                    false,
                )
                .field("Online Users", &online_users, true)
                .field(
                    "API Error Rate",
                    format!("{:.4}%", stats.api_error_rate_avg),
                    true,
                )
                .field("\u{200B}", "\u{200B}", true)
                .field(
                    "Steam Auth",
                    format!("{:.1}%", stats.steam_success_avg),
                    true,
                )
                .field("Meta Auth", format!("{:.1}%", stats.meta_success_avg), true)
                .field("\u{200B}", "\u{200B}", true);

            // Add component group fields
            for (name, value, inline) in component_fields {
                embed = embed.field(name, value, inline);
            }

            let embed = embed
                .footer(CreateEmbedFooter::new("Last 12 hours"))
                .timestamp(Timestamp::now());

            let attachment = CreateAttachment::bytes(png_bytes, "dashboard.png");

            let response = serenity::builder::EditInteractionResponse::new()
                .embed(embed)
                .new_attachment(attachment);

            interaction.edit_response(&ctx.http, response).await?;
        }
        Err(e) => {
            error!(error = %e, "Failed to generate dashboard");

            let embed = CreateEmbed::default()
                .title("Error")
                .description("Failed to generate dashboard. Please try again later.")
                .color(Colour::new(0xed4245));

            let response = serenity::builder::EditInteractionResponse::new().embed(embed);

            interaction.edit_response(&ctx.http, response).await?;
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
fn format_component_groups(components: &[component_logs::Model]) -> Vec<(String, String, bool)> {
    if components.is_empty() {
        return vec![(
            "Components".to_string(),
            "No data available".to_string(),
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

    let format_group = |children: &[&str]| -> String {
        children
            .iter()
            .filter_map(|id| component_map.get(id))
            .map(|(name, status)| format!("{} {}", format_status(status), name))
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
            format!("{} API / Website", api_status),
            format_group(API_WEBSITE_CHILDREN),
            false,
        ),
        (
            format!("{} Realtime Networking", network_status),
            format_group(REALTIME_NETWORKING_CHILDREN),
            false,
        ),
    ]
}
