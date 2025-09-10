use crate::detection::profiles::{ApplicationProfile, ConfigFormat};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration validation and extraction service
#[derive(Debug)]
pub struct ConfigValidator {
    // No fields needed - stateless validator
}

/// Result of configuration validation and extraction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigValidationResult {
    /// Application profile that was validated
    pub application: ApplicationProfile,
    /// Whether the configuration file exists and is valid
    pub is_valid: bool,
    /// Configuration file path that was found (if any)
    pub config_path: Option<PathBuf>,
    /// Configuration format detected
    pub detected_format: Option<ConfigFormat>,
    /// Extracted MCP server configurations
    pub mcp_servers: Vec<McpServerConfig>,
    /// Validation messages and errors
    pub messages: Vec<ValidationMessage>,
    /// Raw configuration data (for debugging)
    pub raw_config: Option<JsonValue>,
    /// Validation timestamp
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

/// MCP server configuration extracted from application config
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServerConfig {
    /// Server identifier/name
    pub name: String,
    /// Server command or executable path
    pub command: Option<String>,
    /// Command arguments
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Working directory
    pub cwd: Option<String>,
    /// Server type (stdio, sse, websocket, etc.)
    pub server_type: ServerType,
    /// Additional metadata
    pub metadata: ServerMetadata,
}

/// Types of MCP server connections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerType {
    /// Standard input/output communication
    Stdio,
    /// Server-sent events
    Sse { url: String },
    /// WebSocket connection
    WebSocket { url: String },
    /// HTTP/REST API
    Http { base_url: String },
    /// Custom/unknown type
    Custom(String),
}

/// Additional server metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerMetadata {
    /// Server description
    pub description: Option<String>,
    /// Server version
    pub version: Option<String>,
    /// Server author/maintainer
    pub author: Option<String>,
    /// Server capabilities
    pub capabilities: Vec<String>,
    /// Whether server is enabled
    pub enabled: bool,
    /// Server configuration source
    pub source: ConfigSource,
}

/// Source of the server configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigSource {
    /// From application's main config file
    MainConfig,
    /// From alternative config file
    AlternativeConfig,
    /// From environment variables
    Environment,
    /// From command line arguments
    CommandLine,
    /// Unknown source
    Unknown,
}

/// Validation message for configuration issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationMessage {
    /// Message severity level
    pub level: MessageLevel,
    /// Message content
    pub message: String,
    /// Configuration path or field that caused the message
    pub path: Option<String>,
    /// Suggested fix or action
    pub suggestion: Option<String>,
}

/// Message severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for MessageLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageLevel::Info => write!(f, "INFO"),
            MessageLevel::Warning => write!(f, "WARN"),
            MessageLevel::Error => write!(f, "ERROR"),
            MessageLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Summary of validation results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationSummary {
    /// Total number of applications validated
    pub total_applications: usize,
    /// Number of applications with valid configurations
    pub valid_configs: usize,
    /// Number of applications with invalid configurations
    pub invalid_configs: usize,
    /// Total number of MCP servers found
    pub total_servers: usize,
    /// Number of applications with MCP servers
    pub applications_with_servers: usize,
    /// Number of applications without MCP servers
    pub applications_without_servers: usize,
    /// Breakdown of configuration formats found
    pub format_breakdown: HashMap<String, usize>,
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Validate and extract configuration for a single application
    pub async fn validate_application_config(&self, application: &ApplicationProfile) -> Result<ConfigValidationResult> {
        let mut messages = Vec::new();
        let mut mcp_servers = Vec::new();
        let mut config_path = None;
        let mut detected_format = None;
        let mut raw_config = None;
        let mut is_valid = false;

        // Try to find and read the configuration file
        if let Some((found_path, format, content)) = self.find_config_file(application).await? {
            config_path = Some(found_path.clone());
            detected_format = Some(format.clone());
            
            match self.parse_config_content(&content, &format) {
                Ok(parsed_config) => {
                    raw_config = Some(parsed_config.clone());
                    
                    // Extract MCP servers from the configuration
                    match self.extract_mcp_servers(&parsed_config, application, &found_path) {
                        Ok(servers) => {
                            mcp_servers = servers;
                            is_valid = true;
                            
                            if mcp_servers.is_empty() {
                                messages.push(ValidationMessage {
                                    level: MessageLevel::Warning,
                                    message: "No MCP servers found in configuration".to_string(),
                                    path: Some(found_path.display().to_string()),
                                    suggestion: Some("Add MCP server configurations to enable MCP functionality".to_string()),
                                });
                            } else {
                                messages.push(ValidationMessage {
                                    level: MessageLevel::Info,
                                    message: format!("Found {} MCP server(s) in configuration", mcp_servers.len()),
                                    path: Some(found_path.display().to_string()),
                                    suggestion: None,
                                });
                            }
                        }
                        Err(e) => {
                            messages.push(ValidationMessage {
                                level: MessageLevel::Error,
                                message: format!("Failed to extract MCP servers: {}", e),
                                path: Some(found_path.display().to_string()),
                                suggestion: Some("Check configuration format and MCP server definitions".to_string()),
                            });
                        }
                    }
                }
                Err(e) => {
                    messages.push(ValidationMessage {
                        level: MessageLevel::Error,
                        message: format!("Failed to parse configuration file: {}", e),
                        path: Some(found_path.display().to_string()),
                        suggestion: Some("Check configuration file syntax and format".to_string()),
                    });
                }
            }
        } else {
            messages.push(ValidationMessage {
                level: MessageLevel::Warning,
                message: "Configuration file not found".to_string(),
                path: None,
                suggestion: Some("Create a configuration file to enable MCP functionality".to_string()),
            });
        }

        Ok(ConfigValidationResult {
            application: application.clone(),
            is_valid,
            config_path,
            detected_format,
            mcp_servers,
            messages,
            raw_config,
            validated_at: chrono::Utc::now(),
        })
    }

    /// Validate configurations for multiple applications
    pub async fn validate_multiple_configs(&self, applications: &[ApplicationProfile]) -> Result<Vec<ConfigValidationResult>> {
        let mut results = Vec::new();
        
        for application in applications {
            let result = self.validate_application_config(application).await?;
            results.push(result);
        }
        
        Ok(results)
    }

    /// Get summary statistics for validation results
    pub fn get_validation_summary(&self, results: &[ConfigValidationResult]) -> ValidationSummary {
        let total_applications = results.len();
        let valid_configs = results.iter().filter(|r| r.is_valid).count();
        let total_servers = results.iter().map(|r| r.mcp_servers.len()).sum();
        let applications_with_servers = results.iter().filter(|r| !r.mcp_servers.is_empty()).count();
        
        let mut format_breakdown = HashMap::new();
        for result in results {
            if let Some(format) = &result.detected_format {
                let format_name = match format {
                    ConfigFormat::Json => "JSON",
                    ConfigFormat::Yaml => "YAML", 
                    ConfigFormat::Toml => "TOML",
                    ConfigFormat::Plist => "Plist",
                    ConfigFormat::Custom(name) => name,
                };
                *format_breakdown.entry(format_name.to_string()).or_insert(0) += 1;
            }
        }

        ValidationSummary {
            total_applications,
            valid_configs,
            invalid_configs: total_applications - valid_configs,
            total_servers,
            applications_with_servers,
            applications_without_servers: total_applications - applications_with_servers,
            format_breakdown,
        }
    }

    // Private helper methods

    /// Find the configuration file for an application
    async fn find_config_file(&self, application: &ApplicationProfile) -> Result<Option<(PathBuf, ConfigFormat, String)>> {
        // Try primary config path
        let primary_path = self.expand_path(&application.config_path)?;
        if primary_path.exists() {
            let content = tokio::fs::read_to_string(&primary_path).await
                .context("Failed to read primary config file")?;
            return Ok(Some((primary_path, application.config_format.clone(), content)));
        }

        // Try alternative config paths
        for alt_path in &application.alt_config_paths {
            let expanded_path = self.expand_path(alt_path)?;
            if expanded_path.exists() {
                let content = tokio::fs::read_to_string(&expanded_path).await
                    .context("Failed to read alternative config file")?;
                return Ok(Some((expanded_path, application.config_format.clone(), content)));
            }
        }

        Ok(None)
    }

    /// Parse configuration content based on format
    fn parse_config_content(&self, content: &str, format: &ConfigFormat) -> Result<JsonValue> {
        match format {
            ConfigFormat::Json => {
                serde_json::from_str(content)
                    .context("Failed to parse JSON configuration")
            }
            ConfigFormat::Yaml => {
                let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
                    .context("Failed to parse YAML configuration")?;
                serde_json::to_value(yaml_value)
                    .context("Failed to convert YAML to JSON")
            }
            ConfigFormat::Toml => {
                let toml_value: toml::Value = content.parse()
                    .context("Failed to parse TOML configuration")?;
                serde_json::to_value(toml_value)
                    .context("Failed to convert TOML to JSON")
            }
            ConfigFormat::Plist => {
                // For now, treat plist as JSON (could be enhanced later)
                serde_json::from_str(content)
                    .context("Failed to parse Plist configuration")
            }
            ConfigFormat::Custom(_) => {
                // Try JSON first, then YAML as fallback
                serde_json::from_str(content)
                    .or_else(|_| {
                        let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
                            .context("Failed to parse as YAML")?;
                        serde_json::to_value(yaml_value)
                            .context("Failed to convert YAML to JSON")
                    })
                    .context("Failed to parse custom configuration format")
            }
        }
    }

    /// Extract MCP server configurations from parsed config
    fn extract_mcp_servers(&self, config: &JsonValue, application: &ApplicationProfile, config_path: &Path) -> Result<Vec<McpServerConfig>> {
        let mut servers = Vec::new();

        // Different applications have different MCP server configuration structures
        match application.id.as_str() {
            "claude-desktop" => {
                servers.extend(self.extract_claude_desktop_servers(config)?);
            }
            "cursor" => {
                servers.extend(self.extract_cursor_servers(config)?);
            }
            "zed" => {
                servers.extend(self.extract_zed_servers(config)?);
            }
            "vscode" => {
                servers.extend(self.extract_vscode_servers(config)?);
            }
            _ => {
                // Generic extraction for custom applications
                servers.extend(self.extract_generic_servers(config)?);
            }
        }

        // Set source for all servers
        for server in &mut servers {
            server.metadata.source = if config_path == Path::new(&application.config_path) {
                ConfigSource::MainConfig
            } else {
                ConfigSource::AlternativeConfig
            };
        }

        Ok(servers)
    }

    /// Extract MCP servers from Claude Desktop configuration
    fn extract_claude_desktop_servers(&self, config: &JsonValue) -> Result<Vec<McpServerConfig>> {
        let mut servers = Vec::new();

        if let Some(mcp_servers) = config.get("mcpServers").and_then(|v| v.as_object()) {
            for (name, server_config) in mcp_servers {
                if let Some(server_obj) = server_config.as_object() {
                    let command = server_obj.get("command").and_then(|v| v.as_str()).map(String::from);
                    let args = server_obj.get("args")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    
                    let env = server_obj.get("env")
                        .and_then(|v| v.as_object())
                        .map(|obj| obj.iter().filter_map(|(k, v)| {
                            v.as_str().map(|s| (k.clone(), s.to_string()))
                        }).collect())
                        .unwrap_or_default();

                    servers.push(McpServerConfig {
                        name: name.clone(),
                        command,
                        args,
                        env,
                        cwd: server_obj.get("cwd").and_then(|v| v.as_str()).map(String::from),
                        server_type: ServerType::Stdio, // Claude Desktop uses stdio
                        metadata: ServerMetadata {
                            description: server_obj.get("description").and_then(|v| v.as_str()).map(String::from),
                            version: None,
                            author: None,
                            capabilities: Vec::new(),
                            enabled: true,
                            source: ConfigSource::MainConfig,
                        },
                    });
                }
            }
        }

        Ok(servers)
    }

    /// Extract MCP servers from Cursor configuration
    fn extract_cursor_servers(&self, config: &JsonValue) -> Result<Vec<McpServerConfig>> {
        let mut servers = Vec::new();

        // Cursor might have MCP servers in extensions or settings
        if let Some(extensions) = config.get("extensions").and_then(|v| v.as_object()) {
            for (name, ext_config) in extensions {
                if name.contains("mcp") || name.contains("model-context-protocol") {
                    if let Some(ext_obj) = ext_config.as_object() {
                        servers.push(McpServerConfig {
                            name: name.clone(),
                            command: ext_obj.get("command").and_then(|v| v.as_str()).map(String::from),
                            args: Vec::new(),
                            env: HashMap::new(),
                            cwd: None,
                            server_type: ServerType::Custom("extension".to_string()),
                            metadata: ServerMetadata {
                                description: Some(format!("Cursor extension: {}", name)),
                                version: ext_obj.get("version").and_then(|v| v.as_str()).map(String::from),
                                author: None,
                                capabilities: Vec::new(),
                                enabled: ext_obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                                source: ConfigSource::MainConfig,
                            },
                        });
                    }
                }
            }
        }

        Ok(servers)
    }

    /// Extract MCP servers from Zed configuration
    fn extract_zed_servers(&self, config: &JsonValue) -> Result<Vec<McpServerConfig>> {
        let mut servers = Vec::new();

        if let Some(mcp_config) = config.get("mcp") {
            if let Some(servers_config) = mcp_config.get("servers").and_then(|v| v.as_object()) {
                for (name, server_config) in servers_config {
                    if let Some(server_obj) = server_config.as_object() {
                        servers.push(McpServerConfig {
                            name: name.clone(),
                            command: server_obj.get("command").and_then(|v| v.as_str()).map(String::from),
                            args: server_obj.get("args")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default(),
                            env: HashMap::new(),
                            cwd: None,
                            server_type: ServerType::Stdio,
                            metadata: ServerMetadata {
                                description: server_obj.get("description").and_then(|v| v.as_str()).map(String::from),
                                version: None,
                                author: None,
                                capabilities: Vec::new(),
                                enabled: server_obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                                source: ConfigSource::MainConfig,
                            },
                        });
                    }
                }
            }
        }

        Ok(servers)
    }

    /// Extract MCP servers from VS Code configuration
    fn extract_vscode_servers(&self, config: &JsonValue) -> Result<Vec<McpServerConfig>> {
        let mut servers = Vec::new();

        // VS Code might have MCP servers in settings or extensions
        if let Some(settings) = config.as_object() {
            for (key, value) in settings {
                if key.contains("mcp") && value.is_object() {
                    if let Some(server_obj) = value.as_object() {
                        servers.push(McpServerConfig {
                            name: key.clone(),
                            command: server_obj.get("command").and_then(|v| v.as_str()).map(String::from),
                            args: Vec::new(),
                            env: HashMap::new(),
                            cwd: None,
                            server_type: ServerType::Custom("vscode-setting".to_string()),
                            metadata: ServerMetadata {
                                description: Some(format!("VS Code MCP setting: {}", key)),
                                version: None,
                                author: None,
                                capabilities: Vec::new(),
                                enabled: true,
                                source: ConfigSource::MainConfig,
                            },
                        });
                    }
                }
            }
        }

        Ok(servers)
    }

    /// Generic MCP server extraction for unknown applications
    fn extract_generic_servers(&self, config: &JsonValue) -> Result<Vec<McpServerConfig>> {
        let mut servers = Vec::new();

        // Look for common MCP server configuration patterns
        let possible_keys = ["mcpServers", "mcp_servers", "mcp", "servers", "modelContextProtocol"];
        
        for key in &possible_keys {
            if let Some(servers_config) = config.get(key) {
                if let Some(servers_obj) = servers_config.as_object() {
                    for (name, server_config) in servers_obj {
                        if let Some(server_obj) = server_config.as_object() {
                            servers.push(McpServerConfig {
                                name: name.clone(),
                                command: server_obj.get("command").and_then(|v| v.as_str()).map(String::from),
                                args: server_obj.get("args")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                    .unwrap_or_default(),
                                env: HashMap::new(),
                                cwd: None,
                                server_type: ServerType::Custom("generic".to_string()),
                                metadata: ServerMetadata {
                                    description: server_obj.get("description").and_then(|v| v.as_str()).map(String::from),
                                    version: None,
                                    author: None,
                                    capabilities: Vec::new(),
                                    enabled: server_obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                                    source: ConfigSource::MainConfig,
                                },
                            });
                        }
                    }
                }
                break; // Found servers, no need to check other keys
            }
        }

        Ok(servers)
    }

    /// Expand path with ~ to home directory
    fn expand_path(&self, path: &str) -> Result<PathBuf> {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                Ok(home.join(stripped))
            } else {
                Err(anyhow::anyhow!("Could not find home directory"))
            }
        } else if path == "~" {
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))
        } else {
            Ok(PathBuf::from(path))
        }
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create default ConfigValidator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::profiles::{ApplicationCategory, ApplicationMetadata, DetectionStrategy, DetectionMethod};
    use tempfile::tempdir;
    use std::fs;

    fn create_test_application() -> ApplicationProfile {
        ApplicationProfile {
            id: "test-app".to_string(),
            name: "Test Application".to_string(),
            bundle_id: "com.test.app".to_string(),
            config_path: "~/test/config.json".to_string(),
            alt_config_paths: vec!["~/.config/test/config.json".to_string()],
            config_format: ConfigFormat::Json,
            executable_paths: vec!["/Applications/Test.app".to_string()],
            alt_executable_paths: vec![],
            detection_strategy: DetectionStrategy {
                use_bundle_lookup: false,
                use_executable_check: false,
                use_config_check: true,
                use_spotlight: false,
                priority_order: vec![DetectionMethod::ConfigCheck],
            },
            metadata: ApplicationMetadata {
                version: None,
                developer: "Test Developer".to_string(),
                category: ApplicationCategory::Other("Test".to_string()),
                mcp_version: "1.0".to_string(),
                notes: None,
                requires_permissions: false,
            },
        }
    }

    #[tokio::test]
    async fn test_config_validator_creation() {
        let validator = ConfigValidator::new();
        assert!(validator.is_ok());
    }

    #[tokio::test]
    async fn test_validate_missing_config() {
        let validator = ConfigValidator::new().unwrap();
        let app = create_test_application();
        
        let result = validator.validate_application_config(&app).await;
        assert!(result.is_ok());
        
        let validation_result = result.unwrap();
        assert!(!validation_result.is_valid);
        assert!(validation_result.config_path.is_none());
        assert!(!validation_result.messages.is_empty());
        assert_eq!(validation_result.messages[0].level, MessageLevel::Warning);
        assert!(validation_result.messages[0].message.contains("Configuration file not found"));
    }

    #[tokio::test]
    async fn test_validate_claude_desktop_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("claude_desktop_config.json");
        
        let config_content = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
                    "env": {
                        "NODE_ENV": "production"
                    }
                },
                "git": {
                    "command": "python",
                    "args": ["-m", "mcp_git"],
                    "description": "Git repository server"
                }
            }
        }"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let mut app = create_test_application();
        app.id = "claude-desktop".to_string();
        app.config_path = config_path.to_string_lossy().to_string();
        
        let validator = ConfigValidator::new().unwrap();
        let result = validator.validate_application_config(&app).await.unwrap();
        
        assert!(result.is_valid);
        assert_eq!(result.mcp_servers.len(), 2);
        assert_eq!(result.mcp_servers[0].name, "filesystem");
        assert_eq!(result.mcp_servers[0].command, Some("npx".to_string()));
        assert_eq!(result.mcp_servers[0].args, vec!["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]);
        assert_eq!(result.mcp_servers[1].name, "git");
        assert_eq!(result.mcp_servers[1].command, Some("python".to_string()));
    }

    #[tokio::test]
    async fn test_parse_yaml_config() {
        let validator = ConfigValidator::new().unwrap();
        let yaml_content = r#"
mcp:
  servers:
    test-server:
      command: "python"
      args: ["-m", "test_server"]
      enabled: true
"#;
        
        let parsed = validator.parse_config_content(yaml_content, &ConfigFormat::Yaml);
        assert!(parsed.is_ok());
        
        let config = parsed.unwrap();
        assert!(config.get("mcp").is_some());
    }

    #[tokio::test]
    async fn test_parse_toml_config() {
        let validator = ConfigValidator::new().unwrap();
        let toml_content = r#"
[mcp.servers.test-server]
command = "python"
args = ["-m", "test_server"]
enabled = true
"#;
        
        let parsed = validator.parse_config_content(toml_content, &ConfigFormat::Toml);
        assert!(parsed.is_ok());
        
        let config = parsed.unwrap();
        assert!(config.get("mcp").is_some());
    }

    #[tokio::test]
    async fn test_validation_summary() {
        let validator = ConfigValidator::new().unwrap();
        let app1 = create_test_application();
        let mut app2 = create_test_application();
        app2.id = "app2".to_string();
        
        let results = vec![
            ConfigValidationResult {
                application: app1,
                is_valid: true,
                config_path: Some(PathBuf::from("/test/config.json")),
                detected_format: Some(ConfigFormat::Json),
                mcp_servers: vec![McpServerConfig {
                    name: "test".to_string(),
                    command: Some("test".to_string()),
                    args: vec![],
                    env: HashMap::new(),
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
                }],
                messages: vec![],
                raw_config: None,
                validated_at: chrono::Utc::now(),
            },
            ConfigValidationResult {
                application: app2,
                is_valid: false,
                config_path: None,
                detected_format: None,
                mcp_servers: vec![],
                messages: vec![],
                raw_config: None,
                validated_at: chrono::Utc::now(),
            },
        ];
        
        let summary = validator.get_validation_summary(&results);
        assert_eq!(summary.total_applications, 2);
        assert_eq!(summary.valid_configs, 1);
        assert_eq!(summary.invalid_configs, 1);
        assert_eq!(summary.total_servers, 1);
        assert_eq!(summary.applications_with_servers, 1);
        assert_eq!(summary.applications_without_servers, 1);
        assert_eq!(summary.format_breakdown.get("JSON"), Some(&1));
    }

    #[test]
    fn test_server_type_serialization() {
        let server_types = vec![
            ServerType::Stdio,
            ServerType::Sse { url: "http://example.com".to_string() },
            ServerType::WebSocket { url: "ws://example.com".to_string() },
            ServerType::Http { base_url: "https://api.example.com".to_string() },
            ServerType::Custom("custom".to_string()),
        ];
        
        for server_type in server_types {
            let serialized = serde_json::to_string(&server_type).unwrap();
            let deserialized: ServerType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(server_type, deserialized);
        }
    }

    #[test]
    fn test_validation_message_levels() {
        let levels = vec![
            MessageLevel::Info,
            MessageLevel::Warning,
            MessageLevel::Error,
            MessageLevel::Critical,
        ];
        
        for level in levels {
            let serialized = serde_json::to_string(&level).unwrap();
            let deserialized: MessageLevel = serde_json::from_str(&serialized).unwrap();
            assert_eq!(level, deserialized);
        }
    }

    #[tokio::test]
    async fn test_multiple_config_validation() {
        let validator = ConfigValidator::new().unwrap();
        let apps = vec![create_test_application()];
        
        let results = validator.validate_multiple_configs(&apps).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_zed_config_extraction() {
        let validator = ConfigValidator::new().unwrap();
        let zed_config = serde_json::json!({
            "mcp": {
                "servers": {
                    "filesystem": {
                        "command": "npx",
                        "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
                        "enabled": true,
                        "description": "File system access"
                    }
                }
            }
        });
        
        let servers = validator.extract_zed_servers(&zed_config).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "filesystem");
        assert_eq!(servers[0].command, Some("npx".to_string()));
        assert!(servers[0].metadata.enabled);
    }

    #[tokio::test]
    async fn test_generic_config_extraction() {
        let validator = ConfigValidator::new().unwrap();
        let generic_config = serde_json::json!({
            "mcpServers": {
                "custom-server": {
                    "command": "custom-mcp-server",
                    "args": ["--port", "8080"],
                    "description": "Custom MCP server"
                }
            }
        });
        
        let servers = validator.extract_generic_servers(&generic_config).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "custom-server");
        assert_eq!(servers[0].command, Some("custom-mcp-server".to_string()));
        assert_eq!(servers[0].args, vec!["--port", "8080"]);
    }

    #[test]
    fn test_expand_path() {
        let validator = ConfigValidator::new().unwrap();
        
        // Test home directory expansion
        let expanded = validator.expand_path("~/test/config.json").unwrap();
        assert!(!expanded.to_string_lossy().starts_with('~'));
        
        // Test absolute path
        let absolute = validator.expand_path("/absolute/path").unwrap();
        assert_eq!(absolute, PathBuf::from("/absolute/path"));
    }

    #[tokio::test]
    async fn test_invalid_json_config() {
        let validator = ConfigValidator::new().unwrap();
        let invalid_json = "{ invalid json content }";
        
        let result = validator.parse_config_content(invalid_json, &ConfigFormat::Json);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_config_source_detection() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let config_content = r#"{
            "mcpServers": {
                "test": {
                    "command": "test-command"
                }
            }
        }"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let mut app = create_test_application();
        app.id = "claude-desktop".to_string();
        app.config_path = config_path.to_string_lossy().to_string();
        
        let validator = ConfigValidator::new().unwrap();
        let result = validator.validate_application_config(&app).await.unwrap();
        
        assert!(result.is_valid);
        assert_eq!(result.mcp_servers[0].metadata.source, ConfigSource::MainConfig);
    }
}
