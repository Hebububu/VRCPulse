//! /config command - Guild and user registration for VRCPulse alerts

mod context;
mod embeds;
mod handlers;
mod validation;

use rust_i18n::t;
use serenity::all::{
    ChannelType, CommandInteraction, CommandOptionType, ComponentInteraction, Context,
    CreateCommand, CreateCommandOption, Permissions, ResolvedValue,
};

use crate::commands::shared::respond_error;
use crate::i18n::resolve_locale;
use context::determine_context;
use handlers::{
    handle_language, handle_setup, handle_show, handle_unregister, handle_unregister_cancel,
    handle_unregister_confirm, is_cancel_button, is_confirm_button,
};

// =============================================================================
// Command Registration
// =============================================================================

/// /config command definition
pub fn register() -> CreateCommand {
    CreateCommand::new("config")
        .description(t!("commands.config.description"))
        .name_localized("ko", t!("commands.config.name", locale = "ko"))
        .description_localized("ko", t!("commands.config.description", locale = "ko"))
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "setup",
                t!("commands.config.setup.description"),
            )
            .name_localized("ko", t!("commands.config.setup.name", locale = "ko"))
            .description_localized("ko", t!("commands.config.setup.description", locale = "ko"))
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    t!("commands.config.setup.option_channel"),
                )
                .name_localized("ko", "채널")
                .description_localized(
                    "ko",
                    t!("commands.config.setup.option_channel", locale = "ko"),
                )
                .channel_types(vec![ChannelType::Text, ChannelType::News])
                .required(false),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "show",
                t!("commands.config.show.description"),
            )
            .name_localized("ko", t!("commands.config.show.name", locale = "ko"))
            .description_localized("ko", t!("commands.config.show.description", locale = "ko")),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "unregister",
                t!("commands.config.unregister.description"),
            )
            .name_localized("ko", t!("commands.config.unregister.name", locale = "ko"))
            .description_localized(
                "ko",
                t!("commands.config.unregister.description", locale = "ko"),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "language",
                t!("commands.config.language.description"),
            )
            .name_localized("ko", t!("commands.config.language.name", locale = "ko"))
            .description_localized(
                "ko",
                t!("commands.config.language.description", locale = "ko"),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "code",
                    t!("commands.config.language.option_code"),
                )
                .name_localized("ko", "코드")
                .description_localized(
                    "ko",
                    t!("commands.config.language.option_code", locale = "ko"),
                )
                .required(false)
                .add_string_choice("English", "en")
                .add_string_choice("한국어 (Korean)", "ko")
                .add_string_choice("Auto-detect (Discord)", "auto"),
            ),
        )
}

// =============================================================================
// Command Handler
// =============================================================================

/// /config command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    let options = &interaction.data.options();
    // Use sync locale resolution for error messages (before defer)
    // Each handler will call resolve_locale_async after deferring for full DB lookup
    let locale = resolve_locale(interaction);

    let Some(subcommand) = options.first() else {
        return respond_error(ctx, interaction, "Missing subcommand", &locale).await;
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
        "language" => {
            let language_code = if let ResolvedValue::SubCommand(opts) = &subcommand.value {
                opts.iter().find_map(|opt| {
                    if opt.name == "code"
                        && let ResolvedValue::String(code) = opt.value
                    {
                        return Some(code.to_string());
                    }
                    None
                })
            } else {
                None
            };
            handle_language(ctx, interaction, config_context, language_code).await
        }
        _ => respond_error(ctx, interaction, "Unknown subcommand", &locale).await,
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
