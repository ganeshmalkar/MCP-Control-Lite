use serde::{Deserialize, Serialize};
use std::process::Child;

use crate::detection::McpServerConfig;

pub mod manager;
pub mod registry;
pub mod process;

pub use manager::ServerManager;
pub use registry::ServerRegistry;

/// Status of an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    /// Server is running and responsive
    Running,
    /// Server is stopped
    Stopped,
    /// Server encountered an error
    Error(String),
    /// Server status is unknown
    Unknown,
}

/// Information about a running server process
#[derive(Debug)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Server configuration
    pub config: McpServerConfig,
    /// Process handle (if local process)
    pub child: Option<Child>,
    /// Start time
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Server management operations result
#[derive(Debug, Clone)]
pub struct ServerOperationResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Server ID that was operated on
    pub server_id: String,
    /// Operation message
    pub message: String,
    /// Any errors that occurred
    pub errors: Vec<String>,
}

impl std::fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerStatus::Running => write!(f, "Running"),
            ServerStatus::Stopped => write!(f, "Stopped"),
            ServerStatus::Error(msg) => write!(f, "Error: {}", msg),
            ServerStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for ServerOperationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(f, "✓ {}: {}", self.server_id, self.message)
        } else {
            write!(f, "✗ {}: {}", self.server_id, self.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_display() {
        assert_eq!(ServerStatus::Running.to_string(), "Running");
        assert_eq!(ServerStatus::Stopped.to_string(), "Stopped");
        assert_eq!(ServerStatus::Error("test error".to_string()).to_string(), "Error: test error");
        assert_eq!(ServerStatus::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_server_operation_result_display() {
        let success_result = ServerOperationResult {
            success: true,
            server_id: "test-server".to_string(),
            message: "Started successfully".to_string(),
            errors: vec![],
        };
        assert_eq!(success_result.to_string(), "✓ test-server: Started successfully");

        let error_result = ServerOperationResult {
            success: false,
            server_id: "test-server".to_string(),
            message: "Failed to start".to_string(),
            errors: vec!["Command not found".to_string()],
        };
        assert_eq!(error_result.to_string(), "✗ test-server: Failed to start");
    }
}
