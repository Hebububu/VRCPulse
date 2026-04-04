use chrono::{DateTime, Utc};
use serenity::all::GuildId;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use vrcpulse_core::{CollectorConfigTx, VrcPulseService};

/// TypeMap key for AppState access
pub struct AppStateKey;

impl serenity::prelude::TypeMapKey for AppStateKey {
    type Value = Arc<RwLock<AppState>>;
}

/// Application global state
/// - Accessible via `TypeMap` in Serenity event handlers
#[allow(dead_code)]
pub struct AppState {
    /// Shared service for data queries (metrics, status, incidents)
    /// Also provides `db_ref()` for bot-specific table access
    pub service: VrcPulseService,
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
    pub fn new(service: VrcPulseService, collector_config: CollectorConfigTx) -> Self {
        Self {
            service,
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
