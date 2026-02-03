//! Interactive setup wizard for Scratchpad
//!
//! Guides users through first-time setup with:
//! - Pre-flight checks (Docker, permissions)
//! - Interactive configuration
//! - Demo scratch creation
//! - Verification

use anyhow::Result;
use colored::Colorize;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::config::{
    Config, DockerConfig, NginxConfig, NginxRouting, ScratchDefaults, ScratchProfile,
    ServerConfig, ServiceConfig,
};
use crate::docker::DockerClient;

/// Run the interactive setup wizard
pub async fn run_setup_wizard(non_interactive: bool) -> Result<()> {
    let term = Term::stdout();
    let theme = ColorfulTheme::default();

    // Clear screen and show welcome
    let _ = term.clear_screen();
    print_welcome();

    if non_interactive {
        return run_quick_setup().await;
    }

    // Check if config already exists
    let config_path = Path::new("scratchpad.toml");
    if config_path.exists() {
        println!();
        let overwrite = Confirm::with_theme(&theme)
            .with_prompt("scratchpad.toml already exists. Overwrite it?")
            .default(false)
            .interact()?;

        if !overwrite {
            println!();
            println!(
                "{}",
                "Setup cancelled. Run 'scratchpad doctor' to check your existing config.".yellow()
            );
            return Ok(());
        }
    }

    println!();
    println!(
        "{}",
        "Let's check your system first...".bold()
    );
    println!();

    // Run pre-flight checks
    let preflight_ok = run_preflight_checks().await?;
    if !preflight_ok {
        println!();
        let continue_anyway = Confirm::with_theme(&theme)
            .with_prompt("Some checks failed. Continue anyway?")
            .default(false)
            .interact()?;

        if !continue_anyway {
            return Ok(());
        }
    }

    println!();
    println!("{}", "Great! Let's configure Scratchpad.".bold());
    println!();

    // Gather configuration through prompts
    let config = gather_config(&theme)?;

    // Show summary
    println!();
    println!("{}", "Configuration Summary".bold().underline());
    println!();
    print_config_summary(&config);

    let confirm = Confirm::with_theme(&theme)
        .with_prompt("Save this configuration?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("{}", "Setup cancelled.".yellow());
        return Ok(());
    }

    // Write config
    write_config(&config)?;
    println!();
    println!("{} Created scratchpad.toml", "✓".green());

    // Create necessary directories
    create_directories(&config)?;

    // Offer to create demo scratch
    println!();
    let create_demo = Confirm::with_theme(&theme)
        .with_prompt("Create a demo scratch to verify everything works?")
        .default(true)
        .interact()?;

    if create_demo {
        create_demo_scratch(&config).await?;
    }

    // Final message
    println!();
    println!("{}", "🎉 Setup complete!".green().bold());
    println!();
    println!("Next steps:");
    println!("  {} Create scratches:", "1.".bold());
    println!("     scratchpad create --branch feature/my-branch");
    println!();
    println!("  {} Start the web UI:", "2.".bold());
    println!("     scratchpad serve");
    println!();
    println!("  {} View your scratches:", "3.".bold());
    println!("     scratchpad list");
    println!();

    Ok(())
}

fn print_welcome() {
    println!();
    println!("{}", "╔═══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║                                                           ║".cyan());
    println!("{}", "║   🚀 Welcome to Scratchpad Setup                          ║".cyan());
    println!("{}", "║                                                           ║".cyan());
    println!("{}", "║   Scratchpad creates isolated environments for your       ║".cyan());
    println!("{}", "║   Git branches - perfect for testing PRs and features.    ║".cyan());
    println!("{}", "║                                                           ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan());
    println!();
}

async fn run_preflight_checks() -> Result<bool> {
    let mut all_passed = true;

    // Check 1: Find Docker socket (supports OrbStack, Docker Desktop, standard Linux)
    print!("  {} Looking for Docker socket... ", "→".blue());
    
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let socket_candidates = vec![
        // OrbStack (macOS)
        format!("{}/.orbstack/run/docker.sock", home_dir),
        // Docker Desktop (macOS)
        format!("{}/.docker/run/docker.sock", home_dir),
        // Default Linux
        "/var/run/docker.sock".to_string(),
    ];

    let mut found_socket: Option<String> = None;
    for candidate in &socket_candidates {
        if Path::new(candidate).exists() {
            found_socket = Some(candidate.clone());
            break;
        }
    }

    match &found_socket {
        Some(socket) => {
            let socket_type = if socket.contains("orbstack") {
                "OrbStack"
            } else if socket.contains(".docker") {
                "Docker Desktop"
            } else {
                "Docker"
            };
            println!("{} ({})", "found".green(), socket_type);
        }
        None => {
            println!("{}", "not found".red());
            println!(
                "    {} No Docker socket found. Checked:",
                "⚠".yellow()
            );
            for candidate in &socket_candidates {
                println!("       - {}", candidate);
            }
            all_passed = false;
        }
    }

    // Check 2: Docker connection
    print!("  {} Connecting to Docker... ", "→".blue());
    
    // Try to connect using the same logic as DockerClient
    let docker_result = if let Some(ref socket) = found_socket {
        bollard::Docker::connect_with_unix(socket, 120, bollard::API_DEFAULT_VERSION)
    } else {
        // Fall back to default connection attempt
        bollard::Docker::connect_with_socket_defaults()
    };

    match docker_result {
        Ok(docker) => {
            match docker.ping().await {
                Ok(_) => println!("{}", "connected".green()),
                Err(e) => {
                    println!("{}", "failed".red());
                    println!("    {} Could not ping Docker: {}", "⚠".yellow(), e);
                    all_passed = false;
                }
            }
        }
        Err(e) => {
            println!("{}", "failed".red());
            println!("    {} Could not connect: {}", "⚠".yellow(), e);
            println!("    {} Is Docker/OrbStack running?", "💡".blue());
            all_passed = false;
        }
    }

    // Check 3: Current directory is writable
    print!("  {} Checking write permissions... ", "→".blue());
    let test_file = Path::new(".scratchpad_test");
    match fs::write(test_file, "test") {
        Ok(_) => {
            let _ = fs::remove_file(test_file);
            println!("{}", "writable".green());
        }
        Err(_) => {
            println!("{}", "not writable".red());
            println!(
                "    {} Cannot write to current directory",
                "⚠".yellow()
            );
            all_passed = false;
        }
    }

    // Check 4: Port 3456 available
    print!("  {} Checking port 3456... ", "→".blue());
    match std::net::TcpListener::bind("0.0.0.0:3456") {
        Ok(_) => println!("{}", "available".green()),
        Err(_) => {
            println!("{}", "in use".yellow());
            println!(
                "    {} Port 3456 is in use (you can change this later)",
                "ℹ".blue()
            );
            // Not a fatal error
        }
    }

    println!();
    if all_passed {
        println!("  {} All pre-flight checks passed!", "✓".green());
    } else {
        println!("  {} Some checks failed", "⚠".yellow());
    }

    Ok(all_passed)
}

fn gather_config(theme: &ColorfulTheme) -> Result<Config> {
    // Domain configuration
    println!("{}", "Domain Configuration".bold());
    println!(
        "{}",
        "Scratches are accessed via URLs like: scratch-name.domain.com".dimmed()
    );
    println!();

    let domain: String = Input::with_theme(theme)
        .with_prompt("Base domain for scratches")
        .default("scratches.localhost".to_string())
        .interact_text()?;

    let routing_options = vec!["Subdomain (feature-branch.domain.com)", "Path (domain.com/feature-branch)"];
    let routing_idx = Select::with_theme(theme)
        .with_prompt("URL routing style")
        .items(&routing_options)
        .default(0)
        .interact()?;

    let routing = if routing_idx == 0 {
        NginxRouting::Subdomain
    } else {
        NginxRouting::Path
    };

    println!();
    println!("{}", "Services".bold());
    println!(
        "{}",
        "Select which services your scratches will use:".dimmed()
    );
    println!();

    let service_options = vec![
        ("PostgreSQL", "postgres", "Database - postgres:16"),
        ("Redis", "redis", "Cache/queue - redis:7-alpine"),
        ("MySQL", "mysql", "Database - mysql:8"),
        ("MongoDB", "mongodb", "NoSQL database - mongo:7"),
    ];

    let service_names: Vec<&str> = service_options
        .iter()
        .map(|(name, _, desc)| {
            // Format for display
            format!("{} - {}", name, desc).leak() as &str
        })
        .collect();

    let defaults = vec![true, true, false, false]; // postgres and redis by default
    let selected_indices = MultiSelect::with_theme(theme)
        .with_prompt("Services (space to toggle, enter to confirm)")
        .items(&service_names)
        .defaults(&defaults)
        .interact()?;

    // Build services map
    let mut services = HashMap::new();

    for idx in selected_indices {
        let (_, key, _) = &service_options[idx];
        let service = match *key {
            "postgres" => ServiceConfig {
                image: "postgres:16".to_string(),
                shared: true,
                port: Some(5432),
                internal_port: None, // derived from image
                env: HashMap::from([
                    ("POSTGRES_USER".to_string(), "postgres".to_string()),
                    ("POSTGRES_PASSWORD".to_string(), "postgres".to_string()),
                ]),
                volumes: vec![],
                healthcheck: Some("pg_isready -U postgres".to_string()),
                auto_create_db: true,
                connection: None,
            },
            "redis" => ServiceConfig {
                image: "redis:7-alpine".to_string(),
                shared: false,
                port: None,
                internal_port: None,
                env: HashMap::new(),
                volumes: vec![],
                healthcheck: Some("redis-cli ping".to_string()),
                auto_create_db: false,
                connection: None,
            },
            "mysql" => ServiceConfig {
                image: "mysql:8".to_string(),
                shared: true,
                port: Some(3306),
                internal_port: None,
                env: HashMap::from([
                    ("MYSQL_ROOT_PASSWORD".to_string(), "mysql".to_string()),
                ]),
                volumes: vec![],
                healthcheck: Some("mysqladmin ping -h localhost".to_string()),
                auto_create_db: false,
                connection: None,
            },
            "mongodb" => ServiceConfig {
                image: "mongo:7".to_string(),
                shared: true,
                port: Some(27017),
                internal_port: None,
                env: HashMap::new(),
                volumes: vec![],
                healthcheck: Some("mongosh --eval 'db.runCommand(\"ping\").ok'".to_string()),
                auto_create_db: false,
                connection: None,
            },
            _ => continue,
        };
        services.insert(key.to_string(), service);
    }

    // Default services for scratches
    let default_services: Vec<String> = services.keys().cloned().collect();

    // Server port
    println!();
    println!("{}", "Server Configuration".bold());
    println!();

    let port: u16 = Input::with_theme(theme)
        .with_prompt("API server port")
        .default(3456)
        .interact_text()?;

    // Build config
    let config = Config {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port,
            releases_dir: "./releases".into(),
        },
        docker: DockerConfig::default(),
        nginx: NginxConfig {
            enabled: true,
            config_path: "./nginx/scratches.conf".into(),
            reload_command: None,
            domain,
            routing,
            container: None,
        },
        github: None,
        services,
        scratch: ScratchDefaults {
            template: "default".to_string(),
            services: default_services.clone(),
            env: HashMap::new(),
            profiles: HashMap::from([
                (
                    "minimal".to_string(),
                    ScratchProfile {
                        template: None,
                        services: default_services.iter().take(1).cloned().collect(),
                        env: HashMap::new(),
                    },
                ),
                (
                    "full".to_string(),
                    ScratchProfile {
                        template: None,
                        services: default_services,
                        env: HashMap::new(),
                    },
                ),
            ]),
        },
    };

    Ok(config)
}

fn print_config_summary(config: &Config) {
    println!("  {} {}", "Domain:".bold(), config.nginx.domain);
    println!(
        "  {} {:?}",
        "Routing:".bold(),
        config.nginx.routing
    );
    println!("  {} {}", "Port:".bold(), config.server.port);
    println!(
        "  {} {}",
        "Services:".bold(),
        config.services.keys().cloned().collect::<Vec<_>>().join(", ")
    );
}

fn write_config(config: &Config) -> Result<()> {
    let toml_content = toml::to_string_pretty(config)?;
    
    // Add a header comment
    let content = format!(
        "# Scratchpad Configuration\n\
         # Generated by 'scratchpad setup'\n\
         # See https://github.com/Krakaw/scratchpad for documentation\n\n\
         {}", 
        toml_content
    );
    
    fs::write("scratchpad.toml", content)?;
    Ok(())
}

fn create_directories(config: &Config) -> Result<()> {
    // Create releases directory
    if !config.server.releases_dir.exists() {
        fs::create_dir_all(&config.server.releases_dir)?;
        println!(
            "{} Created {}",
            "✓".green(),
            config.server.releases_dir.display()
        );
    }

    // Create nginx config directory
    if let Some(parent) = config.nginx.config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            println!("{} Created {}", "✓".green(), parent.display());
        }
    }

    Ok(())
}

async fn create_demo_scratch(config: &Config) -> Result<()> {
    println!();
    println!("{}", "Creating demo scratch...".bold());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    // Connect to Docker
    pb.set_message("Connecting to Docker...");
    let docker = match DockerClient::new(config.docker.clone()) {
        Ok(d) => d,
        Err(e) => {
            pb.finish_with_message(format!("{} Failed to connect to Docker: {}", "✗".red(), e));
            return Ok(());
        }
    };

    // Create the demo scratch
    pb.set_message("Starting services...");

    // For now, just verify Docker works - the actual scratch creation
    // uses the existing create_scratch function
    match crate::scratch::create_scratch(config, &docker, "demo", Some("demo".to_string()), None, None).await {
        Ok(scratch) => {
            pb.finish_and_clear();
            println!("{} Created demo scratch: {}", "✓".green(), scratch.name);
            
            if config.nginx.enabled {
                let url = match config.nginx.routing {
                    NginxRouting::Subdomain => format!("http://{}.{}", scratch.name, config.nginx.domain),
                    NginxRouting::Path => format!("http://{}/{}", config.nginx.domain, scratch.name),
                };
                println!("  {} {}", "URL:".bold(), url.cyan());
            }

            println!();
            println!(
                "  {} Delete it later with: scratchpad delete demo",
                "ℹ".blue()
            );
        }
        Err(e) => {
            pb.finish_and_clear();
            println!("{} Could not create demo scratch: {}", "⚠".yellow(), e);
            println!(
                "  {} This might be normal if services need manual setup",
                "ℹ".blue()
            );
        }
    }

    Ok(())
}

/// Quick non-interactive setup with sensible defaults
async fn run_quick_setup() -> Result<()> {
    println!();
    println!("{}", "Running quick setup with defaults...".bold());
    println!();

    // Run preflight
    let preflight_ok = run_preflight_checks().await?;
    if !preflight_ok {
        println!();
        println!(
            "{}",
            "Some preflight checks failed. Fix the issues above and try again.".red()
        );
        return Ok(());
    }

    // Create default config
    let mut services = HashMap::new();
    services.insert(
        "postgres".to_string(),
        ServiceConfig {
            image: "postgres:16".to_string(),
            shared: true,
            port: Some(5432),
            internal_port: None,
            env: HashMap::from([
                ("POSTGRES_USER".to_string(), "postgres".to_string()),
                ("POSTGRES_PASSWORD".to_string(), "postgres".to_string()),
            ]),
            volumes: vec![],
            healthcheck: Some("pg_isready -U postgres".to_string()),
            auto_create_db: true,
            connection: None,
        },
    );
    services.insert(
        "redis".to_string(),
        ServiceConfig {
            image: "redis:7-alpine".to_string(),
            shared: false,
            port: None,
            internal_port: None,
            env: HashMap::new(),
            volumes: vec![],
            healthcheck: Some("redis-cli ping".to_string()),
            auto_create_db: false,
            connection: None,
        },
    );

    let config = Config {
        server: ServerConfig::default(),
        docker: DockerConfig::default(),
        nginx: NginxConfig::default(),
        github: None,
        services,
        scratch: ScratchDefaults {
            template: "default".to_string(),
            services: vec!["postgres".to_string(), "redis".to_string()],
            env: HashMap::new(),
            profiles: HashMap::from([
                (
                    "minimal".to_string(),
                    ScratchProfile {
                        template: None,
                        services: vec!["postgres".to_string()],
                        env: HashMap::new(),
                    },
                ),
                (
                    "full".to_string(),
                    ScratchProfile {
                        template: None,
                        services: vec!["postgres".to_string(), "redis".to_string()],
                        env: HashMap::new(),
                    },
                ),
            ]),
        },
    };

    write_config(&config)?;
    println!("{} Created scratchpad.toml", "✓".green());

    create_directories(&config)?;

    println!();
    println!("{}", "✓ Quick setup complete!".green().bold());
    println!();
    println!("Run 'scratchpad setup' (without --quick) for interactive configuration.");
    println!();

    Ok(())
}
