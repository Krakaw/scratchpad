//! Integration tests for scratch lifecycle operations
//! Tests the full create, start, stop, delete flow
//!
//! Run with: cargo test --test scratch_operations_tests -- --test-threads=1
//! (Use single thread to avoid Docker port conflicts)

use scratchpad::config::{Config, DockerConfig};
use scratchpad::docker::DockerClient;
use scratchpad::scratch;
use std::fs;

/// Helper to create a test Docker client
fn create_test_docker_client(config: &Config) -> Result<DockerClient, Box<dyn std::error::Error>> {
    let docker_config = DockerConfig {
        socket: "/var/run/docker.sock".to_string(),
        network: config.docker.network.clone(),
        label_prefix: config.docker.label_prefix.clone(),
    };
    DockerClient::new(docker_config).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Helper to clean up test scratch directories
fn cleanup_scratches(config: &Config, names: &[&str]) {
    for name in names {
        let scratch_dir = config.server.releases_dir.join(name);
        if scratch_dir.exists() {
            let _ = fs::remove_dir_all(&scratch_dir);
        }
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_create_scratch() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let test_name = "test-create-scratch";
    cleanup_scratches(&config, &[test_name]);

    match scratch::create_scratch(
        &config,
        &docker,
        "test-branch",
        Some(test_name.to_string()),
        None,
        None,
    )
    .await
    {
        Ok(created) => {
            assert_eq!(created.name, test_name);
            assert_eq!(created.branch, "test-branch");
            println!("✓ Successfully created scratch: {}", test_name);

            // Verify directory exists
            let scratch_dir = config.server.releases_dir.join(test_name);
            assert!(scratch_dir.exists(), "Scratch directory should exist");
            assert!(
                scratch_dir.join(".scratchpad.toml").exists(),
                "Config file should exist"
            );
            println!("✓ Scratch directory and config file created");

            cleanup_scratches(&config, &[test_name]);
        }
        Err(e) => panic!("✗ Failed to create scratch: {}", e),
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_list_scratches() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let test_names = ["list-test-1", "list-test-2"];
    cleanup_scratches(&config, &test_names);

    // Create two scratches
    for (i, name) in test_names.iter().enumerate() {
        if let Err(e) = scratch::create_scratch(
            &config,
            &docker,
            &format!("branch-{}", i),
            Some(name.to_string()),
            None,
            None,
        )
        .await
        {
            println!("Warning: Failed to create test scratch: {}", e);
        }
    }

    // List scratches
    match scratch::list_scratches(&config, &docker).await {
        Ok(scratches) => {
            assert!(
                scratches.len() >= test_names.len(),
                "Should have at least {} scratches",
                test_names.len()
            );
            println!("✓ Listed {} scratches", scratches.len());

            for name in &test_names {
                assert!(
                    scratches.iter().any(|s| &s.name == name),
                    "Scratch {} should be in list",
                    name
                );
            }
            println!("✓ Both test scratches found in list");
        }
        Err(e) => panic!("✗ Failed to list scratches: {}", e),
    }

    cleanup_scratches(&config, &test_names);
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_scratch_status() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let test_name = "status-test-scratch";
    cleanup_scratches(&config, &[test_name]);

    // Create a scratch
    if let Err(e) = scratch::create_scratch(
        &config,
        &docker,
        "status-branch",
        Some(test_name.to_string()),
        None,
        None,
    )
    .await
    {
        println!("Warning: Failed to create test scratch: {}", e);
    }

    // Get status
    match scratch::get_scratch_status(&config, &docker, test_name).await {
        Ok(status) => {
            assert_eq!(status.name, test_name);
            assert_eq!(status.branch, "status-branch");
            println!("✓ Retrieved scratch status: {:?}", status.status);
        }
        Err(e) => panic!("✗ Failed to get scratch status: {}", e),
    }

    cleanup_scratches(&config, &[test_name]);
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_start_stop_scratch() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let test_name = "start-stop-test";
    cleanup_scratches(&config, &[test_name]);

    // Create scratch
    if let Err(e) = scratch::create_scratch(
        &config,
        &docker,
        "feature-branch",
        Some(test_name.to_string()),
        None,
        None,
    )
    .await
    {
        println!("Warning: Failed to create test scratch: {}", e);
    }

    // Stop the scratch
    match scratch::stop_scratch(&config, &docker, test_name).await {
        Ok(()) => {
            println!("✓ Successfully stopped scratch");
        }
        Err(e) => panic!("✗ Failed to stop scratch: {}", e),
    }

    // Start the scratch
    match scratch::start_scratch(&config, &docker, test_name).await {
        Ok(()) => {
            println!("✓ Successfully started scratch");
        }
        Err(e) => panic!("✗ Failed to start scratch: {}", e),
    }

    cleanup_scratches(&config, &[test_name]);
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_restart_scratch() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let test_name = "restart-test";
    cleanup_scratches(&config, &[test_name]);

    // Create scratch
    if let Err(e) = scratch::create_scratch(
        &config,
        &docker,
        "main-branch",
        Some(test_name.to_string()),
        None,
        None,
    )
    .await
    {
        println!("Warning: Failed to create test scratch: {}", e);
    }

    // Restart the scratch
    match scratch::restart_scratch(&config, &docker, test_name).await {
        Ok(()) => {
            println!("✓ Successfully restarted scratch");
        }
        Err(e) => panic!("✗ Failed to restart scratch: {}", e),
    }

    cleanup_scratches(&config, &[test_name]);
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_delete_scratch() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let test_name = "delete-test";
    cleanup_scratches(&config, &[test_name]);

    // Create scratch
    if let Err(e) = scratch::create_scratch(
        &config,
        &docker,
        "delete-branch",
        Some(test_name.to_string()),
        None,
        None,
    )
    .await
    {
        println!("Warning: Failed to create test scratch: {}", e);
    }

    // Verify it exists
    let scratch_dir = config.server.releases_dir.join(test_name);
    assert!(scratch_dir.exists(), "Scratch should exist before deletion");

    // Delete the scratch
    match scratch::delete_scratch(&config, &docker, test_name, false).await {
        Ok(()) => {
            println!("✓ Successfully deleted scratch");
        }
        Err(e) => panic!("✗ Failed to delete scratch: {}", e),
    }

    // Verify it's gone
    assert!(!scratch_dir.exists(), "Scratch directory should be deleted");
    println!("✓ Scratch directory removed successfully");
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_cannot_create_duplicate_scratch() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let test_name = "duplicate-test";
    cleanup_scratches(&config, &[test_name]);

    // Create first scratch
    if let Err(e) = scratch::create_scratch(
        &config,
        &docker,
        "branch-1",
        Some(test_name.to_string()),
        None,
        None,
    )
    .await
    {
        println!("Warning: Failed to create test scratch: {}", e);
    }

    // Try to create duplicate
    match scratch::create_scratch(
        &config,
        &docker,
        "branch-2",
        Some(test_name.to_string()),
        None,
        None,
    )
    .await
    {
        Ok(_) => panic!("✗ Should not allow duplicate scratch creation"),
        Err(_) => {
            println!("✓ Correctly rejected duplicate scratch creation");
        }
    }

    cleanup_scratches(&config, &[test_name]);
}
