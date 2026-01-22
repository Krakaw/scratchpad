//! Docker Compose file handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;

/// Docker Compose file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeFile {
    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub services: HashMap<String, ComposeService>,

    #[serde(default)]
    pub networks: HashMap<String, ComposeNetwork>,

    #[serde(default)]
    pub volumes: HashMap<String, ComposeVolume>,
}

impl Default for ComposeFile {
    fn default() -> Self {
        Self {
            version: Some("3.8".to_string()),
            services: HashMap::new(),
            networks: HashMap::new(),
            volumes: HashMap::new(),
        }
    }
}

/// A service in docker-compose
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComposeService {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<ComposeBuild>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environment: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_file: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub networks: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<ComposeHealthcheck>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<serde_yaml::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<serde_yaml::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComposeBuild {
    Simple(String),
    Full {
        context: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        dockerfile: Option<String>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        args: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeHealthcheck {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<serde_yaml::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComposeNetwork {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComposeVolume {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ComposeFile {
    /// Load a compose file from disk
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let compose: ComposeFile = serde_yaml::from_str(&content)?;
        Ok(compose)
    }

    /// Save the compose file to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add scratchpad labels to all services
    pub fn add_labels(&mut self, scratch_name: &str, label_prefix: &str) {
        for (service_name, service) in &mut self.services {
            service.labels.insert(
                format!("{}.scratch", label_prefix),
                scratch_name.to_string(),
            );
            service.labels.insert(
                format!("{}.service", label_prefix),
                service_name.to_string(),
            );
        }
    }

    /// Add the scratchpad network to all services
    pub fn add_network(&mut self, network_name: &str) {
        // Add network definition
        self.networks.insert(
            network_name.to_string(),
            ComposeNetwork {
                external: Some(true),
                ..Default::default()
            },
        );

        // Add network to all services
        for service in self.services.values_mut() {
            if !service.networks.contains(&network_name.to_string()) {
                service.networks.push(network_name.to_string());
            }
        }
    }
}
