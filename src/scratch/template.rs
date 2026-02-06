//! Template rendering for scratch environments

use minijinja::{context, Environment};
use std::collections::HashMap;

use crate::config::Config;
use crate::error::Result;

use super::Scratch;

/// Default compose template
const DEFAULT_COMPOSE_TEMPLATE: &str = r#"
services:
{% for service in services %}
  {{ service.name }}:
    image: "{{ service.image }}"
    container_name: "{{ scratch.name }}-{{ service.name }}"
    restart: unless-stopped
{% if service.ports %}
    ports:
{% for port in service.ports %}
      - "{{ port }}"
{% endfor %}
{% endif %}
{% if service.environment %}
    environment:
{% for key, value in service.environment|items %}
      {{ key }}: "{{ value }}"
{% endfor %}
{% endif %}
{% if service.volumes %}
    volumes:
{% for volume in service.volumes %}
      - "{{ volume }}"
{% endfor %}
{% endif %}
{% if service.healthcheck %}
    healthcheck:
      test: ["CMD-SHELL", "{{ service.healthcheck }}"]
      interval: 10s
      timeout: 5s
      retries: 3
{% endif %}
    networks:
      - {{ network.name }}
{% endfor %}

networks:
  {{ network.name }}:
    external: true
"#;

/// Render a template for a scratch environment
pub fn render_template(config: &Config, scratch: &Scratch) -> Result<String> {
    let mut env = Environment::new();
    env.add_template("compose", DEFAULT_COMPOSE_TEMPLATE)?;

    // Build service configs
    let mut services_data: Vec<HashMap<String, serde_json::Value>> = Vec::new();

    for service_name in &scratch.services {
        if let Some(service_config) = config.get_service(service_name) {
            // Skip shared services - they're managed separately
            if service_config.shared {
                continue;
            }

            let mut service_data: HashMap<String, serde_json::Value> = HashMap::new();
            service_data.insert("name".to_string(), service_name.clone().into());
            service_data.insert("image".to_string(), service_config.image.clone().into());

            // Environment variables
            let mut env_vars: HashMap<String, String> = service_config.env.clone();

            // Add database connection info if applicable
            if let Some(dbs) = scratch.databases.get("postgres") {
                if let Some(db_name) = dbs.first() {
                    env_vars.insert(
                        "DATABASE_URL".to_string(),
                        format!(
                            "postgres://postgres:postgres@scratchpad-postgres:5432/{}",
                            db_name
                        ),
                    );
                }
            }

            // Add Redis URL if redis is a configured service
            if config.services.contains_key("redis") {
                env_vars.insert(
                    "REDIS_URL".to_string(),
                    "redis://scratchpad-redis:6379".to_string(),
                );
            }

            if !env_vars.is_empty() {
                service_data.insert("environment".to_string(), serde_json::to_value(env_vars)?);
            }

            // Volumes
            if !service_config.volumes.is_empty() {
                service_data.insert(
                    "volumes".to_string(),
                    serde_json::to_value(&service_config.volumes)?,
                );
            }

            // Port mapping (host:container)
            if let Some(host_port) = service_config.port {
                let container_port = service_config.internal_port.unwrap_or(host_port);
                service_data.insert(
                    "ports".to_string(),
                    serde_json::to_value(vec![format!("{}:{}", host_port, container_port)])?,
                );
            }

            // Healthcheck
            if let Some(healthcheck) = &service_config.healthcheck {
                service_data.insert("healthcheck".to_string(), healthcheck.clone().into());
            }

            services_data.push(service_data);
        }
    }

    // Build network info
    let mut network_data: HashMap<String, String> = HashMap::new();
    network_data.insert("name".to_string(), config.docker.network.clone());

    // Build scratch info
    let mut scratch_data: HashMap<String, String> = HashMap::new();
    scratch_data.insert("name".to_string(), scratch.name.clone());
    scratch_data.insert("branch".to_string(), scratch.branch.clone());

    let template = env.get_template("compose")?;
    let rendered = template.render(context! {
        scratch => scratch_data,
        services => services_data,
        network => network_data,
    })?;

    Ok(rendered)
}

/// Load a custom template from a file
#[allow(dead_code)]
pub fn load_custom_template(path: &std::path::Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

/// Render a custom template
#[allow(dead_code)]
pub fn render_custom_template(
    template_content: &str,
    config: &Config,
    scratch: &Scratch,
) -> Result<String> {
    let mut env = Environment::new();
    env.add_template("custom", template_content)?;

    // Build context
    let mut scratch_data: HashMap<String, serde_json::Value> = HashMap::new();
    scratch_data.insert("name".to_string(), scratch.name.clone().into());
    scratch_data.insert("branch".to_string(), scratch.branch.clone().into());
    scratch_data.insert(
        "services".to_string(),
        serde_json::to_value(&scratch.services)?,
    );
    scratch_data.insert(
        "databases".to_string(),
        serde_json::to_value(&scratch.databases)?,
    );
    scratch_data.insert("env".to_string(), serde_json::to_value(&scratch.env)?);

    let mut network_data: HashMap<String, String> = HashMap::new();
    network_data.insert("name".to_string(), config.docker.network.clone());

    let template = env.get_template("custom")?;
    let rendered = template.render(context! {
        scratch => scratch_data,
        network => network_data,
        config => config,
    })?;

    Ok(rendered)
}
