use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value as JsonValue;

use crate::detection::{McpServerConfig, ApplicationProfile, ConfigFormat};
use super::{ApplicationAdapter, ExtractionResult, ApplicationResult};

/// Generic application adapter for unknown applications
pub struct GenericAdapter;

impl GenericAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ApplicationAdapter for GenericAdapter {
    async fn extract_server_configs(&self, config: &JsonValue) -> Result<ExtractionResult> {
        let mut servers = Vec::new();
        let mut messages = Vec::new();
        
        // Try common MCP configuration patterns
        if let Some(mcp_servers) = config.get("mcpServers").and_then(|v| v.as_object()) {
            // Amazon Q / Claude Desktop style
            for (name, server_config) in mcp_servers {
                if let Ok(server) = self.parse_generic_server_config(name, server_config) {
                    servers.push(server);
                } else {
                    messages.push(format!("Failed to parse server '{}'", name));
                }
            }
        } else if let Some(mcp_config) = config.get("mcp") {
            if let Some(mcp_servers) = mcp_config.get("servers").and_then(|v| v.as_object()) {
                // Cursor style
                for (name, server_config) in mcp_servers {
                    if let Ok(server) = self.parse_generic_server_config(name, server_config) {
                        servers.push(server);
                    } else {
                        messages.push(format!("Failed to parse server '{}'", name));
                    }
                }
            }
        } else {
            messages.push("No recognized MCP configuration pattern found".to_string());
        }
        
        let success = !servers.is_empty();
        
        Ok(ExtractionResult {
            servers,
            messages,
            success,
        })
    }
    
    async fn apply_server_configs(&self, config: &JsonValue, servers: &[McpServerConfig]) -> Result<ApplicationResult> {
        let mut new_config = config.clone();
        let mut messages = Vec::new();
        
        // Use mcpServers format as default
        if new_config.get("mcpServers").is_none() {
            new_config["mcpServers"] = serde_json::json!({});
        }
        
        let mcp_servers = new_config["mcpServers"]
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("Failed to get mcpServers as object"))?;
        
        // Clear existing servers
        mcp_servers.clear();
        
        // Add new servers
        for server in servers {
            let server_config = self.format_generic_server_config(server);
            mcp_servers.insert(server.name.clone(), server_config);
            messages.push(format!("Added server '{}'", server.name));
        }
        
        Ok(ApplicationResult {
            config: new_config,
            messages,
            success: true,
        })
    }
    
    async fn validate_config(&self, _config: &JsonValue) -> Result<bool> {
        // Generic validation - just check if it's valid JSON
        Ok(true)
    }
    
    fn get_supported_formats(&self) -> Vec<ConfigFormat> {
        vec![ConfigFormat::Json, ConfigFormat::Yaml, ConfigFormat::Toml]
    }
    
    fn get_name(&self) -> &str {
        "generic"
    }
    
    fn can_handle(&self, _profile: &ApplicationProfile) -> bool {
        // Generic adapter can handle any application as fallback
        true
    }
}

impl GenericAdapter {
    fn parse_generic_server_config(&self, name: &str, config: &JsonValue) -> Result<McpServerConfig> {
        use crate::detection::{ServerType, ServerMetadata, ConfigSource};
        
        let command = config.get("command")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let args = config.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        
        let env = config.get("env")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(McpServerConfig {
            name: name.to_string(),
            command,
            args,
            env,
            cwd: None,
            server_type: ServerType::Stdio,
            metadata: ServerMetadata {
                description: None,
                version: None,
                author: None,
                capabilities: vec![],
                enabled: !config.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false),
                source: ConfigSource::MainConfig,
            },
        })
    }
    
    fn format_generic_server_config(&self, server: &McpServerConfig) -> JsonValue {
        let mut config = serde_json::json!({
            "command": server.command,
            "args": server.args
        });
        
        if !server.env.is_empty() {
            config["env"] = serde_json::json!(server.env);
        }
        
        if !server.metadata.enabled {
            config["disabled"] = serde_json::json!(true);
        }
        
        config
    }
}
