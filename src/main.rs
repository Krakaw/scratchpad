use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod config;
mod docker;
mod error;
mod nginx;
mod scratch;
mod services;

pub mod api;
pub mod ui;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scratchpad=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cli::commands::init().await,
        Commands::Create {
            branch,
            name,
            profile,
            template,
        } => cli::commands::create(&branch, name, profile, template).await,
        Commands::List { format } => cli::commands::list(format).await,
        Commands::Start { name } => cli::commands::start(&name).await,
        Commands::Stop { name } => cli::commands::stop(&name).await,
        Commands::Restart { name } => cli::commands::restart(&name).await,
        Commands::Delete { name, force } => cli::commands::delete(&name, force).await,
        Commands::Logs {
            name,
            service,
            follow,
            tail,
        } => cli::commands::logs(&name, service, follow, tail).await,
        Commands::Serve { host, port } => cli::commands::serve(&host, port).await,
        Commands::Status { name } => cli::commands::status(&name).await,
        Commands::Nginx { action } => cli::commands::nginx(action).await,
        Commands::Services { action } => cli::commands::services(action).await,
    }
}
