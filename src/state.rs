use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// 애플리케이션 전역 상태
/// - Serenity 이벤트 핸들러에서 `TypeMap`을 통해 접근 가능
#[derive(Clone)]
pub struct AppState {
    /// 데이터베이스 연결
    pub database: Arc<DatabaseConnection>,
    /// HTTP 클라이언트 (VRChat API 호출용)
    pub http_client: reqwest::Client,
}

impl AppState {
    /// 새로운 AppState 인스턴스 생성
    pub fn new(database: DatabaseConnection) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            database: Arc::new(database),
            http_client,
        }
    }
}
