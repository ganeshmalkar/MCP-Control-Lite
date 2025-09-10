use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Backup metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Unique backup ID
    pub id: Uuid,
    
    /// Original file path
    pub original_path: PathBuf,
    
    /// Backup file path
    pub backup_path: PathBuf,
    
    /// Timestamp when backup was created
    pub created_at: DateTime<Utc>,
    
    /// User who created the backup
    pub created_by: String,
    
    /// Original file size
    pub original_size: u64,
    
    /// Backup file size
    pub backup_size: u64,
    
    /// SHA-256 hash of original file
    pub original_hash: String,
    
    /// SHA-256 hash of backup file
    pub backup_hash: String,
    
    /// Backup type
    pub backup_type: BackupType,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Whether this backup is compressed
    pub is_compressed: bool,
    
    /// Retention policy for this backup
    pub retention_days: Option<u32>,
}

/// Types of backups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BackupType {
    /// Automatic backup before modification
    Automatic,
    /// Manual backup requested by user
    Manual,
    /// Scheduled backup
    Scheduled,
    /// Backup before system update
    PreUpdate,
}

/// Backup service for managing configuration file backups
pub struct BackupService {
    /// Base directory for storing backups
    backup_dir: PathBuf,
    
    /// Current user ID
    user_id: String,
    
    /// Maximum number of backups to keep per file
    max_backups_per_file: usize,
    
    /// Default retention period in days
    default_retention_days: u32,
    
    /// Whether to compress backups
    compress_backups: bool,
}

impl BackupService {
    /// Create a new backup service
    pub fn new<P: AsRef<Path>>(backup_dir: P, user_id: String) -> Result<Self> {
        let backup_dir = backup_dir.as_ref().to_path_buf();
        
        // Ensure backup directory exists
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("Failed to create backup directory: {}", backup_dir.display()))?;
        
        Ok(Self {
            backup_dir,
            user_id,
            max_backups_per_file: 10, // Default to keeping 10 backups per file
            default_retention_days: 30, // Default 30 day retention
            compress_backups: false, // Default to no compression for simplicity
        })
    }
    
    /// Set maximum number of backups to keep per file
    pub fn set_max_backups_per_file(&mut self, max_backups: usize) {
        self.max_backups_per_file = max_backups;
    }
    
    /// Set default retention period in days
    pub fn set_default_retention_days(&mut self, days: u32) {
        self.default_retention_days = days;
    }
    
    /// Enable or disable backup compression
    pub fn set_compression(&mut self, enabled: bool) {
        self.compress_backups = enabled;
    }
    
    /// Create a backup of a file
    pub fn create_backup<P: AsRef<Path>>(
        &self,
        file_path: P,
        backup_type: BackupType,
        description: Option<String>,
    ) -> Result<BackupMetadata> {
        let file_path = file_path.as_ref();
        
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {}", file_path.display()));
        }
        
        if !file_path.is_file() {
            return Err(anyhow::anyhow!("Path is not a file: {}", file_path.display()));
        }
        
        // Generate backup filename
        let backup_id = Uuid::new_v4();
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let original_filename = file_path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path: {}", file_path.display()))?
            .to_string_lossy();
        
        let backup_filename = format!("{}_{}_{}_{}.backup", 
            original_filename, backup_id, timestamp, backup_type.to_string().to_lowercase());
        let backup_path = self.backup_dir.join(backup_filename);
        
        // Read original file
        let original_content = fs::read(file_path)
            .with_context(|| format!("Failed to read original file: {}", file_path.display()))?;
        
        let original_size = original_content.len() as u64;
        let original_hash = self.calculate_hash(&original_content);
        
        // Create backup content (with optional compression)
        let backup_content = if self.compress_backups {
            self.compress_data(&original_content)?
        } else {
            original_content.clone()
        };
        
        // Write backup file
        fs::write(&backup_path, &backup_content)
            .with_context(|| format!("Failed to write backup file: {}", backup_path.display()))?;
        
        let backup_size = backup_content.len() as u64;
        let backup_hash = self.calculate_hash(&backup_content);
        
        // Create metadata
        let metadata = BackupMetadata {
            id: backup_id,
            original_path: file_path.to_path_buf(),
            backup_path: backup_path.clone(),
            created_at: Utc::now(),
            created_by: self.user_id.clone(),
            original_size,
            backup_size,
            original_hash,
            backup_hash,
            backup_type,
            description,
            is_compressed: self.compress_backups,
            retention_days: Some(self.default_retention_days),
        };
        
        // Save metadata
        self.save_metadata(&metadata)?;
        
        // Clean up old backups if necessary
        self.cleanup_old_backups(file_path)?;
        
        Ok(metadata)
    }
    
    /// Restore a file from backup
    pub fn restore_backup(&self, backup_metadata: &BackupMetadata) -> Result<()> {
        if !backup_metadata.backup_path.exists() {
            return Err(anyhow::anyhow!("Backup file does not exist: {}", backup_metadata.backup_path.display()));
        }
        
        // Read backup content
        let backup_content = fs::read(&backup_metadata.backup_path)
            .with_context(|| format!("Failed to read backup file: {}", backup_metadata.backup_path.display()))?;
        
        // Verify backup integrity
        let backup_hash = self.calculate_hash(&backup_content);
        if backup_hash != backup_metadata.backup_hash {
            return Err(anyhow::anyhow!("Backup file integrity check failed"));
        }
        
        // Decompress if necessary
        let restore_content = if backup_metadata.is_compressed {
            self.decompress_data(&backup_content)?
        } else {
            backup_content
        };
        
        // Ensure target directory exists
        if let Some(parent) = backup_metadata.original_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        // Write restored content to original location
        fs::write(&backup_metadata.original_path, &restore_content)
            .with_context(|| format!("Failed to restore file: {}", backup_metadata.original_path.display()))?;
        
        Ok(())
    }
    
    /// List all backups for a specific file
    pub fn list_backups_for_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<BackupMetadata>> {
        let file_path = file_path.as_ref();
        let mut backups = Vec::new();
        
        // Read all metadata files
        let metadata_dir = self.backup_dir.join("metadata");
        if !metadata_dir.exists() {
            return Ok(backups);
        }
        
        for entry in fs::read_dir(&metadata_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata) = self.load_metadata(&entry.path()) {
                    if metadata.original_path == file_path {
                        backups.push(metadata);
                    }
                }
            }
        }
        
        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }
    
    /// List all backups
    pub fn list_all_backups(&self) -> Result<Vec<BackupMetadata>> {
        let mut backups = Vec::new();
        
        let metadata_dir = self.backup_dir.join("metadata");
        if !metadata_dir.exists() {
            return Ok(backups);
        }
        
        for entry in fs::read_dir(&metadata_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata) = self.load_metadata(&entry.path()) {
                    backups.push(metadata);
                }
            }
        }
        
        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }
    
    /// Delete a specific backup
    pub fn delete_backup(&self, backup_id: &Uuid) -> Result<()> {
        let metadata_path = self.get_metadata_path(backup_id);
        
        if let Ok(metadata) = self.load_metadata(&metadata_path) {
            // Delete backup file
            if metadata.backup_path.exists() {
                fs::remove_file(&metadata.backup_path)
                    .with_context(|| format!("Failed to delete backup file: {}", metadata.backup_path.display()))?;
            }
            
            // Delete metadata file
            fs::remove_file(&metadata_path)
                .with_context(|| format!("Failed to delete metadata file: {}", metadata_path.display()))?;
        }
        
        Ok(())
    }
    
    /// Clean up expired backups
    pub fn cleanup_expired_backups(&self) -> Result<Vec<Uuid>> {
        let mut deleted_backups = Vec::new();
        let now = Utc::now();
        
        for metadata in self.list_all_backups()? {
            if let Some(retention_days) = metadata.retention_days {
                let expiry_date = metadata.created_at + chrono::Duration::days(retention_days as i64);
                if now > expiry_date {
                    self.delete_backup(&metadata.id)?;
                    deleted_backups.push(metadata.id);
                }
            }
        }
        
        Ok(deleted_backups)
    }
    
    /// Get backup statistics
    pub fn get_backup_stats(&self) -> Result<BackupStats> {
        let backups = self.list_all_backups()?;
        
        let total_backups = backups.len();
        let total_size: u64 = backups.iter().map(|b| b.backup_size).sum();
        let oldest_backup = backups.iter().map(|b| b.created_at).min();
        let newest_backup = backups.iter().map(|b| b.created_at).max();
        
        let mut backup_types = std::collections::HashMap::new();
        for backup in &backups {
            *backup_types.entry(backup.backup_type.clone()).or_insert(0) += 1;
        }
        
        Ok(BackupStats {
            total_backups,
            total_size_bytes: total_size,
            oldest_backup,
            newest_backup,
            backup_types,
        })
    }
    
    // Private helper methods
    
    fn save_metadata(&self, metadata: &BackupMetadata) -> Result<()> {
        let metadata_dir = self.backup_dir.join("metadata");
        fs::create_dir_all(&metadata_dir)
            .with_context(|| format!("Failed to create metadata directory: {}", metadata_dir.display()))?;
        
        let metadata_path = self.get_metadata_path(&metadata.id);
        let metadata_json = serde_json::to_string_pretty(metadata)
            .context("Failed to serialize backup metadata")?;
        
        fs::write(&metadata_path, metadata_json)
            .with_context(|| format!("Failed to write metadata file: {}", metadata_path.display()))?;
        
        Ok(())
    }
    
    fn load_metadata(&self, metadata_path: &Path) -> Result<BackupMetadata> {
        let metadata_json = fs::read_to_string(metadata_path)
            .with_context(|| format!("Failed to read metadata file: {}", metadata_path.display()))?;
        
        let metadata: BackupMetadata = serde_json::from_str(&metadata_json)
            .context("Failed to deserialize backup metadata")?;
        
        Ok(metadata)
    }
    
    fn get_metadata_path(&self, backup_id: &Uuid) -> PathBuf {
        self.backup_dir.join("metadata").join(format!("{}.json", backup_id))
    }
    
    fn cleanup_old_backups<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let mut backups = self.list_backups_for_file(file_path)?;
        
        if backups.len() > self.max_backups_per_file {
            // Sort by creation time (oldest first for deletion)
            backups.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            
            // Delete oldest backups beyond the limit
            let to_delete = backups.len() - self.max_backups_per_file;
            for backup in backups.iter().take(to_delete) {
                self.delete_backup(&backup.id)?;
            }
        }
        
        Ok(())
    }
    
    fn calculate_hash(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        format!("{:x}", hash)
    }
    
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // For now, just return the original data
        // In a real implementation, you might use flate2 or similar
        Ok(data.to_vec())
    }
    
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // For now, just return the original data
        // In a real implementation, you might use flate2 or similar
        Ok(data.to_vec())
    }
}

/// Backup statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStats {
    pub total_backups: usize,
    pub total_size_bytes: u64,
    pub oldest_backup: Option<DateTime<Utc>>,
    pub newest_backup: Option<DateTime<Utc>>,
    pub backup_types: std::collections::HashMap<BackupType, usize>,
}

impl std::fmt::Display for BackupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupType::Automatic => write!(f, "automatic"),
            BackupType::Manual => write!(f, "manual"),
            BackupType::Scheduled => write!(f, "scheduled"),
            BackupType::PreUpdate => write!(f, "preupdate"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_create_and_restore_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let test_file = temp_dir.path().join("test.txt");
        
        // Create test file
        let original_content = "This is test content";
        fs::write(&test_file, original_content).unwrap();
        
        // Create backup service
        let backup_service = BackupService::new(&backup_dir, "test_user".to_string()).unwrap();
        
        // Create backup
        let metadata = backup_service.create_backup(
            &test_file,
            BackupType::Manual,
            Some("Test backup".to_string()),
        ).unwrap();
        
        assert_eq!(metadata.original_path, test_file);
        assert!(metadata.backup_path.exists());
        assert_eq!(metadata.backup_type, BackupType::Manual);
        assert_eq!(metadata.description, Some("Test backup".to_string()));
        
        // Modify original file
        fs::write(&test_file, "Modified content").unwrap();
        
        // Restore from backup
        backup_service.restore_backup(&metadata).unwrap();
        
        // Verify restoration
        let restored_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(restored_content, original_content);
    }
    
    #[test]
    fn test_list_backups_for_file() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let test_file = temp_dir.path().join("test.txt");
        
        fs::write(&test_file, "content").unwrap();
        
        let backup_service = BackupService::new(&backup_dir, "test_user".to_string()).unwrap();
        
        // Create multiple backups
        backup_service.create_backup(&test_file, BackupType::Automatic, None).unwrap();
        backup_service.create_backup(&test_file, BackupType::Manual, None).unwrap();
        
        let backups = backup_service.list_backups_for_file(&test_file).unwrap();
        assert_eq!(backups.len(), 2);
        
        // Should be sorted by creation time (newest first)
        assert!(backups[0].created_at >= backups[1].created_at);
    }
    
    #[test]
    fn test_backup_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let test_file = temp_dir.path().join("test.txt");
        
        fs::write(&test_file, "content").unwrap();
        
        let mut backup_service = BackupService::new(&backup_dir, "test_user".to_string()).unwrap();
        backup_service.set_max_backups_per_file(2);
        
        // Create 3 backups (should trigger cleanup)
        let _backup1 = backup_service.create_backup(&test_file, BackupType::Automatic, None).unwrap();
        let _backup2 = backup_service.create_backup(&test_file, BackupType::Automatic, None).unwrap();
        let _backup3 = backup_service.create_backup(&test_file, BackupType::Automatic, None).unwrap();
        
        let backups = backup_service.list_backups_for_file(&test_file).unwrap();
        assert_eq!(backups.len(), 2); // Should only keep 2 backups
    }
    
    #[test]
    fn test_backup_stats() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let test_file = temp_dir.path().join("test.txt");
        
        fs::write(&test_file, "content").unwrap();
        
        let backup_service = BackupService::new(&backup_dir, "test_user".to_string()).unwrap();
        
        // Create backups of different types
        backup_service.create_backup(&test_file, BackupType::Automatic, None).unwrap();
        backup_service.create_backup(&test_file, BackupType::Manual, None).unwrap();
        
        let stats = backup_service.get_backup_stats().unwrap();
        assert_eq!(stats.total_backups, 2);
        assert!(stats.total_size_bytes > 0);
        assert!(stats.backup_types.contains_key(&BackupType::Automatic));
        assert!(stats.backup_types.contains_key(&BackupType::Manual));
    }
}
