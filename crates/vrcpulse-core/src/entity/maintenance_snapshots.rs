use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "maintenance_snapshots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub maintenance_id: String,
    pub title: String,
    pub status: String,
    pub scheduled_for: DateTimeUtc,
    pub scheduled_until: DateTimeUtc,
    #[sea_orm(column_type = "Text")]
    pub raw_json: String,
    pub fetched_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
