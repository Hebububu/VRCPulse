mod alerts;
mod audit;
mod bot;
mod collector;
mod commands;
mod config;
mod database;
mod entity;
mod error;
mod i18n;
mod logging;
mod repository;
mod state;
mod visualization;

// Initialize rust-i18n with locales from the `locales` directory
rust_i18n::i18n!("locales");

use config::Config;
use error::Result;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize logging
    logging::init();

    // 2. Load configuration
    let config = Config::from_env()?;
    config.validate();

    info!("Starting VRCPulse...");

    // 3. Set up and configure the bot
    let mut client = bot::setup(&config).await?;

    // 4. Start bot
    info!("Connecting to Discord...");
    if let Err(e) = client.start().await {
        error!("Client error: {:?}", e);
    }

    Ok(())
}
