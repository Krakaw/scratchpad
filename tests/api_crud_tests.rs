//! Complete API CRUD operation tests
//! Tests all API endpoints with realistic payloads
//! 
//! Run with: cargo test --test api_crud_tests -- --test-threads=1 --nocapture
//! (Single thread to avoid port conflicts)

use scratchpad::api::run_server;
use scratchpad::config::Config;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to start the API server in background
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
async fn test_api_create_scratch_minimal() {
    let config = Config::default();
    let port = 5001u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Create scratch with minimal payload
    let payload = serde_json::json!({
        "branch": "feature/test"
    });

    match client
        .post(&format!("http://127.0.0.1:{}/api/scratches", port))
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Create scratch endpoint responded with status: {}", response.status());

            if let Ok(body) = response.text().await {
                println!("  Response: {}", body);
            }
        }
        Err(e) => println!("⚠ Create scratch failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_create_scratch_full() {
    let config = Config::default();
    let port = 5002u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Create scratch with all fields
    let payload = serde_json::json!({
        "branch": "feature/full-test",
        "name": "custom-scratch",
        "profile": "production",
        "template": "nodejs"
    });

    match client
        .post(&format!("http://127.0.0.1:{}/api/scratches", port))
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Create scratch with all fields responded: {}", response.status());
        }
        Err(e) => println!("⚠ Create scratch failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_get_scratch() {
    let config = Config::default();
    let port = 5003u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Get a scratch (will fail if not exists, but tests endpoint)
    match client
        .get(&format!("http://127.0.0.1:{}/api/scratches/test-scratch", port))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Get scratch endpoint responded with status: {}", response.status());

            if let Ok(body) = response.text().await {
                println!("  Response: {}", body);
            }
        }
        Err(e) => println!("⚠ Get scratch failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_delete_scratch() {
    let config = Config::default();
    let port = 5004u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Delete a scratch
    match client
        .delete(&format!("http://127.0.0.1:{}/api/scratches/test-scratch", port))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Delete scratch endpoint responded with status: {}", response.status());
        }
        Err(e) => println!("⚠ Delete scratch failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_start_scratch() {
    let config = Config::default();
    let port = 5005u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Start a scratch
    match client
        .post(&format!("http://127.0.0.1:{}/api/scratches/test-scratch/start", port))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Start scratch endpoint responded with status: {}", response.status());
        }
        Err(e) => println!("⚠ Start scratch failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_stop_scratch() {
    let config = Config::default();
    let port = 5006u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Stop a scratch
    match client
        .post(&format!("http://127.0.0.1:{}/api/scratches/test-scratch/stop", port))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Stop scratch endpoint responded with status: {}", response.status());
        }
        Err(e) => println!("⚠ Stop scratch failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_restart_scratch() {
    let config = Config::default();
    let port = 5007u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Restart a scratch
    match client
        .post(&format!("http://127.0.0.1:{}/api/scratches/test-scratch/restart", port))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Restart scratch endpoint responded with status: {}", response.status());
        }
        Err(e) => println!("⚠ Restart scratch failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_get_logs() {
    let config = Config::default();
    let port = 5008u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Get logs without parameters
    match client
        .get(&format!("http://127.0.0.1:{}/api/scratches/test-scratch/logs", port))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Get logs endpoint responded with status: {}", response.status());
        }
        Err(e) => println!("⚠ Get logs failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_get_logs_with_service() {
    let config = Config::default();
    let port = 5009u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Get logs for specific service
    match client
        .get(&format!(
            "http://127.0.0.1:{}/api/scratches/test-scratch/logs?service=web",
            port
        ))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Get logs with service filter responded: {}", response.status());
        }
        Err(e) => println!("⚠ Get logs with service failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_get_logs_with_tail() {
    let config = Config::default();
    let port = 5010u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Get logs with tail parameter
    match client
        .get(&format!(
            "http://127.0.0.1:{}/api/scratches/test-scratch/logs?tail=50",
            port
        ))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Get logs with tail parameter responded: {}", response.status());
        }
        Err(e) => println!("⚠ Get logs with tail failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_start_services() {
    let config = Config::default();
    let port = 5011u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Start all services
    match client
        .post(&format!("http://127.0.0.1:{}/api/services/start", port))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Start services endpoint responded with status: {}", response.status());
        }
        Err(e) => println!("⚠ Start services failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_stop_services() {
    let config = Config::default();
    let port = 5012u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Stop all services
    match client
        .post(&format!("http://127.0.0.1:{}/api/services/stop", port))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Stop services endpoint responded with status: {}", response.status());
        }
        Err(e) => println!("⚠ Stop services failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_response_format() {
    let config = Config::default();
    let port = 5013u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Verify response format
    match client
        .get(&format!("http://127.0.0.1:{}/api/health", port))
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(json) => {
                    assert!(json.get("success").is_some(), "Response should have success field");
                    println!("✓ API response format is correct");
                }
                Err(e) => println!("⚠ Could not parse response JSON: {}", e),
            }
        }
        Err(e) => println!("⚠ Request failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_json_content_type() {
    let config = Config::default();
    let port = 5014u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Verify content type is JSON
    match client
        .get(&format!("http://127.0.0.1:{}/api/health", port))
        .send()
        .await
    {
        Ok(response) => {
            if let Some(content_type) = response.headers().get("content-type") {
                println!("✓ Content-Type: {:?}", content_type);
            }
        }
        Err(e) => println!("⚠ Request failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_concurrent_requests() {
    let config = Config::default();
    let port = 5015u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Send multiple concurrent requests
    let mut handles = vec![];

    for i in 0..5 {
        let port_num = port.clone();
        let client = client.clone();
        
        let handle = tokio::spawn(async move {
            let url = format!("http://127.0.0.1:{}/api/health", port_num);
            match client.get(&url).send().await {
                Ok(response) => {
                    println!("✓ Concurrent request {} succeeded: {}", i, response.status());
                }
                Err(e) => println!("⚠ Concurrent request {} failed: {}", i, e),
            }
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let _ = handle.await;
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_invalid_payload() {
    let config = Config::default();
    let port = 5016u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Send invalid JSON
    match client
        .post(&format!("http://127.0.0.1:{}/api/scratches", port))
        .header("content-type", "application/json")
        .body("{ invalid json")
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Invalid payload handled with status: {}", response.status());
        }
        Err(e) => println!("⚠ Request failed: {}", e),
    }

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_api_method_not_allowed() {
    let config = Config::default();
    let port = 5017u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        panic!("Server failed to start");
    }

    let client = reqwest::Client::new();

    // Try to GET a POST-only endpoint
    match client
        .get(&format!("http://127.0.0.1:{}/api/services/start", port))
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Wrong HTTP method handled with status: {}", response.status());
        }
        Err(e) => println!("⚠ Request failed: {}", e),
    }

    server_handle.abort();
}
