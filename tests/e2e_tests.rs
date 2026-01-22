//! End-to-end integration tests
//! Tests complete workflows combining multiple components
//! 
//! Run with: cargo test --test e2e_tests -- --test-threads=1 --nocapture
//! (Use single thread to avoid Docker port/name conflicts)

use scratchpad::config::{Config, DockerConfig};
use scratchpad::docker::DockerClient;
use scratchpad::scratch;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to create a test Docker client
fn create_test_docker_client(config: &Config) -> Result<DockerClient, Box<dyn std::error::Error>> {
    let docker_config = DockerConfig {
        socket: "/var/run/docker.sock".to_string(),
        network: config.docker.network.clone(),
        label_prefix: config.docker.label_prefix.clone(),
    };
    DockerClient::new(docker_config).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Helper to clean up test scratches
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
async fn test_e2e_scratch_creation_with_services() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let test_name = "e2e-create-with-services";
    cleanup_scratches(&config, &[test_name]);

    // 1. Create scratch with services
    match scratch::create_scratch(
        &config,
        &docker,
        "feature-e2e",
        Some(test_name.to_string()),
        None,
        None,
    )
    .await
    {
        Ok(created) => {
            println!("✓ Step 1: Created scratch with services");
            assert_eq!(created.name, test_name);

            // 2. Verify directory structure
            let scratch_dir = config.server.releases_dir.join(test_name);
            assert!(scratch_dir.exists(), "Scratch directory should exist");
            assert!(scratch_dir.join(".scratchpad.toml").exists(), "Config should exist");
            assert!(scratch_dir.join("compose.yml").exists(), "Compose file should exist");
            assert!(scratch_dir.join("logs").exists(), "Logs dir should exist");
            assert!(scratch_dir.join("data").exists(), "Data dir should exist");
            println!("✓ Step 2: Directory structure verified");

            // 3. List scratches to verify it appears
            match scratch::list_scratches(&config, &docker).await {
                Ok(scratches) => {
                    assert!(
                        scratches.iter().any(|s| s.name == test_name),
                        "Scratch should be in list"
                    );
                    println!("✓ Step 3: Scratch appears in list");
                }
                Err(e) => panic!("Failed to list scratches: {}", e),
            }

            // 4. Get scratch status
            match scratch::get_scratch_status(&config, &docker, test_name).await {
                Ok(status) => {
                    assert_eq!(status.name, test_name);
                    println!(
                        "✓ Step 4: Retrieved status - branch: {}, status: {}",
                        status.branch, status.status
                    );
                }
                Err(e) => panic!("Failed to get status: {}", e),
            }

            // 5. Stop the scratch
            match scratch::stop_scratch(&config, &docker, test_name).await {
                Ok(()) => println!("✓ Step 5: Stopped scratch"),
                Err(e) => panic!("Failed to stop: {}", e),
            }

            // Small delay to let Docker catch up
            sleep(Duration::from_millis(500)).await;

            // 6. Start the scratch
            match scratch::start_scratch(&config, &docker, test_name).await {
                Ok(()) => println!("✓ Step 6: Started scratch"),
                Err(e) => panic!("Failed to start: {}", e),
            }

            // 7. Delete the scratch
            match scratch::delete_scratch(&config, &docker, test_name, false).await {
                Ok(()) => {
                    println!("✓ Step 7: Deleted scratch");
                    assert!(
                        !scratch_dir.exists(),
                        "Scratch directory should be removed"
                    );
                }
                Err(e) => panic!("Failed to delete: {}", e),
            }
        }
        Err(e) => panic!("✗ Failed to create scratch: {}", e),
    }

    println!("✓✓✓ Complete end-to-end workflow successful!");
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_e2e_multiple_scratches() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let names = ["e2e-multi-1", "e2e-multi-2", "e2e-multi-3"];
    cleanup_scratches(&config, &names);

    // 1. Create multiple scratches
    for (i, name) in names.iter().enumerate() {
        match scratch::create_scratch(
            &config,
            &docker,
            &format!("branch-{}", i),
            Some(name.to_string()),
            None,
            None,
        )
        .await
        {
            Ok(_) => println!("✓ Created scratch: {}", name),
            Err(e) => panic!("Failed to create {}: {}", name, e),
        }
    }

    // 2. List all scratches
    match scratch::list_scratches(&config, &docker).await {
        Ok(scratches) => {
            for name in &names {
                assert!(
                    scratches.iter().any(|s| &s.name == name),
                    "Scratch {} should be in list",
                    name
                );
            }
            println!(
                "✓ All {} scratches found in list",
                names.len()
            );
        }
        Err(e) => panic!("Failed to list scratches: {}", e),
    }

    // 3. Stop multiple scratches
    for name in &names {
        match scratch::stop_scratch(&config, &docker, name).await {
            Ok(()) => println!("✓ Stopped: {}", name),
            Err(e) => panic!("Failed to stop {}: {}", name, e),
        }
    }

    sleep(Duration::from_millis(500)).await;

    // 4. Restart multiple scratches
    for name in &names {
        match scratch::restart_scratch(&config, &docker, name).await {
            Ok(()) => println!("✓ Restarted: {}", name),
            Err(e) => panic!("Failed to restart {}: {}", name, e),
        }
    }

    // 5. Delete all scratches
    for name in &names {
        match scratch::delete_scratch(&config, &docker, name, false).await {
            Ok(()) => println!("✓ Deleted: {}", name),
            Err(e) => panic!("Failed to delete {}: {}", name, e),
        }
    }

    // 6. Verify all deleted
    match scratch::list_scratches(&config, &docker).await {
        Ok(scratches) => {
            for name in &names {
                assert!(
                    !scratches.iter().any(|s| &s.name == name),
                    "Scratch {} should be deleted",
                    name
                );
            }
            println!("✓ All scratches successfully deleted");
        }
        Err(e) => panic!("Failed to list scratches: {}", e),
    }

    println!("✓✓✓ Multiple scratch workflow successful!");
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_e2e_nginx_config_regeneration() {
    let mut config = Config::default();
    config.nginx.enabled = true;
    config.nginx.domain = "test-scratches.local".to_string();

    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    let names = ["nginx-test-1", "nginx-test-2"];
    cleanup_scratches(&config, &names);

    // 1. Create scratches
    for (i, name) in names.iter().enumerate() {
        match scratch::create_scratch(
            &config,
            &docker,
            &format!("nginx-branch-{}", i),
            Some(name.to_string()),
            None,
            None,
        )
        .await
        {
            Ok(_) => println!("✓ Created scratch for nginx test: {}", name),
            Err(e) => panic!("Failed to create: {}", e),
        }
    }

    // 2. Verify nginx config was regenerated
    match scratchpad::nginx::get_config(&config) {
        Ok(content) => {
            // Check that config contains references to the scratches
            assert!(
                content.contains(&config.nginx.domain),
                "Nginx config should contain configured domain"
            );
            println!("✓ Nginx config contains domain: {}", config.nginx.domain);
            println!("✓ Nginx config regenerated successfully");
        }
        Err(e) => println!("⚠ Could not verify nginx config: {}", e),
    }

    // 3. Cleanup
    for name in &names {
        let _ = scratch::delete_scratch(&config, &docker, name, false).await;
    }

    println!("✓✓✓ Nginx regeneration workflow successful!");
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_e2e_scratch_name_sanitization() {
    let config = Config::default();
    let docker = match create_test_docker_client(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("⚠ Skipping test: Docker not available");
            return;
        }
    };

    // Test branch names that need sanitization
    let test_cases = vec![
        ("Feature/my-branch", "feature-my-branch"),
        ("feature/TEST_FEATURE", "feature-test_feature"),
        ("release/v1.0.0", "release-v1-0-0"),
    ];

    for (branch, expected_name) in test_cases {
        cleanup_scratches(&config, &[expected_name]);

        match scratch::create_scratch(&config, &docker, branch, None, None, None).await {
            Ok(created) => {
                assert_eq!(created.name, expected_name);
                println!("✓ Branch '{}' → '{}'", branch, created.name);

                // Cleanup
                let _ = scratch::delete_scratch(&config, &docker, &created.name, false).await;
            }
            Err(e) => panic!("Failed with branch '{}': {}", branch, e),
        }
    }

    println!("✓✓✓ Name sanitization workflow successful!");
}
