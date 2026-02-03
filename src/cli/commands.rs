//! CLI command implementations

use anyhow::Result;
use std::fs;
use colored::Colorize;

use crate::cli::{error, info, print_scratch_detail, print_scratch_table, success, warn, confirm, NginxAction, OutputFormat, ServicesAction, ConfigAction};
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
        if !confirm(&message)? {
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
            // Generate nginx config before starting services (if nginx is enabled)
            if config.nginx.enabled && config.services.contains_key("nginx") {
                info("Generating nginx configuration...");
                if let Err(e) = nginx::regenerate_config(&config, &docker).await {
                    warn(&format!("Failed to generate nginx config: {}", e));
                }
            }
            
            match services::start_shared_services(&config, &docker).await {
                Ok(_) => success("Started shared services"),
                Err(e) => {
                    let err_str = e.to_string();
                    error(&format!("Failed to start services: {}", e));
                    
                    // Check for common issues and provide helpful messages
                    if err_str.contains("port is already allocated") || err_str.contains("address already in use") {
                        println!();
                        warn("A port conflict was detected. This usually means:");
                        println!("  1. Another service is using the port, OR");
                        println!("  2. An old scratchpad container exists with different port settings");
                        println!();
                        info("To fix, try one of:");
                        println!("  • Change the port in scratchpad.toml");
                        println!("  • Run 'scratchpad services clean' to remove old containers");
                        println!("  • Stop the conflicting service (e.g., local postgres)");
                    }
                    return Err(e.into());
                }
            }
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
        ServicesAction::Clean { force } => {
            let containers = docker.list_shared_service_containers().await?;
            
            if containers.is_empty() {
                info("No shared service containers to clean");
                return Ok(());
            }

            if !force {
                println!("This will remove the following service containers:");
                for c in &containers {
                    let name = c.name.strip_prefix("scratchpad-").unwrap_or(&c.name);
                    println!("  • {} ({})", name, c.state);
                }
                println!();
                
                if !confirm("Are you sure you want to remove these containers?")? {
                    info("Cancelled");
                    return Ok(());
                }
            }

            let removed = services::clean_shared_services(&docker).await?;
            success(&format!("Removed {} service container(s): {}", removed.len(), removed.join(", ")));
            println!();
            info("Run 'scratchpad services start' to recreate with current config");
        }
        ServicesAction::Restart => {
            info("Restarting shared services...");
            services::stop_shared_services(&config, &docker).await?;
            services::start_shared_services(&config, &docker).await?;
            
            // Also start nginx if enabled
            if config.nginx.enabled {
                start_nginx_if_needed(&config, &docker).await?;
            }
            
            success("Restarted shared services");
        }
    }

    Ok(())
}

/// Update an existing scratch environment
pub async fn update(name: &str, restart: bool) -> Result<()> {
    let config = load_config()?;
    let docker = get_docker_client(&config).await?;

    info(&format!("Updating scratch: {}", name));

    match scratch::update_scratch(&config, &docker, name).await {
        Ok(()) => {
            success(&format!("Updated scratch: {}", name));
            
            if restart {
                info("Restarting scratch...");
                scratch::restart_scratch(&config, &docker, name).await?;
                success("Scratch restarted");
            } else {
                info("Run 'scratchpad restart <name>' to apply changes");
            }
            
            Ok(())
        }
        Err(e) => {
            error(&format!("Failed to update scratch: {}", e));
            Err(e.into())
        }
    }
}

/// Configuration management commands
pub async fn config(action: ConfigAction) -> Result<()> {
    use crate::cli::ConfigAction;
    
    match action {
        ConfigAction::Check => {
            println!("{}  Validating configuration...", "→".blue());
            
            match config::load_config() {
                Ok(cfg) => {
                    success("Configuration is valid");
                    println!();
                    
                    // Show summary
                    println!("{}:", "Services".bold());
                    let shared: Vec<_> = cfg.services.iter()
                        .filter(|(_, s)| s.shared)
                        .map(|(n, _)| n.as_str())
                        .collect();
                    let per_scratch: Vec<_> = cfg.services.iter()
                        .filter(|(_, s)| !s.shared)
                        .map(|(n, _)| n.as_str())
                        .collect();
                    
                    if !shared.is_empty() {
                        println!("  Shared: {}", shared.join(", "));
                    }
                    if !per_scratch.is_empty() {
                        println!("  Per-scratch: {}", per_scratch.join(", "));
                    }
                    
                    println!();
                    println!("{}:", "Nginx".bold());
                    println!("  Enabled: {}", cfg.nginx.enabled);
                    if cfg.nginx.enabled {
                        println!("  Domain: {}", cfg.nginx.domain);
                    }
                    
                    println!();
                    println!("{}:", "Docker".bold());
                    println!("  Socket: {}", cfg.docker.socket);
                    println!("  Network: {}", cfg.docker.network);
                    
                    // Validate services have required fields
                    let mut warnings = Vec::new();
                    for (name, svc) in &cfg.services {
                        if svc.image.is_empty() {
                            warnings.push(format!("Service '{}' has no image specified", name));
                        }
                        if !svc.shared && svc.port.is_none() {
                            warnings.push(format!("Per-scratch service '{}' has no port - it won't be accessible", name));
                        }
                    }
                    
                    if !warnings.is_empty() {
                        println!();
                        warn("Warnings:");
                        for w in warnings {
                            println!("  • {}", w);
                        }
                    }
                }
                Err(e) => {
                    error(&format!("Configuration is invalid: {}", e));
                    return Err(anyhow::anyhow!("{}", e));
                }
            }
            
            Ok(())
        }
        ConfigAction::Show => {
            let config_path = std::path::Path::new("scratchpad.toml");
            
            if !config_path.exists() {
                error("No scratchpad.toml found in current directory");
                info("Run 'scratchpad setup' to create one");
                return Ok(());
            }
            
            let content = fs::read_to_string(config_path)?;
            println!("{}", content);
            
            Ok(())
        }
    }
}

/// Start nginx container if not running
async fn start_nginx_if_needed(config: &Config, docker: &DockerClient) -> Result<()> {
    if !config.nginx.enabled {
        return Ok(());
    }

    // Check if nginx container exists in services config
    if let Some(nginx_svc) = config.services.get("nginx") {
        if nginx_svc.shared {
            services::ensure_shared_service_running(config, docker, "nginx").await?;
            return Ok(());
        }
    }

    // If no nginx service defined, try to create a default one
    // For now, just warn
    if config.nginx.container.is_none() {
        warn("Nginx is enabled but no nginx service or container is configured");
        info("Add a shared nginx service to your config, or set nginx.container");
    }

    Ok(())
}

// Helper functions

fn load_config() -> Result<Config> {
    config::load_config().map_err(|e| anyhow::anyhow!("{}", e))
}

async fn get_docker_client(config: &Config) -> Result<DockerClient> {
    match DockerClient::new(config.docker.clone()) {
        Ok(client) => {
            // Test the connection
            if let Err(e) = client.ping().await {
                return Err(anyhow::anyhow!(
                    "Failed to connect to Docker daemon at {}: {}",
                    config.docker.socket,
                    e
                ));
            }
            Ok(client)
        }
        Err(e) => {
            Err(anyhow::anyhow!(
                "Failed to initialize Docker client for socket {}: {}",
                config.docker.socket,
                e
            ))
        }
    }
}

/// Perform health checks and diagnostics
pub async fn doctor() -> Result<()> {
    println!();
    println!("{}", "Scratchpad Health Check".bold().underline());
    println!();

    // 1. Check config file
    println!("{}  Checking configuration...", "→".blue());
    match load_config() {
        Ok(config) => {
            success("Configuration file found and valid");
            println!("    Location: scratchpad.toml");
            println!("    Docker socket: {}", config.docker.socket);
            println!("    Releases directory: {}", config.server.releases_dir.display());

            // 2. Check Docker connection
            println!();
            println!("{}  Checking Docker connection...", "→".blue());
            match get_docker_client(&config).await {
                Ok(docker) => {
                    success("Docker connection successful");
                    
                    // Try to list containers
                    match docker.inner().list_containers::<&str>(None).await {
                        Ok(containers) => {
                            success(&format!("Docker API working ({} containers found)", containers.len()));
                        }
                        Err(e) => {
                            error(&format!("Failed to list containers: {}", e));
                        }
                    }
                }
                Err(e) => {
                    error(&format!("Docker connection failed: {}", e));
                    println!();
                    println!("  {} Make sure:", "⚠".yellow());
                    println!("    1. Docker daemon is running");
                    println!("    2. Socket exists: {}", config.docker.socket);
                    println!("    3. You have permissions to access the socket");
                    println!();
                    println!("  {} Try these commands:", "💡".blue());
                    println!("    - Check if socket exists: ls -la {}", config.docker.socket);
                    println!("    - Check if Docker is running: docker ps");
                    println!("    - Check permissions: id (verify you're in docker group)");
                }
            }

            // 3. Check releases directory
            println!();
            println!("{}  Checking releases directory...", "→".blue());
            match std::fs::metadata(&config.server.releases_dir) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        success(&format!(
                            "Releases directory accessible ({})",
                            config.server.releases_dir.display()
                        ));
                    } else {
                        error("Releases path exists but is not a directory");
                    }
                }
                Err(e) => {
                    warn(&format!(
                        "Releases directory not accessible: {} (will be created when needed)",
                        e
                    ));
                }
            }

            // 4. Check nginx (if enabled)
            if config.nginx.enabled {
                println!();
                println!("{}  Checking nginx...", "→".blue());
                success(&format!("Nginx enabled for domain: {}", config.nginx.domain));
            }

            println!();
            println!("{}", "✓ Health check complete".green());
        }
        Err(e) => {
            error(&format!("Configuration error: {}", e));
            println!();
            println!("  {} Initialize with:", "💡".blue());
            println!("    scratchpad init");
        }
    }

    println!();
    Ok(())
}
