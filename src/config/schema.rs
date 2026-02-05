//! Configuration schema definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub docker: DockerConfig,

    #[serde(default)]
    pub nginx: NginxConfig,

    #[serde(default)]
    pub github: Option<GithubConfig>,

    #[serde(default)]
    pub services: HashMap<String, ServiceConfig>,

    #[serde(default)]
    pub scratch: ScratchDefaults,
}

/// Server configuration for the HTTP API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_releases_dir")]
    pub releases_dir: PathBuf,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3456
}

fn default_releases_dir() -> PathBuf {
    PathBuf::from("./releases")
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            releases_dir: default_releases_dir(),
        }
    }
}

/// Docker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    #[serde(default = "default_socket")]
    pub socket: String,

    #[serde(default = "default_network")]
    pub network: String,

    /// Label prefix for scratch containers
    #[serde(default = "default_label_prefix")]
    pub label_prefix: String,
}

fn default_socket() -> String {
    "/var/run/docker.sock".to_string()
}

fn default_network() -> String {
    "scratchpad-network".to_string()
}

fn default_label_prefix() -> String {
    "scratchpad".to_string()
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            socket: default_socket(),
            network: default_network(),
            label_prefix: default_label_prefix(),
        }
    }
}

/// Nginx configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NginxConfig {
    #[serde(default = "default_nginx_enabled")]
    pub enabled: bool,

    #[serde(default = "default_nginx_config_path")]
    pub config_path: PathBuf,

    #[serde(default)]
    pub reload_command: Option<String>,

    #[serde(default = "default_nginx_domain")]
    pub domain: String,

    #[serde(default = "default_nginx_routing")]
    pub routing: NginxRouting,

    /// Container name for nginx (used for reload)
    #[serde(default)]
    pub container: Option<String>,

    /// Use dynamic routing (default: true)
    /// When true, nginx uses variables to route based on subdomain/path
    /// When false, generates static config entries per scratch
    #[serde(default)]
    pub dynamic: Option<bool>,

    /// The service name that acts as the ingress point for each scratch
    /// e.g., "api" means requests route to <scratch>-api container
    #[serde(default)]
    pub ingress_service: Option<String>,
}

fn default_nginx_enabled() -> bool {
    true
}

fn default_nginx_config_path() -> PathBuf {
    PathBuf::from("./nginx/scratches.conf")
}

fn default_nginx_domain() -> String {
    "scratches.localhost".to_string()
}

fn default_nginx_routing() -> NginxRouting {
    NginxRouting::Subdomain
}

impl Default for NginxConfig {
    fn default() -> Self {
        Self {
            enabled: default_nginx_enabled(),
            config_path: default_nginx_config_path(),
            reload_command: None,
            domain: default_nginx_domain(),
            routing: default_nginx_routing(),
            container: None,
            dynamic: None,
            ingress_service: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NginxRouting {
    #[default]
    Subdomain,
    Path,
}

/// GitHub configuration for webhooks and branch listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubConfig {
    pub token: String,

    #[serde(default)]
    pub api_repo: Option<String>,

    #[serde(default)]
    pub web_repo: Option<String>,
}

/// Shared service configuration (postgres, redis, kafka, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub image: String,

    #[serde(default)]
    pub shared: bool,

    /// Host port to expose the service on
    #[serde(default)]
    pub port: Option<u16>,

    /// Internal container port (defaults to same as port, or standard port for known images)
    #[serde(default)]
    pub internal_port: Option<u16>,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub volumes: Vec<String>,

    #[serde(default)]
    pub healthcheck: Option<String>,

    /// For postgres: automatically create databases
    #[serde(default)]
    pub auto_create_db: bool,

    /// Connection parameters for DB provisioning
    #[serde(default)]
    pub connection: Option<ServiceConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConnection {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

/// Default scratch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchDefaults {
    #[serde(default = "default_template")]
    pub template: String,

    #[serde(default)]
    pub services: Vec<String>,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub profiles: HashMap<String, ScratchProfile>,
}

fn default_template() -> String {
    "default".to_string()
}

impl Default for ScratchDefaults {
    fn default() -> Self {
        Self {
            template: default_template(),
            services: Vec::new(),
            env: HashMap::new(),
            profiles: HashMap::new(),
        }
    }
}

/// A scratch profile - preset configurations for different use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchProfile {
    #[serde(default)]
    pub template: Option<String>,

    #[serde(default)]
    pub services: Vec<String>,

    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Runtime scratch instance configuration (stored per-scratch)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchConfig {
    pub name: String,
    pub branch: String,
    pub template: String,
    pub services: Vec<String>,
    pub databases: HashMap<String, Vec<String>>,
    pub env: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Config {
    /// Get a service configuration by name
    pub fn get_service(&self, name: &str) -> Option<&ServiceConfig> {
        self.services.get(name)
    }

    /// Get a scratch profile by name
    pub fn get_profile(&self, name: &str) -> Option<&ScratchProfile> {
        self.scratch.profiles.get(name)
    }
}
