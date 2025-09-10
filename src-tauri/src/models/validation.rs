use regex::Regex;
use std::collections::HashMap;
use url::Url;
use uuid::Uuid;

/// Type alias for validation rule functions
pub type ValidationRule = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// Common validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Required field is missing or empty
    RequiredField(String),
    /// Field format is invalid
    InvalidFormat { field: String, reason: String },
    /// Field value is out of valid range
    OutOfRange { field: String, min: Option<f64>, max: Option<f64>, value: f64 },
    /// Field length is invalid
    InvalidLength { field: String, min: Option<usize>, max: Option<usize>, actual: usize },
    /// Field contains invalid characters
    InvalidCharacters { field: String, allowed: String },
    /// Field value is not in allowed set
    InvalidValue { field: String, allowed: Vec<String>, actual: String },
    /// Security requirement not met
    SecurityRequirement(String),
    /// Compliance requirement not met
    ComplianceRequirement(String),
    /// Custom validation error
    Custom(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::RequiredField(field) => write!(f, "Required field '{}' is missing or empty", field),
            ValidationError::InvalidFormat { field, reason } => write!(f, "Invalid format for field '{}': {}", field, reason),
            ValidationError::OutOfRange { field, min, max, value } => {
                write!(f, "Field '{}' value {} is out of range", field, value)?;
                if let Some(min) = min {
                    write!(f, " (min: {})", min)?;
                }
                if let Some(max) = max {
                    write!(f, " (max: {})", max)?;
                }
                Ok(())
            },
            ValidationError::InvalidLength { field, min, max, actual } => {
                write!(f, "Field '{}' length {} is invalid", field, actual)?;
                if let Some(min) = min {
                    write!(f, " (min: {})", min)?;
                }
                if let Some(max) = max {
                    write!(f, " (max: {})", max)?;
                }
                Ok(())
            },
            ValidationError::InvalidCharacters { field, allowed } => {
                write!(f, "Field '{}' contains invalid characters. Allowed: {}", field, allowed)
            },
            ValidationError::InvalidValue { field, allowed, actual } => {
                write!(f, "Field '{}' has invalid value '{}'. Allowed: {:?}", field, actual, allowed)
            },
            ValidationError::SecurityRequirement(req) => write!(f, "Security requirement not met: {}", req),
            ValidationError::ComplianceRequirement(req) => write!(f, "Compliance requirement not met: {}", req),
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Result type for validation operations
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Validation context for tracking validation state
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Current field path being validated
    pub field_path: Vec<String>,
    /// Accumulated errors
    pub errors: Vec<ValidationError>,
    /// Validation options
    pub options: ValidationOptions,
}

/// Options for validation behavior
pub struct ValidationOptions {
    /// Whether to stop on first error or collect all errors
    pub fail_fast: bool,
    /// Whether to validate security requirements
    pub validate_security: bool,
    /// Whether to validate compliance requirements
    pub validate_compliance: bool,
    /// Custom validation rules
    pub custom_rules: HashMap<String, ValidationRule>,
}

impl std::fmt::Debug for ValidationOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationOptions")
            .field("fail_fast", &self.fail_fast)
            .field("validate_security", &self.validate_security)
            .field("validate_compliance", &self.validate_compliance)
            .field("custom_rules_count", &self.custom_rules.len())
            .finish()
    }
}

impl Clone for ValidationOptions {
    fn clone(&self) -> Self {
        Self {
            fail_fast: self.fail_fast,
            validate_security: self.validate_security,
            validate_compliance: self.validate_compliance,
            custom_rules: HashMap::new(), // Cannot clone trait objects, so start with empty
        }
    }
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            fail_fast: false,
            validate_security: true,
            validate_compliance: true,
            custom_rules: HashMap::new(),
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new() -> Self {
        Self {
            field_path: Vec::new(),
            errors: Vec::new(),
            options: ValidationOptions::default(),
        }
    }
    
    /// Create validation context with options
    pub fn with_options(options: ValidationOptions) -> Self {
        Self {
            field_path: Vec::new(),
            errors: Vec::new(),
            options,
        }
    }
    
    /// Enter a field context
    pub fn enter_field(&mut self, field_name: &str) {
        self.field_path.push(field_name.to_string());
    }
    
    /// Exit current field context
    pub fn exit_field(&mut self) {
        self.field_path.pop();
    }
    
    /// Get current field path as string
    pub fn current_path(&self) -> String {
        self.field_path.join(".")
    }
    
    /// Add an error to the context
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }
    
    /// Check if validation should continue
    pub fn should_continue(&self) -> bool {
        !self.options.fail_fast || self.errors.is_empty()
    }
    
    /// Get validation result
    pub fn result(self) -> ValidationResult {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }
}

/// Trait for validatable entities
pub trait Validatable {
    /// Validate the entity
    fn validate(&self) -> ValidationResult {
        let mut ctx = ValidationContext::new();
        self.validate_with_context(&mut ctx);
        ctx.result()
    }
    
    /// Validate with custom context
    fn validate_with_context(&self, ctx: &mut ValidationContext);
}

/// Common validation functions
pub struct Validators;

impl Validators {
    /// Validate that a string is not empty
    pub fn not_empty(value: &str, field_name: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            Err(ValidationError::RequiredField(field_name.to_string()))
        } else {
            Ok(())
        }
    }
    
    /// Validate string length
    pub fn string_length(
        value: &str,
        field_name: &str,
        min: Option<usize>,
        max: Option<usize>,
    ) -> Result<(), ValidationError> {
        let len = value.len();
        
        if let Some(min_len) = min {
            if len < min_len {
                return Err(ValidationError::InvalidLength {
                    field: field_name.to_string(),
                    min: Some(min_len),
                    max,
                    actual: len,
                });
            }
        }
        
        if let Some(max_len) = max {
            if len > max_len {
                return Err(ValidationError::InvalidLength {
                    field: field_name.to_string(),
                    min,
                    max: Some(max_len),
                    actual: len,
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate numeric range
    pub fn numeric_range(
        value: f64,
        field_name: &str,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<(), ValidationError> {
        if let Some(min_val) = min {
            if value < min_val {
                return Err(ValidationError::OutOfRange {
                    field: field_name.to_string(),
                    min: Some(min_val),
                    max,
                    value,
                });
            }
        }
        
        if let Some(max_val) = max {
            if value > max_val {
                return Err(ValidationError::OutOfRange {
                    field: field_name.to_string(),
                    min,
                    max: Some(max_val),
                    value,
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate email format
    pub fn email(value: &str, field_name: &str) -> Result<(), ValidationError> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| ValidationError::Custom("Invalid email regex".to_string()))?;
        
        if !email_regex.is_match(value) {
            return Err(ValidationError::InvalidFormat {
                field: field_name.to_string(),
                reason: "Invalid email format".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Validate URL format
    pub fn url(value: &str, field_name: &str) -> Result<(), ValidationError> {
        Url::parse(value).map_err(|_| ValidationError::InvalidFormat {
            field: field_name.to_string(),
            reason: "Invalid URL format".to_string(),
        })?;
        
        Ok(())
    }
    
    /// Validate UUID format
    pub fn uuid(value: &str, field_name: &str) -> Result<(), ValidationError> {
        Uuid::parse_str(value).map_err(|_| ValidationError::InvalidFormat {
            field: field_name.to_string(),
            reason: "Invalid UUID format".to_string(),
        })?;
        
        Ok(())
    }
    
    /// Validate that value is in allowed set
    pub fn in_set(
        value: &str,
        field_name: &str,
        allowed: &[&str],
    ) -> Result<(), ValidationError> {
        if !allowed.contains(&value) {
            return Err(ValidationError::InvalidValue {
                field: field_name.to_string(),
                allowed: allowed.iter().map(|s| s.to_string()).collect(),
                actual: value.to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Validate alphanumeric characters only
    pub fn alphanumeric(value: &str, field_name: &str) -> Result<(), ValidationError> {
        if !value.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ValidationError::InvalidCharacters {
                field: field_name.to_string(),
                allowed: "alphanumeric characters, underscore, and hyphen".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Validate file path format
    pub fn file_path(value: &str, field_name: &str) -> Result<(), ValidationError> {
        // Basic path validation - no null bytes, reasonable length
        if value.contains('\0') {
            return Err(ValidationError::InvalidCharacters {
                field: field_name.to_string(),
                allowed: "valid file path characters (no null bytes)".to_string(),
            });
        }
        
        if value.len() > 4096 {
            return Err(ValidationError::InvalidLength {
                field: field_name.to_string(),
                min: None,
                max: Some(4096),
                actual: value.len(),
            });
        }
        
        Ok(())
    }
    
    /// Validate IP address format
    pub fn ip_address(value: &str, field_name: &str) -> Result<(), ValidationError> {
        use std::net::IpAddr;
        
        value.parse::<IpAddr>().map_err(|_| ValidationError::InvalidFormat {
            field: field_name.to_string(),
            reason: "Invalid IP address format".to_string(),
        })?;
        
        Ok(())
    }
    
    /// Validate port number
    pub fn port(value: u16, field_name: &str) -> Result<(), ValidationError> {
        if value == 0 {
            return Err(ValidationError::OutOfRange {
                field: field_name.to_string(),
                min: Some(1.0),
                max: Some(65535.0),
                value: value as f64,
            });
        }
        
        Ok(())
    }
    
    /// Validate timeout value (in seconds)
    pub fn timeout_seconds(value: u64, field_name: &str) -> Result<(), ValidationError> {
        // Reasonable timeout range: 1 second to 24 hours
        if value == 0 || value > 86400 {
            return Err(ValidationError::OutOfRange {
                field: field_name.to_string(),
                min: Some(1.0),
                max: Some(86400.0),
                value: value as f64,
            });
        }
        
        Ok(())
    }
    
    /// Validate that a collection is not empty
    pub fn not_empty_collection<T>(
        collection: &[T],
        field_name: &str,
    ) -> Result<(), ValidationError> {
        if collection.is_empty() {
            Err(ValidationError::RequiredField(field_name.to_string()))
        } else {
            Ok(())
        }
    }
    
    /// Validate collection size
    pub fn collection_size<T>(
        collection: &[T],
        field_name: &str,
        min: Option<usize>,
        max: Option<usize>,
    ) -> Result<(), ValidationError> {
        let len = collection.len();
        
        if let Some(min_len) = min {
            if len < min_len {
                return Err(ValidationError::InvalidLength {
                    field: field_name.to_string(),
                    min: Some(min_len),
                    max,
                    actual: len,
                });
            }
        }
        
        if let Some(max_len) = max {
            if len > max_len {
                return Err(ValidationError::InvalidLength {
                    field: field_name.to_string(),
                    min,
                    max: Some(max_len),
                    actual: len,
                });
            }
        }
        
        Ok(())
    }
}

/// Security-specific validators
pub struct SecurityValidators;

impl SecurityValidators {
    /// Validate password strength
    pub fn password_strength(password: &str, field_name: &str) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        
        if password.len() < 8 {
            errors.push("Password must be at least 8 characters long".to_string());
        }
        
        if !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }
        
        if !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }
        
        if !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one number".to_string());
        }
        
        if !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            errors.push("Password must contain at least one special character".to_string());
        }
        
        if !errors.is_empty() {
            return Err(ValidationError::SecurityRequirement(format!(
                "Password for field '{}' does not meet security requirements: {}",
                field_name,
                errors.join(", ")
            )));
        }
        
        Ok(())
    }
    
    /// Validate that sensitive data is encrypted
    pub fn encrypted_field(
        value: &str,
        field_name: &str,
        is_encrypted: bool,
    ) -> Result<(), ValidationError> {
        if !value.is_empty() && !is_encrypted {
            return Err(ValidationError::SecurityRequirement(format!(
                "Sensitive field '{}' must be encrypted",
                field_name
            )));
        }
        
        Ok(())
    }
    
    /// Validate access control permissions
    pub fn access_permissions(permissions: &str, field_name: &str) -> Result<(), ValidationError> {
        // Validate Unix-style permissions (e.g., "rwxr--r--")
        if permissions.len() != 9 {
            return Err(ValidationError::InvalidFormat {
                field: field_name.to_string(),
                reason: "Permissions must be 9 characters long (rwxrwxrwx format)".to_string(),
            });
        }
        
        for (i, c) in permissions.chars().enumerate() {
            let valid_chars = match i % 3 {
                0 => ['r', '-'], // read
                1 => ['w', '-'], // write
                2 => ['x', '-'], // execute
                _ => unreachable!(),
            };
            
            if !valid_chars.contains(&c) {
                return Err(ValidationError::InvalidFormat {
                    field: field_name.to_string(),
                    reason: format!("Invalid permission character '{}' at position {}", c, i),
                });
            }
        }
        
        Ok(())
    }
}

/// Compliance-specific validators
pub struct ComplianceValidators;

impl ComplianceValidators {
    /// Validate audit trail completeness
    pub fn audit_trail_complete(
        created_by: &str,
        updated_by: &str,
        field_name: &str,
    ) -> Result<(), ValidationError> {
        if created_by.is_empty() {
            return Err(ValidationError::ComplianceRequirement(format!(
                "Field '{}.created_by' is required for audit compliance",
                field_name
            )));
        }
        
        if updated_by.is_empty() {
            return Err(ValidationError::ComplianceRequirement(format!(
                "Field '{}.updated_by' is required for audit compliance",
                field_name
            )));
        }
        
        Ok(())
    }
    
    /// Validate data classification
    pub fn data_classification(classification: &str, field_name: &str) -> Result<(), ValidationError> {
        let valid_classifications = ["public", "internal", "confidential", "restricted"];
        
        if !valid_classifications.contains(&classification.to_lowercase().as_str()) {
            return Err(ValidationError::InvalidValue {
                field: field_name.to_string(),
                allowed: valid_classifications.iter().map(|s| s.to_string()).collect(),
                actual: classification.to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Validate consent status for GDPR compliance
    pub fn consent_status(
        user_consented: bool,
        consent_date: Option<chrono::DateTime<chrono::Utc>>,
        field_name: &str,
    ) -> Result<(), ValidationError> {
        if user_consented && consent_date.is_none() {
            return Err(ValidationError::ComplianceRequirement(format!(
                "Field '{}.consent_date' is required when user has consented",
                field_name
            )));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_not_empty_validation() {
        assert!(Validators::not_empty("test", "field").is_ok());
        assert!(Validators::not_empty("", "field").is_err());
        assert!(Validators::not_empty("   ", "field").is_err());
    }
    
    #[test]
    fn test_string_length_validation() {
        assert!(Validators::string_length("test", "field", Some(1), Some(10)).is_ok());
        assert!(Validators::string_length("", "field", Some(1), None).is_err());
        assert!(Validators::string_length("toolongstring", "field", None, Some(5)).is_err());
    }
    
    #[test]
    fn test_email_validation() {
        assert!(Validators::email("test@example.com", "email").is_ok());
        assert!(Validators::email("invalid-email", "email").is_err());
        assert!(Validators::email("@example.com", "email").is_err());
    }
    
    #[test]
    fn test_url_validation() {
        assert!(Validators::url("https://example.com", "url").is_ok());
        assert!(Validators::url("http://localhost:8080", "url").is_ok());
        assert!(Validators::url("not-a-url", "url").is_err());
    }
    
    #[test]
    fn test_password_strength() {
        assert!(SecurityValidators::password_strength("StrongP@ss1", "password").is_ok());
        assert!(SecurityValidators::password_strength("weak", "password").is_err());
        assert!(SecurityValidators::password_strength("NoNumbers!", "password").is_err());
    }
    
    #[test]
    fn test_validation_context() {
        let mut ctx = ValidationContext::new();
        ctx.enter_field("user");
        ctx.enter_field("email");
        
        assert_eq!(ctx.current_path(), "user.email");
        
        ctx.add_error(ValidationError::RequiredField("email".to_string()));
        assert_eq!(ctx.errors.len(), 1);
        
        let result = ctx.result();
        assert!(result.is_err());
    }
}
