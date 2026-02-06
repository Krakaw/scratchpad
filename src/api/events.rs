//! Docker event streaming and log streaming tasks
//!
//! This module provides background tasks that stream Docker events and container logs
//! to connected WebSocket clients via the broadcast hub.

use crate::docker::DockerClient;
use crate::error::Result;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::websocket::{ServerMessage, WsBroadcastHub};

/// Start background event streaming tasks
///
/// This spawns tasks that monitor Docker events and stream them to WebSocket clients.
pub fn start_event_streaming(hub: Arc<WsBroadcastHub>, docker: Arc<DockerClient>) {
    // Spawn task to monitor container events
    let hub_clone = hub.clone();
    let docker_clone = docker.clone();
    tokio::spawn(async move {
        if let Err(e) = stream_docker_events(hub_clone, docker_clone).await {
            error!("Docker event streaming error: {}", e);
        }
    });

    info!("Event streaming tasks started");
}

/// Stream Docker events and broadcast them to connected clients
async fn stream_docker_events(hub: Arc<WsBroadcastHub>, docker: Arc<DockerClient>) -> Result<()> {
    use bollard::query_parameters::ListContainersOptions;

    debug!("Starting Docker event stream");

    loop {
        // Get current containers
        let containers = docker
            .inner()
            .list_containers(Some(ListContainersOptions {
                all: true,
                ..Default::default()
            }))
            .await?;

        // For now, we'll create a simple polling mechanism
        // In production, you'd use Docker's event stream API directly
        for container in containers {
            if let Some(id) = &container.id {
                // Get container info to extract status and name
                let full_data = docker.inner().inspect_container(id, None).await?;

                let name = full_data
                    .name
                    .as_ref()
                    .map(|n| n.trim_start_matches('/').to_string())
                    .unwrap_or_default();

                let status = full_data
                    .state
                    .and_then(|s| s.status)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Extract scratch name from container name
                // Typically container names are like "scratch-name-service"
                if let Some(scratch_name) = extract_scratch_name(&name) {
                    let service = extract_service_name(&name);

                    // Broadcast status change
                    let msg = ServerMessage::StatusChange {
                        scratch: scratch_name.to_string(),
                        status: status.clone(),
                        service: Some(service),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };

                    // Broadcast to the status channel for this scratch
                    let channel = format!("status:{}", scratch_name);
                    hub.broadcast(&channel, msg).await;
                }
            }
        }

        // Sleep before next poll (adjust interval as needed)
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

/// Extract scratch name from container name
/// Container names are typically formatted as "scratchpad-{scratch_name}-{service}"
fn extract_scratch_name(container_name: &str) -> Option<&str> {
    // Remove leading "scratchpad-" prefix if present
    if let Some(rest) = container_name.strip_prefix("scratchpad-") {
        // Find the last dash to separate scratch name from service
        if let Some(pos) = rest.rfind('-') {
            return Some(&rest[..pos]);
        }
        return Some(rest);
    }

    // Fallback: try to use the whole name (for non-standard naming)
    if !container_name.is_empty() {
        return Some(container_name);
    }

    None
}

/// Extract service name from container name
fn extract_service_name(container_name: &str) -> String {
    // Remove leading "scratchpad-" prefix if present
    if let Some(rest) = container_name.strip_prefix("scratchpad-") {
        // Get everything after the last dash
        if let Some(pos) = rest.rfind('-') {
            return rest[pos + 1..].to_string();
        }
    }

    // Fallback to the whole name
    container_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_scratch_name() {
        assert_eq!(
            extract_scratch_name("scratchpad-my-feature-api"),
            Some("my-feature")
        );
        assert_eq!(extract_scratch_name("scratchpad-test-db"), Some("test"));
        assert_eq!(
            extract_scratch_name("custom-container"),
            Some("custom-container")
        );
        assert_eq!(extract_scratch_name(""), None);
    }

    #[test]
    fn test_extract_service_name() {
        assert_eq!(extract_service_name("scratchpad-my-feature-api"), "api");
        assert_eq!(extract_service_name("scratchpad-test-db"), "db");
        assert_eq!(extract_service_name("custom-container"), "custom-container");
        assert_eq!(extract_service_name(""), "");
    }
}
