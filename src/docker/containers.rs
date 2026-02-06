//! Container management operations

use bollard::container::LogOutput;
use bollard::models::{ContainerCreateBody, ContainerSummary, HostConfig, PortBinding};
use bollard::query_parameters::{
    CreateContainerOptions, CreateImageOptions, ListContainersOptions, LogsOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use futures_util::StreamExt;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

use super::DockerClient;
use crate::error::Result;

/// Container status information
#[derive(Debug, Clone)]
pub struct ContainerStatus {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub labels: HashMap<String, String>,
}

impl From<ContainerSummary> for ContainerStatus {
    fn from(container: ContainerSummary) -> Self {
        Self {
            id: container.id.unwrap_or_default(),
            name: container
                .names
                .and_then(|n| n.first().cloned())
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string(),
            image: container.image.unwrap_or_default(),
            state: container.state.map(|s| s.to_string()).unwrap_or_default(),
            status: container.status.unwrap_or_default(),
            labels: container.labels.unwrap_or_default(),
        }
    }
}

impl DockerClient {
    /// List all containers with the scratchpad label
    pub async fn list_scratch_containers(
        &self,
        scratch_name: Option<&str>,
    ) -> Result<Vec<ContainerStatus>> {
        let label_prefix = &self.config().label_prefix;
        let mut filters: HashMap<String, Vec<String>> = HashMap::new();

        if let Some(name) = scratch_name {
            filters.insert(
                "label".to_string(),
                vec![format!("{}.scratch={}", label_prefix, name)],
            );
        } else {
            filters.insert(
                "label".to_string(),
                vec![format!("{}.scratch", label_prefix)],
            );
        }

        let options = ListContainersOptions {
            all: true,
            filters: Some(filters),
            ..Default::default()
        };

        let containers: Vec<ContainerSummary> = self.inner().list_containers(Some(options)).await?;
        Ok(containers.into_iter().map(ContainerStatus::from).collect())
    }

    /// List all containers for shared services
    pub async fn list_shared_service_containers(&self) -> Result<Vec<ContainerStatus>> {
        let label_prefix = &self.config().label_prefix;
        let mut filters: HashMap<String, Vec<String>> = HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![format!("{}.shared-service", label_prefix)],
        );

        let options = ListContainersOptions {
            all: true,
            filters: Some(filters),
            ..Default::default()
        };

        let containers: Vec<ContainerSummary> = self.inner().list_containers(Some(options)).await?;
        Ok(containers.into_iter().map(ContainerStatus::from).collect())
    }

    /// Create and start a container
    #[allow(clippy::too_many_arguments)]
    pub async fn create_container(
        &self,
        name: &str,
        image: &str,
        env: Vec<String>,
        labels: HashMap<String, String>,
        ports: Vec<(u16, u16)>, // (host, container)
        volumes: Vec<String>,
        network: Option<&str>,
        healthcheck_cmd: Option<&str>,
    ) -> Result<String> {
        // Pull image if not present
        self.pull_image_if_missing(image).await?;

        // Port bindings
        let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
        // Build port keys for exposed_ports
        let port_keys: Vec<String> = ports
            .iter()
            .map(|(_, container_port)| format!("{}/tcp", container_port))
            .collect();

        for (host_port, container_port) in &ports {
            let key = format!("{}/tcp", container_port);
            port_bindings.insert(
                key,
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(host_port.to_string()),
                }]),
            );
        }

        // Build exposed_ports as Vec<String>
        let exposed_ports: Vec<String> = port_keys.clone();

        // Host config
        let host_config = HostConfig {
            binds: Some(volumes),
            port_bindings: Some(port_bindings),
            network_mode: network.map(|n| n.to_string()),
            restart_policy: Some(bollard::models::RestartPolicy {
                name: Some(bollard::models::RestartPolicyNameEnum::UNLESS_STOPPED),
                maximum_retry_count: None,
            }),
            ..Default::default()
        };

        // Healthcheck
        let healthcheck = healthcheck_cmd.map(|cmd| bollard::models::HealthConfig {
            test: Some(vec!["CMD-SHELL".to_string(), cmd.to_string()]),
            interval: Some(10_000_000_000), // 10s in nanoseconds
            timeout: Some(5_000_000_000),   // 5s
            retries: Some(3),
            start_period: Some(5_000_000_000), // 5s
            start_interval: None,
        });

        let config = ContainerCreateBody {
            image: Some(image.to_string()),
            env: Some(env),
            labels: Some(labels),
            exposed_ports: Some(exposed_ports),
            host_config: Some(host_config),
            healthcheck,
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: Some(name.to_string()),
            platform: String::new(),
        };

        let response = self.inner().create_container(Some(options), config).await?;

        // Start the container
        self.inner()
            .start_container(&response.id, None::<StartContainerOptions>)
            .await?;

        Ok(response.id)
    }

    /// Pull an image if it doesn't exist locally
    pub async fn pull_image_if_missing(&self, image: &str) -> Result<()> {
        // Check if image exists
        if self.inner().inspect_image(image).await.is_ok() {
            return Ok(());
        }

        tracing::info!("Pulling image: {}", image);

        let options = CreateImageOptions {
            from_image: Some(image.to_string()),
            ..Default::default()
        };

        let mut stream = self.inner().create_image(Some(options), None, None);
        while let Some(result) = stream.next().await {
            if let Err(e) = result {
                tracing::warn!("Error pulling image: {}", e);
            }
        }

        Ok(())
    }

    /// Stop a container
    pub async fn stop_container(&self, id: &str) -> Result<()> {
        let options = StopContainerOptions {
            t: Some(10),
            signal: Some("SIGTERM".to_string()),
        };
        self.inner().stop_container(id, Some(options)).await?;
        Ok(())
    }

    /// Start a stopped container
    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.inner()
            .start_container(id, None::<StartContainerOptions>)
            .await?;
        Ok(())
    }

    /// Remove a container
    pub async fn remove_container(&self, id: &str, force: bool) -> Result<()> {
        let options = RemoveContainerOptions {
            force,
            v: true, // Remove volumes
            ..Default::default()
        };
        self.inner().remove_container(id, Some(options)).await?;
        Ok(())
    }

    /// Stream logs from a container to a file
    pub async fn stream_logs_to_file(
        &self,
        container_id: &str,
        log_path: &std::path::Path,
    ) -> Result<()> {
        let options = LogsOptions {
            follow: true,
            stdout: true,
            stderr: true,
            timestamps: true,
            ..Default::default()
        };

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .await?;

        let mut stream = self.inner().logs(container_id, Some(options));

        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    let line = match output {
                        LogOutput::StdOut { message } => message,
                        LogOutput::StdErr { message } => message,
                        LogOutput::Console { message } => message,
                        LogOutput::StdIn { message } => message,
                    };
                    file.write_all(&line).await?;
                    file.write_all(b"\n").await?;
                }
                Err(e) => {
                    tracing::error!("Error streaming logs: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Get logs from a container (non-streaming)
    pub async fn get_logs(&self, container_id: &str, tail: usize) -> Result<Vec<String>> {
        let options = LogsOptions {
            follow: false,
            stdout: true,
            stderr: true,
            timestamps: true,
            tail: tail.to_string(),
            ..Default::default()
        };

        let mut logs = Vec::new();
        let mut stream = self.inner().logs(container_id, Some(options));

        while let Some(result) = stream.next().await {
            if let Ok(output) = result {
                let line = match output {
                    LogOutput::StdOut { message } => String::from_utf8_lossy(&message).to_string(),
                    LogOutput::StdErr { message } => String::from_utf8_lossy(&message).to_string(),
                    LogOutput::Console { message } => String::from_utf8_lossy(&message).to_string(),
                    LogOutput::StdIn { message } => String::from_utf8_lossy(&message).to_string(),
                };
                logs.push(line);
            }
        }

        Ok(logs)
    }

    /// Execute a command in a running container
    pub async fn exec_command(&self, container_id: &str, cmd: Vec<&str>) -> Result<String> {
        use bollard::exec::{CreateExecOptions, StartExecResults};

        let exec = self
            .inner()
            .create_exec(
                container_id,
                CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(cmd),
                    ..Default::default()
                },
            )
            .await?;

        let output = self.inner().start_exec(&exec.id, None).await?;

        let mut result = String::new();
        if let StartExecResults::Attached { mut output, .. } = output {
            while let Some(Ok(msg)) = output.next().await {
                result.push_str(&msg.to_string());
            }
        }

        Ok(result)
    }
}
