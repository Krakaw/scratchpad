//! Scratch lifecycle management (create, start, stop, delete)

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::{Config, ScratchConfig};
use crate::docker::{ComposeFile, DockerClient};
use crate::error::{Error, Result};
use crate::nginx;
use crate::services;

use super::{Scratch, ScratchStatus};

/// Create a new scratch environment
pub async fn create_scratch(
    config: &Config,
    docker: &DockerClient,
    branch: &str,
    name: Option<String>,
    profile: Option<String>,
    template: Option<String>,
) -> Result<Scratch> {
    let scratch_name = name.unwrap_or_else(|| Scratch::sanitize_name(branch));

    // Validate name
    if scratch_name.is_empty() {
        return Err(Error::InvalidScratchName(
            "Scratch name cannot be empty".to_string(),
        ));
    }

    tracing::debug!("Creating scratch '{}' from branch '{}'", scratch_name, branch);

    // Check if scratch already exists
    let releases_dir = &config.server.releases_dir;
    let scratch_dir = releases_dir.join(&scratch_name);

    if scratch_dir.exists() {
        return Err(Error::ScratchAlreadyExists(scratch_name));
    }

    // Determine services to use
    let services = if let Some(profile_name) = &profile {
        tracing::debug!("Using profile: {}", profile_name);
        config
            .get_profile(profile_name)
            .map(|p| p.services.clone())
            .unwrap_or_else(|| config.scratch.services.clone())
    } else {
        config.scratch.services.clone()
    };
    tracing::debug!("Services to include: {:?}", services);

    // Determine template to use
    let template_name = template
        .or_else(|| {
            profile.as_ref().and_then(|p| {
                config
                    .get_profile(p)
                    .and_then(|prof| prof.template.clone())
            })
        })
        .unwrap_or_else(|| config.scratch.template.clone());
    tracing::debug!("Using template: {}", template_name);

    // Create scratch instance
    let mut scratch = Scratch::new(scratch_name.clone(), branch.to_string(), template_name.clone());
    scratch.services = services.clone();

    // Create directory structure
    tracing::debug!("Creating directory structure at {}", scratch_dir.display());
    create_scratch_directories(&scratch_dir)?;

    // Ensure network exists
    tracing::debug!("Ensuring Docker network exists");
    docker.ensure_network().await?;

    // Provision shared services and databases
    let mut databases: HashMap<String, Vec<String>> = HashMap::new();
    for service_name in &services {
        if let Some(service_config) = config.get_service(service_name) {
            if service_config.shared && service_config.auto_create_db {
                // For postgres, create a database
                if service_name == "postgres" {
                    let db_name = format!("scratch_{}", scratch_name);
                    tracing::debug!("Ensuring PostgreSQL service is running");
                    services::ensure_shared_service_running(config, docker, service_name).await?;
                    tracing::debug!("Creating database: {}", db_name);
                    services::create_postgres_database(config, &db_name).await?;
                    databases
                        .entry(service_name.clone())
                        .or_default()
                        .push(db_name);
                }
            }
        }
    }
    scratch.databases = databases;

    // Render and save compose file
    tracing::debug!("Rendering docker-compose file");
    let compose = render_compose_file(config, &scratch)?;
    let compose_path = scratch_dir.join("compose.yml");
    tracing::debug!("Saving docker-compose file to {}", compose_path.display());
    compose.save(&compose_path)?;

    // Save scratch config
    tracing::debug!("Saving scratch configuration");
    let scratch_config = ScratchConfig {
        name: scratch.name.clone(),
        branch: scratch.branch.clone(),
        template: scratch.template.clone(),
        services: scratch.services.clone(),
        databases: scratch.databases.clone(),
        env: scratch.env.clone(),
        created_at: scratch.created_at,
    };
    let config_path = scratch_dir.join(".scratchpad.toml");
    let config_content = toml::to_string_pretty(&scratch_config)
        .map_err(|e| Error::Config(e.to_string()))?;
    fs::write(&config_path, config_content)?;

    // Start the scratch (run docker compose up)
    tracing::info!("Starting containers for scratch '{}'", scratch_name);
    start_scratch_compose(&scratch_dir).await?;

    // Update nginx config
    if config.nginx.enabled {
        tracing::debug!("Regenerating nginx configuration");
        nginx::regenerate_config(config, docker).await?;
        tracing::debug!("Reloading nginx");
        nginx::reload(config, docker).await?;
    }

    tracing::info!("Successfully created scratch: {}", scratch_name);
    Ok(scratch)
}

/// Create the directory structure for a scratch
fn create_scratch_directories(scratch_dir: &Path) -> Result<()> {
    fs::create_dir_all(scratch_dir)?;
    fs::create_dir_all(scratch_dir.join("logs"))?;
    fs::create_dir_all(scratch_dir.join("data"))?;
    Ok(())
}

/// Render the docker-compose file for a scratch
fn render_compose_file(config: &Config, scratch: &Scratch) -> Result<ComposeFile> {
    use super::template::render_template;

    let compose_content = render_template(config, scratch)?;
    let mut compose: ComposeFile = serde_yaml::from_str(&compose_content)?;

    // Add labels and network
    compose.add_labels(&scratch.name, &config.docker.label_prefix);
    compose.add_network(&config.docker.network);

    Ok(compose)
}

/// Start a scratch using docker compose
async fn start_scratch_compose(scratch_dir: &Path) -> Result<()> {
    use tokio::process::Command;

    tracing::debug!("Running 'docker compose up' in {}", scratch_dir.display());
    let output = Command::new("docker")
        .args(["compose", "up", "-d"])
        .current_dir(scratch_dir)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Docker compose up failed: {}", stderr);
        return Err(Error::Other(format!(
            "Failed to start scratch: {}",
            stderr
        )));
    }

    tracing::debug!("Docker compose up completed successfully");
    Ok(())
}

/// Stop a scratch using docker compose
async fn stop_scratch_compose(scratch_dir: &Path) -> Result<()> {
    use tokio::process::Command;

    tracing::debug!("Running 'docker compose down' in {}", scratch_dir.display());
    let output = Command::new("docker")
        .args(["compose", "down"])
        .current_dir(scratch_dir)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Docker compose down failed: {}", stderr);
        return Err(Error::Other(format!(
            "Failed to stop scratch: {}",
            stderr
        )));
    }

    tracing::debug!("Docker compose down completed successfully");
    Ok(())
}

/// Start a stopped scratch
pub async fn start_scratch(config: &Config, _docker: &DockerClient, name: &str) -> Result<()> {
    let scratch_dir = config.server.releases_dir.join(name);

    if !scratch_dir.exists() {
        return Err(Error::ScratchNotFound(name.to_string()));
    }

    tracing::info!("Starting scratch: {}", name);
    start_scratch_compose(&scratch_dir).await?;
    tracing::info!("Successfully started scratch: {}", name);
    Ok(())
}

/// Stop a running scratch
pub async fn stop_scratch(config: &Config, _docker: &DockerClient, name: &str) -> Result<()> {
    let scratch_dir = config.server.releases_dir.join(name);

    if !scratch_dir.exists() {
        return Err(Error::ScratchNotFound(name.to_string()));
    }

    tracing::info!("Stopping scratch: {}", name);
    stop_scratch_compose(&scratch_dir).await?;
    tracing::info!("Successfully stopped scratch: {}", name);
    Ok(())
}

/// Restart a scratch
pub async fn restart_scratch(config: &Config, docker: &DockerClient, name: &str) -> Result<()> {
    stop_scratch(config, docker, name).await?;
    start_scratch(config, docker, name).await?;
    Ok(())
}

/// Delete a scratch environment
pub async fn delete_scratch(
    config: &Config,
    docker: &DockerClient,
    name: &str,
    _force: bool,
) -> Result<()> {
    let scratch_dir = config.server.releases_dir.join(name);

    if !scratch_dir.exists() {
        return Err(Error::ScratchNotFound(name.to_string()));
    }

    tracing::info!("Deleting scratch: {}", name);

    // Load scratch config to find databases
    let config_path = scratch_dir.join(".scratchpad.toml");
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        if let Ok(scratch_config) = toml::from_str::<ScratchConfig>(&content) {
            // Drop databases
            for (service, dbs) in &scratch_config.databases {
                if service == "postgres" {
                    for db in dbs {
                        tracing::debug!("Dropping database: {}", db);
                        if let Err(e) = services::drop_postgres_database(config, db).await {
                            tracing::warn!("Failed to drop database {}: {}", db, e);
                        }
                    }
                }
            }
        }
    }

    // Stop containers
    tracing::debug!("Stopping containers");
    stop_scratch_compose(&scratch_dir).await?;

    // Remove directory
    tracing::debug!("Removing scratch directory");
    fs::remove_dir_all(&scratch_dir)?;

    // Update nginx config
    if config.nginx.enabled {
        tracing::debug!("Regenerating nginx configuration");
        nginx::regenerate_config(config, docker).await?;
        tracing::debug!("Reloading nginx");
        nginx::reload(config, docker).await?;
    }

    tracing::info!("Successfully deleted scratch: {}", name);
    Ok(())
}

/// List all scratches with their status
pub async fn list_scratches(config: &Config, docker: &DockerClient) -> Result<Vec<ScratchStatus>> {
    let releases_dir = &config.server.releases_dir;

    if !releases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut scratches = Vec::new();

    // Get container statuses
    let containers = docker.list_scratch_containers(None).await?;

    // Read all scratch directories
    for entry in fs::read_dir(releases_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let config_path = entry.path().join(".scratchpad.toml");

        let mut status = ScratchStatus::new(name.clone(), name.clone());

        // Load config if exists
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(scratch_config) = toml::from_str::<ScratchConfig>(&content) {
                    status.branch = scratch_config.branch;
                    status.created_at = Some(scratch_config.created_at);
                    status.databases = scratch_config
                        .databases
                        .values()
                        .flatten()
                        .cloned()
                        .collect();
                }
            }
        }

        // Get service statuses from containers
        for container in &containers {
            if let Some(scratch_name) = container.labels.get(&format!(
                "{}.scratch",
                config.docker.label_prefix
            )) {
                if scratch_name == &name {
                    if let Some(service_name) = container.labels.get(&format!(
                        "{}.service",
                        config.docker.label_prefix
                    )) {
                        status
                            .services
                            .insert(service_name.clone(), container.state.clone());
                    }
                }
            }
        }

        // Calculate overall status
        status.calculate_status();

        // Set URL
        if config.nginx.enabled {
            status.url = Some(format!(
                "http://{}.{}",
                name, config.nginx.domain
            ));
        }

        scratches.push(status);
    }

    // Sort by name
    scratches.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(scratches)
}

/// Get status of a single scratch
pub async fn get_scratch_status(
    config: &Config,
    docker: &DockerClient,
    name: &str,
) -> Result<ScratchStatus> {
    let scratches = list_scratches(config, docker).await?;
    scratches
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| Error::ScratchNotFound(name.to_string()))
}
