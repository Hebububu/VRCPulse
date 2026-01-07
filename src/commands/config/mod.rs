//! /config command - Guild and user registration for VRCPulse alerts

mod context;
mod embeds;
mod handlers;
mod validation;

use serenity::all::{
    ChannelType, CommandInteraction, CommandOptionType, ComponentInteraction, Context,
    CreateCommand, CreateCommandOption, Permissions, ResolvedValue,
};

use crate::commands::shared::respond_error;
use context::determine_context;
use handlers::{
    handle_setup, handle_show, handle_unregister, handle_unregister_cancel,
    handle_unregister_confirm, is_cancel_button, is_confirm_button,
};

// =============================================================================
// Command Registration
// =============================================================================

/// /config command definition
pub fn register() -> CreateCommand {
    CreateCommand::new("config")
        .description("Configure VRCPulse alerts for this server or your account")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "setup",
                "Register for VRCPulse alerts",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "Channel to receive alerts (required for servers)",
                )
                .channel_types(vec![ChannelType::Text, ChannelType::News])
                .required(false),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "show",
            "View current configuration",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "unregister",
            "Disable VRCPulse alerts",
        ))
}

// =============================================================================
// Command Handler
// =============================================================================

/// /config command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    let options = &interaction.data.options();

    let Some(subcommand) = options.first() else {
        return respond_error(ctx, interaction, "Missing subcommand").await;
    };

    // Determine context: guild or user install
    let config_context = determine_context(interaction);

    match subcommand.name {
        "setup" => {
            let channel_id = if let ResolvedValue::SubCommand(opts) = &subcommand.value {
                opts.iter().find_map(|opt| {
                    if opt.name == "channel"
                        && let ResolvedValue::Channel(ch) = opt.value
                    {
                        return Some(ch.id);
                    }
                    None
                })
            } else {
                None
            };
            handle_setup(ctx, interaction, config_context, channel_id).await
        }
        "show" => handle_show(ctx, interaction, config_context).await,
        "unregister" => handle_unregister(ctx, interaction, config_context).await,
        _ => respond_error(ctx, interaction, "Unknown subcommand").await,
    }
}

// =============================================================================
// Button Handler
// =============================================================================

/// Handle button interactions for unregister confirmation
pub async fn handle_button(
    ctx: &Context,
    interaction: &ComponentInteraction,
) -> Result<(), serenity::Error> {
    let custom_id = &interaction.data.custom_id;

    if is_confirm_button(custom_id) {
        handle_unregister_confirm(ctx, interaction).await
    } else if is_cancel_button(custom_id) {
        handle_unregister_cancel(ctx, interaction).await
    } else {
        Ok(())
    }
}
