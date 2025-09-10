use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::detection::McpServerConfig;
use super::ConfigurationChange;

/// Persistent configuration store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationStore {
    /// Path to the store file
    #[serde(skip)]
    store_path: PathBuf,
    
    /// MCP server configurations indexed by name
    pub servers: HashMap<String, StoredServerConfig>,
    
    /// Application associations
    pub application_servers: HashMap<String, Vec<String>>,
    
    /// Configuration change history
    pub changes: Vec<ConfigurationChange>,
    
    /// Store metadata
    pub metadata: StoreMetadata,
}

/// Server configuration with storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredServerConfig {
    pub config: McpServerConfig,
    pub application_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

/// Store metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMetadata {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub last_sync: Option<DateTime<Utc>>,
}

impl Default for StoreMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: "1.0.0".to_string(),
            created_at: now,
            last_modified: now,
            last_sync: None,
        }
    }
}

impl ConfigurationStore {
    /// Create a new configuration store
    pub fn new(store_path: PathBuf) -> Result<Self> {
        let mut store = if store_path.exists() {
            Self::load_from_file(&store_path)?
        } else {
            Self {
                store_path: store_path.clone(),
                servers: HashMap::new(),
                application_servers: HashMap::new(),
                changes: Vec::new(),
                metadata: StoreMetadata::default(),
            }
        };
        
        store.store_path = store_path;
        Ok(store)
    }

    /// Load store from file
    fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read store file: {}", path.display()))?;
        
        let mut store: Self = serde_json::from_str(&content)
            .with_context(|| "Failed to parse store file")?;
        
        store.store_path = path.clone();
        Ok(store)
    }

    /// Save store to file
    fn save_to_file(&mut self) -> Result<()> {
        self.metadata.last_modified = Utc::now();
        
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize store")?;
        
        // Ensure parent directory exists
        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create store directory: {}", parent.display()))?;
        }
        
        fs::write(&self.store_path, content)
            .with_context(|| format!("Failed to write store file: {}", self.store_path.display()))?;
        
        Ok(())
    }

    /// Add a new server configuration
    pub fn add_server(&mut self, server: McpServerConfig, application_id: Option<String>) -> Result<()> {
        let now = Utc::now();
        let stored_config = StoredServerConfig {
            config: server.clone(),
            application_id: application_id.clone(),
            created_at: now,
            updated_at: now,
            version: 1,
        };

        self.servers.insert(server.name.clone(), stored_config);

        // Update application associations
        if let Some(app_id) = application_id {
            self.application_servers
                .entry(app_id)
                .or_default()
                .push(server.name.clone());
        }

        self.save_to_file()
    }

    /// Update an existing server configuration
    pub fn update_server(&mut self, server: McpServerConfig) -> Result<()> {
        if let Some(stored) = self.servers.get_mut(&server.name) {
            stored.config = server;
            stored.updated_at = Utc::now();
            stored.version += 1;
            self.save_to_file()
        } else {
            Err(anyhow::anyhow!("Server not found: {}", server.name))
        }
    }

    /// Remove a server configuration
    pub fn remove_server(&mut self, server_name: &str) -> Result<()> {
        if let Some(stored) = self.servers.remove(server_name) {
            // Remove from application associations
            if let Some(app_id) = &stored.application_id {
                if let Some(servers) = self.application_servers.get_mut(app_id) {
                    servers.retain(|name| name != server_name);
                    if servers.is_empty() {
                        self.application_servers.remove(app_id);
                    }
                }
            }
            self.save_to_file()
        } else {
            Err(anyhow::anyhow!("Server not found: {}", server_name))
        }
    }

    /// Get a server configuration by name
    pub fn get_server(&self, server_name: &str) -> Result<Option<McpServerConfig>> {
        Ok(self.servers.get(server_name).map(|stored| stored.config.clone()))
    }

    /// Get all server configurations
    pub fn get_all_servers(&self) -> Result<Vec<McpServerConfig>> {
        Ok(self.servers.values().map(|stored| stored.config.clone()).collect())
    }

    /// Get servers associated with a specific application
    pub fn get_servers_for_application(&self, application_id: &str) -> Result<Vec<McpServerConfig>> {
        let server_names = self.application_servers
            .get(application_id)
            .cloned()
            .unwrap_or_default();

        let servers = server_names
            .iter()
            .filter_map(|name| self.servers.get(name))
            .map(|stored| stored.config.clone())
            .collect();

        Ok(servers)
    }

    /// Get active applications (those with associated servers)
    pub fn get_active_applications(&self) -> Result<Vec<String>> {
        Ok(self.application_servers.keys().cloned().collect())
    }

    /// Record a configuration change
    pub fn record_change(&mut self, change: ConfigurationChange) -> Result<()> {
        self.changes.push(change);
        
        // Keep only recent changes (last 30 days)
        let cutoff = Utc::now() - Duration::days(30);
        self.changes.retain(|change| change.timestamp > cutoff);
        
        self.save_to_file()
    }

    /// Get recent configuration changes
    pub fn get_recent_changes(&self, hours: u32) -> Result<Vec<ConfigurationChange>> {
        let cutoff = Utc::now() - Duration::hours(hours as i64);
        Ok(self.changes
            .iter()
            .filter(|change| change.timestamp > cutoff)
            .cloned()
            .collect())
    }

    /// Get last sync time
    pub fn get_last_sync_time(&self) -> Result<Option<DateTime<Utc>>> {
        Ok(self.metadata.last_sync)
    }

    /// Update last sync time
    pub fn update_last_sync_time(&mut self) -> Result<()> {
        self.metadata.last_sync = Some(Utc::now());
        self.save_to_file()
    }

    /// Get store statistics
    pub fn get_stats(&self) -> StoreStats {
        let total_servers = self.servers.len();
        let active_apps = self.application_servers.len();
        let recent_changes = self.changes
            .iter()
            .filter(|c| c.timestamp > Utc::now() - Duration::hours(24))
            .count();

        StoreStats {
            total_servers,
            active_applications: active_apps,
            recent_changes,
            store_size_bytes: self.estimate_size(),
            oldest_change: self.changes.first().map(|c| c.timestamp),
            newest_change: self.changes.last().map(|c| c.timestamp),
        }
    }

    /// Estimate store size in bytes
    fn estimate_size(&self) -> usize {
        serde_json::to_string(self).map(|s| s.len()).unwrap_or(0)
    }
}

/// Store statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStats {
    pub total_servers: usize,
    pub active_applications: usize,
    pub recent_changes: usize,
    pub store_size_bytes: usize,
    pub oldest_change: Option<DateTime<Utc>>,
    pub newest_change: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::collections::HashMap;

    fn create_test_server(name: &str) -> McpServerConfig {
        McpServerConfig {
            name: name.to_string(),
            command: Some("node".to_string()),
            args: vec!["server.js".to_string()],
            env: HashMap::new(),
            cwd: None,
            server_type: crate::detection::ServerType::Stdio,
            metadata: crate::detection::ServerMetadata {
                version: Some("1.0.0".to_string()),
                description: Some("Test server".to_string()),
                author: None,
                capabilities: Vec::new(),
                enabled: true,
                source: crate::detection::ConfigSource::MainConfig,
            },
        }
    }

    #[test]
    fn test_store_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test_store.json");
        
        let store = ConfigurationStore::new(store_path);
        assert!(store.is_ok());
    }

    #[test]
    fn test_server_operations() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test_store.json");
        let mut store = ConfigurationStore::new(store_path).unwrap();

        let server = create_test_server("test-server");

        // Test adding server
        store.add_server(server.clone(), Some("app1".to_string())).unwrap();
        
        // Test getting server
        let retrieved = store.get_server("test-server").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-server");

        // Test getting all servers
        let all_servers = store.get_all_servers().unwrap();
        assert_eq!(all_servers.len(), 1);

        // Test getting servers for application
        let app_servers = store.get_servers_for_application("app1").unwrap();
        assert_eq!(app_servers.len(), 1);

        // Test updating server
        let mut updated_server = server.clone();
        updated_server.args = vec!["updated.js".to_string()];
        store.update_server(updated_server).unwrap();

        let retrieved = store.get_server("test-server").unwrap().unwrap();
        assert_eq!(retrieved.args, vec!["updated.js".to_string()]);

        // Test removing server
        store.remove_server("test-server").unwrap();
        let retrieved = store.get_server("test-server").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test_store.json");
        
        // Create store and add server
        {
            let mut store = ConfigurationStore::new(store_path.clone()).unwrap();
            let server = create_test_server("persistent-server");
            store.add_server(server, None).unwrap();
        }

        // Load store and verify server exists
        {
            let store = ConfigurationStore::new(store_path).unwrap();
            let retrieved = store.get_server("persistent-server").unwrap();
            assert!(retrieved.is_some());
        }
    }

    #[test]
    fn test_store_stats() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test_store.json");
        let mut store = ConfigurationStore::new(store_path).unwrap();

        let server = create_test_server("stats-server");
        store.add_server(server, Some("app1".to_string())).unwrap();

        let stats = store.get_stats();
        assert_eq!(stats.total_servers, 1);
        assert_eq!(stats.active_applications, 1);
    }
}
