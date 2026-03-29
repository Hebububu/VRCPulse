use std::collections::HashSet;

use chrono::Utc;
use reqwest::Client;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{debug, info};

use crate::entity::{maintenance_snapshots, maintenances};

use super::client::{Result, fetch_json, status_api_url};
use super::models::{Maintenance as ApiMaintenance, MaintenancesResponse};

/// Change events emitted by the maintenance poller.
#[derive(Debug, Clone)]
pub enum MaintenanceChange {
    New {
        maintenance_id: String,
        title: String,
    },
    StatusChanged {
        maintenance_id: String,
        title: String,
        old_status: String,
        new_status: String,
    },
    Rescheduled {
        maintenance_id: String,
        title: String,
    },
}

/// Poll /scheduled-maintenances/upcoming.json and /scheduled-maintenances/active.json.
/// Upserts maintenances, stores snapshots on change, and returns change events.
pub async fn poll(client: &Client, db: &DatabaseConnection) -> Result<Vec<MaintenanceChange>> {
    let upcoming_url = status_api_url("/scheduled-maintenances/upcoming.json");
    let active_url = status_api_url("/scheduled-maintenances/active.json");

    let upcoming: MaintenancesResponse = fetch_json(client, &upcoming_url).await?;
    let active: MaintenancesResponse = fetch_json(client, &active_url).await?;

    let now = Utc::now();
    let mut changes = Vec::new();

    // Upsert all from both endpoints
    for m in upcoming
        .scheduled_maintenances
        .iter()
        .chain(active.scheduled_maintenances.iter())
    {
        let change = upsert_maintenance(db, m).await?;
        if let Some(c) = change {
            // Store snapshot on change
            let raw_json = serde_json::to_string(m).unwrap_or_else(|_| "{}".to_string());
            let snapshot = maintenance_snapshots::ActiveModel {
                maintenance_id: Set(m.id.clone()),
                title: Set(m.name.clone()),
                status: Set(m.status.clone()),
                scheduled_for: Set(m.scheduled_for),
                scheduled_until: Set(m.scheduled_until),
                raw_json: Set(raw_json),
                fetched_at: Set(now),
                ..Default::default()
            };
            snapshot.insert(db).await?;
            changes.push(c);
        }
    }

    // Check for completed maintenances
    let active_ids: HashSet<_> = active
        .scheduled_maintenances
        .iter()
        .map(|m| m.id.as_str())
        .collect();

    let in_progress_in_db = maintenances::Entity::find()
        .filter(maintenances::Column::Status.eq("in_progress"))
        .all(db)
        .await?;

    for m in in_progress_in_db {
        if !active_ids.contains(m.id.as_str()) && now > m.scheduled_until {
            let maintenance_id = m.id.clone();
            let title = m.title.clone();
            let mut active_model: maintenances::ActiveModel = m.into();
            active_model.status = Set("completed".to_string());
            active_model.updated_at = Set(now);
            active_model.update(db).await?;
            info!(maintenance_id = %maintenance_id, "Marked maintenance as completed");

            // Store completion snapshot
            let snapshot = maintenance_snapshots::ActiveModel {
                maintenance_id: Set(maintenance_id.clone()),
                title: Set(title.clone()),
                status: Set("completed".to_string()),
                scheduled_for: Set(now), // use now as placeholder
                scheduled_until: Set(now),
                raw_json: Set("{}".to_string()),
                fetched_at: Set(now),
                ..Default::default()
            };
            snapshot.insert(db).await?;

            changes.push(MaintenanceChange::StatusChanged {
                maintenance_id,
                title,
                old_status: "in_progress".to_string(),
                new_status: "completed".to_string(),
            });
        }
    }

    // Check for skipped maintenances (scheduled -> completed without in_progress)
    let scheduled_in_db = maintenances::Entity::find()
        .filter(maintenances::Column::Status.eq("scheduled"))
        .all(db)
        .await?;

    for m in scheduled_in_db {
        if now > m.scheduled_until {
            let maintenance_id = m.id.clone();
            let title = m.title.clone();
            let mut active_model: maintenances::ActiveModel = m.into();
            active_model.status = Set("completed".to_string());
            active_model.updated_at = Set(now);
            active_model.update(db).await?;
            info!(maintenance_id = %maintenance_id, "Marked skipped maintenance as completed");

            let snapshot = maintenance_snapshots::ActiveModel {
                maintenance_id: Set(maintenance_id.clone()),
                title: Set(title.clone()),
                status: Set("completed".to_string()),
                scheduled_for: Set(now),
                scheduled_until: Set(now),
                raw_json: Set("{}".to_string()),
                fetched_at: Set(now),
                ..Default::default()
            };
            snapshot.insert(db).await?;

            changes.push(MaintenanceChange::StatusChanged {
                maintenance_id,
                title,
                old_status: "scheduled".to_string(),
                new_status: "completed".to_string(),
            });
        }
    }

    if !changes.is_empty() {
        info!(count = changes.len(), "Maintenance changes detected");
    }

    Ok(changes)
}

/// Upsert a maintenance record. Returns a change event if something changed.
async fn upsert_maintenance(
    db: &DatabaseConnection,
    m: &ApiMaintenance,
) -> Result<Option<MaintenanceChange>> {
    let existing = maintenances::Entity::find_by_id(&m.id).one(db).await?;

    match existing {
        Some(existing) => {
            let status_changed = existing.status != m.status;
            let rescheduled = existing.scheduled_for != m.scheduled_for
                || existing.scheduled_until != m.scheduled_until;

            if status_changed || rescheduled || existing.title != m.name {
                let old_status = existing.status.clone();
                let mut active: maintenances::ActiveModel = existing.into();
                active.title = Set(m.name.clone());
                active.status = Set(m.status.clone());
                active.scheduled_for = Set(m.scheduled_for);
                active.scheduled_until = Set(m.scheduled_until);
                active.updated_at = Set(m.updated_at);
                active.update(db).await?;
                debug!(maintenance_id = %m.id, status = %m.status, "Updated maintenance");

                if status_changed {
                    return Ok(Some(MaintenanceChange::StatusChanged {
                        maintenance_id: m.id.clone(),
                        title: m.name.clone(),
                        old_status,
                        new_status: m.status.clone(),
                    }));
                } else if rescheduled {
                    return Ok(Some(MaintenanceChange::Rescheduled {
                        maintenance_id: m.id.clone(),
                        title: m.name.clone(),
                    }));
                }
            }
            Ok(None)
        }
        None => {
            let active = maintenances::ActiveModel {
                id: Set(m.id.clone()),
                title: Set(m.name.clone()),
                status: Set(m.status.clone()),
                scheduled_for: Set(m.scheduled_for),
                scheduled_until: Set(m.scheduled_until),
                created_at: Set(m.created_at),
                updated_at: Set(m.updated_at),
            };
            active.insert(db).await?;
            info!(maintenance_id = %m.id, title = %m.name, "Inserted new maintenance");

            Ok(Some(MaintenanceChange::New {
                maintenance_id: m.id.clone(),
                title: m.name.clone(),
            }))
        }
    }
}
