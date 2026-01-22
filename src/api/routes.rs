//! API route handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use super::server::SharedState;
use crate::scratch;
use crate::services;

// Request/Response types

#[derive(Debug, Deserialize)]
pub struct CreateScratchRequest {
    pub branch: String,
    pub name: Option<String>,
    pub profile: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub service: Option<String>,
    pub tail: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

// Health check

pub async fn health() -> impl IntoResponse {
    Json(ApiResponse::ok("healthy"))
}

// Scratch routes

pub async fn list_scratches(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    
    match scratch::list_scratches(&state.config, &state.docker).await {
        Ok(scratches) => (StatusCode::OK, Json(ApiResponse::ok(scratches))),
        Err(_e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::ok(Vec::<scratch::ScratchStatus>::new())),
        ),
    }
}

pub async fn create_scratch(
    State(state): State<SharedState>,
    Json(req): Json<CreateScratchRequest>,
) -> impl IntoResponse {
    let state = state.read().await;

    match scratch::create_scratch(
        &state.config,
        &state.docker,
        &req.branch,
        req.name,
        req.profile,
        req.template,
    )
    .await
    {
        Ok(scratch_instance) => (StatusCode::CREATED, Json(ApiResponse::ok(scratch_instance.name))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::ok(e.to_string())),
        ),
    }
}

pub async fn get_scratch(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;

    match scratch::get_scratch_status(&state.config, &state.docker, &name).await {
        Ok(status) => (StatusCode::OK, Json(ApiResponse::ok(status))),
        Err(_e) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::ok(scratch::ScratchStatus::new(name, "unknown".to_string()))),
        ),
    }
}

pub async fn delete_scratch(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;

    match scratch::delete_scratch(&state.config, &state.docker, &name, true).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::ok("deleted".to_string()))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::ok(e.to_string())),
        ),
    }
}

pub async fn start_scratch(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;

    match scratch::start_scratch(&state.config, &state.docker, &name).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::ok("started".to_string()))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::ok(e.to_string())),
        ),
    }
}

pub async fn stop_scratch(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;

    match scratch::stop_scratch(&state.config, &state.docker, &name).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::ok("stopped".to_string()))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::ok(e.to_string())),
        ),
    }
}

pub async fn restart_scratch(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;

    match scratch::restart_scratch(&state.config, &state.docker, &name).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::ok("restarted".to_string()))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::ok(e.to_string())),
        ),
    }
}

pub async fn get_logs(
    State(state): State<SharedState>,
    Path(name): Path<String>,
    Query(query): Query<LogsQuery>,
) -> impl IntoResponse {
    let state = state.read().await;
    let tail = query.tail.unwrap_or(100);

    // Get containers for this scratch
    let containers = match state.docker.list_scratch_containers(Some(&name)).await {
        Ok(c) => c,
        Err(_e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::ok(Vec::<String>::new()))),
    };

    // Filter by service if specified
    let containers: Vec<_> = if let Some(ref svc) = query.service {
        containers
            .into_iter()
            .filter(|c| {
                c.labels
                    .get(&format!("{}.service", state.config.docker.label_prefix))
                    .map(|s| s == svc)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        containers
    };

    let mut all_logs = Vec::new();
    for container in containers {
        if let Ok(logs) = state.docker.get_logs(&container.id, tail).await {
            all_logs.extend(logs);
        }
    }

    (StatusCode::OK, Json(ApiResponse::ok(all_logs)))
}

// Webhook handlers

#[derive(Debug, Deserialize)]
pub struct GithubWebhookPayload {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub action: Option<String>,
    pub pull_request: Option<GithubPullRequest>,
}

#[derive(Debug, Deserialize)]
pub struct GithubPullRequest {
    pub head: GithubRef,
}

#[derive(Debug, Deserialize)]
pub struct GithubRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
}

pub async fn github_webhook(
    State(state): State<SharedState>,
    Json(payload): Json<GithubWebhookPayload>,
) -> impl IntoResponse {
    let state = state.read().await;

    // Extract branch name
    let branch = payload
        .ref_name
        .as_ref()
        .map(|r| r.strip_prefix("refs/heads/").unwrap_or(r).to_string())
        .or_else(|| payload.pull_request.as_ref().map(|pr| pr.head.ref_name.clone()));

    let Some(branch) = branch else {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::ok("no branch found".to_string())));
    };

    tracing::info!("GitHub webhook triggered for branch: {}", branch);

    // Create or update scratch
    let name = crate::scratch::Scratch::sanitize_name(&branch);
    
    match scratch::create_scratch(&state.config, &state.docker, &branch, Some(name), None, None).await {
        Ok(s) => (StatusCode::OK, Json(ApiResponse::ok(s.name))),
        Err(e) => {
            tracing::error!("Failed to create scratch from webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::ok(e.to_string())))
        }
    }
}

// Service routes

pub async fn list_services(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;

    match services::get_shared_services_status(&state.docker).await {
        Ok(status) => (StatusCode::OK, Json(ApiResponse::ok(status))),
        Err(_e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::ok(std::collections::HashMap::<String, String>::new())),
        ),
    }
}

pub async fn start_services(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;

    match services::start_shared_services(&state.config, &state.docker).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::ok("started".to_string()))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::ok(e.to_string())),
        ),
    }
}

pub async fn stop_services(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;

    match services::stop_shared_services(&state.config, &state.docker).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::ok("stopped".to_string()))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::ok(e.to_string())),
        ),
    }
}

// Config routes

pub async fn get_config(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    (StatusCode::OK, Json(ApiResponse::ok(state.config.clone())))
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub server: Option<crate::config::ServerConfig>,
    pub docker: Option<crate::config::DockerConfig>,
    pub nginx: Option<crate::config::NginxConfig>,
    pub github: Option<crate::config::GithubConfig>,
}

pub async fn update_config(
    State(state): State<SharedState>,
    Json(req): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    let mut state = state.write().await;
    
    if let Some(server) = req.server {
        state.config.server = server;
    }
    if let Some(docker) = req.docker {
        state.config.docker = docker;
    }
    if let Some(nginx) = req.nginx {
        state.config.nginx = nginx;
    }
    if let Some(github) = req.github {
        state.config.github = Some(github);
    }
    
    // Persist config to file
    if let Err(e) = crate::config::save_config(&state.config) {
        tracing::error!("Failed to save config: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::err("Failed to save configuration")),
        ).into_response();
    }
    
    (StatusCode::OK, Json(ApiResponse::ok("Config updated and persisted"))).into_response()
}

pub async fn start_service(
    State(state): State<SharedState>,
    Path(service): Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    
    // Check if service exists in config
    if !state.config.services.contains_key(&service) {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::err("Service not found")),
        ).into_response();
    }
    
    // TODO: Implement individual service start
    (StatusCode::OK, Json(ApiResponse::ok("Service started"))).into_response()
}

pub async fn stop_service(
    State(state): State<SharedState>,
    Path(service): Path<String>,
) -> impl IntoResponse {
    let state = state.read().await;
    
    // Check if service exists in config
    if !state.config.services.contains_key(&service) {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::err("Service not found")),
        ).into_response();
    }
    
    // TODO: Implement individual service stop
    (StatusCode::OK, Json(ApiResponse::ok("Service stopped"))).into_response()
}
