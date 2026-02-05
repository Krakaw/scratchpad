//! Nginx reload functionality

use crate::config::Config;
use crate::docker::DockerClient;
use crate::error::{Error, Result};

/// Reload nginx configuration
pub async fn reload(config: &Config, docker: &DockerClient) -> Result<()> {
    if !config.nginx.enabled {
        return Ok(());
    }

    // If a custom reload command is specified, use it
    if let Some(reload_cmd) = &config.nginx.reload_command {
        return execute_reload_command(reload_cmd).await;
    }

    // If a container name is specified, exec nginx reload in it
    if let Some(container_name) = &config.nginx.container {
        return reload_via_docker(docker, container_name).await;
    }

    // Try to find an nginx container
    let containers = docker.inner().list_containers::<String>(None).await?;

    for container in containers {
        let name = container
            .names
            .and_then(|n| n.first().cloned())
            .unwrap_or_default();

        if name.contains("nginx") {
            let container_id = container.id.unwrap_or_default();
            return reload_via_docker(docker, &container_id).await;
        }
    }

    tracing::warn!("No nginx container found for reload");
    Ok(())
}

/// Execute a custom reload command
async fn execute_reload_command(command: &str) -> Result<()> {
    use tokio::process::Command;

    let output = Command::new("sh").args(["-c", command]).output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Other(format!(
            "Nginx reload command failed: {}",
            stderr
        )));
    }

    tracing::info!("Nginx reloaded via command: {}", command);
    Ok(())
}

/// Reload nginx by executing a command in the container
async fn reload_via_docker(docker: &DockerClient, container: &str) -> Result<()> {
    let result = docker
        .exec_command(container, vec!["nginx", "-s", "reload"])
        .await;

    match result {
        Ok(_) => {
            tracing::info!("Nginx reloaded in container: {}", container);
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Failed to reload nginx in {}: {}", container, e);
            // Try alternative method
            docker.exec_command(container, vec!["nginx", "-t"]).await?;
            docker
                .exec_command(container, vec!["kill", "-HUP", "1"])
                .await?;
            Ok(())
        }
    }
}

/// Test nginx configuration
#[allow(dead_code)]
pub async fn test_config(config: &Config, docker: &DockerClient) -> Result<()> {
    if let Some(container_name) = &config.nginx.container {
        let output = docker
            .exec_command(container_name, vec!["nginx", "-t"])
            .await?;
        tracing::info!("Nginx config test: {}", output);
    }
    Ok(())
}
