// Application Detection Module
// Provides comprehensive detection and profiling of MCP-enabled applications

pub mod profiles;
pub mod detector;
pub mod registry;
pub mod validator;
pub mod reporter;

pub use profiles::*;
pub use detector::{ApplicationDetector, DetectionResult, DetectionPaths, DetectionMessage as DetectorMessage, MessageLevel as DetectorMessageLevel};
pub use registry::*;
pub use validator::*;
pub use reporter::*;
