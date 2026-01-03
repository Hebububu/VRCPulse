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
use tokio::sync::watch;
use tokio::time::{Interval, MissedTickBehavior, interval};
use tracing::{debug, error, info};

pub use config::{CollectorConfigRx, CollectorConfigTx};

/// Start the data collector with all pollers running concurrently
pub async fn start(client: Client, db: DatabaseConnection, config: CollectorConfigRx) {
    info!("Starting data collector...");

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
    );
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
                        debug!(poller = name, "Poll completed successfully");
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
