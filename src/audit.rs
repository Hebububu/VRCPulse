//! Command audit logging
//!
//! Logs command execution to console and database for auditing purposes.

use chrono::Utc;
use sea_orm::{ActiveModelTrait, Set};
use serenity::all::{CommandDataOptionValue, CommandInteraction, Context};
use tracing::{error, info};

use crate::database;
use crate::entity::command_logs;

/// Log command execution to console and database (non-blocking)
pub fn log_command(ctx: &Context, command: &CommandInteraction) {
    let command_name = &command.data.name;
    let user_id = command.user.id;
    let guild_id = command.guild_id;
    let channel_id = command.channel_id;

    // Extract subcommand if present
    let subcommand = command
        .data
        .options
        .first()
        .and_then(|opt| match &opt.value {
            CommandDataOptionValue::SubCommand(_) | CommandDataOptionValue::SubCommandGroup(_) => {
                Some(opt.name.as_str())
            }
            _ => None,
        });

    // Get guild name if available
    let guild_name = command
        .guild_id
        .and_then(|id| ctx.cache.guild(id).map(|g| g.name.clone()));

    // Console log (sync, fast)
    info!(
        command = command_name,
        subcommand = subcommand,
        user_id = %user_id,
        user_name = %command.user.name,
        guild_id = ?guild_id.map(|g| g.to_string()),
        guild_name = ?guild_name,
        channel_id = %channel_id,
        "Command received"
    );

    // Database audit log (spawn as background task to not block command handling)
    let command_name = command_name.clone();
    let subcommand = subcommand.map(|s| s.to_string());
    let user_id_str = user_id.to_string();
    let guild_id_str = guild_id.map(|g| g.to_string());
    let channel_id_str = channel_id.to_string();
    let ctx = ctx.clone();

    tokio::spawn(async move {
        if let Some(db) = database::try_get_db(&ctx).await {
            let log = command_logs::ActiveModel {
                command_name: Set(command_name),
                subcommand: Set(subcommand),
                user_id: Set(user_id_str),
                guild_id: Set(guild_id_str),
                channel_id: Set(Some(channel_id_str)),
                executed_at: Set(Utc::now()),
                ..Default::default()
            };

            if let Err(e) = log.insert(&*db).await {
                error!(error = %e, "Failed to insert command log");
            }
        }
    });
}
