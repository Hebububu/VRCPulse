pub mod client;
pub mod incident;
pub mod maintenance;
pub mod metrics;
pub mod models;
pub mod status;

use std::time::Duration;

use reqwest::Client;
use sea_orm::DatabaseConnection;
use tokio::time::interval;
use tracing::{error, info};

/// Polling intervals as defined in the spec
const STATUS_INTERVAL: Duration = Duration::from_secs(60);
const INCIDENT_INTERVAL: Duration = Duration::from_secs(30);
const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(300);
const METRICS_INTERVAL: Duration = Duration::from_secs(60);

/// Start the data collector with all pollers running concurrently
pub async fn start(client: Client, db: DatabaseConnection) {
    info!("Starting data collector...");

    tokio::join!(
        poll_loop("status", STATUS_INTERVAL, || { status::poll(&client, &db) }),
        poll_loop("incident", INCIDENT_INTERVAL, || {
            incident::poll(&client, &db)
        }),
        poll_loop("maintenance", MAINTENANCE_INTERVAL, || {
            maintenance::poll(&client, &db)
        }),
        poll_loop("metrics", METRICS_INTERVAL, || {
            metrics::poll(&client, &db)
        }),
    );
}

async fn poll_loop<F, Fut>(name: &'static str, duration: Duration, poll_fn: F)
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = client::Result<()>>,
{
    let mut ticker = interval(duration);

    loop {
        ticker.tick().await;

        match poll_fn().await {
            Ok(()) => {
                tracing::debug!(poller = name, "Poll completed successfully");
            }
            Err(e) => {
                error!(poller = name, error = %e, "Poll failed");
            }
        }
    }
}
