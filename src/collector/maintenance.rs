use std::collections::HashSet;

use chrono::Utc;
use reqwest::Client;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{debug, info};

use crate::entity::maintenances;

use super::client::{Result, fetch_json, status_api_url};
use super::models::{Maintenance as ApiMaintenance, MaintenancesResponse};

/// Poll /scheduled-maintenances/upcoming.json and /scheduled-maintenances/active.json
pub async fn poll(client: &Client, db: &DatabaseConnection) -> Result<()> {
    let upcoming_url = status_api_url("/scheduled-maintenances/upcoming.json");
    let active_url = status_api_url("/scheduled-maintenances/active.json");

    let upcoming: MaintenancesResponse = fetch_json(client, &upcoming_url).await?;
    let active: MaintenancesResponse = fetch_json(client, &active_url).await?;

    let now = Utc::now();

    // Upsert all from both endpoints
    for m in upcoming
        .scheduled_maintenances
        .iter()
        .chain(active.scheduled_maintenances.iter())
    {
        upsert_maintenance(db, m).await?;
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
        // Disappears from /active.json AND NOW() > scheduled_until
        if !active_ids.contains(m.id.as_str()) && now > m.scheduled_until {
            let maintenance_id = m.id.clone();
            let mut active_model: maintenances::ActiveModel = m.into();
            active_model.status = Set("completed".to_string());
            active_model.updated_at = Set(now);
            active_model.update(db).await?;
            info!(maintenance_id = %maintenance_id, "Marked maintenance as completed");
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
            let mut active_model: maintenances::ActiveModel = m.into();
            active_model.status = Set("completed".to_string());
            active_model.updated_at = Set(now);
            active_model.update(db).await?;
            info!(
                maintenance_id = %maintenance_id,
                "Marked skipped maintenance as completed"
            );
        }
    }

    Ok(())
}

async fn upsert_maintenance(db: &DatabaseConnection, m: &ApiMaintenance) -> Result<()> {
    let existing = maintenances::Entity::find_by_id(&m.id).one(db).await?;

    match existing {
        Some(existing) => {
            // Update if status, scheduled_for, or scheduled_until changed
            if should_update(&existing, m) {
                let mut active: maintenances::ActiveModel = existing.into();
                active.title = Set(m.name.clone());
                active.status = Set(m.status.clone());
                active.scheduled_for = Set(m.scheduled_for);
                active.scheduled_until = Set(m.scheduled_until);
                active.updated_at = Set(m.updated_at);
                active.update(db).await?;
                debug!(maintenance_id = %m.id, status = %m.status, "Updated maintenance");
            }
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
        }
    }

    Ok(())
}

fn should_update(existing: &maintenances::Model, incoming: &ApiMaintenance) -> bool {
    existing.status != incoming.status
        || existing.scheduled_for != incoming.scheduled_for
        || existing.scheduled_until != incoming.scheduled_until
}
