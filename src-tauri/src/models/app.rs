use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use crate::models::{ComplianceModel, ComplianceResult, DataClassification};
use crate::models::audit::AuditInfo;
use crate::models::security::AccessControl;
use crate::models::validation::{Validatable, ValidationContext, Validators};

/// Application profile for MCP client applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationProfile {
    /// Unique identifier
    pub id: Uuid,
    
    /// Application name (e.g., "Claude Desktop", "Cursor")
    pub name: String,
    
    /// Configuration file path
    pub config_path: String,
    
    /// Configuration format (json, yaml, etc.)
    pub format: String,
    
    /// Adapter type for this application
    pub adapter: String,
    
    /// Whether the application was detected
    pub detected: bool,
    
    /// Last synchronization time
    pub last_sync: Option<DateTime<Utc>>,
    
    /// Application bundle ID (macOS)
    pub bundle_id: Option<String>,
    
    /// Application version
    pub version: Option<String>,
    
    /// Whether the application is installed
    pub installed: bool,
    
    /// MCP servers configured for this application
    pub mcp_servers: Vec<AppServerConfig>,
    
    /// Last detection time
    pub last_detected: Option<DateTime<Utc>>,
    
    /// Data classification
    pub data_classification: DataClassification,
    
    /// Access control
    pub access_control: AccessControl,
    
    /// Audit information
    pub audit_info: AuditInfo,
}

/// Server configuration specific to an application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppServerConfig {
    /// Server ID reference
    pub server_id: Uuid,
    
    /// Whether the server is enabled for this app
    pub enabled: bool,
    
    /// App-specific parameter overrides
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Synchronization status
    pub sync_status: SyncStatus,
    
    /// Audit information
    pub audit_info: AuditInfo,
}

/// Synchronization status for app server configurations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    /// Configuration is synchronized
    Synced,
    /// Synchronization is pending
    Pending,
    /// Synchronization failed
    Error(String),
    /// Never synchronized
    Never,
}

impl ApplicationProfile {
    /// Create a new application profile
    pub fn new(name: String, config_path: String, created_by: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            config_path,
            format: "json".to_string(),
            adapter: "default".to_string(),
            detected: false,
            last_sync: None,
            bundle_id: None,
            version: None,
            installed: false,
            mcp_servers: Vec::new(),
            last_detected: None,
            data_classification: DataClassification::Internal,
            access_control: AccessControl::new(&created_by),
            audit_info: AuditInfo::new(created_by),
        }
    }
}

impl Validatable for ApplicationProfile {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate name
        ctx.enter_field("name");
        if let Err(e) = Validators::not_empty(&self.name, "name") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate config path
        ctx.enter_field("config_path");
        if let Err(e) = Validators::not_empty(&self.config_path, "config_path") {
            ctx.add_error(e);
        }
        if let Err(e) = Validators::file_path(&self.config_path, "config_path") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate format
        ctx.enter_field("format");
        if let Err(e) = Validators::in_set(&self.format, "format", &["json", "yaml", "toml"]) {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate MCP servers
        ctx.enter_field("mcp_servers");
        for (i, server_config) in self.mcp_servers.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            server_config.validate_with_context(ctx);
            ctx.exit_field();
        }
        ctx.exit_field();
        
        // Validate access control
        ctx.enter_field("access_control");
        self.access_control.validate_with_context(ctx);
        ctx.exit_field();
    }
}

impl Validatable for AppServerConfig {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Server ID is validated by UUID type
        
        // Validate parameters
        ctx.enter_field("parameters");
        if let Err(e) = Validators::collection_size(&self.parameters.keys().collect::<Vec<_>>(), "parameters", None, Some(100)) {
            ctx.add_error(e);
        }
        ctx.exit_field();
    }
}

impl ComplianceModel for ApplicationProfile {
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
        vec![crate::models::audit::AuditEntry {
            id: Uuid::new_v4(),
            entity_type: "ApplicationProfile".to_string(),
            entity_id: self.id.to_string(),
            action: "created".to_string(),
            user_id: self.audit_info.created_by.clone(),
            timestamp: self.audit_info.created_at,
            details: serde_json::json!({
                "name": self.name,
                "config_path": self.config_path,
                "detected": self.detected,
                "installed": self.installed,
                "server_count": self.mcp_servers.len()
            }),
            ip_address: None,
            user_agent: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_application_profile_creation() {
        let profile = ApplicationProfile::new(
            "Test App".to_string(),
            "/path/to/config.json".to_string(),
            "test_user".to_string(),
        );
        
        assert_eq!(profile.name, "Test App");
        assert_eq!(profile.config_path, "/path/to/config.json");
        assert!(!profile.detected);
        assert!(!profile.installed);
    }
    
    #[test]
    fn test_application_profile_validation() {
        let profile = ApplicationProfile::new(
            "Valid App".to_string(),
            "/valid/path/config.json".to_string(),
            "test_user".to_string(),
        );
        
        assert!(profile.validate().is_ok());
    }
}
