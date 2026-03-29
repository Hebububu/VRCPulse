use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "translations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub item_type: String,
    pub item_id: String,
    pub locale: String,
    pub content_hash: String,
    #[sea_orm(column_type = "Text")]
    pub translated_name: String,
    #[sea_orm(column_type = "Text")]
    pub translated_body: String,
    #[sea_orm(column_type = "Text")]
    pub translated_updates: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
