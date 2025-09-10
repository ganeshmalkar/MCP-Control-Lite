use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use serde::{Deserialize, Serialize};
use serde_json;
use toml;
use anyhow::{Result, Context, anyhow};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{ComplianceModel, ComplianceResult, DataClassification};
use crate::models::audit::{AuditInfo, AuditEntry};
use crate::models::security::AccessControl;
use crate::models::validation::{Validatable, ValidationContext, Validators};

/// Supported configuration file formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}

impl ConfigFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Result<Self> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => Ok(ConfigFormat::Json),
            Some("yaml") | Some("yml") => Ok(ConfigFormat::Yaml),
            Some("toml") => Ok(ConfigFormat::Toml),
            Some(ext) => Err(anyhow!("Unsupported file extension: {}", ext)),
            None => Err(anyhow!("No file extension found")),
        }
    }
    
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFormat::Json => "json",
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::Toml => "toml",
        }
    }
}

/// Configuration file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileMetadata {
    /// Unique identifier for this config file
    pub id: Uuid,
    
    /// File path
    pub path: PathBuf,
    
    /// File format
    pub format: ConfigFormat,
    
    /// File size in bytes
    pub size: u64,
    
    /// Last modified timestamp
    pub modified: DateTime<Utc>,
    
    /// File permissions (Unix-style)
    pub permissions: String,
    
    /// Whether the file is readable
    pub readable: bool,
    
    /// Whether the file is writable
    pub writable: bool,
    
    /// Data classification
    pub data_classification: DataClassification,
    
    /// Access control
    pub access_control: AccessControl,
    
    /// Audit information
    pub audit_info: AuditInfo,
}

/// Configuration file operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOperation {
    /// Operation ID
    pub id: Uuid,
    
    /// Operation type
    pub operation_type: ConfigOperationType,
    
    /// File path
    pub file_path: PathBuf,
    
    /// Operation timestamp
    pub timestamp: DateTime<Utc>,
    
    /// User who performed the operation
    pub user_id: String,
    
    /// Whether the operation was successful
    pub success: bool,
    
    /// Error message if operation failed
    pub error_message: Option<String>,
    
    /// Backup file path (if created)
    pub backup_path: Option<PathBuf>,
    
    /// File hash before operation
    pub hash_before: Option<String>,
    
    /// File hash after operation
    pub hash_after: Option<String>,
}

/// Types of configuration operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigOperationType {
    Read,
    Write,
    Backup,
    Restore,
    Validate,
    Watch,
    Unwatch,
}

/// Configuration file service for safe file operations
pub struct ConfigFileService {
    /// Current user ID
    user_id: String,
    
    /// Operation history
    operations: Vec<ConfigOperation>,
    
    /// Whether to create backups before writes
    auto_backup: bool,
    
    /// Backup directory
    backup_dir: PathBuf,
}

impl ConfigFileService {
    /// Create a new configuration file service
    pub fn new(user_id: String, backup_dir: PathBuf) -> Self {
        Self {
            user_id,
            operations: Vec::new(),
            auto_backup: true,
            backup_dir,
        }
    }
    
    /// Read configuration from a file
    pub async fn read_config<T>(&mut self, path: &Path) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let operation_id = Uuid::new_v4();
        let start_time = Utc::now();
        
        let result = self.read_config_internal(path).await;
        
        // Record operation
        let operation = ConfigOperation {
            id: operation_id,
            operation_type: ConfigOperationType::Read,
            file_path: path.to_path_buf(),
            timestamp: start_time,
            user_id: self.user_id.clone(),
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
            backup_path: None,
            hash_before: None,
            hash_after: None,
        };
        
        self.operations.push(operation);
        
        match result {
            Ok(content) => {
                let parsed: T = self.parse_config_content(&content, path)?;
                Ok(parsed)
            }
            Err(e) => Err(e),
        }
    }
    
    /// Write configuration to a file
    pub async fn write_config<T>(&mut self, path: &Path, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let operation_id = Uuid::new_v4();
        let start_time = Utc::now();
        
        // Create backup if file exists and auto_backup is enabled
        let backup_path = if self.auto_backup && path.exists() {
            Some(self.create_backup(path).await?)
        } else {
            None
        };
        
        // Get hash before operation
        let hash_before = if path.exists() {
            Some(self.calculate_file_hash(path)?)
        } else {
            None
        };
        
        let result = self.write_config_internal(path, data).await;
        
        // Get hash after operation
        let hash_after = if result.is_ok() && path.exists() {
            Some(self.calculate_file_hash(path)?)
        } else {
            None
        };
        
        // Record operation
        let operation = ConfigOperation {
            id: operation_id,
            operation_type: ConfigOperationType::Write,
            file_path: path.to_path_buf(),
            timestamp: start_time,
            user_id: self.user_id.clone(),
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
            backup_path,
            hash_before,
            hash_after,
        };
        
        self.operations.push(operation);
        
        result
    }
    
    /// Validate configuration file format and content
    pub async fn validate_config(&mut self, path: &Path) -> Result<ConfigFileMetadata> {
        let operation_id = Uuid::new_v4();
        let start_time = Utc::now();
        
        let result = self.validate_config_internal(path).await;
        
        // Record operation
        let operation = ConfigOperation {
            id: operation_id,
            operation_type: ConfigOperationType::Validate,
            file_path: path.to_path_buf(),
            timestamp: start_time,
            user_id: self.user_id.clone(),
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
            backup_path: None,
            hash_before: None,
            hash_after: None,
        };
        
        self.operations.push(operation);
        
        result
    }
    
    /// Create a backup of a configuration file
    pub async fn create_backup(&mut self, path: &Path) -> Result<PathBuf> {
        let operation_id = Uuid::new_v4();
        let start_time = Utc::now();
        
        let result = self.create_backup_internal(path).await;
        
        // Record operation
        let operation = ConfigOperation {
            id: operation_id,
            operation_type: ConfigOperationType::Backup,
            file_path: path.to_path_buf(),
            timestamp: start_time,
            user_id: self.user_id.clone(),
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
            backup_path: result.as_ref().ok().cloned(),
            hash_before: None,
            hash_after: None,
        };
        
        self.operations.push(operation);
        
        result
    }
    
    /// Restore configuration from a backup
    pub async fn restore_config(&mut self, backup_path: &Path, target_path: &Path) -> Result<()> {
        let operation_id = Uuid::new_v4();
        let start_time = Utc::now();
        
        let hash_before = if target_path.exists() {
            Some(self.calculate_file_hash(target_path)?)
        } else {
            None
        };
        
        let result = self.restore_config_internal(backup_path, target_path).await;
        
        let hash_after = if result.is_ok() && target_path.exists() {
            Some(self.calculate_file_hash(target_path)?)
        } else {
            None
        };
        
        // Record operation
        let operation = ConfigOperation {
            id: operation_id,
            operation_type: ConfigOperationType::Restore,
            file_path: target_path.to_path_buf(),
            timestamp: start_time,
            user_id: self.user_id.clone(),
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
            backup_path: Some(backup_path.to_path_buf()),
            hash_before,
            hash_after,
        };
        
        self.operations.push(operation);
        
        result
    }
    
    /// Get operation history
    pub fn get_operations(&self) -> &[ConfigOperation] {
        &self.operations
    }
    
    /// Set auto-backup behavior
    pub fn set_auto_backup(&mut self, enabled: bool) {
        self.auto_backup = enabled;
    }
    
    // Internal implementation methods
    
    async fn read_config_internal(&self, path: &Path) -> Result<String> {
        // Check if file exists
        if !path.exists() {
            return Err(anyhow!("Configuration file does not exist: {}", path.display()));
        }
        
        // Check if file is readable
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
        
        if metadata.is_dir() {
            return Err(anyhow!("Path is a directory, not a file: {}", path.display()));
        }
        
        // Read file content
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration file: {}", path.display()))?;
        
        Ok(content)
    }
    
    async fn write_config_internal<T>(&self, path: &Path, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        // Determine format and serialize data
        let format = ConfigFormat::from_extension(path)?;
        let content = self.serialize_config_content(data, &format)?;
        
        // Write to temporary file first
        let temp_path = path.with_extension(format!("{}.tmp", path.extension().unwrap_or_default().to_string_lossy()));
        
        {
            let mut file = fs::File::create(&temp_path)
                .with_context(|| format!("Failed to create temporary file: {}", temp_path.display()))?;
            
            file.write_all(content.as_bytes())
                .with_context(|| format!("Failed to write to temporary file: {}", temp_path.display()))?;
            
            file.sync_all()
                .with_context(|| format!("Failed to sync temporary file: {}", temp_path.display()))?;
        }
        
        // Atomically move temporary file to target
        fs::rename(&temp_path, path)
            .with_context(|| format!("Failed to move temporary file to target: {}", path.display()))?;
        
        Ok(())
    }
    
    async fn validate_config_internal(&self, path: &Path) -> Result<ConfigFileMetadata> {
        // Check if file exists
        if !path.exists() {
            return Err(anyhow!("Configuration file does not exist: {}", path.display()));
        }
        
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
        
        // Detect format
        let format = ConfigFormat::from_extension(path)?;
        
        // Try to parse the file to validate format
        let content = self.read_config_internal(path).await?;
        self.validate_config_format(&content, &format)?;
        
        // Get file permissions
        let permissions = self.get_file_permissions(&metadata);
        
        let config_metadata = ConfigFileMetadata {
            id: Uuid::new_v4(),
            path: path.to_path_buf(),
            format,
            size: metadata.len(),
            modified: metadata.modified()
                .map(DateTime::<Utc>::from)
                .unwrap_or_else(|_| Utc::now()),
            permissions: permissions.clone(),
            readable: path.exists() && fs::read(path).is_ok(),
            writable: self.is_writable(path),
            data_classification: DataClassification::Internal, // Default classification
            access_control: AccessControl::new(&self.user_id),
            audit_info: AuditInfo::new(self.user_id.clone()),
        };
        
        Ok(config_metadata)
    }
    
    async fn create_backup_internal(&self, path: &Path) -> Result<PathBuf> {
        if !path.exists() {
            return Err(anyhow!("Cannot backup non-existent file: {}", path.display()));
        }
        
        // Create backup filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = path.file_name()
            .ok_or_else(|| anyhow!("Invalid file path: {}", path.display()))?;
        
        let backup_filename = format!("{}_{}.backup", filename.to_string_lossy(), timestamp);
        let backup_path = self.backup_dir.join(backup_filename);
        
        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_dir)
            .with_context(|| format!("Failed to create backup directory: {}", self.backup_dir.display()))?;
        
        // Copy file to backup location
        fs::copy(path, &backup_path)
            .with_context(|| format!("Failed to create backup: {} -> {}", path.display(), backup_path.display()))?;
        
        Ok(backup_path)
    }
    
    async fn restore_config_internal(&self, backup_path: &Path, target_path: &Path) -> Result<()> {
        if !backup_path.exists() {
            return Err(anyhow!("Backup file does not exist: {}", backup_path.display()));
        }
        
        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        // Copy backup to target location
        fs::copy(backup_path, target_path)
            .with_context(|| format!("Failed to restore backup: {} -> {}", backup_path.display(), target_path.display()))?;
        
        Ok(())
    }
    
    fn parse_config_content<T>(&self, content: &str, path: &Path) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let format = ConfigFormat::from_extension(path)?;
        
        match format {
            ConfigFormat::Json => {
                serde_json::from_str(content)
                    .with_context(|| format!("Failed to parse JSON configuration: {}", path.display()))
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str(content)
                    .with_context(|| format!("Failed to parse YAML configuration: {}", path.display()))
            }
            ConfigFormat::Toml => {
                toml::from_str(content)
                    .with_context(|| format!("Failed to parse TOML configuration: {}", path.display()))
            }
        }
    }
    
    fn serialize_config_content<T>(&self, data: &T, format: &ConfigFormat) -> Result<String>
    where
        T: Serialize,
    {
        match format {
            ConfigFormat::Json => {
                serde_json::to_string_pretty(data)
                    .with_context(|| "Failed to serialize data to JSON")
            }
            ConfigFormat::Yaml => {
                serde_yaml::to_string(data)
                    .with_context(|| "Failed to serialize data to YAML")
            }
            ConfigFormat::Toml => {
                toml::to_string_pretty(data)
                    .with_context(|| "Failed to serialize data to TOML")
            }
        }
    }
    
    fn validate_config_format(&self, content: &str, format: &ConfigFormat) -> Result<()> {
        match format {
            ConfigFormat::Json => {
                serde_json::from_str::<serde_json::Value>(content)
                    .with_context(|| "Invalid JSON format")?;
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str::<serde_yaml::Value>(content)
                    .with_context(|| "Invalid YAML format")?;
            }
            ConfigFormat::Toml => {
                toml::from_str::<toml::Value>(content)
                    .with_context(|| "Invalid TOML format")?;
            }
        }
        Ok(())
    }
    
    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let content = fs::read(path)
            .with_context(|| format!("Failed to read file for hashing: {}", path.display()))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = hasher.finalize();
        
        Ok(format!("{:x}", hash))
    }
    
    fn get_file_permissions(&self, metadata: &fs::Metadata) -> String {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            format!("{:o}", mode & 0o777)
        }
        
        #[cfg(not(unix))]
        {
            if metadata.permissions().readonly() {
                "r--r--r--".to_string()
            } else {
                "rw-rw-rw-".to_string()
            }
        }
    }
    
    fn is_writable(&self, path: &Path) -> bool {
        if let Some(parent) = path.parent() {
            if path.exists() {
                fs::OpenOptions::new().write(true).open(path).is_ok()
            } else {
                fs::create_dir_all(parent).is_ok()
            }
        } else {
            false
        }
    }
}

impl Validatable for ConfigFileMetadata {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate path
        ctx.enter_field("path");
        if let Err(e) = Validators::file_path(&self.path.to_string_lossy(), "path") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate size
        ctx.enter_field("size");
        if let Err(e) = Validators::numeric_range(self.size as f64, "size", Some(0.0), Some(1_000_000_000.0)) {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate permissions
        ctx.enter_field("permissions");
        if let Err(e) = Validators::not_empty(&self.permissions, "permissions") {
            ctx.add_error(e);
        }
        ctx.exit_field();
    }
}

impl ComplianceModel for ConfigFileMetadata {
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
    
    fn get_audit_trail(&self) -> Vec<AuditEntry> {
        vec![AuditEntry {
            id: Uuid::new_v4(),
            entity_type: "ConfigFileMetadata".to_string(),
            entity_id: self.id.to_string(),
            action: "created".to_string(),
            user_id: self.audit_info.created_by.clone(),
            timestamp: self.audit_info.created_at,
            details: serde_json::json!({
                "path": self.path,
                "format": self.format,
                "size": self.size,
                "readable": self.readable,
                "writable": self.writable,
                "data_classification": self.data_classification
            }),
            ip_address: None,
            user_agent: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_config_format_detection() {
        assert_eq!(ConfigFormat::from_extension(Path::new("config.json")).unwrap(), ConfigFormat::Json);
        assert_eq!(ConfigFormat::from_extension(Path::new("config.yaml")).unwrap(), ConfigFormat::Yaml);
        assert_eq!(ConfigFormat::from_extension(Path::new("config.toml")).unwrap(), ConfigFormat::Toml);
    }
    
    #[tokio::test]
    async fn test_read_write_json_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.json");
        let backup_dir = temp_dir.path().join("backups");
        
        let mut service = ConfigFileService::new("test_user".to_string(), backup_dir);
        
        let test_data = json!({
            "name": "test",
            "value": 42
        });
        
        // Write config
        service.write_config(&config_path, &test_data).await.unwrap();
        
        // Read config back
        let read_data: serde_json::Value = service.read_config(&config_path).await.unwrap();
        
        assert_eq!(test_data, read_data);
    }
    
    #[tokio::test]
    async fn test_backup_and_restore() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.json");
        let backup_dir = temp_dir.path().join("backups");
        
        let mut service = ConfigFileService::new("test_user".to_string(), backup_dir);
        
        let original_data = json!({"version": 1});
        let modified_data = json!({"version": 2});
        
        // Write original config
        service.write_config(&config_path, &original_data).await.unwrap();
        
        // Create backup
        let backup_path = service.create_backup(&config_path).await.unwrap();
        
        // Modify config
        service.write_config(&config_path, &modified_data).await.unwrap();
        
        // Restore from backup
        service.restore_config(&backup_path, &config_path).await.unwrap();
        
        // Verify restoration
        let restored_data: serde_json::Value = service.read_config(&config_path).await.unwrap();
        assert_eq!(original_data, restored_data);
    }
    
    #[tokio::test]
    async fn test_validate_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.json");
        let backup_dir = temp_dir.path().join("backups");
        
        let mut service = ConfigFileService::new("test_user".to_string(), backup_dir);
        
        let test_data = json!({"test": true});
        service.write_config(&config_path, &test_data).await.unwrap();
        
        let metadata = service.validate_config(&config_path).await.unwrap();
        
        assert_eq!(metadata.format, ConfigFormat::Json);
        assert!(metadata.readable);
        assert!(metadata.writable);
        assert!(metadata.size > 0);
    }
}
