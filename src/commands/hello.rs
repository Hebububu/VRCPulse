use rust_i18n::t;
use serenity::all::{CommandInteraction, Context, CreateCommand};

use crate::commands::shared::defer;
use crate::i18n::resolve_locale_async;

/// /hello command definition
pub fn register() -> CreateCommand {
    CreateCommand::new("hello")
        .description(t!("commands.hello.description"))
        .name_localized("ko", t!("commands.hello.name", locale = "ko"))
        .description_localized("ko", t!("commands.hello.description", locale = "ko"))
}

/// /hello command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    // Defer first to acknowledge within 3 seconds, then do DB lookup for locale
    defer::defer(ctx, interaction).await?;

    let locale = resolve_locale_async(ctx, interaction).await;

    let response = serenity::builder::EditInteractionResponse::new().content(t!(
        "embeds.hello.message",
        locale = &locale,
        user = &interaction.user.name
    ));

    interaction.edit_response(&ctx.http, response).await?;
    Ok(())
}
