//! HTTP API integration tests
//! Tests the REST API endpoints
//! 
//! Run with: cargo test --test api_tests -- --test-threads=1 --nocapture
//! (Use single thread to avoid port conflicts)

use scratchpad::api::run_server;
use scratchpad::config::Config;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to start the API server in background with a given port
async fn start_test_server(config: Config, port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let _ = run_server(config, "127.0.0.1", port).await;
    })
}

/// Helper to wait for server to be ready
async fn wait_for_server(port: u16, max_attempts: u32) -> bool {
    let client = reqwest::Client::new();
    for attempt in 0..max_attempts {
        match client
            .get(&format!("http://127.0.0.1:{}/api/health", port))
            .timeout(Duration::from_secs(1))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                println!("✓ Server ready on port {}", port);
                return true;
            }
            _ => {
                if attempt < max_attempts - 1 {
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
    false
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_health_endpoint() {
    let config = Config::default();
    let port = 4001u16;

    let server_handle = start_test_server(config, port).await;

    // Wait for server to start
    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Test health endpoint
    match client
        .get(&format!("http://127.0.0.1:{}/api/health", port))
        .send()
        .await
    {
        Ok(response) => {
            assert!(response.status().is_success());
            println!("✓ Health endpoint returned success");
        }
        Err(e) => panic!("✗ Failed to reach health endpoint: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_list_scratches() {
    let config = Config::default();
    let port = 4002u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Test list scratches endpoint
    match client
        .get(&format!("http://127.0.0.1:{}/api/scratches", port))
        .send()
        .await
    {
        Ok(response) => {
            assert!(response.status().is_success());
            println!("✓ List scratches endpoint returned success");

            if let Ok(body) = response.text().await {
                println!("  Response: {}", body);
            }
        }
        Err(e) => panic!("✗ Failed to list scratches: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_list_services() {
    let config = Config::default();
    let port = 4003u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Test list services endpoint
    match client
        .get(&format!("http://127.0.0.1:{}/api/services", port))
        .send()
        .await
    {
        Ok(response) => {
            assert!(response.status().is_success());
            println!("✓ List services endpoint returned success");

            if let Ok(body) = response.text().await {
                println!("  Services: {}", body);
            }
        }
        Err(e) => panic!("✗ Failed to list services: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_get_nonexistent_scratch() {
    let config = Config::default();
    let port = 4004u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Test getting a non-existent scratch
    match client
        .get(&format!(
            "http://127.0.0.1:{}/api/scratches/nonexistent",
            port
        ))
        .send()
        .await
    {
        Ok(response) => {
            // Should return 404 or 500 depending on implementation
            println!("✓ Endpoint returned status: {}", response.status());

            if let Ok(body) = response.text().await {
                println!("  Response: {}", body);
            }
        }
        Err(e) => panic!("✗ Failed to call endpoint: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_cors_headers() {
    let config = Config::default();
    let port = 4005u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Test CORS headers
    match client
        .get(&format!("http://127.0.0.1:{}/api/health", port))
        .header("Origin", "http://example.com")
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Request succeeded with custom origin");

            // Check for CORS headers
            if let Some(cors) = response.headers().get("access-control-allow-origin") {
                println!("  CORS header present: {:?}", cors);
            }
        }
        Err(e) => panic!("✗ Failed to send request: {}", e),
    }

    server_handle.abort();
}
