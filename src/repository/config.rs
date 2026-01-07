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
    pub async fn get(&self, guild_id: GuildId) -> Option<guild_configs::Model> {
        guild_configs::Entity::find_by_id(guild_id.to_string())
            .one(&*self.db)
            .await
            .ok()
            .flatten()
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
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&*self.db).await
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
    pub async fn get(&self, user_id: UserId) -> Option<user_configs::Model> {
        user_configs::Entity::find_by_id(user_id.to_string())
            .one(&*self.db)
            .await
            .ok()
            .flatten()
    }

    /// Create new user config
    pub async fn create(&self, user_id: UserId) -> Result<user_configs::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let model = user_configs::ActiveModel {
            user_id: Set(user_id.to_string()),
            enabled: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(&*self.db).await
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
