use serde::Deserialize;

/// 어플리케이션 환경변수 설정
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Discord 봇 토큰
    pub discord_token: String,
    /// 테스트 길드 ID (선택)
    /// 설정 시 해당 길드에 즉시 슬래시 명령어 등록
    pub test_guild_id: Option<u64>,
    /// SQLite 데이터베이스 연결 URL
    pub database_url: String,
}

impl Config {
    /// 환경변수 로드 및 Config 생성
    pub fn from_env() -> Result<Self, envy::Error> {
        if let Err(e) = dotenvy::dotenv() {
            eprintln!("Failed to load .env file: {e}");
        }

        envy::from_env::<Config>()
    }

    /// 필수 설정값 검증
    pub fn validate(&self) {
        if self.discord_token.is_empty() {
            panic!("DISCORD_TOKEN is required");
        }

        if self.database_url.is_empty() {
            panic!("DATABASE_URL is required");
        }

        if self.test_guild_id.is_some() {
            eprintln!("ℹ️  TEST_GUILD_ID is set. Commands will be registered to this guild only.");
        }
    }
}
