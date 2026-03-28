use thiserror::Error;

/// Bot-specific error types
#[derive(Debug, Error)]
pub enum AppError {
    /// Core library error
    #[error("{0}")]
    Core(#[from] vrcpulse_core::CoreError),

    /// Failed to load environment variables
    #[error("Failed to load config: {0}")]
    Config(#[from] envy::Error),

    /// Database connection failure
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Discord client error
    #[error("Discord error: {0}")]
    Discord(#[from] Box<serenity::Error>),
}

impl From<serenity::Error> for AppError {
    fn from(e: serenity::Error) -> Self {
        AppError::Discord(Box::new(e))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
