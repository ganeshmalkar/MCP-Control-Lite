use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Comprehensive audit entry for tracking all system activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier for this audit entry
    pub id: Uuid,
    
    /// Type of entity being audited (e.g., "ServerConfig", "Session", "User")
    pub entity_type: String,
    
    /// Unique identifier of the entity being audited
    pub entity_id: String,
    
    /// Action performed (e.g., "create", "read", "update", "delete", "login", "logout")
    pub action: String,
    
    /// User who performed the action
    pub user_id: String,
    
    /// Timestamp when the action occurred
    pub timestamp: DateTime<Utc>,
    
    /// Detailed information about the action
    pub details: serde_json::Value,
    
    /// IP address of the user (if applicable)
    pub ip_address: Option<String>,
    
    /// User agent string (if applicable)
    pub user_agent: Option<String>,
}

/// Common audit information included in all data models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInfo {
    /// User who created this entity
    pub created_by: String,
    
    /// User who last updated this entity
    pub updated_by: String,
    
    /// Timestamp when entity was created
    pub created_at: DateTime<Utc>,
    
    /// Timestamp when entity was last updated
    pub updated_at: DateTime<Utc>,
}

/// Audit trail for tracking changes to an entity over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    /// Entity being tracked
    pub entity_id: String,
    
    /// Type of entity
    pub entity_type: String,
    
    /// All audit entries for this entity
    pub entries: Vec<AuditEntry>,
    
    /// When this trail was last updated
    pub last_updated: DateTime<Utc>,
}

/// Security event for tracking security-related activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Unique identifier for this security event
    pub id: Uuid,
    
    /// Type of security event
    pub event_type: SecurityEventType,
    
    /// Severity level of the event
    pub severity: SecuritySeverity,
    
    /// User involved in the event (if applicable)
    pub user_id: Option<String>,
    
    /// IP address involved
    pub ip_address: Option<String>,
    
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    
    /// Description of the event
    pub description: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Whether this event has been resolved
    pub resolved: bool,
    
    /// Resolution details (if resolved)
    pub resolution: Option<String>,
}

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityEventType {
    /// Authentication attempt
    AuthenticationAttempt,
    /// Failed login
    LoginFailure,
    /// Successful login
    LoginSuccess,
    /// Logout
    Logout,
    /// Unauthorized access attempt
    UnauthorizedAccess,
    /// Permission denied
    PermissionDenied,
    /// Data access
    DataAccess,
    /// Configuration change
    ConfigurationChange,
    /// Tool execution
    ToolExecution,
    /// Suspicious activity
    SuspiciousActivity,
    /// Security policy violation
    PolicyViolation,
}

/// Security event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum SecuritySeverity {
    /// Informational event
    Info,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical security event
    Critical,
}

/// Access log entry for tracking data access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLogEntry {
    /// Unique identifier for this access log entry
    pub id: Uuid,
    
    /// User who accessed the data
    pub user_id: String,
    
    /// Type of resource accessed
    pub resource_type: String,
    
    /// Identifier of the resource
    pub resource_id: String,
    
    /// Type of access (read, write, delete, etc.)
    pub access_type: String,
    
    /// Timestamp of access
    pub timestamp: DateTime<Utc>,
    
    /// IP address of the accessor
    pub ip_address: Option<String>,
    
    /// User agent string
    pub user_agent: Option<String>,
    
    /// Whether access was granted
    pub access_granted: bool,
    
    /// Reason if access was denied
    pub denial_reason: Option<String>,
    
    /// Data classification of accessed resource
    pub data_classification: Option<String>,
    
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
}

/// Compliance audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditReport {
    /// Unique identifier for this report
    pub id: Uuid,
    
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    
    /// User who generated the report
    pub generated_by: String,
    
    /// Report period start
    pub period_start: DateTime<Utc>,
    
    /// Report period end
    pub period_end: DateTime<Utc>,
    
    /// Compliance frameworks covered
    pub frameworks: Vec<String>,
    
    /// Summary statistics
    pub summary: ComplianceAuditSummary,
    
    /// Detailed findings
    pub findings: Vec<ComplianceFinding>,
    
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Summary statistics for compliance audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditSummary {
    /// Total number of audit entries reviewed
    pub total_entries: u64,
    
    /// Number of compliant entries
    pub compliant_entries: u64,
    
    /// Number of non-compliant entries
    pub non_compliant_entries: u64,
    
    /// Number of security events
    pub security_events: u64,
    
    /// Number of access violations
    pub access_violations: u64,
    
    /// Compliance percentage
    pub compliance_percentage: f64,
}

/// Individual compliance finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    /// Finding identifier
    pub id: Uuid,
    
    /// Severity of the finding
    pub severity: SecuritySeverity,
    
    /// Compliance framework this relates to
    pub framework: String,
    
    /// Control or requirement that was violated
    pub control: String,
    
    /// Description of the finding
    pub description: String,
    
    /// Evidence supporting the finding
    pub evidence: Vec<String>,
    
    /// Recommended remediation
    pub remediation: String,
    
    /// Status of remediation
    pub status: String,
}

impl AuditInfo {
    /// Create new audit info for entity creation
    pub fn new(created_by: String) -> Self {
        let now = Utc::now();
        Self {
            created_by: created_by.clone(),
            updated_by: created_by,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Update audit info when entity is modified
    pub fn update(&mut self, updated_by: String) {
        self.updated_by = updated_by;
        self.updated_at = Utc::now();
    }
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(
        entity_type: String,
        entity_id: String,
        action: String,
        user_id: String,
        details: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type,
            entity_id,
            action,
            user_id,
            timestamp: Utc::now(),
            details,
            ip_address: None,
            user_agent: None,
        }
    }
    
    /// Create audit entry with network context
    pub fn with_network_context(
        entity_type: String,
        entity_id: String,
        action: String,
        user_id: String,
        details: serde_json::Value,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type,
            entity_id,
            action,
            user_id,
            timestamp: Utc::now(),
            details,
            ip_address,
            user_agent,
        }
    }
}

impl SecurityEvent {
    /// Create a new security event
    pub fn new(
        event_type: SecurityEventType,
        severity: SecuritySeverity,
        description: String,
        user_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            severity,
            user_id,
            ip_address: None,
            timestamp: Utc::now(),
            description,
            metadata: HashMap::new(),
            resolved: false,
            resolution: None,
        }
    }
    
    /// Mark security event as resolved
    pub fn resolve(&mut self, resolution: String) {
        self.resolved = true;
        self.resolution = Some(resolution);
    }
}

impl AccessLogEntry {
    /// Create a new access log entry
    pub fn new(
        user_id: String,
        resource_type: String,
        resource_id: String,
        access_type: String,
        access_granted: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            resource_type,
            resource_id,
            access_type,
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            access_granted,
            denial_reason: None,
            data_classification: None,
            context: HashMap::new(),
        }
    }
    
    /// Create access log entry with denial reason
    pub fn denied(
        user_id: String,
        resource_type: String,
        resource_id: String,
        access_type: String,
        denial_reason: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            resource_type,
            resource_id,
            access_type,
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            access_granted: false,
            denial_reason: Some(denial_reason),
            data_classification: None,
            context: HashMap::new(),
        }
    }
}

/// Trait for entities that can be audited
pub trait Auditable {
    /// Get the entity type for audit logging
    fn entity_type() -> String;
    
    /// Get the entity ID
    fn entity_id(&self) -> String;
    
    /// Create an audit entry for this entity
    fn create_audit_entry(&self, action: String, user_id: String, details: serde_json::Value) -> AuditEntry {
        AuditEntry::new(
            Self::entity_type(),
            self.entity_id(),
            action,
            user_id,
            details,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_info_creation() {
        let audit_info = AuditInfo::new("test_user".to_string());
        assert_eq!(audit_info.created_by, "test_user");
        assert_eq!(audit_info.updated_by, "test_user");
        assert!(audit_info.created_at <= Utc::now());
        assert!(audit_info.updated_at <= Utc::now());
    }
    
    #[test]
    fn test_audit_info_update() {
        let mut audit_info = AuditInfo::new("creator".to_string());
        let original_created_at = audit_info.created_at;
        
        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        audit_info.update("updater".to_string());
        
        assert_eq!(audit_info.created_by, "creator");
        assert_eq!(audit_info.updated_by, "updater");
        assert_eq!(audit_info.created_at, original_created_at);
        assert!(audit_info.updated_at > original_created_at);
    }
    
    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            "TestEntity".to_string(),
            "test-id".to_string(),
            "create".to_string(),
            "test_user".to_string(),
            serde_json::json!({"test": "data"}),
        );
        
        assert_eq!(entry.entity_type, "TestEntity");
        assert_eq!(entry.entity_id, "test-id");
        assert_eq!(entry.action, "create");
        assert_eq!(entry.user_id, "test_user");
    }
    
    #[test]
    fn test_security_event_resolution() {
        let mut event = SecurityEvent::new(
            SecurityEventType::UnauthorizedAccess,
            SecuritySeverity::High,
            "Unauthorized access attempt detected".to_string(),
            Some("suspicious_user".to_string()),
        );
        
        assert!(!event.resolved);
        assert!(event.resolution.is_none());
        
        event.resolve("Access blocked and user account suspended".to_string());
        
        assert!(event.resolved);
        assert!(event.resolution.is_some());
    }
    
    #[test]
    fn test_access_log_entry_denied() {
        let entry = AccessLogEntry::denied(
            "user123".to_string(),
            "ServerConfig".to_string(),
            "server-456".to_string(),
            "delete".to_string(),
            "Insufficient permissions".to_string(),
        );
        
        assert!(!entry.access_granted);
        assert_eq!(entry.denial_reason, Some("Insufficient permissions".to_string()));
    }
}
