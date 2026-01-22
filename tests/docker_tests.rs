//! Docker integration tests

#[cfg(test)]
mod tests {
    use scratchpad::config::DockerConfig;
    use scratchpad::docker::DockerClient;

    #[tokio::test]
    async fn test_docker_ping() {
        let config = DockerConfig {
            socket: "/var/run/docker.sock".to_string(),
            network: "scratchpad-network".to_string(),
            label_prefix: "scratchpad".to_string(),
        };

        match DockerClient::new(config) {
            Ok(client) => {
                match client.ping().await {
                    Ok(_) => println!("✓ Docker ping successful"),
                    Err(e) => eprintln!("✗ Docker ping failed: {}", e),
                }
            }
            Err(e) => eprintln!("✗ Failed to create Docker client: {}", e),
        }
    }

    #[tokio::test]
    async fn test_docker_list_containers() {
        let config = DockerConfig {
            socket: "/var/run/docker.sock".to_string(),
            network: "scratchpad-network".to_string(),
            label_prefix: "scratchpad".to_string(),
        };

        if let Ok(client) = DockerClient::new(config) {
            match client.list_scratch_containers(None).await {
                Ok(containers) => {
                    println!("✓ Found {} containers", containers.len());
                }
                Err(e) => eprintln!("✗ Failed to list containers: {}", e),
            }
        }
    }

    #[tokio::test]
    async fn test_docker_network_exists() {
        let config = DockerConfig {
            socket: "/var/run/docker.sock".to_string(),
            network: "scratchpad-network".to_string(),
            label_prefix: "scratchpad".to_string(),
        };

        if let Ok(client) = DockerClient::new(config) {
            match client.ensure_network().await {
                Ok(_) => println!("✓ Network ensured successfully"),
                Err(e) => eprintln!("✗ Failed to ensure network: {}", e),
            }
        }
    }
}
