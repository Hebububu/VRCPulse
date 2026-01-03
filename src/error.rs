use thiserror::Error;

/// 애플리케이션 에러 타입
#[derive(Debug, Error)]
pub enum AppError {
    /// 환경변수 로드 실패
    #[error("Failed to load config: {0}")]
    Config(#[from] envy::Error),

    /// 데이터베이스 연결 실패
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Discord 클라이언트 에러
    #[error("Discord error: {0}")]
    Discord(#[from] serenity::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
