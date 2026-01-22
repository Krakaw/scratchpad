//! Configuration loading and environment variable interpolation

use crate::error::{Error, Result};
use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;

use super::Config;

const CONFIG_FILENAME: &str = "scratchpad.toml";

/// Load configuration from scratchpad.toml
pub fn load_config() -> Result<Config> {
    let config_path = find_config_file()?;
    load_config_from_path(&config_path)
}

/// Load configuration from a specific path
pub fn load_config_from_path(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path).map_err(|_| Error::ConfigNotFound)?;
    let content = interpolate_env_vars(&content);
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// Find the configuration file, searching upward from current directory
fn find_config_file() -> Result<std::path::PathBuf> {
    let mut current = env::current_dir().map_err(|e| Error::Config(e.to_string()))?;

    loop {
        let config_path = current.join(CONFIG_FILENAME);
        if config_path.exists() {
            return Ok(config_path);
        }

        if !current.pop() {
            return Err(Error::ConfigNotFound);
        }
    }
}

/// Interpolate environment variables in the format ${VAR_NAME} or ${VAR_NAME:-default}
fn interpolate_env_vars(content: &str) -> String {
    // This regex is a compile-time constant, panicking is acceptable here
    // as it indicates a programming error in the codebase, not a runtime issue
    let re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)(?::-([^}]*))?\}")
        .expect("Invalid regex pattern - this is a bug in the codebase");

    re.replace_all(content, |caps: &regex::Captures| {
        let var_name = &caps[1];
        let default = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        env::var(var_name).unwrap_or_else(|_| default.to_string())
    })
    .to_string()
}

/// Generate a default configuration file content
pub fn default_config_content() -> &'static str {
    r#"# Scratchpad Configuration
# See https://github.com/Krakaw/scratchpad for documentation

[server]
host = "0.0.0.0"
port = 3456
releases_dir = "./releases"

[docker]
socket = "/var/run/docker.sock"
network = "scratchpad-network"

[nginx]
enabled = true
config_path = "./nginx/scratches.conf"
domain = "scratches.localhost"
routing = "subdomain"  # or "path"
# container = "nginx"  # Container name for reload
# reload_command = "docker exec nginx nginx -s reload"

# GitHub configuration (optional, for webhooks)
# [github]
# token = "${GITHUB_TOKEN}"
# api_repo = "owner/api-repo"
# web_repo = "owner/web-repo"

# Shared services
[services.postgres]
image = "postgres:16"
shared = true
port = 5432
auto_create_db = true
env = { POSTGRES_PASSWORD = "postgres", POSTGRES_USER = "postgres" }
healthcheck = "pg_isready -U postgres"

[services.redis]
image = "redis:7-alpine"
shared = false  # Each scratch gets its own instance
healthcheck = "redis-cli ping"

# Uncomment to add Kafka support
# [services.kafka]
# image = "bitnami/kafka:3.6"
# shared = true
# env = { ALLOW_PLAINTEXT_LISTENER = "yes" }

# Default scratch settings
[scratch.defaults]
template = "default"
services = ["postgres", "redis"]

# Custom profiles for different use cases
[scratch.profiles.minimal]
services = ["postgres"]

[scratch.profiles.full]
services = ["postgres", "redis", "kafka"]

# Profile-specific environment variables
# [scratch.profiles.full.env]
# ENABLE_KAFKA = "true"
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_interpolation() {
        env::set_var("TEST_VAR", "hello");
        let content = "value = \"${TEST_VAR}\"";
        let result = interpolate_env_vars(content);
        assert_eq!(result, "value = \"hello\"");
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_env_interpolation_with_default() {
        let content = "value = \"${NONEXISTENT_VAR:-default_value}\"";
        let result = interpolate_env_vars(content);
        assert_eq!(result, "value = \"default_value\"");
    }
}
