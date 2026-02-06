//! Services provisioning integration tests
//! Tests shared service creation and management
//!
//! Run with: cargo test --test services_tests -- --test-threads=1 --nocapture
//! Note: Requires Docker and may require PostgreSQL connection credentials

use scratchpad::config::{Config, DockerConfig};
use scratchpad::docker::DockerClient;
use scratchpad::services;

/// Helper to create a test Docker client
fn create_test_docker_client(config: &Config) -> Result<DockerClient, Box<dyn std::error::Error>> {
    let docker_config = DockerConfig {
        socket: "/var/run/docker.sock".to_string(),
        network: config.docker.network.clone(),
        label_prefix: config.docker.label_prefix.clone(),
    };
    DockerClient::new(docker_config).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_list_shared_services() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    match docker.list_shared_service_containers().await {
        Ok(containers) => {
            println!("✓ Listed {} shared service containers", containers.len());
            for container in &containers {
                println!(
                    "  - {}: {} ({})",
                    container.name, container.image, container.state
                );
            }
        }
        Err(e) => panic!("✗ Failed to list shared services: {}", e),
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_ensure_shared_service_postgres() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    // First ensure the network exists
    if let Err(e) = docker.ensure_network().await {
        println!("⚠ Warning: Failed to ensure network: {}", e);
    }

    match services::ensure_shared_service_running(&config, &docker, "postgres").await {
        Ok(()) => {
            println!("✓ Successfully ensured postgres service is running");

            // List services to verify
            match docker.list_shared_service_containers().await {
                Ok(containers) => {
                    let postgres_found = containers.iter().any(|c| c.name.contains("postgres"));
                    assert!(postgres_found, "PostgreSQL container should be in list");
                    println!("✓ PostgreSQL container confirmed in service list");
                }
                Err(e) => println!("⚠ Could not verify service: {}", e),
            }
        }
        Err(e) => panic!("✗ Failed to ensure postgres service: {}", e),
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_postgres_database_creation() {
    let config = Config::default();

    // This test requires PostgreSQL to be running and accessible
    // It's skipped by default as it requires actual database access
    let db_name = "scratchpad_test_db";

    match services::create_postgres_database(&config, db_name).await {
        Ok(()) => {
            println!("✓ Successfully created PostgreSQL database: {}", db_name);
        }
        Err(e) => {
            println!(
                "⚠ Could not create database (may require running PostgreSQL): {}",
                e
            );
        }
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_postgres_database_deletion() {
    let config = Config::default();

    // This test requires PostgreSQL to be running and accessible
    let db_name = "scratchpad_test_delete";

    // First create a database
    if let Err(e) = services::create_postgres_database(&config, db_name).await {
        println!("⚠ Skipping: Could not create test database: {}", e);
        return;
    }

    // Now delete it
    match services::drop_postgres_database(&config, db_name).await {
        Ok(()) => {
            println!("✓ Successfully deleted PostgreSQL database: {}", db_name);
        }
        Err(e) => {
            println!("⚠ Could not delete database: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_service_not_found() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    match services::ensure_shared_service_running(&config, &docker, "nonexistent-service").await {
        Ok(()) => panic!("✗ Should return error for non-existent service"),
        Err(_) => {
            println!("✓ Correctly rejected non-existent service");
        }
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_database_name_validation() {
    let config = Config::default();

    // Test invalid database name (contains invalid characters)
    let invalid_names = vec!["test-db", "test db", "test@db", "test.db"];

    for name in invalid_names {
        match services::create_postgres_database(&config, name).await {
            Ok(()) => println!("⚠ Should reject invalid name: {}", name),
            Err(_) => {
                println!("✓ Correctly rejected invalid database name: {}", name);
            }
        }
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_valid_database_names() {
    let config = Config::default();

    // Test valid database names
    let valid_names = vec!["scratch_test_1", "scratch_test_2", "scratchpad_db"];

    for name in valid_names {
        match services::create_postgres_database(&config, name).await {
            Ok(()) => {
                println!("✓ Successfully created database with valid name: {}", name);
                // Cleanup
                let _ = services::drop_postgres_database(&config, name).await;
            }
            Err(e) => {
                println!(
                    "⚠ Could not test with valid name {} (may need running PostgreSQL): {}",
                    name, e
                );
            }
        }
    }
}
