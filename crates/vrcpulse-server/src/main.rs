use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database};
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;

use vrcpulse_core::VrcPulseService;

struct AppState {
    service: VrcPulseService,
}

#[derive(Deserialize)]
struct RangeQuery {
    range: Option<String>,
}

#[derive(Deserialize)]
struct StatusFilterQuery {
    status: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,vrcpulse=debug".into()),
        )
        .init();

    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Failed to load .env file: {e}");
    }

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/vrcpulse.db".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    // Connect to database
    let mut db_opts = ConnectOptions::new(&database_url);
    db_opts
        .max_connections(5)
        .min_connections(1)
        .sqlx_logging(false);

    let database = Database::connect(db_opts)
        .await
        .expect("Failed to connect to database");

    database
        .execute_unprepared("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
        .await
        .expect("Failed to set SQLite pragmas");

    info!("Database connected");

    // Run migrations
    use sea_orm_migration::MigratorTrait;
    migration::Migrator::up(&database, None)
        .await
        .expect("Failed to run migrations");
    info!("Migrations applied");

    // Start collector in background
    let (_config_tx, config_rx) = vrcpulse_core::collector::config::init(&database)
        .await
        .expect("Failed to load collector config");

    let http_client = reqwest::Client::builder()
        .user_agent("vrcpulse-server/1.0.0")
        .build()
        .expect("Failed to create HTTP client");

    tokio::spawn(vrcpulse_core::collector::start(
        http_client,
        database.clone(),
        config_rx,
    ));

    info!("Collector started");

    let state = Arc::new(AppState {
        service: VrcPulseService::new(database),
    });

    // Build router
    let api_routes = Router::new()
        .route("/status", get(get_status))
        .route("/metrics/dashboard", get(get_dashboard))
        .route("/metrics/{name}", get(get_metrics))
        .route("/incidents", get(get_incidents))
        .route("/maintenances", get(get_maintenances));

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new("static").fallback(get(|| async {
            match tokio::fs::read_to_string("static/index.html").await {
                Ok(html) => axum::response::Html(html).into_response(),
                Err(_) => (
                    StatusCode::OK,
                    "VRCPulse Server is running. No static files found.",
                )
                    .into_response(),
            }
        })))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind");

    info!("VRCPulse server listening on http://0.0.0.0:{port}");

    axum::serve(listener, app).await.expect("Server error");
}

async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.service.get_status().await {
        Ok(status) => Json(status).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_metrics(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let range = params.range.as_deref().unwrap_or("12h");
    match state.service.get_metrics(&name, range).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let range = params.range.as_deref().unwrap_or("12h");
    match state.service.get_dashboard(range).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_incidents(
    State(state): State<Arc<AppState>>,
    Query(params): Query<StatusFilterQuery>,
) -> impl IntoResponse {
    let status = params.status.as_deref().unwrap_or("active");
    match state.service.get_incidents(status).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_maintenances(
    State(state): State<Arc<AppState>>,
    Query(params): Query<StatusFilterQuery>,
) -> impl IntoResponse {
    let status = params.status.as_deref().unwrap_or("upcoming");
    match state.service.get_maintenances(status).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
