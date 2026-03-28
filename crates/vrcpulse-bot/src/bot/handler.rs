//! Discord event handler
//!
//! Handles all Discord gateway events (ready, interactions, guild joins, etc.)

use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serenity::all::{
    ActivityData, ComponentInteraction, EventHandler, Guild, Interaction, Permissions, Ready,
};
use tracing::{error, info, warn};

use crate::commands;
use crate::entity::guild_configs;
use crate::error::Result;
use crate::state::AppStateKey;

use super::intro::{
    BUTTON_SET_KOREAN, BUTTON_VIEW_KOREAN, create_admin_only_error_response, create_intro_message,
    create_korean_intro_response, create_set_korean_success_response,
};

/// Serenity event handler
pub struct Handler {
    /// Test guild ID (for development)
    pub test_guild_id: Option<u64>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    /// Called when the bot connects to Discord
    async fn ready(&self, ctx: serenity::all::Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // Set bot activity status
        ctx.set_activity(Some(ActivityData::watching("VRChat Status")));

        // Register slash commands
        match self.test_guild_id {
            Some(guild_id) => {
                // Development: register all commands including admin to test guild
                let guild_id = serenity::all::GuildId::new(guild_id);
                let mut cmds = commands::all();
                cmds.extend(commands::admin::all());

                match guild_id.set_commands(&ctx.http, cmds).await {
                    Ok(registered) => {
                        info!(
                            "Registered {} commands to test guild {} (includes admin)",
                            registered.len(),
                            guild_id
                        );
                    }
                    Err(e) => error!("Failed to register commands: {:?}", e),
                }
            }
            None => {
                // Production: global commands only (no admin)
                if let Err(e) = commands::register_global(&ctx).await {
                    error!("Failed to register commands: {:?}", e);
                }
            }
        }
    }

    /// Handle interactions (slash commands and buttons)
    async fn interaction_create(&self, ctx: serenity::all::Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                // Log command request (fire-and-forget, don't block command handling)
                crate::audit::log_command(&ctx, &command);

                // Check if this guild has a pending intro (but don't send yet)
                let pending_intro = if let Some(guild_id) = command.guild_id {
                    let data = ctx.data.read().await;
                    if let Some(state) = data.get::<AppStateKey>() {
                        if state.write().await.remove_pending_intro(guild_id) {
                            // Get guild's preferred locale from cache
                            let locale = guild_id
                                .to_guild_cached(&ctx.cache)
                                .map(|g| g.preferred_locale.clone())
                                .unwrap_or_else(|| "en-US".to_string());
                            Some((guild_id, command.channel_id, locale))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Run the command first
                let result = match command.data.name.as_str() {
                    "hello" => commands::hello::run(&ctx, &command).await,
                    "admin" => commands::admin::config::run(&ctx, &command).await,
                    "config" => commands::config::run(&ctx, &command).await,
                    "report" => commands::report::run(&ctx, &command).await,
                    "status" => commands::status::run(&ctx, &command).await,
                    _ => Ok(()),
                };

                if let Err(e) = result {
                    error!("Command error: {:?}", e);
                }

                // Send pending intro AFTER command completes
                if let Some((guild_id, channel_id, locale)) = pending_intro {
                    let message = create_intro_message(&locale);
                    if let Err(e) = channel_id.send_message(&ctx.http, message).await {
                        warn!(
                            guild_id = %guild_id,
                            error = %e,
                            "Failed to send pending intro message"
                        );
                    } else {
                        info!(guild_id = %guild_id, locale = %locale, "Sent pending intro message");
                    }
                }
            }
            Interaction::Component(component) => {
                // Handle intro button interactions
                if component.data.custom_id.starts_with("intro_") {
                    if let Err(e) = handle_intro_button(&ctx, &component).await {
                        error!("Intro button error: {:?}", e);
                    }
                    return;
                }

                // Handle button interactions for /config unregister
                if component.data.custom_id.starts_with("config_")
                    && let Err(e) = commands::config::handle_button(&ctx, &component).await
                {
                    error!("Button interaction error: {:?}", e);
                }
            }
            _ => {}
        }
    }

    /// Called when bot joins a new guild
    async fn guild_create(&self, ctx: serenity::all::Context, guild: Guild, is_new: Option<bool>) {
        // Only send intro for newly joined guilds (not on reconnect)
        let is_new = is_new.unwrap_or(false);
        if !is_new {
            return;
        }

        // Deduplication: prevent duplicate intro messages from rapid guild_create events
        // This can happen due to Discord sending duplicate events or race conditions
        let should_send = {
            let data = ctx.data.read().await;
            if let Some(state) = data.get::<AppStateKey>() {
                state.write().await.try_mark_intro_sent(guild.id)
            } else {
                true // No state, proceed (shouldn't happen)
            }
        };

        if !should_send {
            info!(
                guild_id = %guild.id,
                "Skipping intro - already sent or being processed"
            );
            return;
        }

        info!(guild_id = %guild.id, guild_name = %guild.name, "Processing new guild join");

        // Try to send intro message to system channel
        // Use guild's preferred locale - Discord sends "ko" for Korean, "en-US"/"en-GB" for English
        let intro_sent = if let Some(system_channel_id) = guild.system_channel_id {
            let message = create_intro_message(&guild.preferred_locale);

            match system_channel_id.send_message(&ctx.http, message).await {
                Ok(_) => {
                    info!(
                        guild_id = %guild.id,
                        locale = %guild.preferred_locale,
                        "Sent intro message to system channel"
                    );
                    true
                }
                Err(e) => {
                    warn!(
                        guild_id = %guild.id,
                        error = %e,
                        "Failed to send intro to system channel"
                    );
                    false
                }
            }
        } else {
            false
        };

        // If intro wasn't sent, add to pending intros for first command
        if !intro_sent {
            let data = ctx.data.read().await;
            if let Some(state) = data.get::<AppStateKey>() {
                state.write().await.add_pending_intro(guild.id);
                info!(guild_id = %guild.id, "Added to pending intros");
            }
        }
    }
}

/// Handle intro button interactions
async fn handle_intro_button(
    ctx: &serenity::all::Context,
    component: &ComponentInteraction,
) -> Result<()> {
    match component.data.custom_id.as_str() {
        BUTTON_VIEW_KOREAN => {
            // Send Korean intro with "set language" button (ephemeral)
            let response = create_korean_intro_response();
            component.create_response(&ctx.http, response).await?;
        }
        BUTTON_SET_KOREAN => {
            // Check if user has MANAGE_GUILD permission
            let has_permission = component.member.as_ref().is_some_and(|m| {
                m.permissions
                    .is_some_and(|p| p.contains(Permissions::MANAGE_GUILD))
            });

            if !has_permission {
                let response = create_admin_only_error_response();
                component.create_response(&ctx.http, response).await?;
                return Ok(());
            }

            // Get guild_id
            let Some(guild_id) = component.guild_id else {
                return Ok(());
            };

            // Update guild config to set language to Korean
            let data = ctx.data.read().await;
            if let Some(state) = data.get::<AppStateKey>() {
                let db = &*state.read().await.database;

                // Upsert guild config with language = "ko"
                let existing = guild_configs::Entity::find_by_id(guild_id.to_string())
                    .one(db)
                    .await?;

                let now = Utc::now();
                match existing {
                    Some(config) => {
                        let mut active: guild_configs::ActiveModel = config.into();
                        active.language = Set(Some("ko".to_string()));
                        active.updated_at = Set(now);
                        active.update(db).await?;
                    }
                    None => {
                        let active = guild_configs::ActiveModel {
                            guild_id: Set(guild_id.to_string()),
                            language: Set(Some("ko".to_string())),
                            enabled: Set(false),
                            created_at: Set(now),
                            updated_at: Set(now),
                            ..Default::default()
                        };
                        active.insert(db).await?;
                    }
                }

                info!(guild_id = %guild_id, "Set guild language to Korean via intro button");
            }

            // Send confirmation (public)
            let response = create_set_korean_success_response();
            component.create_response(&ctx.http, response).await?;
        }
        _ => {}
    }

    Ok(())
}
