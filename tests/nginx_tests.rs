//! Nginx configuration tests

#[cfg(test)]
mod tests {
    use scratchpad::config::{Config, NginxRouting};

    #[test]
    fn test_nginx_routing_enum_subdomain() {
        let routing = NginxRouting::Subdomain;
        assert_eq!(routing, NginxRouting::Subdomain);
        println!("✓ NginxRouting::Subdomain enum works");
    }

    #[test]
    fn test_nginx_routing_enum_path() {
        let routing = NginxRouting::Path;
        assert_eq!(routing, NginxRouting::Path);
        println!("✓ NginxRouting::Path enum works");
    }

    #[test]
    fn test_nginx_config_default() {
        let config = Config::default();
        assert!(config.nginx.enabled);
        assert_eq!(config.nginx.domain, "scratches.localhost");
        assert_eq!(config.nginx.routing, NginxRouting::Subdomain);
        println!("✓ Default nginx config has correct defaults");
    }

    #[test]
    fn test_nginx_config_custom_domain() {
        let mut config = Config::default();
        config.nginx.domain = "example.com".to_string();
        assert_eq!(config.nginx.domain, "example.com");
        println!("✓ Can set custom nginx domain");
    }

    #[test]
    fn test_nginx_config_path_routing() {
        let mut config = Config::default();
        config.nginx.routing = NginxRouting::Path;
        assert_eq!(config.nginx.routing, NginxRouting::Path);
        println!("✓ Can set nginx routing to path");
    }

    #[tokio::test]
    async fn test_nginx_regenerate_with_docker() {
        use scratchpad::config::DockerConfig;
        use scratchpad::docker::DockerClient;

        let mut config = Config::default();
        config.nginx.ingress_service = Some("api".to_string());

        let docker_config = DockerConfig {
            socket: "/var/run/docker.sock".to_string(),
            network: "scratchpad-network".to_string(),
            label_prefix: "scratchpad".to_string(),
        };

        let docker = match DockerClient::new(docker_config) {
            Ok(client) => client,
            Err(_) => {
                println!("⚠ Skipping Docker integration test: Docker not available");
                return;
            }
        };

        // This creates the nginx config file
        match scratchpad::nginx::regenerate_config(&config, &docker).await {
            Ok(()) => {
                println!("✓ Successfully regenerated nginx config with Docker client");
                // The config file is created in the target directory
            }
            Err(e) => panic!("✗ Failed to regenerate nginx config: {}", e),
        }
    }
}
