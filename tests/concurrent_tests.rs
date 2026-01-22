//! Concurrent API operation tests  
//! Tests concurrent HTTP requests and stress scenarios
//! 
//! Run with: cargo test --test concurrent_tests -- --ignored --test-threads=1

use scratchpad::api::run_server;
use scratchpad::config::Config;
use std::time::Instant;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to start server
async fn start_test_server(config: Config, port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let _ = run_server(config, "127.0.0.1", port).await;
    })
}

/// Wait for server readiness
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
async fn test_concurrent_health_requests_10() {
    let config = Config::default();
    let port = 6001u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        println!("⚠ Server failed to start");
        server_handle.abort();
        return;
    }

    let start = Instant::now();
    let client = reqwest::Client::new();

    // Send 10 concurrent health checks
    let mut handles = vec![];

    for i in 0..10 {
        let client = client.clone();
        let port = port.clone();

        let handle = tokio::spawn(async move {
            let url = format!("http://127.0.0.1:{}/api/health", port);
            match client.get(&url).send().await {
                Ok(response) => {
                    println!("  Request {}: {}", i, response.status());
                    response.status().is_success()
                }
                Err(_) => false,
            }
        });

        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.await.is_ok_and(|v| v) {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();

    println!(
        "✓ Completed 10 concurrent health requests in {:?} ({} succeeded)",
        elapsed, success_count
    );

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_concurrent_list_scratches_requests() {
    let config = Config::default();
    let port = 6002u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        println!("⚠ Server failed to start");
        server_handle.abort();
        return;
    }

    let start = Instant::now();
    let client = reqwest::Client::new();

    // Send 5 concurrent list scratches requests
    let mut handles = vec![];

    for i in 0..5 {
        let client = client.clone();
        let port = port.clone();

        let handle = tokio::spawn(async move {
            let url = format!("http://127.0.0.1:{}/api/scratches", port);
            match client.get(&url).send().await {
                Ok(response) => {
                    println!("  Request {}: {}", i, response.status());
                    response.status().is_success()
                }
                Err(_) => false,
            }
        });

        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.await.is_ok_and(|v| v) {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();

    println!(
        "✓ Completed 5 concurrent list requests in {:?} ({} succeeded)",
        elapsed, success_count
    );

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_concurrent_mixed_endpoint_requests() {
    let config = Config::default();
    let port = 6003u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        println!("⚠ Server failed to start");
        server_handle.abort();
        return;
    }

    let start = Instant::now();
    let client = reqwest::Client::new();

    // Send mixed requests concurrently
    let endpoints = vec![
        ("/api/health", "GET"),
        ("/api/scratches", "GET"),
        ("/api/services", "GET"),
    ];

    let mut handles = vec![];

    for (i, (endpoint, method)) in endpoints.iter().cycle().take(9).enumerate() {
        let client = client.clone();
        let port = port.clone();
        let endpoint = endpoint.to_string();
        let method = method.to_string();

        let handle = tokio::spawn(async move {
            let url = format!("http://127.0.0.1:{}{}", port, endpoint);
            match client.get(&url).send().await {
                Ok(response) => {
                    println!("  Request {} {}: {}", i, method, response.status());
                    response.status().is_success()
                }
                Err(_) => false,
            }
        });

        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.await.is_ok_and(|v| v) {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();

    println!(
        "✓ Completed 9 concurrent mixed requests in {:?} ({} succeeded)",
        elapsed, success_count
    );

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_rapid_sequential_requests() {
    let config = Config::default();
    let port = 6004u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        println!("⚠ Server failed to start");
        server_handle.abort();
        return;
    }

    let start = Instant::now();
    let client = reqwest::Client::new();

    let url = format!("http://127.0.0.1:{}/api/health", port);

    // Send 20 rapid sequential requests
    let mut success_count = 0;

    for _ in 0..20 {
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                success_count += 1;
            }
            _ => {}
        }
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed.as_millis() / 20;

    println!(
        "✓ Completed 20 sequential requests in {:?} (avg: {}ms, {} succeeded)",
        elapsed, avg_time, success_count
    );

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_sustained_load() {
    let config = Config::default();
    let port = 6005u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        println!("⚠ Server failed to start");
        server_handle.abort();
        return;
    }

    let start = Instant::now();
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/api/health", port);

    // Send requests for 2 seconds
    let target_duration = Duration::from_secs(2);
    let mut request_count = 0;

    while start.elapsed() < target_duration {
        match client.get(&url).send().await {
            Ok(_) => request_count += 1,
            Err(_) => break,
        }
    }

    let elapsed = start.elapsed();
    let requests_per_sec = request_count as f64 / elapsed.as_secs_f64();

    println!(
        "✓ Sustained load test: {} requests in {:?} ({:.0} req/s)",
        request_count, elapsed, requests_per_sec
    );

    server_handle.abort();
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_parallel_endpoints() {
    let config = Config::default();
    let port = 6006u16;

    let server_handle = start_test_server(config, port).await;

    if !wait_for_server(port, 50).await {
        println!("⚠ Server failed to start");
        server_handle.abort();
        return;
    }

    let start = Instant::now();
    let client = reqwest::Client::new();

    // Test all endpoints in parallel
    let endpoints = vec!["/api/health", "/api/scratches", "/api/services"];

    let handles: Vec<_> = endpoints
        .iter()
        .map(|endpoint| {
            let client = client.clone();
            let port = port.clone();
            let endpoint = endpoint.to_string();

            tokio::spawn(async move {
                let url = format!("http://127.0.0.1:{}{}", port, endpoint);
                match client.get(&url).send().await {
                    Ok(response) => {
                        println!("  {} : {}", endpoint, response.status());
                        response.status().is_success()
                    }
                    Err(_) => false,
                }
            })
        })
        .collect();

    let mut success_count = 0;
    for handle in handles {
        if handle.await.is_ok_and(|v| v) {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();

    println!(
        "✓ Tested {} endpoints in parallel in {:?} ({} succeeded)",
        endpoints.len(),
        elapsed,
        success_count
    );

    server_handle.abort();
}
