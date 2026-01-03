use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

/// /hello command definition
pub fn register() -> CreateCommand {
    CreateCommand::new("hello").description("Say hello to VRCPulse!")
}

/// /hello command handler
pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    let response = CreateInteractionResponseMessage::new()
        .content(format!("Hello, {}! 👋", interaction.user.name));

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
}
