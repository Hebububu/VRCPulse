use thiserror::Error;

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    /// Failed to load environment variables
    #[error("Failed to load config: {0}")]
    Config(#[from] envy::Error),

    /// Database connection failure
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Discord client error
    #[error("Discord error: {0}")]
    Discord(#[from] serenity::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
