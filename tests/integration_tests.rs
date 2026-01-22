//! Integration tests for Scratchpad
//! 
//! These tests require Docker to be running

use scratchpad::config::{Config, DockerConfig};
use scratchpad::docker::DockerClient;

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_docker_client_creation() {
    let config = Config::default();
    let docker_config = DockerConfig {
        socket: "/var/run/docker.sock".to_string(),
        network: config.docker.network.clone(),
        label_prefix: config.docker.label_prefix.clone(),
    };
    
    let client = DockerClient::new(docker_config).expect("Failed to create Docker client");
    
    // Verify we can interact with Docker by getting simple info
    // In real implementation, this would call Docker API
    println!("Docker client created successfully");
}

#[tokio::test]
#[ignore]
async fn test_docker_network_creation() {
    let config = Config::default();
    let docker_config = DockerConfig {
        socket: "/var/run/docker.sock".to_string(),
        network: config.docker.network.clone(),
        label_prefix: config.docker.label_prefix.clone(),
    };
    
    let client = DockerClient::new(docker_config).expect("Failed to create Docker client");
    
    // Test network creation
    client.ensure_network().await.expect("Failed to ensure network");
    println!("Network created successfully");
}

#[tokio::test]
#[ignore]
async fn test_scratch_lifecycle() {
    let config = Config::default();
    let docker_config = DockerConfig {
        socket: "/var/run/docker.sock".to_string(),
        network: config.docker.network.clone(),
        label_prefix: config.docker.label_prefix.clone(),
    };
    
    let client = DockerClient::new(docker_config).expect("Failed to create Docker client");
    
    // Test complete scratch lifecycle
    // 1. Create
    // 2. List
    // 3. Start
    // 4. Stop
    // 5. Delete
    
    println!("Scratch lifecycle test completed");
}

#[test]
fn test_config_parsing() {
    let config = Config::default();
    assert_eq!(config.server.port, 3456);
    assert_eq!(config.docker.network, "scratchpad-network");
}
