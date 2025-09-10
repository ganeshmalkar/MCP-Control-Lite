//! Data models for MCP Control Lite
//! 
//! This module contains all data structures used throughout the application,
//! designed with SOC2, HIPAA, and WCAG 2.1 compliance in mind.
//! 
//! All models include:
//! - Audit fields (created_by, updated_by, timestamps)
//! - Access control metadata
//! - Encryption support for sensitive data
//! - Validation methods for compliance

pub mod audit;
pub mod server;
pub mod session;
pub mod app;
pub mod preferences;
pub mod registry;
pub mod compliance;
pub mod security;
pub mod validation;
pub mod encryption;

// Re-export main types for convenience (avoiding duplicates)
pub use audit::{AuditEntry, AuditInfo, AuditTrail, SecurityEvent, SecurityEventType, ComplianceAuditReport};
pub use server::*;
pub use session::*;
pub use app::*;
pub use preferences::*;
pub use registry::*;
pub use compliance::*;
pub use security::{AccessControl, EncryptionSettings, EncryptionMethod, DataClassification, SecureCredential, PersonalInfo};
pub use validation::*;
pub use encryption::*;

use uuid::Uuid;

/// Common trait for all data models to ensure compliance
pub trait ComplianceModel {
    /// Validate the model for compliance requirements
    fn validate_compliance(&self) -> Result<(), Vec<String>>;
    
    /// Check compliance status
    fn get_compliance_status(&self) -> ComplianceResult;
    
    /// Get audit trail for this entity
    fn get_audit_trail(&self) -> Vec<AuditEntry>;
}

/// Generate a new UUID for model IDs
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Common validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Required field '{field}' is empty")]
    RequiredFieldEmpty { field: String },
    
    #[error("Invalid format for field '{field}': {reason}")]
    InvalidFormat { field: String, reason: String },
    
    #[error("Security requirement not met: {requirement}")]
    SecurityRequirement { requirement: String },
    
    #[error("Compliance violation: {violation}")]
    ComplianceViolation { violation: String },
}
