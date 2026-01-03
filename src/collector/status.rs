use chrono::Utc;
use reqwest::Client;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{debug, info};

use crate::entity::{component_logs, status_logs};

use super::client::{Result, fetch_json, status_api_url};
use super::models::SummaryResponse;

/// Poll /summary.json and store status and component logs
pub async fn poll(client: &Client, db: &DatabaseConnection) -> Result<()> {
    let url = status_api_url("/summary.json");
    let response: SummaryResponse = fetch_json(client, &url).await?;

    let source_timestamp = response.page.updated_at;
    let now = Utc::now();

    // Check if status log already exists for this timestamp
    let existing_status = status_logs::Entity::find()
        .filter(status_logs::Column::SourceTimestamp.eq(source_timestamp))
        .one(db)
        .await?;

    if existing_status.is_none() {
        // Insert new status log
        let status_log = status_logs::ActiveModel {
            indicator: Set(response.status.indicator.clone()),
            description: Set(response.status.description.clone()),
            source_timestamp: Set(source_timestamp),
            created_at: Set(now),
            ..Default::default()
        };
        status_log.insert(db).await?;
        info!(
            indicator = %response.status.indicator,
            "Inserted new status log"
        );
    } else {
        debug!("Status log already exists for timestamp, skipping");
    }

    // Process components
    for component in response.components {
        let existing_component = component_logs::Entity::find()
            .filter(component_logs::Column::ComponentId.eq(&component.id))
            .filter(component_logs::Column::SourceTimestamp.eq(source_timestamp))
            .one(db)
            .await?;

        if existing_component.is_none() {
            let component_log = component_logs::ActiveModel {
                component_id: Set(component.id.clone()),
                name: Set(component.name.clone()),
                status: Set(component.status.clone()),
                source_timestamp: Set(source_timestamp),
                created_at: Set(now),
                ..Default::default()
            };
            component_log.insert(db).await?;
            debug!(
                component_id = %component.id,
                name = %component.name,
                status = %component.status,
                "Inserted component log"
            );
        }
    }

    Ok(())
}
