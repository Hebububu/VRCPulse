pub mod client;
pub mod config;
pub mod incident;
pub mod maintenance;
pub mod metrics;
pub mod models;
pub mod status;

use std::time::Duration;

use reqwest::Client;
use sea_orm::DatabaseConnection;
use tokio::sync::{mpsc, watch};
use tokio::time::{Interval, MissedTickBehavior, interval};
use tracing::{debug, error, info};

use crate::insight::InsightTrigger;

pub use config::{CollectorConfigRx, CollectorConfigTx};

/// Start the data collector with all pollers running concurrently.
/// The `insight_tx` sender is used to notify the analysis task when new incidents are detected.
pub async fn start(
    client: Client,
    db: DatabaseConnection,
    config: CollectorConfigRx,
    insight_tx: Option<mpsc::Sender<InsightTrigger>>,
) {
    info!("Starting data collector...");
    info!(
        status = config.status.borrow().as_secs(),
        incident = config.incident.borrow().as_secs(),
        maintenance = config.maintenance.borrow().as_secs(),
        metrics = config.metrics.borrow().as_secs(),
        history = config.history.borrow().as_secs(),
        "Polling intervals (seconds)"
    );

    tokio::join!(
        poll_loop_dynamic("status", config.status.clone(), || {
            status::poll(&client, &db)
        }),
        poll_loop_dynamic("incident", config.incident.clone(), || {
            incident::poll(&client, &db)
        }),
        poll_loop_dynamic("maintenance", config.maintenance.clone(), || {
            maintenance::poll(&client, &db)
        }),
        poll_loop_dynamic("metrics", config.metrics.clone(), || {
            metrics::poll(&client, &db)
        }),
        poll_loop_dynamic_with_trigger("history", config.history.clone(), &insight_tx, || async {
            incident::poll_history(&client, &db).await
        }),
    );
}

/// Poll loop variant that sends IncidentChange events to the insight task.
async fn poll_loop_dynamic_with_trigger<F, Fut>(
    name: &'static str,
    mut interval_rx: watch::Receiver<Duration>,
    insight_tx: &Option<mpsc::Sender<InsightTrigger>>,
    poll_fn: F,
) where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = client::Result<Vec<incident::IncidentChange>>>,
{
    let mut ticker = create_interval(*interval_rx.borrow());

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                match poll_fn().await {
                    Ok(changes) => {
                        debug!(poller = name, "Polled");
                        if let Some(tx) = insight_tx {
                            for change in &changes {
                                let trigger = match change {
                                    incident::IncidentChange::New { incident_id, title, .. } => {
                                        Some(InsightTrigger::IncidentDetected {
                                            incident_id: incident_id.clone(),
                                            title: title.clone(),
                                        })
                                    }
                                    incident::IncidentChange::StatusChanged { incident_id, title, .. } => {
                                        Some(InsightTrigger::IncidentDetected {
                                            incident_id: incident_id.clone(),
                                            title: title.clone(),
                                        })
                                    }
                                    _ => None,
                                };
                                if let Some(t) = trigger {
                                    let _ = tx.try_send(t);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(poller = name, error = %e, "Poll failed");
                    }
                }
            }
            _ = interval_rx.changed() => {
                let new_duration = *interval_rx.borrow();
                ticker = create_interval(new_duration);
                info!(
                    poller = name,
                    interval_secs = new_duration.as_secs(),
                    "Polling interval updated"
                );
            }
        }
    }
}

/// Poll loop with dynamic interval from watch channel
async fn poll_loop_dynamic<F, Fut>(
    name: &'static str,
    mut interval_rx: watch::Receiver<Duration>,
    poll_fn: F,
) where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = client::Result<()>>,
{
    let mut ticker = create_interval(*interval_rx.borrow());

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                match poll_fn().await {
                    Ok(()) => {
                        debug!(poller = name, "Polled");
                    }
                    Err(e) => {
                        error!(poller = name, error = %e, "Poll failed");
                    }
                }
            }
            _ = interval_rx.changed() => {
                let new_duration = *interval_rx.borrow();
                ticker = create_interval(new_duration);
                info!(
                    poller = name,
                    interval_secs = new_duration.as_secs(),
                    "Polling interval updated"
                );
            }
        }
    }
}

fn create_interval(duration: Duration) -> Interval {
    let mut ticker = interval(duration);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    ticker
}
