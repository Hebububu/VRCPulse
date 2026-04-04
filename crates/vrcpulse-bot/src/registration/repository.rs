//! Repository for guild and user configuration

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use serenity::all::{ChannelId, GuildId, UserId};
use std::sync::Arc;

use crate::entity::{guild_configs, user_configs};

// =============================================================================
// Guild Config Repository
// =============================================================================

/// Repository for guild configuration operations
pub struct GuildConfigRepository {
    db: Arc<DatabaseConnection>,
}

impl GuildConfigRepository {
    /// Create a new repository instance
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Get guild config by ID
    pub async fn get(
        &self,
        guild_id: GuildId,
    ) -> Result<Option<guild_configs::Model>, sea_orm::DbErr> {
        guild_configs::Entity::find_by_id(guild_id.to_string())
            .one(&*self.db)
            .await
    }

    /// Create a disabled guild config with language preference only (no channel).
    /// Used when setting language before the guild runs /config setup.
    pub async fn create_with_language(
        &self,
        guild_id: GuildId,
        language: String,
    ) -> Result<guild_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = guild_configs::ActiveModel {
            guild_id: Set(guild_id.to_string()),
            channel_id: Set(None),
            enabled: Set(false),
            language: Set(Some(language)),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&*self.db).await
    }

    /// Create new guild config
    pub async fn create(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<guild_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = guild_configs::ActiveModel {
            guild_id: Set(guild_id.to_string()),
            channel_id: Set(Some(channel_id.to_string())),
            enabled: Set(true),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&*self.db).await
    }

    /// Update guild language preference
    pub async fn update_language(
        &self,
        guild_id: GuildId,
        language: Option<String>,
    ) -> Result<guild_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = guild_configs::ActiveModel {
            guild_id: Set(guild_id.to_string()),
            language: Set(language),
            updated_at: Set(now),
            ..Default::default()
        };
        model.update(&*self.db).await
    }

    /// Re-enable existing guild config with new channel
    pub async fn reenable(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<guild_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = guild_configs::ActiveModel {
            guild_id: Set(guild_id.to_string()),
            channel_id: Set(Some(channel_id.to_string())),
            enabled: Set(true),
            updated_at: Set(now),
            ..Default::default()
        };
        model.update(&*self.db).await
    }

    /// Update guild channel
    pub async fn update_channel(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<guild_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = guild_configs::ActiveModel {
            guild_id: Set(guild_id.to_string()),
            channel_id: Set(Some(channel_id.to_string())),
            updated_at: Set(now),
            ..Default::default()
        };
        model.update(&*self.db).await
    }

    /// Disable guild config (soft delete)
    pub async fn disable(&self, guild_id: GuildId) -> Result<guild_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = guild_configs::ActiveModel {
            guild_id: Set(guild_id.to_string()),
            enabled: Set(false),
            updated_at: Set(now),
            ..Default::default()
        };
        model.update(&*self.db).await
    }

    /// Count enabled guild configs
    pub async fn count_enabled(&self) -> Result<u64, sea_orm::DbErr> {
        guild_configs::Entity::find()
            .filter(guild_configs::Column::Enabled.eq(true))
            .count(&*self.db)
            .await
    }
}

// =============================================================================
// User Config Repository
// =============================================================================

/// Repository for user configuration operations
pub struct UserConfigRepository {
    db: Arc<DatabaseConnection>,
}

impl UserConfigRepository {
    /// Create a new repository instance
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Get user config by ID
    pub async fn get(
        &self,
        user_id: UserId,
    ) -> Result<Option<user_configs::Model>, sea_orm::DbErr> {
        user_configs::Entity::find_by_id(user_id.to_string())
            .one(&*self.db)
            .await
    }

    /// Create new user config
    pub async fn create(&self, user_id: UserId) -> Result<user_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = user_configs::ActiveModel {
            user_id: Set(user_id.to_string()),
            enabled: Set(true),
            language: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&*self.db).await
    }

    /// Update user language preference
    pub async fn update_language(
        &self,
        user_id: UserId,
        language: Option<String>,
    ) -> Result<user_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = user_configs::ActiveModel {
            user_id: Set(user_id.to_string()),
            language: Set(language),
            updated_at: Set(now),
            ..Default::default()
        };
        model.update(&*self.db).await
    }

    /// Re-enable existing user config
    pub async fn reenable(&self, user_id: UserId) -> Result<user_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = user_configs::ActiveModel {
            user_id: Set(user_id.to_string()),
            enabled: Set(true),
            updated_at: Set(now),
            ..Default::default()
        };
        model.update(&*self.db).await
    }

    /// Disable user config (soft delete)
    pub async fn disable(&self, user_id: UserId) -> Result<user_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = user_configs::ActiveModel {
            user_id: Set(user_id.to_string()),
            enabled: Set(false),
            updated_at: Set(now),
            ..Default::default()
        };
        model.update(&*self.db).await
    }

    /// Count enabled user configs
    pub async fn count_enabled(&self) -> Result<u64, sea_orm::DbErr> {
        user_configs::Entity::find()
            .filter(user_configs::Column::Enabled.eq(true))
            .count(&*self.db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::MigratorTrait;
    use sea_orm::Database;
    use serenity::all::{ChannelId, GuildId, UserId};
    use std::sync::Arc;

    async fn setup_test_db() -> Arc<DatabaseConnection> {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory SQLite");
        migration::Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");
        Arc::new(db)
    }

    // =========================================================================
    // Guild Config Repository
    // =========================================================================

    #[tokio::test]
    async fn guild_create_and_get() {
        let db = setup_test_db().await;
        let repo = GuildConfigRepository::new(db);
        let guild_id = GuildId::new(111);
        let channel_id = ChannelId::new(222);

        let created = repo.create(guild_id, channel_id).await.unwrap();
        assert!(created.enabled);
        assert_eq!(created.channel_id, Some("222".to_string()));

        let fetched = repo.get(guild_id).await.unwrap().unwrap();
        assert_eq!(fetched.guild_id, "111");
        assert!(fetched.enabled);
    }

    #[tokio::test]
    async fn guild_get_nonexistent_returns_none() {
        let db = setup_test_db().await;
        let repo = GuildConfigRepository::new(db);

        let result = repo.get(GuildId::new(999)).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn guild_create_with_language() {
        let db = setup_test_db().await;
        let repo = GuildConfigRepository::new(db);
        let guild_id = GuildId::new(111);

        let created = repo
            .create_with_language(guild_id, "ko".to_string())
            .await
            .unwrap();
        assert!(!created.enabled);
        assert_eq!(created.language, Some("ko".to_string()));
        assert!(created.channel_id.is_none());
    }

    #[tokio::test]
    async fn guild_disable_and_reenable() {
        let db = setup_test_db().await;
        let repo = GuildConfigRepository::new(db);
        let guild_id = GuildId::new(111);
        let channel_id = ChannelId::new(222);

        repo.create(guild_id, channel_id).await.unwrap();

        // Disable
        let disabled = repo.disable(guild_id).await.unwrap();
        assert!(!disabled.enabled);

        // Count should be 0
        assert_eq!(repo.count_enabled().await.unwrap(), 0);

        // Re-enable with new channel
        let new_channel = ChannelId::new(333);
        let reenabled = repo.reenable(guild_id, new_channel).await.unwrap();
        assert!(reenabled.enabled);
        assert_eq!(reenabled.channel_id, Some("333".to_string()));

        assert_eq!(repo.count_enabled().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn guild_update_language() {
        let db = setup_test_db().await;
        let repo = GuildConfigRepository::new(db);
        let guild_id = GuildId::new(111);
        let channel_id = ChannelId::new(222);

        repo.create(guild_id, channel_id).await.unwrap();

        let updated = repo
            .update_language(guild_id, Some("ko".to_string()))
            .await
            .unwrap();
        assert_eq!(updated.language, Some("ko".to_string()));

        // Reset to auto-detect
        let reset = repo.update_language(guild_id, None).await.unwrap();
        assert!(reset.language.is_none());
    }

    #[tokio::test]
    async fn guild_update_channel() {
        let db = setup_test_db().await;
        let repo = GuildConfigRepository::new(db);
        let guild_id = GuildId::new(111);

        repo.create(guild_id, ChannelId::new(222)).await.unwrap();

        let updated = repo
            .update_channel(guild_id, ChannelId::new(333))
            .await
            .unwrap();
        assert_eq!(updated.channel_id, Some("333".to_string()));
    }

    #[tokio::test]
    async fn guild_count_enabled() {
        let db = setup_test_db().await;
        let repo = GuildConfigRepository::new(db);

        assert_eq!(repo.count_enabled().await.unwrap(), 0);

        repo.create(GuildId::new(1), ChannelId::new(10))
            .await
            .unwrap();
        repo.create(GuildId::new(2), ChannelId::new(20))
            .await
            .unwrap();
        repo.create(GuildId::new(3), ChannelId::new(30))
            .await
            .unwrap();

        assert_eq!(repo.count_enabled().await.unwrap(), 3);

        repo.disable(GuildId::new(2)).await.unwrap();
        assert_eq!(repo.count_enabled().await.unwrap(), 2);
    }

    // =========================================================================
    // User Config Repository
    // =========================================================================

    #[tokio::test]
    async fn user_create_and_get() {
        let db = setup_test_db().await;
        let repo = UserConfigRepository::new(db);
        let user_id = UserId::new(111);

        let created = repo.create(user_id).await.unwrap();
        assert!(created.enabled);
        assert!(created.language.is_none());

        let fetched = repo.get(user_id).await.unwrap().unwrap();
        assert_eq!(fetched.user_id, "111");
    }

    #[tokio::test]
    async fn user_disable_and_reenable() {
        let db = setup_test_db().await;
        let repo = UserConfigRepository::new(db);
        let user_id = UserId::new(111);

        repo.create(user_id).await.unwrap();

        let disabled = repo.disable(user_id).await.unwrap();
        assert!(!disabled.enabled);

        let reenabled = repo.reenable(user_id).await.unwrap();
        assert!(reenabled.enabled);
    }

    #[tokio::test]
    async fn user_update_language() {
        let db = setup_test_db().await;
        let repo = UserConfigRepository::new(db);
        let user_id = UserId::new(111);

        repo.create(user_id).await.unwrap();

        let updated = repo
            .update_language(user_id, Some("ko".to_string()))
            .await
            .unwrap();
        assert_eq!(updated.language, Some("ko".to_string()));
    }

    #[tokio::test]
    async fn user_count_enabled() {
        let db = setup_test_db().await;
        let repo = UserConfigRepository::new(db);

        assert_eq!(repo.count_enabled().await.unwrap(), 0);

        repo.create(UserId::new(1)).await.unwrap();
        repo.create(UserId::new(2)).await.unwrap();

        assert_eq!(repo.count_enabled().await.unwrap(), 2);

        repo.disable(UserId::new(1)).await.unwrap();
        assert_eq!(repo.count_enabled().await.unwrap(), 1);
    }
}
