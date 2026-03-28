use serde::Deserialize;

/// Application environment configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Discord bot token
    pub discord_token: String,
    /// Test guild ID (optional)
    /// When set, registers slash commands to this guild immediately
    pub test_guild_id: Option<u64>,
    /// SQLite database connection URL
    pub database_url: String,
}

impl Config {
    /// Load environment variables and create Config
    pub fn from_env() -> Result<Self, envy::Error> {
        if let Err(e) = dotenvy::dotenv() {
            eprintln!("Failed to load .env file: {e}");
        }

        envy::from_env::<Config>()
    }

    /// Validate required configuration values
    pub fn validate(&self) {
        if self.discord_token.is_empty() {
            panic!("DISCORD_TOKEN is required");
        }

        if self.database_url.is_empty() {
            panic!("DATABASE_URL is required");
        }

        if self.test_guild_id.is_some() {
            eprintln!("TEST_GUILD_ID is set. Commands will be registered to this guild only.");
        }
    }
}
