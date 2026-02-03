//! CLI interface for Scratchpad

pub mod commands;
mod output;
pub mod setup;

pub use output::*;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "scratchpad")]
#[command(author = "Krakaw")]
#[command(version = "2.0.0")]
#[command(about = "Deploy scratch environments easily", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive setup wizard (recommended for first-time users)
    Setup {
        /// Skip interactive prompts and use defaults
        #[arg(short, long)]
        quick: bool,
    },

    /// Initialize a new scratchpad.toml configuration file (basic)
    Init,

    /// Create a new scratch environment from a branch
    Create {
        /// The branch name to create the scratch from
        #[arg(short, long)]
        branch: String,

        /// Optional custom name for the scratch (defaults to sanitized branch name)
        #[arg(short, long)]
        name: Option<String>,

        /// Use a specific profile from the config
        #[arg(short, long)]
        profile: Option<String>,

        /// Override the template to use
        #[arg(short, long)]
        template: Option<String>,
    },

    /// List all scratch environments
    List {
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },

    /// Start a stopped scratch environment
    Start {
        /// Name of the scratch to start
        name: String,
    },

    /// Stop a running scratch environment
    Stop {
        /// Name of the scratch to stop
        name: String,
    },

    /// Restart a scratch environment
    Restart {
        /// Name of the scratch to restart
        name: String,
    },

    /// Delete a scratch environment
    Delete {
        /// Name of the scratch to delete
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// View logs from a scratch environment
    Logs {
        /// Name of the scratch
        name: String,

        /// Specific service to view logs from (optional)
        #[arg(short, long)]
        service: Option<String>,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show from the end
        #[arg(short, long, default_value = "100")]
        tail: usize,
    },

    /// Start the HTTP API server and web UI
    Serve {
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value = "3456")]
        port: u16,
    },

    /// Show detailed status of a scratch environment
    Status {
        /// Name of the scratch
        name: String,
    },

    /// Manage nginx configuration
    Nginx {
        #[command(subcommand)]
        action: NginxAction,
    },

    /// Manage shared services
    Services {
        #[command(subcommand)]
        action: ServicesAction,
    },

    /// Check system health and Docker connectivity
    Doctor,
}

#[derive(Subcommand)]
pub enum NginxAction {
    /// Regenerate nginx configuration
    Generate,

    /// Reload nginx configuration
    Reload,

    /// Show current nginx configuration
    Show,
}

#[derive(Subcommand)]
pub enum ServicesAction {
    /// Start all shared services
    Start,

    /// Stop all shared services
    Stop,

    /// Show status of shared services
    Status,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}
