//! Docker container provider implementation
//!
//! Implements the ContainerProvider trait using the Docker API via bollard.

use anyhow::{Context, Result};
use async_trait::async_trait;
use bollard::{
    container::{
        Config, CreateContainerOptions, LogOutput, LogsOptions, RemoveContainerOptions,
        StatsOptions, StopContainerOptions,
    },
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    network::CreateNetworkOptions,
    service::{
        HostConfig, Mount, MountTypeEnum, PortBinding, RestartPolicy, RestartPolicyNameEnum,
    },
    Docker,
};
use chrono::Utc;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::default::Default;
use tracing::{debug, info, trace, warn};

use crate::container::{
    config::RestartPolicy as ConfigRestartPolicy, Container, ContainerConfig, ContainerProvider,
    ContainerStats, ContainerStatus, DiskIO, LogEntry, LogStream, NetworkIO, NetworkMode,
    ResourceLimits,
};

/// Docker container provider
pub struct DockerContainerProvider {
    docker: Docker,
    network_id: Option<String>,
}

impl DockerContainerProvider {
    /// Create a new Docker container provider
    pub async fn new() -> Result<Self> {
        info!("Initializing Docker container provider");

        // Try multiple Docker socket paths
        let docker = Self::connect_to_docker().await?;

        // Verify Docker is accessible
        debug!("Verifying Docker daemon accessibility");
        docker
            .ping()
            .await
            .context("Failed to ping Docker daemon")?;

        info!("Successfully connected to Docker");

        // Create or ensure network exists
        let network_id = Self::ensure_network(&docker).await?;

        info!("Docker container provider initialized successfully");
        Ok(Self {
            docker,
            network_id: Some(network_id),
        })
    }

    /// Try to connect to Docker using multiple socket paths
    async fn connect_to_docker() -> Result<Docker> {
        // Common Docker socket paths
        let socket_paths = vec![
            // Default Docker Desktop for Mac/Linux
            "/var/run/docker.sock",
            // Docker Desktop for Mac (alternative)
            "/Users/$USER/.docker/run/docker.sock",
            // Colima
            "/Users/$USER/.colima/default/docker.sock",
            "/Users/$USER/.colima/docker.sock",
            // Podman
            "/run/user/$UID/podman/podman.sock",
            // Lima
            "/Users/$USER/.lima/default/sock/docker.sock",
            // Rancher Desktop
            "/Users/$USER/.rd/docker.sock",
        ];

        // Try environment variable first
        if let Ok(docker_host) = std::env::var("DOCKER_HOST") {
            debug!("Trying DOCKER_HOST: {}", docker_host);
            if docker_host.starts_with("unix://") {
                let path = docker_host.strip_prefix("unix://").unwrap();
                if let Ok(docker) =
                    Docker::connect_with_unix(path, 120, bollard::API_DEFAULT_VERSION)
                {
                    info!("Connected to Docker via DOCKER_HOST: {}", docker_host);
                    return Ok(docker);
                }
            }
        }

        // Expand environment variables in paths
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let uid = users::get_current_uid().to_string();

        // Try each socket path
        for socket_path in socket_paths {
            let expanded_path = socket_path.replace("$USER", &user).replace("$UID", &uid);

            debug!("Trying Docker socket: {}", expanded_path);

            if std::path::Path::new(&expanded_path).exists() {
                match Docker::connect_with_unix(&expanded_path, 120, bollard::API_DEFAULT_VERSION) {
                    Ok(docker) => {
                        info!("Connected to Docker via socket: {}", expanded_path);
                        return Ok(docker);
                    }
                    Err(e) => {
                        debug!("Failed to connect via {}: {}", expanded_path, e);
                    }
                }
            }
        }

        // Try default connection as last resort
        debug!("Trying default Docker connection");
        Docker::connect_with_socket_defaults().context("Failed to connect to Docker via any method")
    }

    /// Ensure ccswarm network exists
    async fn ensure_network(docker: &Docker) -> Result<String> {
        let network_name = "ccswarm-network";

        debug!("Checking if network '{}' exists", network_name);

        // Check if network exists
        match docker
            .inspect_network(
                network_name,
                None::<bollard::network::InspectNetworkOptions<String>>,
            )
            .await
        {
            Ok(network) => {
                let network_id = network.id.unwrap_or_else(|| network_name.to_string());
                info!(
                    "Using existing network: {} (ID: {})",
                    network_name, network_id
                );
                Ok(network_id)
            }
            Err(e) => {
                debug!("Network not found: {}, creating new network", e);

                // Create network
                info!("Creating network: {}", network_name);
                let options = CreateNetworkOptions {
                    name: network_name,
                    driver: "bridge",
                    labels: {
                        let mut labels = HashMap::new();
                        labels.insert("app", "ccswarm");
                        labels.insert("managed-by", "ccswarm");
                        labels
                    },
                    ..Default::default()
                };

                trace!("Network creation options: {:?}", options);

                let response = docker
                    .create_network(options)
                    .await
                    .context("Failed to create network")?;

                let network_id = response.id.unwrap_or_else(|| network_name.to_string());
                info!(
                    "Network created successfully: {} (ID: {})",
                    network_name, network_id
                );
                Ok(network_id)
            }
        }
    }

    /// Convert ContainerConfig to bollard Config
    fn to_docker_config(&self, _name: &str, config: &ContainerConfig) -> Config<String> {
        // Convert environment variables
        let env: Vec<String> = config
            .env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Convert port mappings
        let mut exposed_ports = HashMap::new();
        let mut port_bindings = HashMap::new();

        for port in &config.network.ports {
            let container_port = format!("{}/{}", port.container_port, port.protocol);
            exposed_ports.insert(container_port.clone(), HashMap::new());

            let binding = PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(port.host_port.to_string()),
            };
            port_bindings.insert(container_port, Some(vec![binding]));
        }

        // Convert volume mappings to mounts
        let mounts: Vec<Mount> = config
            .volumes
            .iter()
            .map(|v| Mount {
                target: Some(v.container_path.clone()),
                source: Some(v.host_path.clone()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(v.read_only),
                ..Default::default()
            })
            .collect();

        // Convert restart policy
        let restart_policy = match &config.restart_policy {
            ConfigRestartPolicy::No => RestartPolicy {
                name: Some(RestartPolicyNameEnum::NO),
                maximum_retry_count: None,
            },
            ConfigRestartPolicy::Always => RestartPolicy {
                name: Some(RestartPolicyNameEnum::ALWAYS),
                maximum_retry_count: None,
            },
            ConfigRestartPolicy::OnFailure { max_retries } => RestartPolicy {
                name: Some(RestartPolicyNameEnum::ON_FAILURE),
                maximum_retry_count: Some(*max_retries as i64),
            },
            ConfigRestartPolicy::UnlessStopped => RestartPolicy {
                name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
                maximum_retry_count: None,
            },
        };

        // Create host config
        let mut host_config = HostConfig {
            mounts: Some(mounts),
            port_bindings: Some(port_bindings),
            restart_policy: Some(restart_policy),
            security_opt: Some(config.security_opts.clone()),
            cap_add: Some(config.cap_add.clone()),
            cap_drop: Some(config.cap_drop.clone()),
            ..Default::default()
        };

        // Set resource limits
        if let Some(cpu_limit) = config.resources.cpu_limit {
            host_config.nano_cpus = Some((cpu_limit * 1_000_000_000.0) as i64);
        }
        if let Some(memory_limit) = config.resources.memory_limit {
            host_config.memory = Some(memory_limit);
        }
        if let Some(memory_swap_limit) = config.resources.memory_swap_limit {
            host_config.memory_swap = Some(memory_swap_limit);
        }
        if let Some(cpu_shares) = config.resources.cpu_shares {
            host_config.cpu_shares = Some(cpu_shares);
        }

        // Set network mode
        match &config.network.mode {
            NetworkMode::Bridge => {
                if let Some(network_id) = &self.network_id {
                    host_config.network_mode = Some(network_id.clone());
                }
            }
            NetworkMode::Host => {
                host_config.network_mode = Some("host".to_string());
            }
            NetworkMode::None => {
                host_config.network_mode = Some("none".to_string());
            }
            NetworkMode::Custom(_name) => {
                host_config.network_mode = config.network.network_name.clone();
            }
        }

        Config {
            image: Some(config.image.clone()),
            cmd: config.command.clone(),
            env: Some(env),
            working_dir: Some(config.working_dir.clone()),
            exposed_ports: Some(exposed_ports),
            host_config: Some(host_config),
            labels: Some(config.labels.clone()),
            ..Default::default()
        }
    }

    /// Convert Docker container info to our Container struct
    fn from_docker_container(&self, container: bollard::models::ContainerSummary) -> Container {
        let status = match container.state.as_deref() {
            Some("created") => ContainerStatus::Created,
            Some("running") => ContainerStatus::Running,
            Some("paused") => ContainerStatus::Paused,
            Some("exited") => ContainerStatus::Stopped,
            Some("removing") => ContainerStatus::Removing,
            Some(state) => ContainerStatus::Error(format!("Unknown state: {}", state)),
            None => ContainerStatus::Error("No state information".to_string()),
        };

        Container {
            id: container.id.unwrap_or_default(),
            name: container
                .names
                .unwrap_or_default()
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default(),
            status,
            image: container.image.unwrap_or_default(),
            env: HashMap::new(), // Would need to inspect container for full env
            volumes: Vec::new(), // Would need to inspect container for volumes
            network: crate::container::NetworkConfig {
                mode: NetworkMode::Bridge,
                network_name: None,
                ports: Vec::new(),
            },
            resources: ResourceLimits::default(),
        }
    }
}

#[async_trait]
impl ContainerProvider for DockerContainerProvider {
    async fn create_container(&self, name: &str, config: &ContainerConfig) -> Result<Container> {
        info!("Creating container: {} with image: {}", name, config.image);
        debug!("Container configuration: environment variables: {}, volumes: {}, resource limits: CPU={:?}, Memory={:?}",
            config.env.len(),
            config.volumes.len(),
            config.resources.cpu_limit,
            config.resources.memory_limit
        );

        // Ensure image exists
        info!("Pulling image if not present: {}", config.image);
        let create_image_options = CreateImageOptions {
            from_image: config.image.clone(),
            ..Default::default()
        };

        let mut stream = self
            .docker
            .create_image(Some(create_image_options), None, None);
        while let Some(info) = stream.next().await {
            match info {
                Ok(info) => trace!("Image pull progress: {:?}", info),
                Err(e) => warn!("Image pull warning: {}", e),
            }
        }
        info!("Image {} is ready", config.image);

        // Create container
        let docker_config = self.to_docker_config(name, config);
        let options = CreateContainerOptions {
            name,
            ..Default::default()
        };

        trace!("Docker container config: {:?}", docker_config);

        let container_info = self
            .docker
            .create_container(Some(options), docker_config)
            .await
            .context("Failed to create container")?;

        info!(
            "Container created successfully - Name: {}, ID: {}",
            name, container_info.id
        );

        // Return container info
        Ok(Container {
            id: container_info.id,
            name: name.to_string(),
            status: ContainerStatus::Created,
            image: config.image.clone(),
            env: config.env.clone(),
            volumes: config.volumes.clone(),
            network: config.network.clone(),
            resources: config.resources.clone(),
        })
    }

    async fn start_container(&self, container_id: &str) -> Result<()> {
        info!("Starting container: {}", container_id);

        self.docker
            .start_container::<String>(container_id, None)
            .await
            .context("Failed to start container")?;

        info!("Container {} started successfully", container_id);
        Ok(())
    }

    async fn stop_container(&self, container_id: &str) -> Result<()> {
        info!("Stopping container: {} (30s timeout)", container_id);

        let options = StopContainerOptions {
            t: 30, // 30 second timeout
        };

        self.docker
            .stop_container(container_id, Some(options))
            .await
            .context("Failed to stop container")?;

        info!("Container {} stopped successfully", container_id);
        Ok(())
    }

    async fn remove_container(&self, container_id: &str) -> Result<()> {
        info!(
            "Removing container: {} (force=true, remove_volumes=true)",
            container_id
        );

        let options = RemoveContainerOptions {
            force: true,
            v: true, // Remove volumes
            ..Default::default()
        };

        self.docker
            .remove_container(container_id, Some(options))
            .await
            .context("Failed to remove container")?;

        info!("Container {} removed successfully", container_id);
        Ok(())
    }

    async fn exec_in_container(&self, container_id: &str, command: Vec<String>) -> Result<String> {
        info!(
            "Executing command in container {}: {:?}",
            container_id, command
        );
        trace!("Full command details: {:?}", command);

        let exec_options = CreateExecOptions {
            cmd: Some(command),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec_instance = self
            .docker
            .create_exec(container_id, exec_options)
            .await
            .context("Failed to create exec instance")?;

        let output = match self
            .docker
            .start_exec(&exec_instance.id, None)
            .await
            .context("Failed to start exec")?
        {
            StartExecResults::Attached { mut output, .. } => {
                let mut result = String::new();
                while let Some(msg) = output.next().await {
                    match msg {
                        Ok(LogOutput::StdOut { message }) => {
                            result.push_str(&String::from_utf8_lossy(&message));
                        }
                        Ok(LogOutput::StdErr { message }) => {
                            result.push_str(&String::from_utf8_lossy(&message));
                        }
                        Err(e) => return Err(anyhow::anyhow!("Exec error: {}", e)),
                        _ => {}
                    }
                }
                result
            }
            StartExecResults::Detached => {
                return Err(anyhow::anyhow!("Exec was detached unexpectedly"));
            }
        };

        debug!(
            "Command execution completed, output length: {} bytes",
            output.len()
        );
        trace!("Command output: {}", output);
        Ok(output)
    }

    async fn get_logs(&self, container_id: &str, tail: Option<usize>) -> Result<Vec<LogEntry>> {
        info!(
            "Fetching logs for container: {} (tail: {:?})",
            container_id, tail
        );
        let options = LogsOptions {
            stdout: true,
            stderr: true,
            timestamps: true,
            tail: tail
                .map(|t| t.to_string())
                .unwrap_or_else(|| "all".to_string()),
            ..Default::default()
        };

        let mut stream = self.docker.logs(container_id, Some(options));
        let mut logs = Vec::new();

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(LogOutput::StdOut { message }) => {
                    let text = String::from_utf8_lossy(&message);
                    logs.push(LogEntry {
                        timestamp: Utc::now(),
                        message: text.to_string(),
                        stream: LogStream::Stdout,
                    });
                }
                Ok(LogOutput::StdErr { message }) => {
                    let text = String::from_utf8_lossy(&message);
                    logs.push(LogEntry {
                        timestamp: Utc::now(),
                        message: text.to_string(),
                        stream: LogStream::Stderr,
                    });
                }
                Err(e) => warn!("Error reading logs: {}", e),
                _ => {}
            }
        }

        info!(
            "Retrieved {} log entries from container {}",
            logs.len(),
            container_id
        );
        Ok(logs)
    }

    async fn get_status(&self, container_id: &str) -> Result<ContainerStatus> {
        debug!("Getting status for container: {}", container_id);

        let container_info = self
            .docker
            .inspect_container(container_id, None)
            .await
            .context("Failed to inspect container")?;

        let status = match container_info
            .state
            .as_ref()
            .and_then(|s| s.status.as_ref())
        {
            Some(bollard::models::ContainerStateStatusEnum::CREATED) => ContainerStatus::Created,
            Some(bollard::models::ContainerStateStatusEnum::RUNNING) => ContainerStatus::Running,
            Some(bollard::models::ContainerStateStatusEnum::PAUSED) => ContainerStatus::Paused,
            Some(bollard::models::ContainerStateStatusEnum::RESTARTING) => ContainerStatus::Running,
            Some(bollard::models::ContainerStateStatusEnum::REMOVING) => ContainerStatus::Removing,
            Some(bollard::models::ContainerStateStatusEnum::EXITED) => ContainerStatus::Stopped,
            Some(bollard::models::ContainerStateStatusEnum::DEAD) => {
                ContainerStatus::Error("Container is dead".to_string())
            }
            _ => ContainerStatus::Error("Unknown status".to_string()),
        };

        debug!("Container {} status: {:?}", container_id, status);
        Ok(status)
    }

    async fn list_containers(&self, filter: Option<String>) -> Result<Vec<Container>> {
        info!("Listing containers with filter: {:?}", filter);

        let mut filters = HashMap::new();

        // Add label filter
        filters.insert("label", vec!["app=ccswarm"]);

        // Add name filter if provided
        if let Some(ref f) = filter {
            filters.insert("name", vec![f.as_str()]);
        }

        trace!("Container list filters: {:?}", filters);

        let options = bollard::container::ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .context("Failed to list containers")?;

        let result: Vec<Container> = containers
            .into_iter()
            .map(|c| self.from_docker_container(c))
            .collect();

        info!("Found {} containers matching filters", result.len());
        Ok(result)
    }

    async fn get_stats(&self, container_id: &str) -> Result<ContainerStats> {
        debug!("Getting stats for container: {}", container_id);
        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stream = self.docker.stats(container_id, Some(options));

        if let Some(stats_result) = stream.next().await {
            let stats = stats_result.context("Failed to get container stats")?;

            // Calculate CPU percentage
            let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
                - stats.precpu_stats.cpu_usage.total_usage as f64;
            let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
                - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
            let cpu_percent = if system_delta > 0.0 {
                (cpu_delta / system_delta) * stats.cpu_stats.online_cpus.unwrap_or(1) as f64 * 100.0
            } else {
                0.0
            };

            // Memory usage
            let memory_stats = stats.memory_stats;
            let memory_usage = memory_stats.usage.unwrap_or(0);
            let memory_limit = memory_stats.limit.unwrap_or(0);

            // Network I/O
            let mut rx_bytes = 0u64;
            let mut tx_bytes = 0u64;
            if let Some(networks) = stats.networks {
                for (_, network) in networks {
                    rx_bytes += network.rx_bytes;
                    tx_bytes += network.tx_bytes;
                }
            }

            // Disk I/O
            let mut read_bytes = 0u64;
            let mut write_bytes = 0u64;
            let blkio_stats = stats.blkio_stats;
            if let Some(io_service_bytes) = blkio_stats.io_service_bytes_recursive {
                for entry in io_service_bytes {
                    match entry.op.as_str() {
                        "read" => read_bytes += entry.value,
                        "write" => write_bytes += entry.value,
                        _ => {}
                    }
                }
            }

            let stats = ContainerStats {
                cpu_percent,
                memory_usage,
                memory_limit,
                network_io: NetworkIO { rx_bytes, tx_bytes },
                disk_io: DiskIO {
                    read_bytes,
                    write_bytes,
                },
            };

            debug!("Container {} stats - CPU: {:.2}%, Memory: {}MB/{}, Network: RX/TX={}/{} bytes, Disk: R/W={}/{} bytes",
                container_id,
                stats.cpu_percent,
                stats.memory_usage / 1_048_576,
                stats.memory_limit / 1_048_576,
                stats.network_io.rx_bytes,
                stats.network_io.tx_bytes,
                stats.disk_io.read_bytes,
                stats.disk_io.write_bytes
            );

            Ok(stats)
        } else {
            Err(anyhow::anyhow!("No stats data received"))
        }
    }

    async fn copy_to_container(
        &self,
        container_id: &str,
        src_path: &str,
        dest_path: &str,
    ) -> Result<()> {
        use std::fs::File;
        use std::path::Path;
        use tar::Builder;

        info!(
            "Copying {} to container {}:{}",
            src_path, container_id, dest_path
        );

        // Create a tar archive in memory
        let mut tar_data = Vec::new();
        {
            let mut tar = Builder::new(&mut tar_data);

            let src = Path::new(src_path);
            if src.is_file() {
                // Single file
                let mut file =
                    File::open(src).context(format!("Failed to open source file: {}", src_path))?;
                let file_name = src
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
                tar.append_file(file_name, &mut file)
                    .context("Failed to add file to tar archive")?;
            } else if src.is_dir() {
                // Directory
                tar.append_dir_all(".", src)
                    .context("Failed to add directory to tar archive")?;
            } else {
                return Err(anyhow::anyhow!("Source path does not exist: {}", src_path));
            }

            tar.finish().context("Failed to finish tar archive")?;
        }

        // Upload the tar archive to the container
        let options = bollard::container::UploadToContainerOptions {
            path: dest_path.to_string(),
            ..Default::default()
        };

        self.docker
            .upload_to_container(container_id, Some(options), tar_data.into())
            .await
            .context("Failed to upload file to container")?;

        info!("Successfully copied file to container");
        Ok(())
    }

    async fn copy_from_container(
        &self,
        container_id: &str,
        src_path: &str,
        dest_path: &str,
    ) -> Result<()> {
        use futures_util::TryStreamExt;
        use std::fs::create_dir_all;
        use std::path::Path;
        use tar::Archive;

        info!(
            "Copying from container {}:{} to {}",
            container_id, src_path, dest_path
        );

        // Download file from container
        let options = bollard::container::DownloadFromContainerOptions {
            path: src_path.to_string(),
        };

        let stream = self
            .docker
            .download_from_container(container_id, Some(options));

        // Collect the tar data as bytes
        let tar_data = stream
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .context("Failed to download file from container")?;

        // Extract tar archive
        let mut archive = Archive::new(&tar_data[..]);
        let dest = Path::new(dest_path);

        // Ensure destination directory exists
        if let Some(parent) = dest.parent() {
            create_dir_all(parent).context("Failed to create destination directory")?;
        }

        // Extract files
        for entry in archive.entries().context("Failed to read tar archive")? {
            let mut entry = entry.context("Failed to read tar entry")?;
            let entry_path = entry
                .path()
                .context("Failed to get entry path")?
                .to_path_buf(); // Convert to owned PathBuf

            // If destination is a directory, extract into it
            // If destination is a file path, extract the first file to that path
            let target_path = if dest.exists() && dest.is_dir() {
                dest.join(&entry_path)
            } else {
                dest.to_path_buf()
            };

            // Ensure parent directory exists
            if let Some(parent) = target_path.parent() {
                create_dir_all(parent).context("Failed to create parent directory")?;
            }

            entry
                .unpack(&target_path)
                .context(format!("Failed to extract file to {:?}", target_path))?;

            info!("Extracted: {:?} -> {:?}", entry_path, target_path);

            // If we're extracting to a specific file, only extract the first entry
            if !dest.is_dir() {
                break;
            }
        }

        info!("Successfully copied files from container");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_connection() {
        // This test requires Docker to be running
        match DockerContainerProvider::new().await {
            Ok(_) => {
                // Docker is available
            }
            Err(e) => {
                eprintln!("Docker not available for testing: {}", e);
            }
        }
    }
}
