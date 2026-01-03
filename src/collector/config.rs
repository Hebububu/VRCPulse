use std::time::Duration;

use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use thiserror::Error;
use tokio::sync::watch;
use tracing::info;

use crate::entity::bot_config;

use super::client::Result;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing config key: {0}")]
    MissingKey(String),

    #[error("Invalid config value for {key}: {value}")]
    InvalidValue { key: String, value: String },

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

/// Minimum polling interval (1 minute)
pub const MIN_INTERVAL: u64 = 60;

/// Maximum polling interval (1 hour)
pub const MAX_INTERVAL: u64 = 3600;

/// Default polling interval for reset (1 minute)
pub const DEFAULT_INTERVAL: u64 = 60;

/// Database keys for polling intervals
pub mod keys {
    pub const STATUS: &str = "polling.status";
    pub const INCIDENT: &str = "polling.incident";
    pub const MAINTENANCE: &str = "polling.maintenance";
    pub const METRICS: &str = "polling.metrics";
}

/// Poller type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollerType {
    Status,
    Incident,
    Maintenance,
    Metrics,
}

impl PollerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Incident => "incident",
            Self::Maintenance => "maintenance",
            Self::Metrics => "metrics",
        }
    }

    pub fn db_key(&self) -> &'static str {
        match self {
            Self::Status => keys::STATUS,
            Self::Incident => keys::INCIDENT,
            Self::Maintenance => keys::MAINTENANCE,
            Self::Metrics => keys::METRICS,
        }
    }

    pub fn min_interval(&self) -> u64 {
        MIN_INTERVAL
    }

    pub fn all() -> &'static [PollerType] {
        &[
            Self::Status,
            Self::Incident,
            Self::Maintenance,
            Self::Metrics,
        ]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "status" => Some(Self::Status),
            "incident" => Some(Self::Incident),
            "maintenance" => Some(Self::Maintenance),
            "metrics" => Some(Self::Metrics),
            _ => None,
        }
    }
}

/// Sender side of the config channels (for command handlers)
#[derive(Clone)]
pub struct CollectorConfigTx {
    pub status: watch::Sender<Duration>,
    pub incident: watch::Sender<Duration>,
    pub maintenance: watch::Sender<Duration>,
    pub metrics: watch::Sender<Duration>,
}

impl CollectorConfigTx {
    /// Get sender for a specific poller type
    pub fn get(&self, poller: PollerType) -> &watch::Sender<Duration> {
        match poller {
            PollerType::Status => &self.status,
            PollerType::Incident => &self.incident,
            PollerType::Maintenance => &self.maintenance,
            PollerType::Metrics => &self.metrics,
        }
    }

    /// Update interval for a poller and persist to database
    pub async fn update(
        &self,
        db: &DatabaseConnection,
        poller: PollerType,
        seconds: u64,
    ) -> Result<()> {
        let duration = Duration::from_secs(seconds);

        // Update watch channel
        self.get(poller).send(duration).ok();

        // Persist to database
        set_interval(db, poller, seconds).await?;

        info!(
            poller = poller.as_str(),
            seconds = seconds,
            "Updated polling interval"
        );

        Ok(())
    }

    /// Reset all polling intervals to default value
    pub async fn reset_all(&self, db: &DatabaseConnection) -> Result<()> {
        let duration = Duration::from_secs(DEFAULT_INTERVAL);

        for poller in PollerType::all() {
            self.get(*poller).send(duration).ok();
            set_interval(db, *poller, DEFAULT_INTERVAL).await?;
        }

        info!(
            seconds = DEFAULT_INTERVAL,
            "Reset all polling intervals to default"
        );

        Ok(())
    }
}

/// Receiver side of the config channels (for collector)
#[derive(Clone)]
pub struct CollectorConfigRx {
    pub status: watch::Receiver<Duration>,
    pub incident: watch::Receiver<Duration>,
    pub maintenance: watch::Receiver<Duration>,
    pub metrics: watch::Receiver<Duration>,
}

/// Create config channel pair and load initial values from database
pub async fn init(
    db: &DatabaseConnection,
) -> std::result::Result<(CollectorConfigTx, CollectorConfigRx), ConfigError> {
    let status_interval = load_interval(db, PollerType::Status).await?;
    let incident_interval = load_interval(db, PollerType::Incident).await?;
    let maintenance_interval = load_interval(db, PollerType::Maintenance).await?;
    let metrics_interval = load_interval(db, PollerType::Metrics).await?;

    let (status_tx, status_rx) = watch::channel(Duration::from_secs(status_interval));
    let (incident_tx, incident_rx) = watch::channel(Duration::from_secs(incident_interval));
    let (maintenance_tx, maintenance_rx) =
        watch::channel(Duration::from_secs(maintenance_interval));
    let (metrics_tx, metrics_rx) = watch::channel(Duration::from_secs(metrics_interval));

    let tx = CollectorConfigTx {
        status: status_tx,
        incident: incident_tx,
        maintenance: maintenance_tx,
        metrics: metrics_tx,
    };

    let rx = CollectorConfigRx {
        status: status_rx,
        incident: incident_rx,
        maintenance: maintenance_rx,
        metrics: metrics_rx,
    };

    info!(
        status = status_interval,
        incident = incident_interval,
        maintenance = maintenance_interval,
        metrics = metrics_interval,
        "Loaded polling intervals from database"
    );

    Ok((tx, rx))
}

/// Load interval from database, error if not found
async fn load_interval(
    db: &DatabaseConnection,
    poller: PollerType,
) -> std::result::Result<u64, ConfigError> {
    let key = poller.db_key();

    let config = bot_config::Entity::find_by_id(key)
        .one(db)
        .await?
        .ok_or_else(|| ConfigError::MissingKey(key.to_string()))?;

    config
        .value
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidValue {
            key: key.to_string(),
            value: config.value,
        })
}

/// Get current interval for a poller from database
pub async fn get_interval(
    db: &DatabaseConnection,
    poller: PollerType,
) -> std::result::Result<u64, ConfigError> {
    load_interval(db, poller).await
}

/// Set interval for a poller in database
pub async fn set_interval(db: &DatabaseConnection, poller: PollerType, seconds: u64) -> Result<()> {
    let key = poller.db_key();

    let existing = bot_config::Entity::find_by_id(key).one(db).await?;

    match existing {
        Some(existing) => {
            let mut active: bot_config::ActiveModel = existing.into();
            active.value = Set(seconds.to_string());
            active.updated_at = Set(Utc::now());
            active.update(db).await?;
        }
        None => {
            let config = bot_config::ActiveModel {
                key: Set(key.to_string()),
                value: Set(seconds.to_string()),
                updated_at: Set(Utc::now()),
            };
            config.insert(db).await?;
        }
    }

    Ok(())
}

/// Validate interval for a poller type
pub fn validate_interval(seconds: u64) -> std::result::Result<(), String> {
    if seconds < MIN_INTERVAL {
        return Err(format!(
            "Interval must be at least {} seconds",
            MIN_INTERVAL
        ));
    }

    if seconds > MAX_INTERVAL {
        return Err(format!("Interval must be at most {} seconds", MAX_INTERVAL));
    }

    Ok(())
}
