use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{ComplianceModel, ComplianceResult, DataClassification};
use crate::models::security::{AccessControl, EncryptionSettings};
use crate::models::audit::AuditInfo;
use crate::models::validation::{Validatable, ValidationContext, Validators, ComplianceValidators};

/// Represents the status of an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    /// Server is running and responsive
    Active,
    /// Server is stopped
    Inactive,
    /// Server is starting up
    Starting,
    /// Server is shutting down
    Stopping,
    /// Server encountered an error
    Error(String),
    /// Server status is unknown
    Unknown,
}

/// Represents the type of MCP server connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    /// Local process connection
    Process {
        command: String,
        args: Vec<String>,
        working_directory: Option<String>,
    },
    /// Network connection (HTTP/WebSocket)
    Network {
        url: String,
        headers: HashMap<String, String>,
        timeout_ms: u64,
    },
    /// Docker container connection
    Docker {
        image: String,
        container_name: String,
        ports: HashMap<u16, u16>,
        environment: HashMap<String, String>,
    },
}

/// Configuration for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Unique identifier for the server
    pub id: Uuid,
    
    /// Human-readable name for the server
    pub name: String,
    
    /// Optional description of the server's purpose
    pub description: Option<String>,
    
    /// Connection configuration
    pub connection: ConnectionType,
    
    /// Current status of the server
    pub status: ServerStatus,
    
    /// Whether the server should auto-start
    pub auto_start: bool,
    
    /// Maximum restart attempts on failure
    pub max_restart_attempts: u32,
    
    /// Current restart attempt count
    pub restart_count: u32,
    
    /// Server capabilities and metadata
    pub capabilities: ServerCapabilities,
    
    /// Environment variables for the server
    pub environment: HashMap<String, String>,
    
    /// Security and access control settings
    pub access_control: AccessControl,
    
    /// Data classification for this server's data
    pub data_classification: DataClassification,
    
    /// Encryption settings for server communication
    pub encryption: EncryptionSettings,
    
    /// Audit information
    pub audit_info: AuditInfo,
    
    /// Last successful health check
    pub last_health_check: Option<DateTime<Utc>>,
    
    /// Health check interval in seconds
    pub health_check_interval: u64,
    
    /// Server-specific configuration options
    pub custom_config: HashMap<String, serde_json::Value>,
}

/// Capabilities and metadata for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tools provided by this server
    pub tools: Vec<ToolInfo>,
    
    /// Resources provided by this server
    pub resources: Vec<ResourceInfo>,
    
    /// Prompts provided by this server
    pub prompts: Vec<PromptInfo>,
    
    /// Server version
    pub version: Option<String>,
    
    /// Server protocol version
    pub protocol_version: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Information about a tool provided by an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    
    /// Tool description
    pub description: Option<String>,
    
    /// JSON schema for tool parameters
    pub input_schema: serde_json::Value,
    
    /// Data classification for tool usage
    pub data_classification: DataClassification,
    
    /// Whether this tool requires special permissions
    pub requires_approval: bool,
    
    /// Usage statistics
    pub usage_count: u64,
    
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
}

/// Information about a resource provided by an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Resource URI
    pub uri: String,
    
    /// Resource name
    pub name: String,
    
    /// Resource description
    pub description: Option<String>,
    
    /// MIME type of the resource
    pub mime_type: Option<String>,
    
    /// Data classification for this resource
    pub data_classification: DataClassification,
    
    /// Access count
    pub access_count: u64,
    
    /// Last accessed timestamp
    pub last_accessed: Option<DateTime<Utc>>,
}

/// Information about a prompt provided by an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInfo {
    /// Prompt name
    pub name: String,
    
    /// Prompt description
    pub description: Option<String>,
    
    /// Prompt arguments schema
    pub arguments: Vec<PromptArgument>,
    
    /// Data classification for prompt usage
    pub data_classification: DataClassification,
    
    /// Usage count
    pub usage_count: u64,
    
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
}

/// Argument definition for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,
    
    /// Argument description
    pub description: Option<String>,
    
    /// Whether the argument is required
    pub required: bool,
    
    /// Default value if any
    pub default: Option<serde_json::Value>,
}

/// Server health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Server ID
    pub server_id: Uuid,
    
    /// Timestamp of the health check
    pub timestamp: DateTime<Utc>,
    
    /// Whether the server is healthy
    pub is_healthy: bool,
    
    /// Response time in milliseconds
    pub response_time_ms: u64,
    
    /// Error message if unhealthy
    pub error_message: Option<String>,
    
    /// Additional health metrics
    pub metrics: HashMap<String, serde_json::Value>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            description: None,
            connection: ConnectionType::Process {
                command: String::new(),
                args: Vec::new(),
                working_directory: None,
            },
            status: ServerStatus::Inactive,
            auto_start: false,
            max_restart_attempts: 3,
            restart_count: 0,
            capabilities: ServerCapabilities::default(),
            environment: HashMap::new(),
            access_control: AccessControl::new("system"),
            data_classification: DataClassification::Internal,
            encryption: EncryptionSettings::new("default-key"),
            audit_info: AuditInfo::new("system".to_string()),
            last_health_check: None,
            health_check_interval: 30, // 30 seconds default
            custom_config: HashMap::new(),
        }
    }
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            tools: Vec::new(),
            resources: Vec::new(),
            prompts: Vec::new(),
            version: None,
            protocol_version: "2024-11-05".to_string(), // Current MCP protocol version
            metadata: HashMap::new(),
        }
    }
}

impl Validatable for ServerConfig {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate server name
        ctx.enter_field("name");
        if let Err(e) = Validators::not_empty(&self.name, "name") {
            ctx.add_error(e);
        }
        if let Err(e) = Validators::string_length(&self.name, "name", Some(1), Some(255)) {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate connection configuration
        ctx.enter_field("connection");
        match &self.connection {
            ConnectionType::Process { command, args: _, working_directory } => {
                if let Err(e) = Validators::not_empty(command, "command") {
                    ctx.add_error(e);
                }
                if let Some(wd) = working_directory {
                    if let Err(e) = Validators::file_path(wd, "working_directory") {
                        ctx.add_error(e);
                    }
                }
            }
            ConnectionType::Network { url, timeout_ms, .. } => {
                if let Err(e) = Validators::url(url, "url") {
                    ctx.add_error(e);
                }
                if let Err(e) = Validators::numeric_range(*timeout_ms as f64, "timeout_ms", Some(1000.0), Some(300000.0)) {
                    ctx.add_error(e);
                }
            }
            ConnectionType::Docker { image, container_name, .. } => {
                if let Err(e) = Validators::not_empty(image, "image") {
                    ctx.add_error(e);
                }
                if let Err(e) = Validators::not_empty(container_name, "container_name") {
                    ctx.add_error(e);
                }
                if let Err(e) = Validators::alphanumeric(container_name, "container_name") {
                    ctx.add_error(e);
                }
            }
        }
        ctx.exit_field();
        
        // Validate restart attempts
        ctx.enter_field("max_restart_attempts");
        if let Err(e) = Validators::numeric_range(self.max_restart_attempts as f64, "max_restart_attempts", Some(0.0), Some(10.0)) {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate health check interval
        ctx.enter_field("health_check_interval");
        if let Err(e) = Validators::timeout_seconds(self.health_check_interval, "health_check_interval") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate access control if present
        if ctx.options.validate_security {
            ctx.enter_field("access_control");
            self.access_control.validate_with_context(ctx);
            ctx.exit_field();
        }
        
        // Validate compliance requirements
        if ctx.options.validate_compliance {
            ctx.enter_field("audit_info");
            if let Err(e) = ComplianceValidators::audit_trail_complete(&self.audit_info.created_by, &self.audit_info.updated_by, "audit_info") {
                ctx.add_error(e);
            }
            ctx.exit_field();
            
            // Validate encryption for sensitive data classifications
            if matches!(self.data_classification, DataClassification::Confidential | DataClassification::Restricted) {
                ctx.enter_field("encryption");
                self.encryption.validate_with_context(ctx);
                ctx.exit_field();
            }
        }
    }
}

impl Validatable for ServerCapabilities {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate protocol version
        ctx.enter_field("protocol_version");
        if let Err(e) = Validators::not_empty(&self.protocol_version, "protocol_version") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate tools
        ctx.enter_field("tools");
        for (i, tool) in self.tools.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            tool.validate_with_context(ctx);
            ctx.exit_field();
        }
        ctx.exit_field();
        
        // Validate resources
        ctx.enter_field("resources");
        for (i, resource) in self.resources.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            resource.validate_with_context(ctx);
            ctx.exit_field();
        }
        ctx.exit_field();
        
        // Validate prompts
        ctx.enter_field("prompts");
        for (i, prompt) in self.prompts.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            prompt.validate_with_context(ctx);
            ctx.exit_field();
        }
        ctx.exit_field();
    }
}

impl Validatable for ToolInfo {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate tool name
        ctx.enter_field("name");
        if let Err(e) = Validators::not_empty(&self.name, "name") {
            ctx.add_error(e);
        }
        if let Err(e) = Validators::alphanumeric(&self.name, "name") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate usage count
        ctx.enter_field("usage_count");
        if let Err(e) = Validators::numeric_range(self.usage_count as f64, "usage_count", Some(0.0), None) {
            ctx.add_error(e);
        }
        ctx.exit_field();
    }
}

impl Validatable for ResourceInfo {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate resource URI
        ctx.enter_field("uri");
        if let Err(e) = Validators::not_empty(&self.uri, "uri") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate resource name
        ctx.enter_field("name");
        if let Err(e) = Validators::not_empty(&self.name, "name") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate access count
        ctx.enter_field("access_count");
        if let Err(e) = Validators::numeric_range(self.access_count as f64, "access_count", Some(0.0), None) {
            ctx.add_error(e);
        }
        ctx.exit_field();
    }
}

impl Validatable for PromptInfo {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate prompt name
        ctx.enter_field("name");
        if let Err(e) = Validators::not_empty(&self.name, "name") {
            ctx.add_error(e);
        }
        if let Err(e) = Validators::alphanumeric(&self.name, "name") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate arguments
        ctx.enter_field("arguments");
        for (i, arg) in self.arguments.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            arg.validate_with_context(ctx);
            ctx.exit_field();
        }
        ctx.exit_field();
        
        // Validate usage count
        ctx.enter_field("usage_count");
        if let Err(e) = Validators::numeric_range(self.usage_count as f64, "usage_count", Some(0.0), None) {
            ctx.add_error(e);
        }
        ctx.exit_field();
    }
}

impl Validatable for PromptArgument {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate argument name
        ctx.enter_field("name");
        if let Err(e) = Validators::not_empty(&self.name, "name") {
            ctx.add_error(e);
        }
        if let Err(e) = Validators::alphanumeric(&self.name, "name") {
            ctx.add_error(e);
        }
        ctx.exit_field();
    }
}

impl Validatable for HealthCheckResult {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate response time
        ctx.enter_field("response_time_ms");
        if let Err(e) = Validators::numeric_range(self.response_time_ms as f64, "response_time_ms", Some(0.0), Some(300000.0)) {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate that error message is present if not healthy
        if !self.is_healthy && self.error_message.is_none() {
            ctx.enter_field("error_message");
            ctx.add_error(crate::models::validation::ValidationError::RequiredField("error_message".to_string()));
            ctx.exit_field();
        }
    }
}

impl ComplianceModel for ServerConfig {
    fn validate_compliance(&self) -> Result<(), Vec<String>> {
        match self.validate() {
            Ok(_) => Ok(()),
            Err(errors) => Err(errors.into_iter().map(|e| e.to_string()).collect()),
        }
    }
    
    fn get_compliance_status(&self) -> ComplianceResult {
        match self.validate_compliance() {
            Ok(_) => ComplianceResult::Compliant,
            Err(_) => ComplianceResult::NonCompliant,
        }
    }
    
    fn get_audit_trail(&self) -> Vec<crate::models::audit::AuditEntry> {
        // This would typically query an audit log database
        // For now, return basic audit info
        vec![crate::models::audit::AuditEntry {
            id: Uuid::new_v4(),
            entity_type: "ServerConfig".to_string(),
            entity_id: self.id.to_string(),
            action: "created".to_string(),
            user_id: self.audit_info.created_by.clone(),
            timestamp: self.audit_info.created_at,
            details: serde_json::json!({
                "name": self.name,
                "connection_type": match &self.connection {
                    ConnectionType::Process { .. } => "process",
                    ConnectionType::Network { .. } => "network",
                    ConnectionType::Docker { .. } => "docker",
                },
                "data_classification": self.data_classification
            }),
            ip_address: None,
            user_agent: None,
        }]
    }
}

impl ServerConfig {
    /// Create a new server configuration
    pub fn new(name: String, connection: ConnectionType, created_by: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            connection,
            audit_info: AuditInfo::new(created_by),
            ..Default::default()
        }
    }
    
    /// Update server status
    pub fn update_status(&mut self, status: ServerStatus, updated_by: String) {
        self.status = status;
        self.audit_info.update(updated_by);
    }
    
    /// Record a health check result
    pub fn record_health_check(&mut self, result: HealthCheckResult) {
        self.last_health_check = Some(result.timestamp);
        if result.is_healthy {
            self.restart_count = 0; // Reset restart count on successful health check
        }
    }
    
    /// Check if server should be restarted
    pub fn should_restart(&self) -> bool {
        matches!(self.status, ServerStatus::Error(_)) && 
        self.restart_count < self.max_restart_attempts
    }
    
    /// Increment restart count
    pub fn increment_restart_count(&mut self) {
        self.restart_count += 1;
    }
    
    /// Get server display name with status
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.status_display())
    }
    
    /// Get human-readable status
    pub fn status_display(&self) -> String {
        match &self.status {
            ServerStatus::Active => "Active".to_string(),
            ServerStatus::Inactive => "Inactive".to_string(),
            ServerStatus::Starting => "Starting".to_string(),
            ServerStatus::Stopping => "Stopping".to_string(),
            ServerStatus::Error(msg) => format!("Error: {}", msg),
            ServerStatus::Unknown => "Unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_config_creation() {
        let config = ServerConfig::new(
            "Test Server".to_string(),
            ConnectionType::Process {
                command: "node".to_string(),
                args: vec!["server.js".to_string()],
                working_directory: None,
            },
            "test_user".to_string(),
        );
        
        assert_eq!(config.name, "Test Server");
        assert_eq!(config.status, ServerStatus::Inactive);
        assert_eq!(config.restart_count, 0);
    }
    
    #[test]
    fn test_compliance_validation() {
        let mut config = ServerConfig::default();
        
        // Should fail validation with empty name
        assert!(config.validate_compliance().is_err());
        
        // Should pass with valid configuration
        config.name = "Valid Server".to_string();
        config.connection = ConnectionType::Process {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            working_directory: None,
        };
        
        assert!(config.validate_compliance().is_ok());
    }
    
    #[test]
    fn test_restart_logic() {
        let mut config = ServerConfig { 
            max_restart_attempts: 3, 
            ..Default::default() 
        };
        
        // Should not restart when inactive
        assert!(!config.should_restart());
        
        // Should restart when in error state and under limit
        config.status = ServerStatus::Error("Test error".to_string());
        assert!(config.should_restart());
        
        // Should not restart when over limit
        config.restart_count = 3;
        assert!(!config.should_restart());
    }
}
