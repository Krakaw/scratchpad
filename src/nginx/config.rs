//! Nginx configuration generation

use std::fs;

use crate::config::{Config, NginxRouting};
use crate::docker::DockerClient;
use crate::error::Result;
use crate::scratch;

/// Nginx configuration template
const NGINX_TEMPLATE: &str = r#"
# Scratchpad Nginx Configuration
# Auto-generated - do not edit manually

{% for scratch in scratches %}
upstream scratch_{{ scratch.name }} {
    server {{ scratch.name }}-api:3000;
}

{% endfor %}

{% if routing == "subdomain" %}
{% for scratch in scratches %}
server {
    listen 80;
    server_name {{ scratch.name }}.{{ domain }};

    location / {
        proxy_pass http://scratch_{{ scratch.name }};
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}

{% endfor %}
{% else %}
server {
    listen 80;
    server_name {{ domain }};

{% for scratch in scratches %}
    location /{{ scratch.name }}/ {
        rewrite ^/{{ scratch.name }}/(.*) /$1 break;
        proxy_pass http://scratch_{{ scratch.name }};
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

{% endfor %}
}
{% endif %}
"#;

/// Regenerate the nginx configuration file
pub async fn regenerate_config(config: &Config, docker: &DockerClient) -> Result<()> {
    if !config.nginx.enabled {
        return Ok(());
    }

    // Get list of scratches
    let scratches = scratch::list_scratches(config, docker).await?;

    // Render template
    use minijinja::{context, Environment};

    let mut env = Environment::new();
    env.add_template("nginx", NGINX_TEMPLATE)?;

    let template = env.get_template("nginx")?;
    let rendered = template.render(context! {
        scratches => scratches,
        domain => config.nginx.domain,
        routing => match config.nginx.routing {
            NginxRouting::Subdomain => "subdomain",
            NginxRouting::Path => "path",
        },
    })?;

    // Write config file
    if let Some(parent) = config.nginx.config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&config.nginx.config_path, rendered)?;

    tracing::info!("Generated nginx config: {:?}", config.nginx.config_path);
    Ok(())
}

/// Get the current nginx configuration
pub fn get_config(config: &Config) -> Result<String> {
    let content = fs::read_to_string(&config.nginx.config_path)?;
    Ok(content)
}
