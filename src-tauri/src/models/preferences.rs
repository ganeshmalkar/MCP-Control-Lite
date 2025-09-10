use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{ComplianceModel, ComplianceResult};
use crate::models::audit::AuditInfo;
use crate::models::validation::{Validatable, ValidationContext, Validators};

/// User preferences for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Unique identifier
    pub id: Uuid,
    
    /// User ID these preferences belong to
    pub user_id: String,
    
    /// Auto-sync configurations
    pub auto_sync: bool,
    
    /// Backup before making changes
    pub backup_before_changes: bool,
    
    /// Backup location
    pub backup_location: String,
    
    /// Check for updates automatically
    pub check_for_updates: bool,
    
    /// Application theme
    pub theme: Theme,
    
    /// Start application at login
    pub start_at_login: bool,
    
    /// Show menu bar icon
    pub menu_bar_icon: bool,
    
    /// Favorite server IDs
    pub favorite_servers: Vec<Uuid>,
    
    /// Notification preferences
    pub notifications: NotificationPreferences,
    
    /// Security preferences
    pub security_preferences: SecurityPreferences,
    
    /// Accessibility preferences
    pub accessibility_preferences: AccessibilityPreferences,
    
    /// Audit information
    pub audit_info: AuditInfo,
}

/// Application theme options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Notify on updates
    pub on_update: bool,
    
    /// Notify on errors
    pub on_error: bool,
    
    /// Notify on sync completion
    pub on_sync: bool,
    
    /// Notify on server status changes
    pub on_server_status_change: bool,
}

/// Security-related preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPreferences {
    /// Session timeout in minutes
    pub session_timeout_minutes: u32,
    
    /// Require password for sensitive operations
    pub require_password_for_sensitive_operations: bool,
    
    /// Audit logging level
    pub audit_level: AuditLevel,
    
    /// Encryption key rotation interval in days
    pub encryption_key_rotation_days: Option<u32>,
    
    /// Auto-lock after inactivity
    pub auto_lock_minutes: Option<u32>,
}

/// Audit logging levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditLevel {
    Minimal,
    Standard,
    Verbose,
}

/// Accessibility preferences for WCAG 2.1 compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityPreferences {
    /// High contrast mode
    pub high_contrast: bool,
    
    /// Font size preference
    pub font_size: FontSize,
    
    /// Reduce motion animations
    pub reduce_motion: bool,
    
    /// Screen reader compatibility mode
    pub screen_reader_compatible: bool,
    
    /// Keyboard navigation only
    pub keyboard_navigation: bool,
    
    /// Focus indicators
    pub enhanced_focus_indicators: bool,
    
    /// Color blind friendly mode
    pub color_blind_friendly: bool,
}

/// Font size options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FontSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: String::new(),
            auto_sync: true,
            backup_before_changes: true,
            backup_location: "~/Documents/MCP-Control-Backups".to_string(),
            check_for_updates: true,
            theme: Theme::System,
            start_at_login: false,
            menu_bar_icon: true,
            favorite_servers: Vec::new(),
            notifications: NotificationPreferences::default(),
            security_preferences: SecurityPreferences::default(),
            accessibility_preferences: AccessibilityPreferences::default(),
            audit_info: AuditInfo::new("system".to_string()),
        }
    }
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            on_update: true,
            on_error: true,
            on_sync: false,
            on_server_status_change: true,
        }
    }
}

impl Default for SecurityPreferences {
    fn default() -> Self {
        Self {
            session_timeout_minutes: 60, // 1 hour
            require_password_for_sensitive_operations: true,
            audit_level: AuditLevel::Standard,
            encryption_key_rotation_days: Some(90), // 90 days
            auto_lock_minutes: Some(15), // 15 minutes
        }
    }
}

impl Default for AccessibilityPreferences {
    fn default() -> Self {
        Self {
            high_contrast: false,
            font_size: FontSize::Medium,
            reduce_motion: false,
            screen_reader_compatible: false,
            keyboard_navigation: false,
            enhanced_focus_indicators: false,
            color_blind_friendly: false,
        }
    }
}

impl UserPreferences {
    /// Create new user preferences
    pub fn new(user_id: String) -> Self {
        Self {
            user_id: user_id.clone(),
            audit_info: AuditInfo::new(user_id),
            ..Default::default()
        }
    }
}

impl Validatable for UserPreferences {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate user ID
        ctx.enter_field("user_id");
        if let Err(e) = Validators::not_empty(&self.user_id, "user_id") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate backup location
        ctx.enter_field("backup_location");
        if let Err(e) = Validators::not_empty(&self.backup_location, "backup_location") {
            ctx.add_error(e);
        }
        if let Err(e) = Validators::file_path(&self.backup_location, "backup_location") {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate security preferences
        ctx.enter_field("security_preferences");
        self.security_preferences.validate_with_context(ctx);
        ctx.exit_field();
        
        // Validate accessibility preferences
        ctx.enter_field("accessibility_preferences");
        self.accessibility_preferences.validate_with_context(ctx);
        ctx.exit_field();
    }
}

impl Validatable for SecurityPreferences {
    fn validate_with_context(&self, ctx: &mut ValidationContext) {
        // Validate session timeout
        ctx.enter_field("session_timeout_minutes");
        if let Err(e) = Validators::numeric_range(
            self.session_timeout_minutes as f64,
            "session_timeout_minutes",
            Some(5.0),   // Minimum 5 minutes
            Some(1440.0), // Maximum 24 hours
        ) {
            ctx.add_error(e);
        }
        ctx.exit_field();
        
        // Validate key rotation interval
        if let Some(rotation_days) = self.encryption_key_rotation_days {
            ctx.enter_field("encryption_key_rotation_days");
            if let Err(e) = Validators::numeric_range(
                rotation_days as f64,
                "encryption_key_rotation_days",
                Some(1.0),   // Minimum 1 day
                Some(365.0), // Maximum 1 year
            ) {
                ctx.add_error(e);
            }
            ctx.exit_field();
        }
        
        // Validate auto-lock timeout
        if let Some(auto_lock) = self.auto_lock_minutes {
            ctx.enter_field("auto_lock_minutes");
            if let Err(e) = Validators::numeric_range(
                auto_lock as f64,
                "auto_lock_minutes",
                Some(1.0),   // Minimum 1 minute
                Some(120.0), // Maximum 2 hours
            ) {
                ctx.add_error(e);
            }
            ctx.exit_field();
        }
    }
}

impl Validatable for AccessibilityPreferences {
    fn validate_with_context(&self, _ctx: &mut ValidationContext) {
        // All accessibility preferences are boolean or enum values
        // No additional validation needed beyond type checking
    }
}

impl ComplianceModel for UserPreferences {
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
            entity_type: "UserPreferences".to_string(),
            entity_id: self.id.to_string(),
            action: "created".to_string(),
            user_id: self.audit_info.created_by.clone(),
            timestamp: self.audit_info.created_at,
            details: serde_json::json!({
                "user_id": self.user_id,
                "theme": self.theme,
                "auto_sync": self.auto_sync,
                "accessibility_enabled": self.accessibility_preferences.screen_reader_compatible || 
                                       self.accessibility_preferences.high_contrast ||
                                       self.accessibility_preferences.keyboard_navigation
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
    fn test_user_preferences_creation() {
        let prefs = UserPreferences::new("test_user".to_string());
        
        assert_eq!(prefs.user_id, "test_user");
        assert!(prefs.auto_sync);
        assert_eq!(prefs.theme, Theme::System);
    }
    
    #[test]
    fn test_user_preferences_validation() {
        let prefs = UserPreferences::new("valid_user".to_string());
        assert!(prefs.validate().is_ok());
    }
    
    #[test]
    fn test_security_preferences_validation() {
        let mut security_prefs = SecurityPreferences::default();
        
        // Valid timeout
        assert!(security_prefs.validate().is_ok());
        
        // Invalid timeout (too short)
        security_prefs.session_timeout_minutes = 1;
        assert!(security_prefs.validate().is_err());
    }
}
