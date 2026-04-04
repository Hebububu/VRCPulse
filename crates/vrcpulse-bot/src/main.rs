mod admin;
mod alerting;
mod discord;
mod entity;
mod hello;
mod i18n;
mod infrastructure;
mod onboarding;
mod registration;
mod reporting;
mod status;

// Initialize rust-i18n with locales from the `locales` directory
rust_i18n::i18n!("locales");

use infrastructure::config::Config;
use infrastructure::error::Result;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize logging
    infrastructure::logging::init();

    // 2. Load configuration
    let config = Config::from_env()?;
    config.validate();

    info!("Starting VRCPulse...");

    // 3. Set up and configure the bot
    let mut client = discord::setup(&config).await?;

    // 4. Start bot
    info!("Connecting to Discord...");
    if let Err(e) = client.start().await {
        error!("Client error: {:?}", e);
    }

    Ok(())
}
