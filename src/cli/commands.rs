//! CLI command implementations

use anyhow::Result;
use std::fs;

use crate::cli::{error, info, print_scratch_detail, print_scratch_table, success, warn, confirm, NginxAction, OutputFormat, ServicesAction};
use crate::config::{self, Config};
use crate::docker::DockerClient;
use crate::nginx;
use crate::scratch;
use crate::services;

/// Initialize a new scratchpad.toml configuration file
pub async fn init() -> Result<()> {
    let config_path = std::path::Path::new("scratchpad.toml");

    if config_path.exists() {
        warn("scratchpad.toml already exists");
        return Ok(());
    }

    let content = config::loader::default_config_content();
    fs::write(config_path, content)?;

    success("Created scratchpad.toml");
    info("Edit the configuration file and run 'scratchpad create --branch <branch>' to create your first scratch");

    Ok(())
}

/// Create a new scratch environment
pub async fn create(
    branch: &str,
    name: Option<String>,
    profile: Option<String>,
    template: Option<String>,
) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    info(&format!("Creating scratch from branch: {}", branch));

    match scratch::create_scratch(&config, &docker, branch, name, profile, template).await {
        Ok(scratch_instance) => {
            success(&format!("Created scratch: {}", scratch_instance.name));
            if config.nginx.enabled {
                info(&format!(
                    "Access at: http://{}.{}",
                    scratch_instance.name, config.nginx.domain
                ));
            }
            Ok(())
        }
        Err(e) => {
            error(&format!("Failed to create scratch: {}", e));
            Err(e.into())
        }
    }
}

/// List all scratch environments
pub async fn list(format: OutputFormat) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    let scratches = scratch::list_scratches(&config, &docker).await?;

    match format {
        OutputFormat::Table => {
            print_scratch_table(&scratches);
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&scratches)?;
            println!("{}", json);
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&scratches)?;
            println!("{}", yaml);
        }
    }

    Ok(())
}

/// Start a scratch environment
pub async fn start(name: &str) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    info(&format!("Starting scratch: {}", name));

    match scratch::start_scratch(&config, &docker, name).await {
        Ok(()) => {
            success(&format!("Started scratch: {}", name));
            Ok(())
        }
        Err(e) => {
            error(&format!("Failed to start scratch: {}", e));
            Err(e.into())
        }
    }
}

/// Stop a scratch environment
pub async fn stop(name: &str) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    info(&format!("Stopping scratch: {}", name));

    match scratch::stop_scratch(&config, &docker, name).await {
        Ok(()) => {
            success(&format!("Stopped scratch: {}", name));
            Ok(())
        }
        Err(e) => {
            error(&format!("Failed to stop scratch: {}", e));
            Err(e.into())
        }
    }
}

/// Restart a scratch environment
pub async fn restart(name: &str) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    info(&format!("Restarting scratch: {}", name));

    match scratch::restart_scratch(&config, &docker, name).await {
        Ok(()) => {
            success(&format!("Restarted scratch: {}", name));
            Ok(())
        }
        Err(e) => {
            error(&format!("Failed to restart scratch: {}", e));
            Err(e.into())
        }
    }
}

/// Delete a scratch environment
pub async fn delete(name: &str, force: bool) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    if !force {
        let message = format!(
            "Are you sure you want to delete scratch '{}'? This will also delete associated databases.",
            name
        );
        if !confirm(&message) {
            info("Cancelled");
            return Ok(());
        }
    }

    info(&format!("Deleting scratch: {}", name));

    match scratch::delete_scratch(&config, &docker, name, force).await {
        Ok(()) => {
            success(&format!("Deleted scratch: {}", name));
            Ok(())
        }
        Err(e) => {
            error(&format!("Failed to delete scratch: {}", e));
            Err(e.into())
        }
    }
}

/// View logs from a scratch
pub async fn logs(name: &str, service: Option<String>, follow: bool, tail: usize) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    // Get containers for this scratch
    let containers = docker.list_scratch_containers(Some(name)).await?;

    if containers.is_empty() {
        error(&format!("No containers found for scratch: {}", name));
        return Ok(());
    }

    // Filter by service if specified
    let containers: Vec<_> = if let Some(ref svc) = service {
        containers
            .into_iter()
            .filter(|c| {
                c.labels
                    .get(&format!("{}.service", config.docker.label_prefix))
                    .map(|s| s == svc)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        containers
    };

    if containers.is_empty() {
        error(&format!(
            "No container found for service: {}",
            service.unwrap_or_default()
        ));
        return Ok(());
    }

    for container in containers {
        println!("=== Logs from {} ===", container.name);
        
        if follow {
            // Streaming logs - this will block
            use futures_util::StreamExt;
            use bollard::container::LogsOptions;

            let options = LogsOptions::<String> {
                follow: true,
                stdout: true,
                stderr: true,
                timestamps: true,
                tail: tail.to_string(),
                ..Default::default()
            };

            let mut stream = docker.inner().logs(&container.id, Some(options));

            while let Some(result) = stream.next().await {
                if let Ok(output) = result {
                    print!("{}", output);
                }
            }
        } else {
            let logs = docker.get_logs(&container.id, tail).await?;
            for line in logs {
                println!("{}", line);
            }
        }
    }

    Ok(())
}

/// Show detailed status of a scratch
pub async fn status(name: &str) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    match scratch::get_scratch_status(&config, &docker, name).await {
        Ok(scratch_status) => {
            print_scratch_detail(&scratch_status);
            Ok(())
        }
        Err(e) => {
            error(&format!("Failed to get status: {}", e));
            Err(e.into())
        }
    }
}

/// Start the HTTP API server
pub async fn serve(host: &str, port: u16) -> Result<()> {
    let config = load_config()?;
    
    info(&format!("Starting server at http://{}:{}", host, port));

    crate::api::run_server(config, host, port).await?;
    Ok(())
}

/// Nginx management commands
pub async fn nginx(action: NginxAction) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    match action {
        NginxAction::Generate => {
            nginx::regenerate_config(&config, &docker).await?;
            success("Generated nginx configuration");
        }
        NginxAction::Reload => {
            nginx::reload(&config, &docker).await?;
            success("Reloaded nginx");
        }
        NginxAction::Show => {
            match nginx::get_config(&config) {
                Ok(content) => println!("{}", content),
                Err(_) => warn("No nginx configuration found"),
            }
        }
    }

    Ok(())
}

/// Shared services management commands
pub async fn services(action: ServicesAction) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    match action {
        ServicesAction::Start => {
            services::start_shared_services(&config, &docker).await?;
            success("Started shared services");
        }
        ServicesAction::Stop => {
            services::stop_shared_services(&config, &docker).await?;
            success("Stopped shared services");
        }
        ServicesAction::Status => {
            let status = services::get_shared_services_status(&docker).await?;
            if status.is_empty() {
                info("No shared services running");
            } else {
                println!("Shared Services:");
                for (name, state) in status {
                    let icon = if state == "running" { "●" } else { "○" };
                    println!("  {} {} ({})", icon, name, state);
                }
            }
        }
    }

    Ok(())
}

// Helper functions

fn load_config() -> Result<Config> {
    config::load_config().map_err(|e| anyhow::anyhow!("{}", e))
}

async fn get_docker_client(config: &Config) -> Result<DockerClient> {
    DockerClient::new(config.docker.clone()).map_err(|e| anyhow::anyhow!("{}", e))
}
