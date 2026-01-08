//! Database access utilities

use sea_orm::DatabaseConnection;
use serenity::all::Context;
use std::sync::Arc;

use crate::state::AppStateKey;

/// Get database connection from Serenity context
///
/// # Panics
/// Panics if AppState is not found in TypeMap (should never happen after bot initialization)
pub async fn get_db(ctx: &Context) -> Arc<DatabaseConnection> {
    let data = ctx.data.read().await;
    let state = data
        .get::<AppStateKey>()
        .expect("AppState not found in TypeMap");
    state.read().await.database.clone()
}

/// Try to get database connection from Serenity context
///
/// Returns `None` if AppState is not found (useful for non-critical operations like logging)
pub async fn try_get_db(ctx: &Context) -> Option<Arc<DatabaseConnection>> {
    let data = ctx.data.read().await;
    let state = data.get::<AppStateKey>()?;
    Some(state.read().await.database.clone())
}
