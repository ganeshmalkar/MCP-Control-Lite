//! Security-related data structures for access control and audit logging

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{ComplianceModel, ComplianceResult};
use crate::models::audit::AuditInfo;
use crate::models::validation::{Validatable, ValidationContext, Validators, SecurityValidators};
use crate::models::encryption::EncryptedField;

/// Access control information for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    pub owner_id: String,
    pub group_ids: Vec<String>,
    pub permissions: String, // e.g., "rw-r--r--" Unix-style permissions
    pub restricted_to_roles: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AccessControl {
    pub fn new(owner_id: &str) -> Self {
        let now = Utc::now();
        Self {
            owner_id: owner_id.to_string(),
            group_ids: Vec::new(),
            permissions: "rw-r--r--".to_string(), // Default: owner read/write, others read
            restricted_to_roles: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn can_read(&self, user_id: &str, user_groups: &[String], user_roles: &[String]) -> bool {
        // Owner can always read
        if self.owner_id == user_id {
            return true;
        }
        
        // Check role restrictions
        if let Some(ref restricted_roles) = self.restricted_to_roles {
            if !user_roles.iter().any(|role| restricted_roles.contains(role)) {
                return false;
            }
        }
        
        // Check group permissions (simplified - would need full permission parsing)
        if self.group_ids.iter().any(|group| user_groups.contains(group)) {
            return self.permissions.chars().nth(4) == Some('r');
        }
        
        // Check other permissions
        self.permissions.chars().nth(7) == Some('r')
    }
    
    pub fn can_write(&self, user_id: &str, user_groups: &[String], user_roles: &[String]) -> bool {
        // Owner can always write
        if self.owner_id == user_id {
            return true;
        }
        
        // Check role restrictions
        if let Some(ref restricted_roles) = self.restricted_to_roles {
            if !user_roles.iter().any(|role| restricted_roles.contains(role)) {
                return false;
            }
        }
        
        // Check group permissions
        if self.group_ids.iter().any(|group| user_groups.contains(group)) {
            return self.permissions.chars().nth(5) == Some('w');
        }
        
        // Check other permissions
        self.permissions.chars().nth(8) == Some('w')
    }
}

/// Audit log entry for tracking all access and modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub action: AccessAction,
    pub resource_type: String,
    pub resource_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub additional_metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessAction {
    Read,
    Create,
    Update,
    Delete,
    Execute,
    Export,
    Import,
    Backup,
    Restore,
}

impl AccessAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessAction::Read => "read",
            AccessAction::Create => "create",
            AccessAction::Update => "update",
            AccessAction::Delete => "delete",
            AccessAction::Execute => "execute",
            AccessAction::Export => "export",
            AccessAction::Import => "import",
            AccessAction::Backup => "backup",
            AccessAction::Restore => "restore",
        }
    }
}

impl AccessLogEntry {
    pub fn new(
        user_id: &str,
        action: AccessAction,
        resource_type: &str,
        resource_id: &str,
    ) -> Self {
        Self {
            id: crate::models::generate_id(),
            timestamp: Utc::now(),
            user_id: user_id.to_string(),
            action,
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            ip_address: None,
            user_agent: None,
            success: true,
            failure_reason: None,
            additional_metadata: None,
        }
    }
    
    pub fn with_failure(mut self, reason: &str) -> Self {
        self.success = false;
        self.failure_reason = Some(reason.to_string());
        self
    }
    
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        if let Some(ref mut metadata) = self.additional_metadata {
            metadata.insert(key.to_string(), value.to_string());
        } else {
            let mut metadata = HashMap::new();
            metadata.insert(key.to_string(), value.to_string());
            self.additional_metadata = Some(metadata);
        }
        self
    }
}

/// Encryption settings for sensitive data fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSettings {
    pub encrypted_fields: Vec<String>,
    pub encryption_method: EncryptionMethod,
    pub key_id: String,
    pub last_rotated: DateTime<Utc>,
    pub rotation_interval_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionMethod {
    #[serde(rename = "AES-256-GCM")]
    Aes256Gcm,
    #[serde(rename = "ChaCha20-Poly1305")]
    ChaCha20Poly1305,
}

impl EncryptionMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            EncryptionMethod::Aes256Gcm => "AES-256-GCM",
            EncryptionMethod::ChaCha20Poly1305 => "ChaCha20-Poly1305",
        }
    }
}

impl EncryptionSettings {
    pub fn new(key_id: &str) -> Self {
        Self {
            encrypted_fields: Vec::new(),
            encryption_method: EncryptionMethod::Aes256Gcm,
            key_id: key_id.to_string(),
            last_rotated: Utc::now(),
            rotation_interval_days: 90, // Default 90-day rotation
        }
    }
    
    pub fn needs_rotation(&self) -> bool {
        let rotation_due = self.last_rotated + chrono::Duration::days(self.rotation_interval_days as i64);
        Utc::now() > rotation_due
    }
    
    pub fn add_encrypted_field(&mut self, field_name: &str) {
        if !self.encrypted_fields.contains(&field_name.to_string()) {
            self.encrypted_fields.push(field_name.to_string());
        }
    }
}

/// Data classification levels for information governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl DataClassification {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataClassification::Public => "public",
            DataClassification::Internal => "internal",
            DataClassification::Confidential => "confidential",
            DataClassification::Restricted => "restricted",
        }
    }
    
    pub fn requires_encryption(&self) -> bool {
        matches!(self, DataClassification::Confidential | DataClassification::Restricted)
    }
    
    pub fn requires_audit_logging(&self) -> bool {
        matches!(self, DataClassification::Internal | DataClassification::Confidential | DataClassification::Restricted)
    }
}

impl Validatable for AccessControl {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate owner ID
        ctx.enter_field("owner_id");
        if let Err(e) = Validators::not_empty(&self.owner_id, "owner_id") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate permissions format
        ctx.enter_field("permissions");
        if let Err(e) = SecurityValidators::access_permissions(&self.permissions, "permissions") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate group IDs are not empty if present
        ctx.enter_field("group_ids");
        for (i, group_id) in self.group_ids.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            if let Err(e) = Validators::not_empty(group_id, "group_id") {
                ctx.add_error(e);
            }
            ctx.exit_field();
        }
        ctx.exit_field();
    }
}

impl ComplianceModel for AccessControl {
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
            entity_type: "AccessControl".to_string(),
            entity_id: self.owner_id.clone(),
            action: "created".to_string(),
            user_id: self.owner_id.clone(),
            timestamp: self.created_at,
            details: serde_json::json!({
                "permissions": self.permissions,
                "group_count": self.group_ids.len(),
                "has_role_restrictions": self.restricted_to_roles.is_some()
            }),
            ip_address: None,
            user_agent: None,
        }]
    }
}

impl Validatable for EncryptionSettings {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate key ID
        ctx.enter_field("key_id");
        if let Err(e) = Validators::not_empty(&self.key_id, "key_id") {
            ctx.add_error(e);
        }
        if let Err(e) = Validators::uuid(&self.key_id, "key_id") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate encrypted fields list
        ctx.enter_field("encrypted_fields");
        if let Err(e) = Validators::not_empty_collection(&self.encrypted_fields, "encrypted_fields") {
            ctx.add_error(e);
        }
        for (i, field_name) in self.encrypted_fields.iter().enumerate() {
            ctx.enter_field(&format!("[{}]", i));
            if let Err(e) = Validators::not_empty(field_name, "field_name") {
                ctx.add_error(e);
            }
            ctx.exit_field();
        }
        ctx.exit_field();
        
        // Validate rotation interval
        ctx.enter_field("rotation_interval_days");
        if let Err(e) = Validators::numeric_range(
            self.rotation_interval_days as f64,
            "rotation_interval_days",
            Some(1.0),
            Some(365.0),
        ) {
            ctx.add_error(e);
        }
        ctx.exit_field();
    }
}

impl ComplianceModel for EncryptionSettings {
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
            entity_type: "EncryptionSettings".to_string(),
            entity_id: self.key_id.clone(),
            action: "created".to_string(),
            user_id: "system".to_string(),
            timestamp: self.last_rotated,
            details: serde_json::json!({
                "encryption_method": self.encryption_method.as_str(),
                "encrypted_field_count": self.encrypted_fields.len(),
                "encrypted_fields": self.encrypted_fields,
                "rotation_interval_days": self.rotation_interval_days
            }),
            ip_address: None,
            user_agent: None,
        }]
    }
}

/// Example of a secure user credential with encrypted fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureCredential {
    /// Unique identifier
    pub id: Uuid,
    
    /// Username (not encrypted)
    pub username: String,
    
    /// Encrypted password hash
    pub password_hash: EncryptedField<String>,
    
    /// Encrypted API keys
    pub api_keys: EncryptedField<HashMap<String, String>>,
    
    /// Encrypted personal information
    pub personal_info: EncryptedField<PersonalInfo>,
    
    /// Data classification
    pub data_classification: DataClassification,
    
    /// Access control
    pub access_control: AccessControl,
    
    /// Audit information
    pub audit_info: AuditInfo,
}

/// Personal information that should be encrypted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalInfo {
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
}

impl SecureCredential {
    /// Create a new secure credential
    pub fn new(
        username: String,
        password_hash: String,
        created_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            password_hash: EncryptedField::new(password_hash),
            api_keys: EncryptedField::new(HashMap::new()),
            personal_info: EncryptedField::new(PersonalInfo {
                full_name: String::new(),
                email: String::new(),
                phone: None,
                address: None,
            }),
            data_classification: DataClassification::Confidential,
            access_control: AccessControl::new(&created_by),
            audit_info: AuditInfo::new(created_by),
        }
    }
}

impl Validatable for SecureCredential {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate username
        ctx.enter_field("username");
        if let Err(e) = Validators::not_empty(&self.username, "username") {
            ctx.add_error(e);
        }
        if let Err(e) = Validators::string_length(&self.username, "username", Some(3), Some(50)) {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate that sensitive fields are encrypted
        if ctx.options.validate_security {
            ctx.enter_field("password_hash");
            if let Err(e) = SecurityValidators::encrypted_field(
                "", // We can't access the actual value without decryption
                "password_hash",
                self.password_hash.is_encrypted(),
            ) {
                ctx.add_error(e);
            }
            ctx.exit_field();
            
            ctx.enter_field("api_keys");
            if let Err(e) = SecurityValidators::encrypted_field(
                "",
                "api_keys",
                self.api_keys.is_encrypted(),
            ) {
                ctx.add_error(e);
            }
            ctx.exit_field();
            
            ctx.enter_field("personal_info");
            if let Err(e) = SecurityValidators::encrypted_field(
                "",
                "personal_info",
                self.personal_info.is_encrypted(),
            ) {
                ctx.add_error(e);
            }
            ctx.exit_field();
        }
        
        // Validate access control
        ctx.enter_field("access_control");
        self.access_control.validate_with_context(ctx);
        ctx.exit_field();
    }
}

impl ComplianceModel for SecureCredential {
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
            entity_type: "SecureCredential".to_string(),
            entity_id: self.id.to_string(),
            action: "created".to_string(),
            user_id: self.audit_info.created_by.clone(),
            timestamp: self.audit_info.created_at,
            details: serde_json::json!({
                "username": self.username,
                "data_classification": self.data_classification.as_str(),
                "has_encrypted_password": self.password_hash.is_encrypted(),
                "has_encrypted_api_keys": self.api_keys.is_encrypted(),
                "has_encrypted_personal_info": self.personal_info.is_encrypted()
            }),
            ip_address: None,
            user_agent: None,
        }]
    }
}
