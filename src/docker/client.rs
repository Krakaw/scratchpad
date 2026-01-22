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
        let inner = if config.socket.starts_with("tcp://") {
            Docker::connect_with_http(&config.socket, 120, bollard::API_DEFAULT_VERSION)?
        } else {
            Docker::connect_with_unix(&config.socket, 120, bollard::API_DEFAULT_VERSION)?
        };

        Ok(Self { inner, config })
    }

    /// Get or create the global Docker client
    pub async fn get_or_init(config: DockerConfig) -> Result<Arc<DockerClient>> {
        DOCKER_CLIENT
            .get_or_try_init(|| async {
                let client = DockerClient::new(config)?;
                Ok(Arc::new(client))
            })
            .await
            .map(|c| c.clone())
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
