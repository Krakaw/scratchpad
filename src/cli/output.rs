//! CLI output formatting utilities

use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

use crate::scratch::ScratchStatus;

/// Print a success message
pub fn success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

/// Print an error message
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red(), message);
}

/// Print a warning message
pub fn warn(message: &str) {
    println!("{} {}", "⚠".yellow(), message);
}

/// Print an info message
pub fn info(message: &str) {
    println!("{} {}", "ℹ".blue(), message);
}

/// Format scratch status as a colored string
pub fn format_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "running" => status.green().to_string(),
        "stopped" | "exited" => status.red().to_string(),
        "starting" | "restarting" => status.yellow().to_string(),
        _ => status.to_string(),
    }
}

/// Print a table of scratches
pub fn print_scratch_table(scratches: &[ScratchStatus]) {
    if scratches.is_empty() {
        info("No scratches found. Create one with 'scratchpad create --branch <branch>'");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Name").fg(Color::Cyan),
            Cell::new("Branch").fg(Color::Cyan),
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Services").fg(Color::Cyan),
            Cell::new("URL").fg(Color::Cyan),
            Cell::new("Created").fg(Color::Cyan),
        ]);

    for scratch in scratches {
        let status_color = match scratch.status.as_str() {
            "running" => Color::Green,
            "stopped" | "exited" => Color::Red,
            _ => Color::Yellow,
        };

        let services = scratch
            .services
            .iter()
            .map(|(name, status)| {
                if status == "running" {
                    format!("{}✓", name)
                } else {
                    format!("{}✗", name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        let created = scratch
            .created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        table.add_row(vec![
            Cell::new(&scratch.name),
            Cell::new(&scratch.branch),
            Cell::new(&scratch.status).fg(status_color),
            Cell::new(services),
            Cell::new(scratch.url.as_deref().unwrap_or("-")),
            Cell::new(created),
        ]);
    }

    println!("{table}");
}

/// Print detailed scratch status
pub fn print_scratch_detail(scratch: &ScratchStatus) {
    println!("{}", "Scratch Details".bold().underline());
    println!();
    println!("  {} {}", "Name:".bold(), scratch.name);
    println!("  {} {}", "Branch:".bold(), scratch.branch);
    println!("  {} {}", "Status:".bold(), format_status(&scratch.status));

    if let Some(url) = &scratch.url {
        println!("  {} {}", "URL:".bold(), url.cyan());
    }

    if let Some(created) = scratch.created_at {
        println!(
            "  {} {}",
            "Created:".bold(),
            created.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    println!();
    println!("  {}", "Services:".bold());
    for (name, status) in &scratch.services {
        let status_icon = if status == "running" {
            "●".green()
        } else {
            "○".red()
        };
        println!("    {} {} ({})", status_icon, name, status);
    }

    if !scratch.databases.is_empty() {
        println!();
        println!("  {}", "Databases:".bold());
        for db in &scratch.databases {
            println!("    - {}", db);
        }
    }
}

/// Confirm an action with the user
pub fn confirm(message: &str) -> bool {
    use std::io::{self, Write};

    print!("{} [y/N] ", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
