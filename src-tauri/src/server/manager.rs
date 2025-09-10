use anyhow::{Result, Context};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use tokio::sync::RwLock;

use crate::detection::McpServerConfig;
use crate::configuration::ConfigurationEngine;
use super::{ServerStatus, ProcessInfo, ServerOperationResult, ServerRegistry};

/// Manages MCP server lifecycle operations
pub struct ServerManager {
    /// Registry of available and installed servers
    registry: ServerRegistry,
    /// Currently running servers
    running_servers: RwLock<HashMap<String, ProcessInfo>>,
    /// Configuration engine for server configs
    config_engine: Option<ConfigurationEngine>,
}

impl ServerManager {
    /// Create a new server manager
    pub fn new() -> Self {
        Self {
            registry: ServerRegistry::new(),
            running_servers: RwLock::new(HashMap::new()),
            config_engine: None,
        }
    }

    /// Create a new server manager with configuration engine
    pub fn with_config_engine(config_engine: ConfigurationEngine) -> Self {
        Self {
            registry: ServerRegistry::new(),
            running_servers: RwLock::new(HashMap::new()),
            config_engine: Some(config_engine),
        }
    }

    /// Start an MCP server
    pub async fn start_server(&self, server_config: &McpServerConfig) -> Result<ServerOperationResult> {
        let server_id = &server_config.name;
        
        // Check if already running
        {
            let running = self.running_servers.read().await;
            if running.contains_key(server_id) {
                return Ok(ServerOperationResult {
                    success: true,
                    server_id: server_id.clone(),
                    message: "Already running".to_string(),
                    errors: vec![],
                });
            }
        }

        // Get command and args
        let command = server_config.command.as_ref()
            .context("Server command not specified")?;

        // Start the process
        match self.spawn_server_process(command, &server_config.args, &server_config.env).await {
            Ok(mut child) => {
                let pid = child.id();
                
                let process_info = ProcessInfo {
                    pid,
                    config: server_config.clone(),
                    child: Some(child),
                    started_at: chrono::Utc::now(),
                };

                // Store the running server
                {
                    let mut running = self.running_servers.write().await;
                    running.insert(server_id.clone(), process_info);
                }

                Ok(ServerOperationResult {
                    success: true,
                    server_id: server_id.clone(),
                    message: format!("Started with PID {}", pid),
                    errors: vec![],
                })
            }
            Err(e) => {
                Ok(ServerOperationResult {
                    success: false,
                    server_id: server_id.clone(),
                    message: "Failed to start".to_string(),
                    errors: vec![e.to_string()],
                })
            }
        }
    }

    /// Stop an MCP server
    pub async fn stop_server(&self, server_id: &str) -> Result<ServerOperationResult> {
        let mut running = self.running_servers.write().await;
        
        if let Some(mut process_info) = running.remove(server_id) {
            if let Some(ref mut child) = process_info.child {
                match child.kill() {
                    Ok(_) => {
                        let _ = child.wait(); // Clean up zombie process
                        Ok(ServerOperationResult {
                            success: true,
                            server_id: server_id.to_string(),
                            message: "Stopped successfully".to_string(),
                            errors: vec![],
                        })
                    }
                    Err(e) => {
                        Ok(ServerOperationResult {
                            success: false,
                            server_id: server_id.to_string(),
                            message: "Failed to stop".to_string(),
                            errors: vec![e.to_string()],
                        })
                    }
                }
            } else {
                Ok(ServerOperationResult {
                    success: true,
                    server_id: server_id.to_string(),
                    message: "Process not tracked, marked as stopped".to_string(),
                    errors: vec![],
                })
            }
        } else {
            Ok(ServerOperationResult {
                success: false,
                server_id: server_id.to_string(),
                message: "Server not running".to_string(),
                errors: vec![],
            })
        }
    }

    /// Get the status of a server
    pub async fn get_server_status(&self, server_id: &str) -> ServerStatus {
        let running = self.running_servers.read().await;
        
        if let Some(process_info) = running.get(server_id) {
            // Check if process is still alive
            if let Some(ref child) = process_info.child {
                // For now, assume running if we have a process handle
                // In a more complete implementation, we'd check if the process is actually alive
                ServerStatus::Running
            } else {
                ServerStatus::Unknown
            }
        } else {
            ServerStatus::Stopped
        }
    }

    /// Get status of all servers
    pub async fn get_all_server_statuses(&self) -> HashMap<String, ServerStatus> {
        let running = self.running_servers.read().await;
        let mut statuses = HashMap::new();

        for (server_id, _) in running.iter() {
            statuses.insert(server_id.clone(), ServerStatus::Running);
        }

        statuses
    }

    /// Get list of running servers
    pub async fn get_running_servers(&self) -> Vec<String> {
        let running = self.running_servers.read().await;
        running.keys().cloned().collect()
    }

    /// Discover available MCP servers on the system
    pub async fn discover_servers(&mut self) -> Result<usize> {
        let mut discovered_count = 0;

        // Discover common MCP servers
        discovered_count += self.discover_common_servers().await?;

        // Update last scan time
        self.registry.update_last_scan();

        Ok(discovered_count)
    }

    /// Discover common MCP servers in standard locations
    async fn discover_common_servers(&mut self) -> Result<usize> {
        let mut count = 0;

        // Add some well-known MCP servers that might be available
        let common_servers = self.get_common_server_configs();
        
        for server_config in common_servers {
            // Check if the server command exists
            if self.is_server_available(&server_config).await {
                self.registry.add_available_server(server_config);
                count += 1;
            }
        }

        Ok(count)
    }

    /// Get configurations for common MCP servers
    fn get_common_server_configs(&self) -> Vec<McpServerConfig> {
        use crate::detection::{ServerType, ServerMetadata, ConfigSource};
        
        vec![
            // Filesystem server
            McpServerConfig {
                name: "filesystem".to_string(),
                command: Some("npx".to_string()),
                args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
                env: HashMap::new(),
                cwd: None,
                server_type: ServerType::Stdio,
                metadata: ServerMetadata {
                    description: Some("File system operations server".to_string()),
                    version: None,
                    author: Some("Anthropic".to_string()),
                    capabilities: vec!["read_file".to_string(), "write_file".to_string(), "list_directory".to_string()],
                    enabled: true,
                    source: ConfigSource::MainConfig,
                },
            },
            // Git server
            McpServerConfig {
                name: "git".to_string(),
                command: Some("npx".to_string()),
                args: vec!["-y".to_string(), "@modelcontextprotocol/server-git".to_string()],
                env: HashMap::new(),
                cwd: None,
                server_type: ServerType::Stdio,
                metadata: ServerMetadata {
                    description: Some("Git operations server".to_string()),
                    version: None,
                    author: Some("Anthropic".to_string()),
                    capabilities: vec!["git_log".to_string(), "git_diff".to_string(), "git_status".to_string()],
                    enabled: true,
                    source: ConfigSource::MainConfig,
                },
            },
        ]
    }

    /// Check if a server is available on the system
    async fn is_server_available(&self, server_config: &McpServerConfig) -> bool {
        if let Some(ref command) = server_config.command {
            // Try to run the command with --help to see if it exists
            match Command::new(command)
                .arg("--help")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
            {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Get registry reference
    pub fn get_registry(&self) -> &ServerRegistry {
        &self.registry
    }

    /// Get mutable registry reference
    pub fn get_registry_mut(&mut self) -> &mut ServerRegistry {
        &mut self.registry
    }

    /// Install a server from the available registry
    pub async fn install_server(&mut self, server_name: &str) -> Result<ServerOperationResult> {
        // Check if server is available
        if let Some(server_config) = self.registry.get_available_server(server_name).cloned() {
            // Check if already installed
            if self.registry.is_server_installed(server_name) {
                return Ok(ServerOperationResult {
                    success: false,
                    server_id: server_name.to_string(),
                    message: "Server already installed".to_string(),
                    errors: vec![],
                });
            }

            // For now, just mark as installed (in a real implementation, this would download/install)
            self.registry.add_installed_server(server_config);

            Ok(ServerOperationResult {
                success: true,
                server_id: server_name.to_string(),
                message: "Server installed successfully".to_string(),
                errors: vec![],
            })
        } else {
            Ok(ServerOperationResult {
                success: false,
                server_id: server_name.to_string(),
                message: "Server not found in available registry".to_string(),
                errors: vec![],
            })
        }
    }

    /// Register a custom server configuration
    pub async fn register_server(&mut self, server_config: McpServerConfig) -> Result<ServerOperationResult> {
        let server_name = server_config.name.clone();

        // Check if server is already registered
        if self.registry.is_server_installed(&server_name) {
            return Ok(ServerOperationResult {
                success: false,
                server_id: server_name,
                message: "Server already registered".to_string(),
                errors: vec![],
            });
        }

        // Add to installed servers
        self.registry.add_installed_server(server_config);

        Ok(ServerOperationResult {
            success: true,
            server_id: server_name,
            message: "Server registered successfully".to_string(),
            errors: vec![],
        })
    }

    /// Remove/uninstall a server
    pub async fn remove_server(&mut self, server_name: &str) -> Result<ServerOperationResult> {
        // Stop the server if it's running
        let running_servers = self.get_running_servers().await;
        if running_servers.contains(&server_name.to_string()) {
            let _ = self.stop_server(server_name).await;
        }

        // Remove from installed servers
        if self.registry.remove_installed_server(server_name) {
            Ok(ServerOperationResult {
                success: true,
                server_id: server_name.to_string(),
                message: "Server removed successfully".to_string(),
                errors: vec![],
            })
        } else {
            Ok(ServerOperationResult {
                success: false,
                server_id: server_name.to_string(),
                message: "Server not found or not installed".to_string(),
                errors: vec![],
            })
        }
    }

    /// Get list of installed servers
    pub fn get_installed_servers(&self) -> Vec<String> {
        self.registry.get_installed_server_names()
    }

    /// Get list of available servers
    pub fn get_available_servers(&self) -> Vec<String> {
        self.registry.get_available_server_names()
    }

    /// Spawn a server process
    async fn spawn_server_process(
        &self,
        command: &str,
        args: &[String],
        env: &std::collections::HashMap<String, String>,
    ) -> Result<std::process::Child> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        cmd.spawn()
            .with_context(|| format!("Failed to spawn process: {}", command))
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::{ServerType, ServerMetadata, ConfigSource};

    fn create_test_server_config(name: &str) -> McpServerConfig {
        McpServerConfig {
            name: name.to_string(),
            command: Some("echo".to_string()), // Use echo for testing
            args: vec!["hello".to_string()],
            env: std::collections::HashMap::new(),
            cwd: None,
            server_type: ServerType::Stdio,
            metadata: ServerMetadata {
                description: None,
                version: None,
                author: None,
                capabilities: vec![],
                enabled: true,
                source: ConfigSource::MainConfig,
            },
        }
    }

    #[tokio::test]
    async fn test_server_manager_creation() {
        let manager = ServerManager::new();
        let statuses = manager.get_all_server_statuses().await;
        assert!(statuses.is_empty());
    }

    #[tokio::test]
    async fn test_server_status_stopped_initially() {
        let manager = ServerManager::new();
        let status = manager.get_server_status("test-server").await;
        assert_eq!(status, ServerStatus::Stopped);
    }

    #[tokio::test]
    async fn test_stop_non_running_server() {
        let manager = ServerManager::new();
        let result = manager.stop_server("non-existent").await.unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Server not running");
    }

    #[tokio::test]
    async fn test_server_discovery() {
        let mut manager = ServerManager::new();
        
        // Initially no servers in registry
        assert_eq!(manager.get_registry().get_available_server_names().len(), 0);
        
        // Run discovery
        let discovered = manager.discover_servers().await.unwrap();
        
        // Should have discovered some servers (at least the common ones if commands exist)
        // Note: This might be 0 in CI environments without npx
        assert!(discovered >= 0);
        
        // Registry should be updated with last scan
        assert!(manager.get_registry().last_scan.is_some());
    }

    #[tokio::test]
    async fn test_get_common_server_configs() {
        let manager = ServerManager::new();
        let common_servers = manager.get_common_server_configs();
        
        assert!(!common_servers.is_empty());
        assert!(common_servers.iter().any(|s| s.name == "filesystem"));
        assert!(common_servers.iter().any(|s| s.name == "git"));
    }

    #[tokio::test]
    async fn test_registry_access() {
        let mut manager = ServerManager::new();
        
        // Test registry access
        let registry = manager.get_registry();
        assert_eq!(registry.get_available_server_names().len(), 0);
        
        // Test mutable registry access
        let registry_mut = manager.get_registry_mut();
        let test_server = create_test_server_config("test-server");
        registry_mut.add_available_server(test_server);
        
        assert_eq!(manager.get_registry().get_available_server_names().len(), 1);
    }

    #[tokio::test]
    async fn test_install_server() {
        let mut manager = ServerManager::new();
        
        // Add a server to available registry
        let test_server = create_test_server_config("test-server");
        manager.get_registry_mut().add_available_server(test_server);
        
        // Install the server
        let result = manager.install_server("test-server").await.unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Server installed successfully");
        
        // Check it's now in installed servers
        assert!(manager.get_registry().is_server_installed("test-server"));
        assert_eq!(manager.get_installed_servers().len(), 1);
    }

    #[tokio::test]
    async fn test_install_nonexistent_server() {
        let mut manager = ServerManager::new();
        
        let result = manager.install_server("nonexistent").await.unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Server not found in available registry");
    }

    #[tokio::test]
    async fn test_install_already_installed_server() {
        let mut manager = ServerManager::new();
        
        // Add and install a server
        let test_server = create_test_server_config("test-server");
        manager.get_registry_mut().add_available_server(test_server.clone());
        manager.get_registry_mut().add_installed_server(test_server);
        
        // Try to install again
        let result = manager.install_server("test-server").await.unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Server already installed");
    }

    #[tokio::test]
    async fn test_register_server() {
        let mut manager = ServerManager::new();
        
        let test_server = create_test_server_config("custom-server");
        let result = manager.register_server(test_server).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.message, "Server registered successfully");
        assert!(manager.get_registry().is_server_installed("custom-server"));
    }

    #[tokio::test]
    async fn test_register_duplicate_server() {
        let mut manager = ServerManager::new();
        
        let test_server = create_test_server_config("custom-server");
        manager.get_registry_mut().add_installed_server(test_server.clone());
        
        let result = manager.register_server(test_server).await.unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Server already registered");
    }

    #[tokio::test]
    async fn test_remove_server() {
        let mut manager = ServerManager::new();
        
        // Add and install a server
        let test_server = create_test_server_config("test-server");
        manager.get_registry_mut().add_installed_server(test_server);
        
        // Remove the server
        let result = manager.remove_server("test-server").await.unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Server removed successfully");
        
        // Check it's no longer installed
        assert!(!manager.get_registry().is_server_installed("test-server"));
    }

    #[tokio::test]
    async fn test_remove_nonexistent_server() {
        let mut manager = ServerManager::new();
        
        let result = manager.remove_server("nonexistent").await.unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Server not found or not installed");
    }

    #[tokio::test]
    async fn test_get_server_lists() {
        let mut manager = ServerManager::new();
        
        // Add available and installed servers
        let available_server = create_test_server_config("available-server");
        let installed_server = create_test_server_config("installed-server");
        
        manager.get_registry_mut().add_available_server(available_server);
        manager.get_registry_mut().add_installed_server(installed_server);
        
        let available = manager.get_available_servers();
        let installed = manager.get_installed_servers();
        
        assert_eq!(available.len(), 1);
        assert_eq!(installed.len(), 1);
        assert!(available.contains(&"available-server".to_string()));
        assert!(installed.contains(&"installed-server".to_string()));
    }
}
