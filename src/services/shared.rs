//! Shared service management

use std::collections::HashMap;

use crate::config::Config;
use crate::docker::DockerClient;
use crate::error::{Error, Result};

/// Ensure a shared service is running
pub async fn ensure_shared_service_running(
    config: &Config,
    docker: &DockerClient,
    service_name: &str,
) -> Result<()> {
    let service_config = config
        .get_service(service_name)
        .ok_or_else(|| Error::ServiceNotFound(service_name.to_string()))?;

    if !service_config.shared {
        return Err(Error::Config(format!(
            "Service {} is not configured as shared",
            service_name
        )));
    }

    let container_name = format!("scratchpad-{}", service_name);
    let label_prefix = &config.docker.label_prefix;

    // Check if container already exists
    let containers = docker.list_shared_service_containers().await?;
    let existing = containers
        .iter()
        .find(|c| c.name == container_name);

    if let Some(container) = existing {
        if container.state == "running" {
            tracing::debug!("Shared service {} is already running", service_name);
            return Ok(());
        }

        // Start existing container
        docker.start_container(&container.id).await?;
        tracing::info!("Started shared service: {}", service_name);
        return Ok(());
    }

    // Create and start the container
    let mut labels = HashMap::new();
    labels.insert(
        format!("{}.shared-service", label_prefix),
        service_name.to_string(),
    );

    let env: Vec<String> = service_config
        .env
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    // Determine internal port (container port) - use internal_port if set,
    // otherwise derive from known images, or fall back to host port
    let internal_port = service_config.internal_port.or_else(|| {
        // Known default ports for common images
        let image_lower = service_config.image.to_lowercase();
        if image_lower.contains("postgres") {
            Some(5432)
        } else if image_lower.contains("mysql") || image_lower.contains("mariadb") {
            Some(3306)
        } else if image_lower.contains("redis") {
            Some(6379)
        } else if image_lower.contains("mongo") {
            Some(27017)
        } else if image_lower.contains("kafka") {
            Some(9092)
        } else {
            service_config.port
        }
    });

    let ports: Vec<(u16, u16)> = match (service_config.port, internal_port) {
        (Some(host), Some(container)) => vec![(host, container)],
        (Some(p), None) => vec![(p, p)],
        (None, Some(p)) => vec![(p, p)],
        (None, None) => vec![],
    };

    let volumes = service_config.volumes.clone();

    docker
        .create_container(
            &container_name,
            &service_config.image,
            env,
            labels,
            ports,
            volumes,
            Some(&config.docker.network),
            service_config.healthcheck.as_deref(),
        )
        .await?;

    tracing::info!("Created shared service: {}", service_name);

    // Wait for healthcheck if configured
    if service_config.healthcheck.is_some() {
        wait_for_healthy(docker, &container_name, 30).await?;
    }

    Ok(())
}

/// Wait for a container to become healthy
async fn wait_for_healthy(docker: &DockerClient, container_name: &str, timeout_secs: u32) -> Result<()> {
    use tokio::time::{sleep, Duration};

    for _ in 0..timeout_secs {
        let info = docker.inner().inspect_container(container_name, None).await?;
        
        if let Some(state) = info.state {
            if let Some(health) = state.health {
                if health.status == Some(bollard::models::HealthStatusEnum::HEALTHY) {
                    return Ok(());
                }
            } else {
                // No healthcheck configured, just check if running
                if state.running == Some(true) {
                    return Ok(());
                }
            }
        }

        sleep(Duration::from_secs(1)).await;
    }

    Err(Error::Other(format!(
        "Timeout waiting for {} to become healthy",
        container_name
    )))
}

/// Stop all shared services
pub async fn stop_shared_services(_config: &Config, docker: &DockerClient) -> Result<()> {
    let containers = docker.list_shared_service_containers().await?;

    for container in containers {
        docker.stop_container(&container.id).await?;
        tracing::info!("Stopped shared service: {}", container.name);
    }

    Ok(())
}

/// Start all shared services
pub async fn start_shared_services(config: &Config, docker: &DockerClient) -> Result<()> {
    // Ensure network exists
    docker.ensure_network().await?;

    for (name, service_config) in &config.services {
        if service_config.shared {
            ensure_shared_service_running(config, docker, name).await?;
        }
    }

    Ok(())
}

/// Get status of all shared services
pub async fn get_shared_services_status(
    docker: &DockerClient,
) -> Result<HashMap<String, String>> {
    let containers = docker.list_shared_service_containers().await?;
    let mut status = HashMap::new();

    for container in containers {
        let service_name = container.name.strip_prefix("scratchpad-").unwrap_or(&container.name);
        status.insert(service_name.to_string(), container.state);
    }

    Ok(status)
}

/// Start a specific shared service
pub async fn start_service(
    config: &Config,
    docker: &DockerClient,
    service_name: &str,
) -> Result<()> {
    // Ensure network exists
    docker.ensure_network().await?;

    // Ensure the service is running (creates it if needed)
    ensure_shared_service_running(config, docker, service_name).await
}

/// Stop a specific shared service
pub async fn stop_service(
    docker: &DockerClient,
    service_name: &str,
) -> Result<()> {
    let container_name = format!("scratchpad-{}", service_name);
    let containers = docker.list_shared_service_containers().await?;
    
    let container = containers
        .iter()
        .find(|c| c.name == container_name)
        .ok_or_else(|| Error::ServiceNotFound(service_name.to_string()))?;

    docker.stop_container(&container.id).await?;
    tracing::info!("Stopped shared service: {}", service_name);

    Ok(())
}

