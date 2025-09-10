//! Compliance-related data structures for SOC2, HIPAA, and WCAG 2.1

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Simple compliance result enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplianceResult {
    Compliant,
    NonCompliant,
    PartiallyCompliant,
    Unknown,
}

/// Overall compliance status for any entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub soc2_compliant: bool,
    pub hipaa_compliant: bool,
    pub wcag_compliant: bool,
    pub compliance_gaps: Option<Vec<String>>,
    pub last_assessed: DateTime<Utc>,
}

impl ComplianceStatus {
    pub fn new() -> Self {
        Self {
            soc2_compliant: false,
            hipaa_compliant: false,
            wcag_compliant: false,
            compliance_gaps: None,
            last_assessed: Utc::now(),
        }
    }
    
    pub fn is_fully_compliant(&self) -> bool {
        self.soc2_compliant && self.hipaa_compliant && self.wcag_compliant
    }
    
    pub fn get_result(&self) -> ComplianceResult {
        if self.is_fully_compliant() {
            ComplianceResult::Compliant
        } else if self.soc2_compliant || self.hipaa_compliant || self.wcag_compliant {
            ComplianceResult::PartiallyCompliant
        } else {
            ComplianceResult::NonCompliant
        }
    }
    
    pub fn add_gap(&mut self, gap: String) {
        if let Some(ref mut gaps) = self.compliance_gaps {
            gaps.push(gap);
        } else {
            self.compliance_gaps = Some(vec![gap]);
        }
    }
}

impl Default for ComplianceStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Security issue tracking for compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub severity: SecuritySeverity,
    pub description: String,
    pub remediation: String,
    pub affected_component: String,
    pub discovered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl SecuritySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecuritySeverity::Low => "low",
            SecuritySeverity::Medium => "medium",
            SecuritySeverity::High => "high",
            SecuritySeverity::Critical => "critical",
        }
    }
}

/// Configuration validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidation {
    pub is_valid: bool,
    pub errors: Option<Vec<String>>,
    pub warnings: Option<Vec<String>>,
    pub security_issues: Option<Vec<SecurityIssue>>,
    pub compliance_status: Option<ComplianceStatus>,
    pub validated_at: DateTime<Utc>,
}

impl ConfigValidation {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: None,
            warnings: None,
            security_issues: None,
            compliance_status: None,
            validated_at: Utc::now(),
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        if let Some(ref mut errors) = self.errors {
            errors.push(error);
        } else {
            self.errors = Some(vec![error]);
        }
    }
    
    pub fn add_warning(&mut self, warning: String) {
        if let Some(ref mut warnings) = self.warnings {
            warnings.push(warning);
        } else {
            self.warnings = Some(vec![warning]);
        }
    }
    
    pub fn add_security_issue(&mut self, issue: SecurityIssue) {
        if let Some(ref mut issues) = self.security_issues {
            issues.push(issue);
        } else {
            self.security_issues = Some(vec![issue]);
        }
    }
}

impl Default for ConfigValidation {
    fn default() -> Self {
        Self::new()
    }
}

/// User consent tracking for privacy compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentStatus {
    pub user_consented: bool,
    pub consent_date: Option<DateTime<Utc>>,
    pub consent_version: String,
    pub data_usage_purposes: Vec<String>,
    pub data_retention_period_days: Option<u32>,
    pub withdrawal_date: Option<DateTime<Utc>>,
}

impl ConsentStatus {
    pub fn new(version: &str) -> Self {
        Self {
            user_consented: false,
            consent_date: None,
            consent_version: version.to_string(),
            data_usage_purposes: Vec::new(),
            data_retention_period_days: None,
            withdrawal_date: None,
        }
    }
    
    pub fn grant_consent(&mut self, purposes: Vec<String>, retention_days: Option<u32>) {
        self.user_consented = true;
        self.consent_date = Some(Utc::now());
        self.data_usage_purposes = purposes;
        self.data_retention_period_days = retention_days;
        self.withdrawal_date = None;
    }
    
    pub fn withdraw_consent(&mut self) {
        self.user_consented = false;
        self.withdrawal_date = Some(Utc::now());
    }
    
    pub fn is_valid(&self) -> bool {
        self.user_consented && self.withdrawal_date.is_none()
    }
}
