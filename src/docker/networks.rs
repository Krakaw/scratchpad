//! Docker network management

use bollard::network::CreateNetworkOptions;
use std::collections::HashMap;

use super::DockerClient;
use crate::error::Result;

impl DockerClient {
    /// Ensure the scratchpad network exists
    pub async fn ensure_network(&self) -> Result<()> {
        let network_name = &self.config().network;

        // Check if network exists
        if self.inner().inspect_network::<String>(network_name, None).await.is_ok() {
            tracing::debug!("Network {} already exists", network_name);
            return Ok(());
        }

        // Create the network
        tracing::info!("Creating network: {}", network_name);

        let options = CreateNetworkOptions {
            name: network_name.as_str(),
            driver: "bridge",
            ..Default::default()
        };

        self.inner().create_network(options).await?;
        Ok(())
    }

    /// Connect a container to the scratchpad network
    pub async fn connect_to_network(&self, container_id: &str) -> Result<()> {
        use bollard::network::ConnectNetworkOptions;

        let options = ConnectNetworkOptions {
            container: container_id,
            endpoint_config: Default::default(),
        };

        self.inner()
            .connect_network(&self.config().network, options)
            .await?;
        Ok(())
    }

    /// Disconnect a container from the scratchpad network
    pub async fn disconnect_from_network(&self, container_id: &str) -> Result<()> {
        use bollard::network::DisconnectNetworkOptions;

        let options = DisconnectNetworkOptions {
            container: container_id,
            force: true,
        };

        self.inner()
            .disconnect_network(&self.config().network, options)
            .await?;
        Ok(())
    }

    /// List all containers in the scratchpad network
    pub async fn list_network_containers(&self) -> Result<Vec<String>> {
        let network = self
            .inner()
            .inspect_network::<String>(&self.config().network, None)
            .await?;

        let containers: Vec<String> = network
            .containers
            .unwrap_or_default()
            .keys()
            .cloned()
            .collect();

        Ok(containers)
    }

    /// Remove the scratchpad network (only if empty)
    pub async fn remove_network(&self) -> Result<()> {
        self.inner().remove_network(&self.config().network).await?;
        Ok(())
    }

    /// Get network information as a HashMap for template rendering
    pub fn get_network_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("name".to_string(), self.config().network.clone());
        info
    }
}
