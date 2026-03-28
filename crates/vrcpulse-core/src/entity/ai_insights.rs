use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_insights")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub scope: String,
    pub trigger_type: String,
    pub trigger_id: Option<String>,
    pub window_start: DateTimeUtc,
    pub window_end: DateTimeUtc,
    #[sea_orm(column_type = "Text")]
    pub headline: String,
    #[sea_orm(column_type = "Text")]
    pub summary_json: String,
    pub confidence: f64,
    #[sea_orm(column_type = "Text")]
    pub signals_json: String,
    pub model_id: String,
    pub source_hash: String,
    pub created_at: DateTimeUtc,
    pub expires_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
