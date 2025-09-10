use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{ComplianceModel, ComplianceResult, DataClassification};
use crate::models::security::{AccessControl, EncryptionSettings};
use crate::models::audit::AuditInfo;

/// Represents the state of an MCP session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    /// Session is being initialized
    Initializing,
    /// Session is active and ready for communication
    Active,
    /// Session is temporarily disconnected but can be resumed
    Disconnected,
    /// Session is being terminated
    Terminating,
    /// Session has been terminated
    Terminated,
    /// Session encountered an error
    Error(String),
}

/// Type of MCP session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionType {
    /// Direct client-server session
    Direct,
    /// Proxied session through another service
    Proxied,
    /// Shared session with multiple clients
    Shared,
}

/// MCP client session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: Uuid,
    
    /// Associated server configuration
    pub server_id: Uuid,
    
    /// Session type
    pub session_type: SessionType,
    
    /// Current session state
    pub state: SessionState,
    
    /// Session start time
    pub started_at: DateTime<Utc>,
    
    /// Session end time (if terminated)
    pub ended_at: Option<DateTime<Utc>>,
    
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    
    /// Session timeout in seconds
    pub timeout_seconds: u64,
    
    /// Client information
    pub client_info: ClientInfo,
    
    /// Session capabilities negotiated with server
    pub negotiated_capabilities: SessionCapabilities,
    
    /// Active tool calls in this session
    pub active_tool_calls: HashMap<Uuid, ToolCall>,
    
    /// Session statistics
    pub statistics: SessionStatistics,
    
    /// Security and access control
    pub access_control: AccessControl,
    
    /// Data classification for session data
    pub data_classification: DataClassification,
    
    /// Encryption settings for session communication
    pub encryption: EncryptionSettings,
    
    /// Audit information
    pub audit_info: AuditInfo,
    
    /// Session-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Information about the MCP client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name/identifier
    pub name: String,
    
    /// Client version
    pub version: String,
    
    /// Client user agent
    pub user_agent: Option<String>,
    
    /// Client IP address (if network connection)
    pub ip_address: Option<String>,
    
    /// Client process ID (if local connection)
    pub process_id: Option<u32>,
    
    /// Additional client metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Capabilities negotiated for this session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCapabilities {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    
    /// Supported content types
    pub supported_content_types: Vec<String>,
    
    /// Whether streaming is supported
    pub supports_streaming: bool,
    
    /// Whether progress notifications are supported
    pub supports_progress: bool,
    
    /// Whether cancellation is supported
    pub supports_cancellation: bool,
    
    /// Protocol version being used
    pub protocol_version: String,
    
    /// Additional capability flags
    pub extensions: HashMap<String, serde_json::Value>,
}

/// Statistics for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    /// Total messages sent
    pub messages_sent: u64,
    
    /// Total messages received
    pub messages_received: u64,
    
    /// Total bytes sent
    pub bytes_sent: u64,
    
    /// Total bytes received
    pub bytes_received: u64,
    
    /// Number of tool calls made
    pub tool_calls_made: u64,
    
    /// Number of successful tool calls
    pub tool_calls_successful: u64,
    
    /// Number of failed tool calls
    pub tool_calls_failed: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Number of errors encountered
    pub error_count: u64,
    
    /// Last error message
    pub last_error: Option<String>,
}

/// Represents an active tool call within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique call identifier
    pub id: Uuid,
    
    /// Tool name being called
    pub tool_name: String,
    
    /// Tool arguments
    pub arguments: serde_json::Value,
    
    /// Call start time
    pub started_at: DateTime<Utc>,
    
    /// Call completion time
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Current call state
    pub state: ToolCallState,
    
    /// Progress information (0.0 to 1.0)
    pub progress: Option<f64>,
    
    /// Progress message
    pub progress_message: Option<String>,
    
    /// Call result (if completed)
    pub result: Option<serde_json::Value>,
    
    /// Error information (if failed)
    pub error: Option<String>,
    
    /// Data classification for this tool call
    pub data_classification: DataClassification,
    
    /// Whether this call was approved by user
    pub user_approved: bool,
}

/// State of a tool call
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolCallState {
    /// Call is pending user approval
    PendingApproval,
    /// Call is being executed
    Executing,
    /// Call completed successfully
    Completed,
    /// Call failed with error
    Failed,
    /// Call was cancelled
    Cancelled,
}

/// Session event for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    /// Event ID
    pub id: Uuid,
    
    /// Session ID
    pub session_id: Uuid,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Event type
    pub event_type: SessionEventType,
    
    /// Event details
    pub details: serde_json::Value,
    
    /// User who triggered the event
    pub user_id: Option<String>,
}

/// Types of session events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEventType {
    /// Session was created
    SessionCreated,
    /// Session state changed
    StateChanged,
    /// Tool call was made
    ToolCallMade,
    /// Tool call completed
    ToolCallCompleted,
    /// Tool call failed
    ToolCallFailed,
    /// Message sent
    MessageSent,
    /// Message received
    MessageReceived,
    /// Error occurred
    ErrorOccurred,
    /// Session terminated
    SessionTerminated,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            server_id: Uuid::new_v4(),
            session_type: SessionType::Direct,
            state: SessionState::Initializing,
            started_at: Utc::now(),
            ended_at: None,
            last_activity: Utc::now(),
            timeout_seconds: 3600, // 1 hour default
            client_info: ClientInfo::default(),
            negotiated_capabilities: SessionCapabilities::default(),
            active_tool_calls: HashMap::new(),
            statistics: SessionStatistics::default(),
            access_control: AccessControl::new("system"),
            data_classification: DataClassification::Internal,
            encryption: EncryptionSettings::new("default_key"),
            audit_info: AuditInfo::new("system".to_string()),
            metadata: HashMap::new(),
        }
    }
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            name: "Unknown Client".to_string(),
            version: "0.0.0".to_string(),
            user_agent: None,
            ip_address: None,
            process_id: None,
            metadata: HashMap::new(),
        }
    }
}

impl Default for SessionCapabilities {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB default
            supported_content_types: vec![
                "application/json".to_string(),
                "text/plain".to_string(),
            ],
            supports_streaming: false,
            supports_progress: true,
            supports_cancellation: true,
            protocol_version: "2024-11-05".to_string(),
            extensions: HashMap::new(),
        }
    }
}

impl Default for SessionStatistics {
    fn default() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            tool_calls_made: 0,
            tool_calls_successful: 0,
            tool_calls_failed: 0,
            avg_response_time_ms: 0.0,
            error_count: 0,
            last_error: None,
        }
    }
}

impl ComplianceModel for Session {
    fn validate_compliance(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate session has valid server ID
        if self.server_id.is_nil() {
            errors.push("Session must have a valid server ID".to_string());
        }
        
        // Validate client information
        if self.client_info.name.trim().is_empty() {
            errors.push("Client name cannot be empty".to_string());
        }
        
        // Validate timeout is reasonable
        if self.timeout_seconds == 0 || self.timeout_seconds > 86400 {
            errors.push("Session timeout must be between 1 second and 24 hours".to_string());
        }
        
        // Validate access control
        self.access_control.validate_compliance()
            .unwrap_or_else(|errs| errors.extend(errs));
        
        // Validate encryption for sensitive sessions
        if matches!(self.data_classification, DataClassification::Confidential | DataClassification::Restricted) {
            self.encryption.validate_compliance()
                .unwrap_or_else(|errs| errors.extend(errs));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    fn get_compliance_status(&self) -> ComplianceResult {
        match self.validate_compliance() {
            Ok(_) => ComplianceResult::Compliant,
            Err(_) => ComplianceResult::NonCompliant,
        }
    }
    
    fn get_audit_trail(&self) -> Vec<crate::models::audit::AuditEntry> {
        // This would typically query session events from database
        vec![crate::models::audit::AuditEntry {
            id: Uuid::new_v4(),
            entity_type: "Session".to_string(),
            entity_id: self.id.to_string(),
            action: "created".to_string(),
            user_id: self.audit_info.created_by.clone(),
            timestamp: self.audit_info.created_at,
            details: serde_json::json!({
                "server_id": self.server_id,
                "session_type": self.session_type,
                "client_name": self.client_info.name,
                "data_classification": self.data_classification
            }),
            ip_address: self.client_info.ip_address.clone(),
            user_agent: self.client_info.user_agent.clone(),
        }]
    }
}

impl Session {
    /// Create a new session
    pub fn new(server_id: Uuid, client_info: ClientInfo, created_by: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            server_id,
            client_info,
            audit_info: AuditInfo::new(created_by),
            ..Default::default()
        }
    }
    
    /// Update session state
    pub fn update_state(&mut self, state: SessionState, updated_by: String) {
        self.state = state;
        self.last_activity = Utc::now();
        self.audit_info.update(updated_by);
        
        if matches!(self.state, SessionState::Terminated) {
            self.ended_at = Some(Utc::now());
        }
    }
    
    /// Check if session has timed out
    pub fn is_timed_out(&self) -> bool {
        let timeout_duration = chrono::Duration::seconds(self.timeout_seconds as i64);
        Utc::now() - self.last_activity > timeout_duration
    }
    
    /// Add a tool call to the session
    pub fn add_tool_call(&mut self, tool_call: ToolCall) {
        self.active_tool_calls.insert(tool_call.id, tool_call);
        self.statistics.tool_calls_made += 1;
        self.last_activity = Utc::now();
    }
    
    /// Complete a tool call
    pub fn complete_tool_call(&mut self, call_id: Uuid, result: serde_json::Value) {
        if let Some(call) = self.active_tool_calls.get_mut(&call_id) {
            call.state = ToolCallState::Completed;
            call.completed_at = Some(Utc::now());
            call.result = Some(result);
            self.statistics.tool_calls_successful += 1;
        }
        self.last_activity = Utc::now();
    }
    
    /// Fail a tool call
    pub fn fail_tool_call(&mut self, call_id: Uuid, error: String) {
        if let Some(call) = self.active_tool_calls.get_mut(&call_id) {
            call.state = ToolCallState::Failed;
            call.completed_at = Some(Utc::now());
            call.error = Some(error);
            self.statistics.tool_calls_failed += 1;
        }
        self.last_activity = Utc::now();
    }
    
    /// Update session statistics
    pub fn update_statistics(&mut self, bytes_sent: u64, bytes_received: u64, response_time_ms: u64) {
        self.statistics.bytes_sent += bytes_sent;
        self.statistics.bytes_received += bytes_received;
        
        if bytes_sent > 0 {
            self.statistics.messages_sent += 1;
        }
        if bytes_received > 0 {
            self.statistics.messages_received += 1;
        }
        
        // Update average response time
        let total_messages = self.statistics.messages_sent + self.statistics.messages_received;
        if total_messages > 0 {
            self.statistics.avg_response_time_ms = 
                (self.statistics.avg_response_time_ms * (total_messages - 1) as f64 + response_time_ms as f64) / total_messages as f64;
        }
        
        self.last_activity = Utc::now();
    }
    
    /// Get session duration
    pub fn duration(&self) -> chrono::Duration {
        match self.ended_at {
            Some(end) => end - self.started_at,
            None => Utc::now() - self.started_at,
        }
    }
    
    /// Get active tool call count
    pub fn active_tool_call_count(&self) -> usize {
        self.active_tool_calls.len()
    }
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(tool_name: String, arguments: serde_json::Value, data_classification: DataClassification) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool_name,
            arguments,
            started_at: Utc::now(),
            completed_at: None,
            state: ToolCallState::PendingApproval,
            progress: None,
            progress_message: None,
            result: None,
            error: None,
            data_classification,
            user_approved: false,
        }
    }
    
    /// Update tool call progress
    pub fn update_progress(&mut self, progress: f64, message: Option<String>) {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self.progress_message = message;
    }
    
    /// Get call duration
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.completed_at.map(|end| end - self.started_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_creation() {
        let client_info = ClientInfo {
            name: "Test Client".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };
        
        let session = Session::new(
            Uuid::new_v4(),
            client_info,
            "test_user".to_string(),
        );
        
        assert_eq!(session.state, SessionState::Initializing);
        assert_eq!(session.client_info.name, "Test Client");
        assert!(session.active_tool_calls.is_empty());
    }
    
    #[test]
    fn test_session_timeout() {
        let session = Session { 
            timeout_seconds: 1, 
            last_activity: Utc::now() - chrono::Duration::seconds(2), 
            ..Default::default() 
        };
        
        assert!(session.is_timed_out());
    }
    
    #[test]
    fn test_tool_call_lifecycle() {
        let mut session = Session::default();
        let tool_call = ToolCall::new(
            "test_tool".to_string(),
            serde_json::json!({"param": "value"}),
            DataClassification::Internal,
        );
        let call_id = tool_call.id;
        
        session.add_tool_call(tool_call);
        assert_eq!(session.statistics.tool_calls_made, 1);
        
        session.complete_tool_call(call_id, serde_json::json!({"result": "success"}));
        assert_eq!(session.statistics.tool_calls_successful, 1);
        
        if let Some(call) = session.active_tool_calls.get(&call_id) {
            assert_eq!(call.state, ToolCallState::Completed);
        }
    }
}
