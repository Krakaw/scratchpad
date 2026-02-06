//! Docker client wrapper using bollard

use bollard::Docker;
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::config::DockerConfig;
use crate::error::Result;

static DOCKER_CLIENT: OnceCell<Arc<DockerClient>> = OnceCell::const_new();

/// Wrapper around the bollard Docker client
#[derive(Clone)]
pub struct DockerClient {
    inner: Docker,
    config: DockerConfig,
}

impl DockerClient {
    /// Create a new Docker client
    pub fn new(config: DockerConfig) -> Result<Self> {
        tracing::debug!("Attempting to connect to Docker");

        // Try to get socket from environment first
        let socket_path = std::env::var("DOCKER_HOST")
            .ok()
            .map(|h| {
                // DOCKER_HOST might be in format unix:///path/to/socket or tcp://host:port
                h.strip_prefix("unix://")
                    .map(|s| s.to_string())
                    .unwrap_or(h)
            })
            .unwrap_or_else(|| config.socket.clone());

        // Get home directory for OrbStack and Docker Desktop paths
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

        // List of common socket locations to try
        let socket_candidates = vec![
            socket_path.clone(),
            // OrbStack
            format!("{}/.orbstack/run/docker.sock", home_dir),
            // Docker Desktop
            format!("{}/.docker/run/docker.sock", home_dir),
            // Default Linux
            "/var/run/docker.sock".to_string(),
            // TCP fallback
            "tcp://127.0.0.1:2375".to_string(),
        ];

        let mut last_error = None;
        let mut attempted = Vec::new();

        for candidate in &socket_candidates {
            if candidate.is_empty() {
                continue;
            }

            attempted.push(candidate.clone());
            tracing::debug!("Trying Docker socket: {}", candidate);

            let result = if candidate.starts_with("tcp://") {
                Docker::connect_with_http(candidate, 120, bollard::API_DEFAULT_VERSION)
            } else {
                // Check if socket exists before trying to connect
                if !std::path::Path::new(candidate).exists() {
                    tracing::debug!("Socket does not exist: {}", candidate);
                    continue;
                }
                Docker::connect_with_unix(candidate, 120, bollard::API_DEFAULT_VERSION)
            };

            match result {
                Ok(inner) => {
                    tracing::info!("Successfully connected to Docker at: {}", candidate);
                    return Ok(Self {
                        inner,
                        config: DockerConfig {
                            socket: candidate.clone(),
                            ..config
                        },
                    });
                }
                Err(e) => {
                    tracing::debug!("Failed to connect to {}: {}", candidate, e);
                    last_error = Some(e);
                }
            }
        }

        // If we get here, none of the sockets worked
        let error_msg = if let Some(e) = last_error {
            format!("Failed to connect to Docker socket: {}", e)
        } else {
            "No Docker sockets found. Make sure Docker is running.".to_string()
        };

        tracing::error!("{}", error_msg);
        tracing::error!("Tried sockets:");
        for candidate in attempted {
            if !candidate.is_empty() {
                tracing::error!("  - {}", candidate);
            }
        }

        Err(crate::error::Error::Other(error_msg))
    }

    /// Get or create the global Docker client
    pub async fn get_or_init(config: DockerConfig) -> Result<Arc<DockerClient>> {
        DOCKER_CLIENT
            .get_or_try_init(|| async {
                let client = DockerClient::new(config)?;
                Ok(Arc::new(client))
            })
            .await
            .cloned()
    }

    /// Get the underlying bollard Docker client
    pub fn inner(&self) -> &Docker {
        &self.inner
    }

    /// Get the Docker configuration
    pub fn config(&self) -> &DockerConfig {
        &self.config
    }

    /// Test the Docker connection
    pub async fn ping(&self) -> Result<()> {
        self.inner.ping().await?;
        Ok(())
    }
}
