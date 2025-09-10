use anyhow::{Result, Context};
use serde_json::Value as JsonValue;

use crate::detection::{ApplicationProfile, McpServerConfig};
use crate::filesystem::ConfigFileService;
use crate::adapters::AdapterFactory;

/// Manages synchronization between central store and application configurations
#[derive(Debug)]
pub struct SyncManager {
    // Stateless sync manager - no fields needed for basic operations
}

/// Synchronization result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub servers_synced: usize,
    pub conflicts: Vec<SyncConflict>,
    pub errors: Vec<String>,
}

/// Configuration synchronization conflict
#[derive(Debug, Clone)]
pub struct SyncConflict {
    pub server_name: String,
    pub conflict_type: ConflictType,
    pub central_version: Option<McpServerConfig>,
    pub app_version: Option<McpServerConfig>,
    pub resolution: ConflictResolution,
}

/// Type of synchronization conflict
#[derive(Debug, Clone)]
pub enum ConflictType {
    ServerModifiedInBoth,
    ServerRemovedFromApp,
    ServerAddedToApp,
    ConfigurationMismatch,
}

/// How to resolve a conflict
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    UseCentral,
    UseApplication,
    Merge,
    Skip,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new() -> Self {
        Self {}
    }

    /// Sync servers from central store to application configuration
    pub async fn sync_to_application(
        &self,
        app: &ApplicationProfile,
        servers: &[McpServerConfig],
        file_service: &mut ConfigFileService,
    ) -> Result<SyncResult> {
        let mut result = SyncResult {
            success: false,
            servers_synced: 0,
            conflicts: Vec::new(),
            errors: Vec::new(),
        };

        // Read current application configuration
        let current_config = match self.read_app_config(app, file_service).await {
            Ok(config) => config,
            Err(e) => {
                result.errors.push(format!("Failed to read app config: {}", e));
                return Ok(result);
            }
        };

        // Apply servers to configuration based on application type
        let updated_config = match self.apply_servers_to_config(app, &current_config, servers) {
            Ok(config) => config,
            Err(e) => {
                result.errors.push(format!("Failed to apply servers: {}", e));
                return Ok(result);
            }
        };

        // Write updated configuration back to application
        match self.write_app_config(app, &updated_config, file_service).await {
            Ok(_) => {
                result.success = true;
                result.servers_synced = servers.len();
            }
            Err(e) => {
                result.errors.push(format!("Failed to write app config: {}", e));
            }
        }

        Ok(result)
    }

    /// Read application configuration
    async fn read_app_config(
        &self,
        app: &ApplicationProfile,
        file_service: &mut ConfigFileService,
    ) -> Result<JsonValue> {
        use crate::filesystem::paths::PathUtils;
        let expanded_path = PathUtils::expand_tilde(&app.config_path)?;
        file_service.read_config(&expanded_path).await
    }

    /// Write application configuration
    async fn write_app_config(
        &self,
        app: &ApplicationProfile,
        config: &JsonValue,
        file_service: &mut ConfigFileService,
    ) -> Result<()> {
        use crate::filesystem::paths::PathUtils;
        let expanded_path = PathUtils::expand_tilde(&app.config_path)?;
        
        // Create backup before writing
        self.create_backup_before_write(&expanded_path).await?;
        
        file_service.write_config(&expanded_path, config).await
    }

    /// Create backup of config file before modification
    async fn create_backup_before_write(&self, config_path: &std::path::Path) -> Result<()> {
        if !config_path.exists() {
            return Ok(()); // No backup needed if file doesn't exist
        }

        // Create backup directory
        let backup_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".mcp-control-backups");
        
        tokio::fs::create_dir_all(&backup_dir).await
            .context("Failed to create backup directory")?;

        // Generate backup filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let original_name = config_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("config");
        let backup_name = format!("{}_{}.backup", original_name, timestamp);
        let backup_path = backup_dir.join(backup_name);

        // Copy original file to backup
        tokio::fs::copy(config_path, &backup_path).await
            .with_context(|| format!("Failed to create backup: {} -> {}", 
                config_path.display(), backup_path.display()))?;

        println!("📁 Backup created: {}", backup_path.display());
        Ok(())
    }

    /// Apply MCP servers to application configuration based on app type
    fn apply_servers_to_config(
        &self,
        app: &ApplicationProfile,
        current_config: &JsonValue,
        servers: &[McpServerConfig],
    ) -> Result<JsonValue> {
        let mut config = current_config.clone();

        match app.id.as_str() {
            "claude-desktop" => self.apply_claude_desktop_servers(&mut config, servers)?,
            "cursor" => self.apply_cursor_servers(&mut config, servers)?,
            "zed" => self.apply_zed_servers(&mut config, servers)?,
            "vscode" => self.apply_vscode_servers(&mut config, servers)?,
            "amazon-q" => self.apply_amazon_q_servers(&mut config, servers)?,
            _ => self.apply_generic_servers(&mut config, servers)?,
        }

        Ok(config)
    }

    /// Apply servers to Claude Desktop configuration
    fn apply_claude_desktop_servers(&self, config: &mut JsonValue, servers: &[McpServerConfig]) -> Result<()> {
        // Ensure mcpServers object exists
        if config.get("mcpServers").is_none() {
            config["mcpServers"] = serde_json::json!({});
        }
        
        let mcp_servers = config.get_mut("mcpServers")
            .context("Failed to get mcpServers object")?;

        // Clear existing servers and add new ones
        *mcp_servers = serde_json::json!({});
        
        for server in servers {
            let server_config = serde_json::json!({
                "command": server.command,
                "args": server.args,
                "env": server.env
            });
            
            mcp_servers[&server.name] = server_config;
        }

        Ok(())
    }

    /// Apply servers to Cursor configuration
    fn apply_cursor_servers(&self, config: &mut JsonValue, servers: &[McpServerConfig]) -> Result<()> {
        // Ensure mcp.servers structure exists
        if config.get("mcp").is_none() {
            config["mcp"] = serde_json::json!({ "servers": {} });
        }
        
        let mcp_config = config.get_mut("mcp")
            .context("Failed to get mcp config")?;

        let servers_obj = mcp_config.get_mut("servers")
            .context("Failed to get servers object")?;

        *servers_obj = serde_json::json!({});
        
        for server in servers {
            servers_obj[&server.name] = serde_json::json!({
                "command": server.command,
                "args": server.args,
                "env": server.env
            });
        }

        Ok(())
    }

    /// Apply servers to Zed configuration
    fn apply_zed_servers(&self, config: &mut JsonValue, servers: &[McpServerConfig]) -> Result<()> {
        // Ensure language_servers object exists
        if config.get("language_servers").is_none() {
            config["language_servers"] = serde_json::json!({});
        }
        
        let mcp_servers = config.get_mut("language_servers")
            .context("Failed to get language_servers")?;

        for server in servers {
            mcp_servers[&server.name] = serde_json::json!({
                "command": server.command,
                "args": server.args,
                "env": server.env
            });
        }

        Ok(())
    }

    /// Apply servers to VS Code configuration
    fn apply_vscode_servers(&self, config: &mut JsonValue, servers: &[McpServerConfig]) -> Result<()> {
        // Ensure mcp.servers object exists
        if config.get("mcp.servers").is_none() {
            config["mcp.servers"] = serde_json::json!({});
        }
        
        let mcp_config = config.get_mut("mcp.servers")
            .context("Failed to get mcp.servers")?;

        *mcp_config = serde_json::json!({});
        
        for server in servers {
            mcp_config[&server.name] = serde_json::json!({
                "command": server.command,
                "args": server.args,
                "env": server.env
            });
        }

        Ok(())
    }

    /// Apply servers to Amazon Q Developer configuration
    fn apply_amazon_q_servers(&self, config: &mut JsonValue, servers: &[McpServerConfig]) -> Result<()> {
        // Amazon Q Developer uses mcpServers format
        if config.get("mcpServers").is_none() {
            config["mcpServers"] = serde_json::json!({});
        }
        
        let servers_obj = config.get_mut("mcpServers")
            .context("Failed to get mcpServers object")?;

        *servers_obj = serde_json::json!({});
        
        for server in servers {
            let mut server_config = serde_json::json!({
                "command": server.command,
                "args": server.args
            });
            
            if !server.env.is_empty() {
                server_config["env"] = serde_json::json!(server.env);
            }
            
            servers_obj[&server.name] = server_config;
        }

        Ok(())
    }

    /// Apply servers to generic application configuration
    fn apply_generic_servers(&self, config: &mut JsonValue, servers: &[McpServerConfig]) -> Result<()> {
        // Generic approach - look for common MCP configuration patterns
        let mcp_key = if config.get("mcpServers").is_some() {
            "mcpServers"
        } else if config.get("mcp").is_some() {
            "mcp"
        } else {
            "mcpServers" // Default
        };

        config[mcp_key] = serde_json::json!({});
        
        for server in servers {
            config[mcp_key][&server.name] = serde_json::json!({
                "command": server.command,
                "args": server.args,
                "env": server.env
            });
        }

        Ok(())
    }

    /// Synchronize servers to application using adapters (new method)
    pub async fn sync_to_application_with_adapter(
        &self,
        app: &ApplicationProfile,
        servers: &[McpServerConfig],
        file_service: &mut ConfigFileService,
    ) -> Result<SyncResult> {
        let mut result = SyncResult {
            success: false,
            servers_synced: 0,
            conflicts: Vec::new(),
            errors: Vec::new(),
        };

        // Create adapter for this application
        let adapter = match AdapterFactory::create_adapter(app) {
            Ok(adapter) => adapter,
            Err(e) => {
                result.errors.push(format!("Failed to create adapter: {}", e));
                return Ok(result);
            }
        };
        
        // Read current application config
        let current_config = match self.read_app_config(app, file_service).await {
            Ok(config) => config,
            Err(e) => {
                result.errors.push(format!("Failed to read config: {}", e));
                return Ok(result);
            }
        };

        // Apply servers using adapter
        match adapter.apply_server_configs(&current_config, servers).await {
            Ok(adapter_result) => {
                if adapter_result.success {
                    // Write updated config back
                    if let Err(e) = self.write_app_config(app, &adapter_result.config, file_service).await {
                        result.errors.push(format!("Failed to write config: {}", e));
                        return Ok(result);
                    }
                    
                    result.success = true;
                    result.servers_synced = servers.len();
                } else {
                    result.errors.extend(adapter_result.messages);
                }
            }
            Err(e) => {
                result.errors.push(format!("Adapter failed to apply config: {}", e));
            }
        }

        Ok(result)
    }

    /// Extract servers from application using adapters (new method)
    pub async fn extract_from_application_with_adapter(
        &self,
        app: &ApplicationProfile,
        file_service: &mut ConfigFileService,
    ) -> Result<Vec<McpServerConfig>> {
        // Create adapter for this application
        let adapter = AdapterFactory::create_adapter(app)?;
        
        // Read application config
        let config = self.read_app_config(app, file_service).await?;
        
        // Extract servers using adapter
        let result = adapter.extract_server_configs(&config).await?;
        
        if result.success {
            Ok(result.servers)
        } else {
            // Return empty vec if extraction failed but don't error
            Ok(vec![])
        }
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn create_test_app(id: &str, name: &str) -> ApplicationProfile {
        ApplicationProfile {
            id: id.to_string(),
            name: name.to_string(),
            bundle_id: format!("com.test.{}", id),
            config_path: format!("~/Library/Application Support/{}/config.json", name),
            alt_config_paths: Vec::new(),
            config_format: crate::detection::ConfigFormat::Json,
            executable_paths: vec![format!("/Applications/{}.app", name)],
            alt_executable_paths: Vec::new(),
            detection_strategy: crate::detection::DetectionStrategy {
                use_bundle_lookup: true,
                use_executable_check: true,
                use_config_check: true,
                use_spotlight: false,
                priority_order: vec![
                    crate::detection::DetectionMethod::BundleLookup,
                    crate::detection::DetectionMethod::ExecutableCheck,
                ],
            },
            metadata: crate::detection::ApplicationMetadata {
                version: Some("1.0.0".to_string()),
                developer: "Test Developer".to_string(),
                category: crate::detection::ApplicationCategory::CodeEditor,
                mcp_version: "1.0".to_string(),
                notes: None,
                requires_permissions: false,
            },
        }
    }

    #[test]
    fn test_sync_manager_creation() {
        let _sync_manager = SyncManager::new();
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_claude_desktop_config_application() {
        let sync_manager = SyncManager::new();
        let mut config = serde_json::json!({});
        let servers = vec![create_test_server("test-server")];

        sync_manager.apply_claude_desktop_servers(&mut config, &servers).unwrap();

        assert!(config.get("mcpServers").is_some());
        assert!(config["mcpServers"].get("test-server").is_some());
        assert_eq!(config["mcpServers"]["test-server"]["command"], "node");
    }

    #[test]
    fn test_cursor_config_application() {
        let sync_manager = SyncManager::new();
        let mut config = serde_json::json!({});
        let servers = vec![create_test_server("test-server")];

        sync_manager.apply_cursor_servers(&mut config, &servers).unwrap();

        assert!(config.get("mcp").is_some());
        assert!(config["mcp"]["servers"].get("test-server").is_some());
    }

    #[test]
    fn test_generic_config_application() {
        let sync_manager = SyncManager::new();
        let mut config = serde_json::json!({});
        let servers = vec![create_test_server("test-server")];

        sync_manager.apply_generic_servers(&mut config, &servers).unwrap();

        assert!(config.get("mcpServers").is_some());
        assert!(config["mcpServers"].get("test-server").is_some());
    }
}


