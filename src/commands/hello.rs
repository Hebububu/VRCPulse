use rust_i18n::t;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

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
    // Resolve locale from database preference, then Discord locale, then default
    let locale = resolve_locale_async(ctx, interaction).await;

    let response = CreateInteractionResponseMessage::new().content(t!(
        "embeds.hello.message",
        locale = &locale,
        user = &interaction.user.name
    ));

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}
