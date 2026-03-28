//! Embed builders for /admin command responses

use serenity::all::{Colour, CreateEmbed, CreateEmbedFooter, Timestamp};

use crate::commands::shared::colors;

/// Build embed showing current polling intervals
pub fn show_intervals(
    status: &str,
    incident: &str,
    maintenance: &str,
    metrics: &str,
) -> CreateEmbed {
    CreateEmbed::default()
        .title("Polling Intervals")
        .color(Colour::new(colors::BRAND))
        .field("Status", status, true)
        .field("Incident", incident, true)
        .field("Maintenance", maintenance, true)
        .field("Metrics", metrics, true)
        .footer(CreateEmbedFooter::new("Use /admin config set to change"))
}

/// Build embed for successful config update
pub fn config_updated(poller: &str, seconds: u64) -> CreateEmbed {
    CreateEmbed::default()
        .title("Configuration Updated")
        .description("Polling interval has been changed.")
        .color(Colour::new(colors::SUCCESS))
        .field("Poller", poller, true)
        .field("New Interval", format!("{}s", seconds), true)
        .timestamp(Timestamp::now())
}

/// Build embed for successful config reset
pub fn config_reset(default_interval: u64) -> CreateEmbed {
    let default_str = format!("{}s", default_interval);

    CreateEmbed::default()
        .title("Configuration Reset")
        .description("All polling intervals have been reset to default values.")
        .color(Colour::new(colors::SUCCESS))
        .field("Status", &default_str, true)
        .field("Incident", &default_str, true)
        .field("Maintenance", &default_str, true)
        .field("Metrics", &default_str, true)
        .timestamp(Timestamp::now())
}

/// Build embed for /admin show - bot info and command summary
pub fn admin_show(
    version: &str,
    uptime: &str,
    guild_count: u64,
    registered_guilds: u64,
    registered_users: u64,
    status_interval: &str,
    incident_interval: &str,
    maintenance_interval: &str,
    metrics_interval: &str,
) -> CreateEmbed {
    CreateEmbed::default()
        .title("VRCPulse Admin")
        .color(Colour::new(colors::BRAND))
        .field("Version", version, true)
        .field("Uptime", uptime, true)
        .field("Guilds", guild_count.to_string(), true)
        .field("Registered Guilds", registered_guilds.to_string(), true)
        .field("Registered Users", registered_users.to_string(), true)
        .field("\u{200b}", "\u{200b}", true) // Empty field for alignment
        .field(
            "Polling Intervals",
            format!(
                "Status: {}\nIncident: {}\nMaintenance: {}\nMetrics: {}",
                status_interval, incident_interval, maintenance_interval, metrics_interval
            ),
            false,
        )
        .field(
            "Commands",
            "`/admin show` - Display bot information\n\
             `/admin config show` - View polling intervals\n\
             `/admin config set <poller> <seconds>` - Update interval\n\
             `/admin config reset` - Reset all intervals to default",
            false,
        )
        .footer(CreateEmbedFooter::new("Owner-only commands"))
}
