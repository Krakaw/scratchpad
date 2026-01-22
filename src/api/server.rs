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

use super::{events, routes, websocket};

/// Application state shared across handlers
pub struct AppState {
    pub config: Config,
    pub docker: DockerClient,
    pub ws_hub: Arc<websocket::WsBroadcastHub>,
}

pub type SharedState = Arc<RwLock<AppState>>;

/// Run the HTTP API server
pub async fn run_server(config: Config, host: &str, port: u16) -> Result<()> {
    let docker = DockerClient::new(config.docker.clone())?;
    let docker_arc = Arc::new(docker);

    let ws_hub = Arc::new(websocket::WsBroadcastHub::new());

    let state = Arc::new(RwLock::new(AppState {
        config,
        docker: (*docker_arc).clone(),
        ws_hub: ws_hub.clone(),
    }));

    // Start background event streaming tasks
    events::start_event_streaming(ws_hub, docker_arc);

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
        .route("/api/config", get(routes::get_config))
        .route("/api/config", post(routes::update_config))
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
        .route("/api/services/:service/start", post(routes::start_service))
        .route("/api/services/:service/stop", post(routes::stop_service))
        // WebSocket route
        .route("/ws", get(websocket::ws_handler))
        // UI routes
        .route("/", get(crate::ui::dashboard))
        .route("/config", get(crate::ui::config_editor))
        .route("/services", get(crate::ui::service_manager))
        .route("/scratches/create", get(crate::ui::create_scratch))
        .route("/scratches/:name", get(crate::ui::scratch_detail))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
