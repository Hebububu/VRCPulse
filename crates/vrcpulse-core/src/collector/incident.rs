use std::collections::HashSet;

use chrono::Utc;
use reqwest::Client;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use tracing::{debug, info, warn};

use crate::entity::{incident_snapshots, incident_updates, incidents};

use super::client::{Result, fetch_json, status_api_url};
use super::models::{IncidentsResponse, UnresolvedIncidentsResponse};

/// Change events detected during history polling
#[derive(Debug)]
pub enum IncidentChange {
    New {
        incident_id: String,
        title: String,
        impact: String,
    },
    StatusChanged {
        incident_id: String,
        title: String,
        old_status: String,
        new_status: String,
    },
    UpdateAdded {
        incident_id: String,
        title: String,
        new_update_count: i32,
    },
}

/// Poll /incidents/unresolved.json and handle incident resolution detection
pub async fn poll(client: &Client, db: &DatabaseConnection) -> Result<()> {
    let url = status_api_url("/incidents/unresolved.json");

    // Fetch API - abort on failure (do NOT modify DB on failure)
    let response: UnresolvedIncidentsResponse = match fetch_json(client, &url).await {
        Ok(r) => r,
        Err(e) => {
            warn!("API fetch failed, skipping resolution detection: {}", e);
            return Err(e);
        }
    };

    let api_ids: HashSet<_> = response.incidents.iter().map(|i| i.id.as_str()).collect();
    let now = Utc::now();

    // Query unresolved incidents from DB
    let unresolved_in_db = incidents::Entity::find()
        .filter(incidents::Column::Status.ne("resolved"))
        .all(db)
        .await?;

    // Mark missing incidents as resolved
    for incident in unresolved_in_db {
        if !api_ids.contains(incident.id.as_str()) {
            let incident_id = incident.id.clone();
            let mut active: incidents::ActiveModel = incident.into();
            active.status = Set("resolved".to_string());
            active.resolved_at = Set(Some(now));
            active.updated_at = Set(now);
            active.update(db).await?;
            info!(incident_id = %incident_id, "Marked incident as resolved");
        }
    }

    // Upsert API response
    for incident in response.incidents {
        upsert_incident(db, &incident).await?;

        // Process incident updates
        for update in &incident.incident_updates {
            upsert_incident_update(db, &incident.id, update).await?;
        }
    }

    Ok(())
}

async fn upsert_incident(
    db: &DatabaseConnection,
    incident: &super::models::Incident,
) -> Result<()> {
    let existing = incidents::Entity::find_by_id(&incident.id).one(db).await?;

    match existing {
        Some(existing) => {
            // Update if status, impact, title, or updated_at changed
            let needs_update = existing.status != incident.status
                || existing.impact != incident.impact
                || existing.title != incident.name
                || existing.updated_at != incident.updated_at;

            if needs_update {
                let mut active: incidents::ActiveModel = existing.into();
                active.title = Set(incident.name.clone());
                active.impact = Set(incident.impact.clone());
                active.status = Set(incident.status.clone());
                active.updated_at = Set(incident.updated_at);
                active.update(db).await?;
                debug!(incident_id = %incident.id, "Updated incident");
            }
        }
        None => {
            // Insert new incident
            let active = incidents::ActiveModel {
                id: Set(incident.id.clone()),
                title: Set(incident.name.clone()),
                impact: Set(incident.impact.clone()),
                status: Set(incident.status.clone()),
                started_at: Set(incident.created_at),
                resolved_at: Set(None),
                created_at: Set(incident.created_at),
                updated_at: Set(incident.updated_at),
            };
            active.insert(db).await?;
            info!(incident_id = %incident.id, title = %incident.name, "Inserted new incident");
        }
    }

    Ok(())
}

/// Poll /incidents.json for full history with snapshot storage
pub async fn poll_history(client: &Client, db: &DatabaseConnection) -> Result<Vec<IncidentChange>> {
    let url = status_api_url("/incidents.json");
    let response: IncidentsResponse = fetch_json(client, &url).await?;

    let now = Utc::now();
    let mut changes = Vec::new();

    for incident in &response.incidents {
        let update_count = incident.incident_updates.len() as i32;
        let raw_json = serde_json::to_string(incident).unwrap_or_else(|_| "{}".to_string());

        // Find latest snapshot for this incident
        let prev = incident_snapshots::Entity::find()
            .filter(incident_snapshots::Column::IncidentId.eq(&incident.id))
            .order_by_desc(incident_snapshots::Column::FetchedAt)
            .one(db)
            .await?;

        let changed = match &prev {
            None => {
                changes.push(IncidentChange::New {
                    incident_id: incident.id.clone(),
                    title: incident.name.clone(),
                    impact: incident.impact.clone(),
                });
                true
            }
            Some(prev) => {
                let mut did_change = false;
                if prev.status != incident.status {
                    changes.push(IncidentChange::StatusChanged {
                        incident_id: incident.id.clone(),
                        title: incident.name.clone(),
                        old_status: prev.status.clone(),
                        new_status: incident.status.clone(),
                    });
                    did_change = true;
                }
                if prev.update_count != update_count {
                    changes.push(IncidentChange::UpdateAdded {
                        incident_id: incident.id.clone(),
                        title: incident.name.clone(),
                        new_update_count: update_count,
                    });
                    did_change = true;
                }
                did_change
            }
        };

        if changed {
            let active = incident_snapshots::ActiveModel {
                incident_id: Set(incident.id.clone()),
                title: Set(incident.name.clone()),
                impact: Set(incident.impact.clone()),
                status: Set(incident.status.clone()),
                started_at: Set(incident.started_at),
                resolved_at: Set(incident.resolved_at),
                update_count: Set(update_count),
                raw_json: Set(raw_json),
                fetched_at: Set(now),
                ..Default::default()
            };
            active.insert(db).await?;
            debug!(incident_id = %incident.id, "Saved incident snapshot");
        }
    }

    if !changes.is_empty() {
        info!(count = changes.len(), "Incident changes detected");
    }

    Ok(changes)
}

async fn upsert_incident_update(
    db: &DatabaseConnection,
    incident_id: &str,
    update: &super::models::IncidentUpdate,
) -> Result<()> {
    // Incident updates are immutable - skip if exists
    let existing = incident_updates::Entity::find_by_id(&update.id)
        .one(db)
        .await?;

    if existing.is_none() {
        let active = incident_updates::ActiveModel {
            id: Set(update.id.clone()),
            incident_id: Set(incident_id.to_string()),
            body: Set(update.body.clone()),
            status: Set(update.status.clone()),
            published_at: Set(update.created_at),
            created_at: Set(update.created_at),
            updated_at: Set(update.created_at),
        };
        active.insert(db).await?;
        debug!(update_id = %update.id, "Inserted incident update");
    }

    Ok(())
}
