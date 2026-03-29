pub use sea_orm_migration::prelude::*;

mod m20260103_001_create_table;
mod m20260108_001_add_language_column;
mod m20260328_001_add_incident_snapshots;
mod m20260328_002_add_incident_translations;
mod m20260328_003_add_ai_insights;
mod m20260329_001_add_insight_i18n;
mod m20260329_003_add_maintenance_snapshots;
mod m20260329_004_add_maintenance_description;
mod m20260330_001_replace_translations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260103_001_create_table::Migration),
            Box::new(m20260108_001_add_language_column::Migration),
            Box::new(m20260328_001_add_incident_snapshots::Migration),
            Box::new(m20260328_002_add_incident_translations::Migration),
            Box::new(m20260328_003_add_ai_insights::Migration),
            Box::new(m20260329_001_add_insight_i18n::Migration),
            Box::new(m20260329_003_add_maintenance_snapshots::Migration),
            Box::new(m20260329_004_add_maintenance_description::Migration),
            Box::new(m20260330_001_replace_translations::Migration),
        ]
    }
}
