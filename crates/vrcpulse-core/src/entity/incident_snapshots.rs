use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "incident_snapshots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub incident_id: String,
    pub title: String,
    pub impact: String,
    pub status: String,
    pub started_at: DateTimeUtc,
    pub resolved_at: Option<DateTimeUtc>,
    pub update_count: i32,
    #[sea_orm(column_type = "Text")]
    pub raw_json: String,
    pub fetched_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
