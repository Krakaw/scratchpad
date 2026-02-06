//! Nginx configuration generation

use std::fs;

use crate::config::{Config, NginxRouting};
use crate::docker::DockerClient;
use crate::error::{Error, Result};
use crate::scratch;

/// Dynamic nginx configuration template using variables to route to scratches
/// Uses the subdomain or path as the scratch name to find the upstream
const NGINX_DYNAMIC_TEMPLATE: &str = r#"
# Scratchpad Nginx Configuration
# Auto-generated - do not edit manually
# 
# This config dynamically routes requests based on subdomain/path
# No regeneration needed when creating new scratches!
#
# Ingress service: {{ ingress_service }}
# Upstream port: {{ upstream_port }}

# Resolver for dynamic upstream resolution (Docker DNS)
resolver 127.0.0.11 valid=10s ipv6=off;

{% if routing == "subdomain" %}
# Wildcard subdomain routing: <scratch-name>.{{ domain }} -> <scratch-name>-{{ ingress_service }}:{{ upstream_port }}
server {
    listen 80;
    server_name ~^(?<scratch>.+)\.{{ domain_escaped }}$;

    location / {
        set $upstream ${scratch}-{{ ingress_service }}:{{ upstream_port }};
        proxy_pass http://$upstream;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # Handle upstream not found
        proxy_intercept_errors on;
        error_page 502 503 504 = @scratch_not_found;
    }
    
    location @scratch_not_found {
        return 404 'Scratch "$scratch" not found or not running\n';
        add_header Content-Type text/plain;
    }
}

{% else %}
# Path-based routing: {{ domain }}/<scratch-name>/* -> <scratch-name>-{{ ingress_service }}:{{ upstream_port }}
server {
    listen 80;
    server_name {{ domain }};

    # Extract scratch name from path and proxy
    location ~ ^/(?<scratch>[^/]+)(?:/(?<path>.*))?$ {
        set $upstream ${scratch}-{{ ingress_service }}:{{ upstream_port }};
        
        # Rewrite to remove scratch prefix
        rewrite ^/[^/]+/?(.*)$ /$1 break;
        
        proxy_pass http://$upstream;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Scratchpad-Name $scratch;
        proxy_cache_bypass $http_upgrade;
        
        # Handle upstream not found
        proxy_intercept_errors on;
        error_page 502 503 504 = @scratch_not_found;
    }
    
    location @scratch_not_found {
        return 404 'Scratch not found or not running\n';
        add_header Content-Type text/plain;
    }
    
    location = / {
        return 200 'Scratchpad is running. Access scratches at: {{ domain }}/<scratch-name>/\n';
        add_header Content-Type text/plain;
    }
}
{% endif %}
"#;

/// Static nginx configuration template (one entry per scratch)
/// Used when dynamic resolution isn't desired
const NGINX_STATIC_TEMPLATE: &str = r#"
# Scratchpad Nginx Configuration
# Auto-generated - regenerate with 'scratchpad nginx generate'
#
# Ingress service: {{ ingress_service }}
# Upstream port: {{ upstream_port }}

{% for scratch in scratches %}
upstream scratch_{{ scratch.name }} {
    server {{ scratch.name }}-{{ ingress_service }}:{{ upstream_port }};
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

    use minijinja::{context, Environment};

    // Get ingress service name - required for nginx to work
    let ingress_service = config.nginx.ingress_service.clone().ok_or_else(|| {
        Error::Config(
            "nginx.ingress_service must be set to specify which service handles incoming requests"
                .to_string(),
        )
    })?;

    // Get the port from the ingress service config
    let upstream_port = config
        .services
        .get(&ingress_service)
        .and_then(|svc| svc.internal_port.or(svc.port))
        .unwrap_or(3000);

    let mut env = Environment::new();

    // Use dynamic config by default, static if explicitly requested
    let use_dynamic = config.nginx.dynamic.unwrap_or(true);

    let rendered = if use_dynamic {
        env.add_template("nginx", NGINX_DYNAMIC_TEMPLATE)?;
        let template = env.get_template("nginx")?;

        // Escape dots in domain for regex
        let domain_escaped = config.nginx.domain.replace('.', r"\.");

        template.render(context! {
            domain => config.nginx.domain,
            domain_escaped => domain_escaped,
            ingress_service => ingress_service,
            upstream_port => upstream_port,
            routing => match config.nginx.routing {
                NginxRouting::Subdomain => "subdomain",
                NginxRouting::Path => "path",
            },
        })?
    } else {
        // Static config - needs scratch list
        let scratches = scratch::list_scratches(config, docker).await?;

        env.add_template("nginx", NGINX_STATIC_TEMPLATE)?;
        let template = env.get_template("nginx")?;

        template.render(context! {
            scratches => scratches,
            domain => config.nginx.domain,
            ingress_service => ingress_service,
            upstream_port => upstream_port,
            routing => match config.nginx.routing {
                NginxRouting::Subdomain => "subdomain",
                NginxRouting::Path => "path",
            },
        })?
    };

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
