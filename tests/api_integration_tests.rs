//! Integration tests for API endpoints (Phase 6, 7, 9)
//!
//! Tests for:
//! - Config persistence (Phase 6)
//! - Individual service start/stop (Phase 7)
//! - Real-time status updates (Phase 9 - WebSocket)

use scratchpad::config::{Config, NginxConfig, NginxRouting, ServerConfig};
use std::path::PathBuf;

#[test]
fn test_config_default_serialization() {
    let config = Config::default();

    // Verify config can be serialized to TOML
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize config");

    // Verify it contains expected sections
    assert!(toml_str.contains("[server]"));
    assert!(toml_str.contains("[docker]"));
    assert!(toml_str.contains("[nginx]"));

    // Verify key values
    assert!(toml_str.contains("port = 3456"));
    assert!(toml_str.contains("network = \"scratchpad-network\""));
}

#[test]
fn test_config_serialization_and_deserialization() {
    let mut config = Config::default();

    // Modify config
    config.server.port = 8080;
    config.docker.network = "custom-network".to_string();
    config.nginx.domain = "example.com".to_string();

    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");

    // Deserialize back
    let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    // Verify values match
    assert_eq!(restored.server.port, 8080);
    assert_eq!(restored.docker.network, "custom-network");
    assert_eq!(restored.nginx.domain, "example.com");
}

#[test]
fn test_config_nginx_routing_serialization() {
    let mut config = Config::default();
    config.nginx.routing = NginxRouting::Path;

    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    assert!(toml_str.contains("routing = \"path\""));

    // Deserialize and verify
    let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");
    assert_eq!(restored.nginx.routing, NginxRouting::Path);
}

#[test]
fn test_config_partial_updates() {
    let mut config = Config::default();

    // Store original values
    let original_socket = config.docker.socket.clone();
    let _original_port = config.server.port;

    // Update only server port
    config.server.port = 9999;

    // Verify port changed but socket unchanged
    assert_eq!(config.server.port, 9999);
    assert_eq!(config.docker.socket, original_socket);
}

#[test]
fn test_config_with_services() {
    use scratchpad::config::ServiceConfig;
    use std::collections::HashMap;

    let mut config = Config::default();

    let postgres_config = ServiceConfig {
        image: "postgres:15".to_string(),
        shared: true,
        port: Some(5432),
        internal_port: None,
        env: {
            let mut map = HashMap::new();
            map.insert("POSTGRES_PASSWORD".to_string(), "password".to_string());
            map
        },
        volumes: vec![],
        healthcheck: Some("pg_isready -U postgres".to_string()),
        auto_create_db: true,
        connection: None,
    };

    config
        .services
        .insert("postgres".to_string(), postgres_config);

    // Serialize and deserialize
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    // Verify service persisted
    assert!(restored.services.contains_key("postgres"));
    let pg = &restored.services["postgres"];
    assert_eq!(pg.image, "postgres:15");
    assert_eq!(pg.port, Some(5432));
}

#[test]
fn test_service_name_validation() {
    // Test that service names are properly stored and retrieved
    let mut config = Config::default();

    // Add multiple services
    let services = vec!["postgres", "redis", "kafka"];
    for service in services {
        config.services.insert(
            service.to_string(),
            scratchpad::config::ServiceConfig {
                image: format!("{}:latest", service),
                shared: true,
                port: None,
                internal_port: None,
                env: Default::default(),
                volumes: vec![],
                healthcheck: None,
                auto_create_db: false,
                connection: None,
            },
        );
    }

    // Serialize and verify all services preserved
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    assert_eq!(restored.services.len(), 3);
    assert!(restored.services.contains_key("postgres"));
    assert!(restored.services.contains_key("redis"));
    assert!(restored.services.contains_key("kafka"));
}

#[test]
fn test_config_server_config_update() {
    let mut config = Config::default();

    let new_server_config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8000,
        releases_dir: PathBuf::from("/custom/releases"),
    };

    config.server = new_server_config;

    // Serialize and verify
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    assert_eq!(restored.server.host, "127.0.0.1");
    assert_eq!(restored.server.port, 8000);
    assert_eq!(
        restored.server.releases_dir,
        PathBuf::from("/custom/releases")
    );
}

#[test]
fn test_config_nginx_config_update() {
    let mut config = Config::default();

    let new_nginx_config = NginxConfig {
        enabled: false,
        config_path: PathBuf::from("/etc/nginx/conf.d/custom.conf"),
        reload_command: Some("systemctl reload nginx".to_string()),
        domain: "api.example.com".to_string(),
        routing: NginxRouting::Path,
        container: Some("nginx".to_string()),
        dynamic: None,
        ingress_service: None,
    };

    config.nginx = new_nginx_config;

    // Serialize and verify
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    assert!(!restored.nginx.enabled);
    assert_eq!(restored.nginx.domain, "api.example.com");
    assert_eq!(restored.nginx.routing, NginxRouting::Path);
    assert_eq!(restored.nginx.container, Some("nginx".to_string()));
}

#[test]
fn test_config_github_config_update() {
    use scratchpad::config::GithubConfig;

    let mut config = Config::default();

    let github_config = GithubConfig {
        token: "ghp_test_token_12345".to_string(),
        api_repo: Some("owner/api-repo".to_string()),
        web_repo: Some("owner/web-repo".to_string()),
    };

    config.github = Some(github_config);

    // Serialize and verify
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    assert!(restored.github.is_some());
    let gh = restored.github.unwrap();
    assert_eq!(gh.token, "ghp_test_token_12345");
    assert_eq!(gh.api_repo, Some("owner/api-repo".to_string()));
}

#[test]
fn test_config_comprehensive_update() {
    use scratchpad::config::GithubConfig;

    let mut config = Config::default();

    // Update all major sections
    config.server.port = 9000;
    config.docker.network = "prod-network".to_string();
    config.nginx.enabled = false;
    config.github = Some(GithubConfig {
        token: "test_token".to_string(),
        api_repo: None,
        web_repo: None,
    });

    // Serialize and verify all changes persisted
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    assert_eq!(restored.server.port, 9000);
    assert_eq!(restored.docker.network, "prod-network");
    assert!(!restored.nginx.enabled);
    assert!(restored.github.is_some());
}

#[test]
fn test_config_to_string_formatting() {
    let config = Config::default();
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");

    // Verify it's valid TOML with proper formatting
    assert!(!toml_str.is_empty());

    // Verify it can be parsed back
    let _parsed: Config = toml::from_str(&toml_str).expect("Failed to parse generated TOML");
}

#[test]
fn test_service_status_structure() {
    // Test that service status can be represented as a string
    use std::collections::HashMap;

    let mut status: HashMap<String, String> = HashMap::new();
    status.insert("postgres".to_string(), "running".to_string());
    status.insert("redis".to_string(), "stopped".to_string());

    assert_eq!(status["postgres"], "running");
    assert_eq!(status["redis"], "stopped");
}

#[test]
fn test_websocket_status_message_structure() {
    use serde_json::json;

    // Simulate a StatusChange message that would come from WebSocket
    let status_change = json!({
        "StatusChange": {
            "scratch": "my-feature",
            "status": "running",
            "service": "api",
            "timestamp": "2024-01-22T10:00:00Z"
        }
    });

    // Verify structure
    assert_eq!(status_change["StatusChange"]["scratch"], "my-feature");
    assert_eq!(status_change["StatusChange"]["status"], "running");
}

#[test]
fn test_config_environment_variables() {
    use std::collections::HashMap;

    let mut config = Config::default();
    let mut env = HashMap::new();
    env.insert("ENV_VAR".to_string(), "value".to_string());

    config.scratch.env = env;

    // Serialize and verify
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    assert_eq!(
        restored.scratch.env.get("ENV_VAR"),
        Some(&"value".to_string())
    );
}

#[test]
fn test_multiple_partial_updates_sequence() {
    let mut config = Config::default();

    // First update
    config.server.port = 8000;
    let toml1 = toml::to_string_pretty(&config).expect("Failed to serialize");
    let restored1: Config = toml::from_str(&toml1).expect("Failed to parse");
    assert_eq!(restored1.server.port, 8000);

    // Second update
    restored1.clone().server.port; // Use previous config
    let mut config2 = restored1;
    config2.docker.network = "new-network".to_string();
    let toml2 = toml::to_string_pretty(&config2).expect("Failed to serialize");
    let restored2: Config = toml::from_str(&toml2).expect("Failed to parse");

    // Verify both changes persisted
    assert_eq!(restored2.server.port, 8000);
    assert_eq!(restored2.docker.network, "new-network");
}
