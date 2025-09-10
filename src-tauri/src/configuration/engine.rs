use std::path::PathBuf;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::detection::{ApplicationDetector, ConfigValidator, McpServerConfig, ApplicationProfile};
use crate::filesystem::ConfigFileService;
use super::{ConfigurationStore, SyncManager};

/// Central configuration management engine
pub struct ConfigurationEngine {
    store: ConfigurationStore,
    sync_manager: SyncManager,
    detector: ApplicationDetector,
    validator: ConfigValidator,
    file_service: ConfigFileService,
}

/// Configuration change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChange {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub change_type: ChangeType,
    pub server_id: String,
    pub application_id: Option<String>,
    pub details: String,
}

/// Type of configuration change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    ServerAdded,
    ServerUpdated,
    ServerRemoved,
    ApplicationSynced,
    ConflictResolved,
}

/// Configuration engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub total_servers: usize,
    pub active_applications: usize,
    pub sync_conflicts: usize,
    pub last_sync: Option<DateTime<Utc>>,
    pub changes_today: usize,
}

impl ConfigurationEngine {
    /// Create a new configuration engine
    pub fn new(store_path: PathBuf, backup_dir: PathBuf) -> Result<Self> {
        let store = ConfigurationStore::new(store_path)?;
        let sync_manager = SyncManager::new();
        let detector = ApplicationDetector::new()?;
        let validator = ConfigValidator::new()?;
        let file_service = ConfigFileService::new(Uuid::new_v4().to_string(), backup_dir);

        Ok(Self {
            store,
            sync_manager,
            detector,
            validator,
            file_service,
        })
    }

    /// Initialize the engine by detecting applications and importing configurations
    pub async fn initialize(&mut self) -> Result<()> {
        // Detect applications
        let detection_results = self.detector.detect_all_applications().await?;
        
        // Import existing configurations from detected applications
        for result in detection_results {
            if let Err(e) = self.import_application_config(&result.profile).await {
                eprintln!("Failed to import config from {}: {}", result.profile.name, e);
            }
        }

        Ok(())
    }

    /// Import configuration from a detected application
    async fn import_application_config(&mut self, app: &ApplicationProfile) -> Result<()> {
        // Validate and extract MCP servers from application config
        let validation_result = self.validator.validate_application_config(app).await?;
        
        if validation_result.is_valid {
            // Add extracted servers to our store
            for server in validation_result.mcp_servers {
                self.store.add_server(server, Some(app.id.to_string()))?;
            }
            
            // Record the import
            self.record_change(ChangeType::ApplicationSynced, 
                "imported".to_string(), 
                Some(app.id.to_string()))?;
        }

        Ok(())
    }

    /// Get all MCP server configurations
    pub fn get_all_servers(&self) -> Result<Vec<McpServerConfig>> {
        self.store.get_all_servers()
    }

    /// Get a specific server configuration
    pub fn get_server(&self, server_id: &str) -> Result<Option<McpServerConfig>> {
        self.store.get_server(server_id)
    }

    /// Add a new MCP server configuration
    pub fn add_server(&mut self, server: McpServerConfig, application_id: Option<String>) -> Result<()> {
        self.store.add_server(server.clone(), application_id.clone())?;
        self.record_change(ChangeType::ServerAdded, server.name.clone(), application_id)?;
        Ok(())
    }

    /// Update an existing MCP server configuration
    pub fn update_server(&mut self, server: McpServerConfig) -> Result<()> {
        self.store.update_server(server.clone())?;
        self.record_change(ChangeType::ServerUpdated, server.name.clone(), None)?;
        Ok(())
    }

    /// Remove an MCP server configuration
    pub fn remove_server(&mut self, server_id: &str) -> Result<()> {
        self.store.remove_server(server_id)?;
        self.record_change(ChangeType::ServerRemoved, server_id.to_string(), None)?;
        Ok(())
    }

    /// Synchronize configurations with all detected applications
    pub async fn sync_all_applications(&mut self) -> Result<Vec<String>> {
        let detection_results = self.detector.detect_all_applications().await?;
        let mut sync_results = Vec::new();

        for result in detection_results {
            match self.sync_application(&result.profile).await {
                Ok(_) => sync_results.push(format!("✓ {}", result.profile.name)),
                Err(e) => sync_results.push(format!("✗ {}: {}", result.profile.name, e)),
            }
        }

        Ok(sync_results)
    }

    /// Synchronize configuration with a specific application
    async fn sync_application(&mut self, app: &ApplicationProfile) -> Result<()> {
        // Get servers associated with this application
        let app_servers = self.store.get_servers_for_application(&app.id.to_string())?;
        
        if !app_servers.is_empty() {
            // Use adapter-based sync manager to update application config
            let sync_result = self.sync_manager.sync_to_application_with_adapter(app, &app_servers, &mut self.file_service).await?;
            
            if sync_result.success {
                self.record_change(ChangeType::ApplicationSynced, 
                    format!("synced {} servers", sync_result.servers_synced), 
                    Some(app.id.to_string()))?;
            } else {
                // Return error instead of just logging
                let error_msg = sync_result.errors.join("; ");
                return Err(anyhow::anyhow!("Sync failed: {}", error_msg));
            }
        }

        Ok(())
    }

    /// Get engine statistics
    pub fn get_stats(&self) -> Result<EngineStats> {
        let servers = self.store.get_all_servers()?;
        let applications = self.store.get_active_applications()?;
        let changes = self.store.get_recent_changes(24)?; // Last 24 hours

        Ok(EngineStats {
            total_servers: servers.len(),
            active_applications: applications.len(),
            sync_conflicts: 0, // TODO: Implement conflict tracking
            last_sync: self.store.get_last_sync_time()?,
            changes_today: changes.len(),
        })
    }



    /// Record a configuration change
    fn record_change(&mut self, change_type: ChangeType, details: String, application_id: Option<String>) -> Result<()> {
        let change = ConfigurationChange {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            change_type,
            server_id: details.clone(),
            application_id,
            details,
        };

        self.store.record_change(change)
    }

    /// Get recent configuration changes
    pub fn get_recent_changes(&self, hours: u32) -> Result<Vec<ConfigurationChange>> {
        self.store.get_recent_changes(hours)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("config_store.json");
        let backup_dir = temp_dir.path().join("backups");
        
        let engine = ConfigurationEngine::new(store_path, backup_dir);
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_server_management() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("config_store.json");
        let backup_dir = temp_dir.path().join("backups");
        let mut engine = ConfigurationEngine::new(store_path, backup_dir).unwrap();

        // Create a test server
        let server = McpServerConfig {
            name: "test-server".to_string(),
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
        };

        // Test adding server
        engine.add_server(server.clone(), None).unwrap();
        
        // Test getting server
        let retrieved = engine.get_server("test-server").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-server");

        // Test getting all servers
        let all_servers = engine.get_all_servers().unwrap();
        assert_eq!(all_servers.len(), 1);
    }

    #[test]
    fn test_change_recording() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("config_store.json");
        let backup_dir = temp_dir.path().join("backups");
        let mut engine = ConfigurationEngine::new(store_path, backup_dir).unwrap();

        // Record a change
        engine.record_change(
            ChangeType::ServerAdded,
            "test-server".to_string(),
            None
        ).unwrap();

        // Get recent changes
        let changes = engine.get_recent_changes(1).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].server_id, "test-server");
    }
}


