use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "incident_translations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub incident_id: String,
    pub locale: String,
    #[sea_orm(column_type = "Text")]
    pub translated_title: String,
    #[sea_orm(column_type = "Text")]
    pub translated_updates: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
