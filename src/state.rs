use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serenity::all::GuildId;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::collector::CollectorConfigTx;

/// TypeMap key for AppState access
pub struct AppStateKey;

impl serenity::prelude::TypeMapKey for AppStateKey {
    type Value = Arc<RwLock<AppState>>;
}

/// Application global state
/// - Accessible via `TypeMap` in Serenity event handlers
pub struct AppState {
    /// Database connection
    pub database: Arc<DatabaseConnection>,
    /// Collector config sender for dynamic interval updates
    pub collector_config: CollectorConfigTx,
    /// Bot startup timestamp
    pub started_at: DateTime<Utc>,
    /// Guilds awaiting intro message (failed to send on join)
    pending_intros: HashSet<GuildId>,
    /// Guilds that have already received intro (prevents duplicate sends)
    intro_sent_guilds: HashSet<GuildId>,
}

impl AppState {
    /// Create a new AppState instance
    pub fn new(database: DatabaseConnection, collector_config: CollectorConfigTx) -> Self {
        Self {
            database: Arc::new(database),
            collector_config,
            started_at: Utc::now(),
            pending_intros: HashSet::new(),
            intro_sent_guilds: HashSet::new(),
        }
    }

    /// Add a guild to the pending intros set
    pub fn add_pending_intro(&mut self, guild_id: GuildId) {
        self.pending_intros.insert(guild_id);
    }

    /// Remove a guild from the pending intros set
    /// Returns true if the guild was in the set
    pub fn remove_pending_intro(&mut self, guild_id: GuildId) -> bool {
        self.pending_intros.remove(&guild_id)
    }

    /// Check if intro was already sent to this guild and mark it as sent
    /// Returns true if this is the first time (should send intro)
    /// Returns false if intro was already sent (skip)
    pub fn try_mark_intro_sent(&mut self, guild_id: GuildId) -> bool {
        self.intro_sent_guilds.insert(guild_id)
    }
}
