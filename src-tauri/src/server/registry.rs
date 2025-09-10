use serde::{Deserialize, Serialize};

use crate::detection::McpServerConfig;

/// Registry for tracking available and installed MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRegistry {
    /// Available servers that can be installed
    pub available_servers: Vec<McpServerConfig>,
    /// Currently installed servers
    pub installed_servers: Vec<McpServerConfig>,
    /// Last time the registry was scanned
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
    /// Registry metadata
    pub metadata: RegistryMetadata,
}

/// Metadata about the server registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    /// Total number of available servers
    pub available_count: usize,
    /// Total number of installed servers
    pub installed_count: usize,
    /// Registry version
    pub version: String,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl ServerRegistry {
    /// Create a new server registry
    pub fn new() -> Self {
        Self {
            available_servers: Vec::new(),
            installed_servers: Vec::new(),
            last_scan: None,
            metadata: RegistryMetadata {
                available_count: 0,
                installed_count: 0,
                version: "1.0.0".to_string(),
                last_updated: chrono::Utc::now(),
            },
        }
    }

    /// Add a server to the available servers list
    pub fn add_available_server(&mut self, server: McpServerConfig) {
        if !self.available_servers.iter().any(|s| s.name == server.name) {
            self.available_servers.push(server);
            self.update_metadata();
        }
    }

    /// Add a server to the installed servers list
    pub fn add_installed_server(&mut self, server: McpServerConfig) {
        if !self.installed_servers.iter().any(|s| s.name == server.name) {
            self.installed_servers.push(server);
            self.update_metadata();
        }
    }

    /// Remove a server from installed servers
    pub fn remove_installed_server(&mut self, server_name: &str) -> bool {
        let initial_len = self.installed_servers.len();
        self.installed_servers.retain(|s| s.name != server_name);
        let removed = self.installed_servers.len() != initial_len;
        if removed {
            self.update_metadata();
        }
        removed
    }

    /// Get an installed server by name
    pub fn get_installed_server(&self, server_name: &str) -> Option<&McpServerConfig> {
        self.installed_servers.iter().find(|s| s.name == server_name)
    }

    /// Get an available server by name
    pub fn get_available_server(&self, server_name: &str) -> Option<&McpServerConfig> {
        self.available_servers.iter().find(|s| s.name == server_name)
    }

    /// Check if a server is installed
    pub fn is_server_installed(&self, server_name: &str) -> bool {
        self.installed_servers.iter().any(|s| s.name == server_name)
    }

    /// Get all installed server names
    pub fn get_installed_server_names(&self) -> Vec<String> {
        self.installed_servers.iter().map(|s| s.name.clone()).collect()
    }

    /// Get all available server names
    pub fn get_available_server_names(&self) -> Vec<String> {
        self.available_servers.iter().map(|s| s.name.clone()).collect()
    }

    /// Update the last scan time
    pub fn update_last_scan(&mut self) {
        self.last_scan = Some(chrono::Utc::now());
        self.update_metadata();
    }

    /// Update registry metadata
    fn update_metadata(&mut self) {
        self.metadata.available_count = self.available_servers.len();
        self.metadata.installed_count = self.installed_servers.len();
        self.metadata.last_updated = chrono::Utc::now();
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> RegistryStats {
        RegistryStats {
            available_servers: self.available_servers.len(),
            installed_servers: self.installed_servers.len(),
            last_scan: self.last_scan,
            last_updated: self.metadata.last_updated,
        }
    }
}

/// Statistics about the server registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub available_servers: usize,
    pub installed_servers: usize,
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for ServerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::{ServerType, ServerMetadata, ConfigSource};

    fn create_test_server(name: &str) -> McpServerConfig {
        McpServerConfig {
            name: name.to_string(),
            command: Some("test-command".to_string()),
            args: vec![],
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

    #[test]
    fn test_registry_creation() {
        let registry = ServerRegistry::new();
        assert_eq!(registry.available_servers.len(), 0);
        assert_eq!(registry.installed_servers.len(), 0);
        assert!(registry.last_scan.is_none());
    }

    #[test]
    fn test_add_available_server() {
        let mut registry = ServerRegistry::new();
        let server = create_test_server("test-server");
        
        registry.add_available_server(server);
        assert_eq!(registry.available_servers.len(), 1);
        assert_eq!(registry.metadata.available_count, 1);
    }

    #[test]
    fn test_add_installed_server() {
        let mut registry = ServerRegistry::new();
        let server = create_test_server("test-server");
        
        registry.add_installed_server(server);
        assert_eq!(registry.installed_servers.len(), 1);
        assert_eq!(registry.metadata.installed_count, 1);
        assert!(registry.is_server_installed("test-server"));
    }

    #[test]
    fn test_remove_installed_server() {
        let mut registry = ServerRegistry::new();
        let server = create_test_server("test-server");
        
        registry.add_installed_server(server);
        assert!(registry.is_server_installed("test-server"));
        
        let removed = registry.remove_installed_server("test-server");
        assert!(removed);
        assert!(!registry.is_server_installed("test-server"));
        assert_eq!(registry.metadata.installed_count, 0);
    }

    #[test]
    fn test_get_server_names() {
        let mut registry = ServerRegistry::new();
        let server1 = create_test_server("server1");
        let server2 = create_test_server("server2");
        
        registry.add_available_server(server1);
        registry.add_installed_server(server2);
        
        let available_names = registry.get_available_server_names();
        let installed_names = registry.get_installed_server_names();
        
        assert_eq!(available_names, vec!["server1"]);
        assert_eq!(installed_names, vec!["server2"]);
    }

    #[test]
    fn test_registry_stats() {
        let mut registry = ServerRegistry::new();
        let server = create_test_server("test-server");
        
        registry.add_available_server(server);
        registry.update_last_scan();
        
        let stats = registry.get_stats();
        assert_eq!(stats.available_servers, 1);
        assert_eq!(stats.installed_servers, 0);
        assert!(stats.last_scan.is_some());
    }
}
