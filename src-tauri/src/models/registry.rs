use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{ComplianceModel, ComplianceResult};
use crate::models::audit::AuditInfo;
use crate::models::server::ServerConfig;
use crate::models::validation::{Validatable, ValidationContext, Validators};

/// Registry of available and installed MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRegistry {
    /// Unique identifier
    pub id: Uuid,
    
    /// Available servers (from registry/marketplace)
    pub available_servers: Vec<ServerRegistryEntry>,
    
    /// Installed servers (local configurations)
    pub installed_servers: Vec<ServerConfig>,
    
    /// Last scan/update time
    pub last_scan: Option<DateTime<Utc>>,
    
    /// Registry metadata
    pub metadata: RegistryMetadata,
    
    /// Audit information
    pub audit_info: AuditInfo,
}

/// Entry in the server registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRegistryEntry {
    /// Unique identifier
    pub id: Uuid,
    
    /// Server name
    pub name: String,
    
    /// Server description
    pub description: String,
    
    /// Server version
    pub version: String,
    
    /// Author/maintainer
    pub author: String,
    
    /// Installation source
    pub source: InstallationSource,
    
    /// Server category/tags
    pub tags: Vec<String>,
    
    /// Installation instructions
    pub installation: InstallationInfo,
    
    /// Server capabilities
    pub capabilities: Vec<String>,
    
    /// Compatibility information
    pub compatibility: CompatibilityInfo,
    
    /// Download/usage statistics
    pub statistics: ServerStatistics,
    
    /// Whether this server is verified/trusted
    pub verified: bool,
    
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Installation source for servers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallationSource {
    /// NPM package
    Npm { package_name: String },
    /// GitHub repository
    GitHub { repository: String, branch: Option<String> },
    /// Local file/directory
    Local { path: String },
    /// Docker image
    Docker { image: String, tag: Option<String> },
    /// Custom URL
    Url { url: String },
}

/// Installation information and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    /// Installation command or script
    pub command: Option<String>,
    
    /// Required dependencies
    pub dependencies: Vec<String>,
    
    /// System requirements
    pub system_requirements: SystemRequirements,
    
    /// Configuration template
    pub config_template: Option<serde_json::Value>,
    
    /// Post-installation steps
    pub post_install_steps: Vec<String>,
}

/// System requirements for server installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    /// Supported operating systems
    pub os: Vec<String>,
    
    /// Minimum Node.js version (if applicable)
    pub node_version: Option<String>,
    
    /// Minimum Python version (if applicable)
    pub python_version: Option<String>,
    
    /// Required system tools
    pub tools: Vec<String>,
    
    /// Minimum RAM in MB
    pub min_ram_mb: Option<u32>,
    
    /// Minimum disk space in MB
    pub min_disk_mb: Option<u32>,
}

/// Compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    /// Supported MCP protocol versions
    pub mcp_versions: Vec<String>,
    
    /// Compatible client applications
    pub compatible_clients: Vec<String>,
    
    /// Known issues or limitations
    pub known_issues: Vec<String>,
    
    /// Last tested date
    pub last_tested: Option<DateTime<Utc>>,
}

/// Server usage and download statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerStatistics {
    /// Download count
    pub downloads: u64,
    
    /// Active installations
    pub active_installations: u64,
    
    /// User rating (1-5)
    pub rating: Option<f32>,
    
    /// Number of ratings
    pub rating_count: u32,
    
    /// Last 30 days downloads
    pub recent_downloads: u64,
}

/// Registry metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    /// Registry version
    pub version: String,
    
    /// Registry sources/URLs
    pub sources: Vec<String>,
    
    /// Update interval in hours
    pub update_interval_hours: u32,
    
    /// Cache settings
    pub cache_settings: CacheSettings,
}

/// Cache configuration for registry data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    /// Cache TTL in seconds
    pub ttl_seconds: u32,
    
    /// Maximum cache size in MB
    pub max_size_mb: u32,
    
    /// Cache directory
    pub cache_dir: String,
}

impl Default for ServerRegistry {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            available_servers: Vec::new(),
            installed_servers: Vec::new(),
            last_scan: None,
            metadata: RegistryMetadata::default(),
            audit_info: AuditInfo::new("system".to_string()),
        }
    }
}

impl Default for RegistryMetadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            sources: vec![
                "https://registry.mcp-control.dev".to_string(),
                "https://github.com/mcp-servers/registry".to_string(),
            ],
            update_interval_hours: 24, // Daily updates
            cache_settings: CacheSettings::default(),
        }
    }
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            ttl_seconds: 86400, // 24 hours
            max_size_mb: 100,   // 100 MB
            cache_dir: "~/.mcp-control/cache".to_string(),
        }
    }
}

impl Default for SystemRequirements {
    fn default() -> Self {
        Self {
            os: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            node_version: None,
            python_version: None,
            tools: Vec::new(),
            min_ram_mb: None,
            min_disk_mb: None,
        }
    }
}

impl ServerRegistry {
    /// Create a new server registry
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a server to the registry
    pub fn add_available_server(&mut self, server: ServerRegistryEntry) {
        self.available_servers.push(server);
        self.audit_info.update("system".to_string());
    }
    
    /// Install a server from the registry
    pub fn install_server(&mut self, server_id: Uuid, config: ServerConfig) -> Result<(), String> {
        // Check if server exists in available servers
        if !self.available_servers.iter().any(|s| s.id == server_id) {
            return Err("Server not found in registry".to_string());
        }
        
        // Check if already installed
        if self.installed_servers.iter().any(|s| s.id == config.id) {
            return Err("Server already installed".to_string());
        }
        
        self.installed_servers.push(config);
        self.audit_info.update("system".to_string());
        Ok(())
    }
}

impl Validatable for ServerRegistry {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate available servers
        ctx.enter_field("available_servers");
        for (i, server) in self.available_servers.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            server.validate_with_context(ctx);
            ctx.exit_field();
        }
        ctx.exit_field();
        
        // Validate installed servers
        ctx.enter_field("installed_servers");
        for (i, server) in self.installed_servers.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            server.validate_with_context(ctx);
            ctx.exit_field();
        }
        ctx.exit_field();
        
        // Validate metadata
        ctx.enter_field("metadata");
        self.metadata.validate_with_context(ctx);
        ctx.exit_field();
    }
}

impl Validatable for ServerRegistryEntry {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate name
        ctx.enter_field("name");
        if let Err(e) = Validators::not_empty(&self.name, "name") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate description
        ctx.enter_field("description");
        if let Err(e) = Validators::not_empty(&self.description, "description") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate version
        ctx.enter_field("version");
        if let Err(e) = Validators::not_empty(&self.version, "version") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate author
        ctx.enter_field("author");
        if let Err(e) = Validators::not_empty(&self.author, "author") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate installation info
        ctx.enter_field("installation");
        self.installation.validate_with_context(ctx);
        ctx.exit_field();
    }
}

impl Validatable for InstallationInfo {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate system requirements
        ctx.enter_field("system_requirements");
        self.system_requirements.validate_with_context(ctx);
        ctx.exit_field();
    }
}

impl Validatable for SystemRequirements {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate OS list is not empty
        ctx.enter_field("os");
        if let Err(e) = Validators::not_empty_collection(&self.os, "os") {
            ctx.add_error(e);
        }
        ctx.exit_field();
    }
}

impl Validatable for RegistryMetadata {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate version
        ctx.enter_field("version");
        if let Err(e) = Validators::not_empty(&self.version, "version") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate sources
        ctx.enter_field("sources");
        if let Err(e) = Validators::not_empty_collection(&self.sources, "sources") {
            ctx.add_error(e);
        }
        for (i, source) in self.sources.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            if let Err(e) = Validators::url(source, "source") {
                ctx.add_error(e);
            }
            ctx.exit_field();
        }
        ctx.exit_field();
        
        // Validate update interval
        ctx.enter_field("update_interval_hours");
        if let Err(e) = Validators::numeric_range(
            self.update_interval_hours as f64,
            "update_interval_hours",
            Some(1.0),   // Minimum 1 hour
            Some(168.0), // Maximum 1 week
        ) {
            ctx.add_error(e);
        }
        ctx.exit_field();
    }
}

impl ComplianceModel for ServerRegistry {
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
            entity_type: "ServerRegistry".to_string(),
            entity_id: self.id.to_string(),
            action: "created".to_string(),
            user_id: self.audit_info.created_by.clone(),
            timestamp: self.audit_info.created_at,
            details: serde_json::json!({
                "available_server_count": self.available_servers.len(),
                "installed_server_count": self.installed_servers.len(),
                "registry_version": self.metadata.version
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
    fn test_server_registry_creation() {
        let registry = ServerRegistry::new();
        
        assert!(registry.available_servers.is_empty());
        assert!(registry.installed_servers.is_empty());
        assert!(registry.last_scan.is_none());
    }
    
    #[test]
    fn test_server_registry_validation() {
        let registry = ServerRegistry::new();
        assert!(registry.validate().is_ok());
    }
}
