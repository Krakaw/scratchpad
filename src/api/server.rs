//! HTTP API server

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::docker::DockerClient;
use crate::error::Result;

use super::routes;

/// Application state shared across handlers
pub struct AppState {
    pub config: Config,
    pub docker: DockerClient,
}

pub type SharedState = Arc<RwLock<AppState>>;

/// Run the HTTP API server
pub async fn run_server(config: Config, host: &str, port: u16) -> Result<()> {
    let docker = DockerClient::new(config.docker.clone())?;

    let state = Arc::new(RwLock::new(AppState { config, docker }));

    let app = create_router(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the router with all routes
fn create_router(state: SharedState) -> Router {
    Router::new()
        // API routes
        .route("/api/health", get(routes::health))
        .route("/api/scratches", get(routes::list_scratches))
        .route("/api/scratches", post(routes::create_scratch))
        .route("/api/scratches/:name", get(routes::get_scratch))
        .route("/api/scratches/:name", delete(routes::delete_scratch))
        .route("/api/scratches/:name/start", post(routes::start_scratch))
        .route("/api/scratches/:name/stop", post(routes::stop_scratch))
        .route("/api/scratches/:name/restart", post(routes::restart_scratch))
        .route("/api/scratches/:name/logs", get(routes::get_logs))
        // Webhook routes
        .route("/api/webhooks/github", post(routes::github_webhook))
        // Service routes
        .route("/api/services", get(routes::list_services))
        .route("/api/services/start", post(routes::start_services))
        .route("/api/services/stop", post(routes::stop_services))
        // UI routes
        .route("/", get(crate::ui::dashboard))
        .route("/scratches/:name", get(crate::ui::scratch_detail))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
