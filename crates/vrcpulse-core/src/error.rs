use thiserror::Error;

/// Core error types (no Discord/Serenity dependency)
#[derive(Debug, Error)]
pub enum CoreError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
