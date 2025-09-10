use crate::detection::profiles::ApplicationProfile;
use crate::detection::detector::{ApplicationDetector, DetectionResult};
use crate::detection::registry::ManualRegistryManager;
use crate::detection::validator::{ConfigValidator, ConfigValidationResult};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Comprehensive application detection report generator
pub struct ReportGenerator {
    /// Application detector for discovery
    detector: ApplicationDetector,
    /// Configuration validator for analysis
    validator: ConfigValidator,
    /// Manual registry for custom applications
    registry: ManualRegistryManager,
}

/// Complete application detection report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectionReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Summary statistics
    pub summary: DetectionSummary,
    /// Detailed application profiles
    pub applications: Vec<ApplicationReport>,
    /// Overall recommendations
    pub recommendations: Vec<String>,
    /// Export timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Report metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportMetadata {
    /// Report version
    pub version: String,
    /// System information
    pub system_info: SystemInfo,
    /// Detection configuration used
    pub detection_config: DetectionConfig,
}

/// System information for the report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    /// OS version
    pub os_version: Option<String>,
    /// Architecture
    pub arch: String,
    /// Home directory path
    pub home_dir: Option<PathBuf>,
}

/// Detection configuration used for the report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectionConfig {
    /// Whether bundle lookup was used
    pub bundle_lookup_enabled: bool,
    /// Whether executable checks were used
    pub executable_checks_enabled: bool,
    /// Whether config checks were used
    pub config_checks_enabled: bool,
    /// Whether Spotlight search was used
    pub spotlight_enabled: bool,
    /// Manual applications included
    pub manual_applications_count: usize,
}

/// Summary statistics for the detection report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectionSummary {
    /// Total applications checked
    pub total_applications: usize,
    /// Applications detected as installed
    pub detected_applications: usize,
    /// Applications with valid configurations
    pub valid_configurations: usize,
    /// Applications with MCP servers found
    pub applications_with_servers: usize,
    /// Total MCP servers discovered
    pub total_mcp_servers: usize,
    /// Detection success rate (0.0 to 1.0)
    pub detection_rate: f64,
    /// Configuration validation rate (0.0 to 1.0)
    pub validation_rate: f64,
    /// Configuration format breakdown
    pub format_breakdown: HashMap<String, usize>,
    /// Category breakdown
    pub category_breakdown: HashMap<String, usize>,
}

/// Individual application report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplicationReport {
    /// Application profile
    pub profile: ApplicationProfile,
    /// Detection result
    pub detection: DetectionResult,
    /// Configuration validation result
    pub validation: Option<ConfigValidationResult>,
    /// Overall status
    pub status: ApplicationStatus,
    /// Recommendations for this application
    pub recommendations: Vec<String>,
}

/// Overall status of an application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApplicationStatus {
    /// Fully functional with MCP servers
    FullyFunctional,
    /// Installed but no MCP configuration
    InstalledNoConfig,
    /// Installed with invalid configuration
    InstalledInvalidConfig,
    /// Not installed
    NotInstalled,
    /// Manually registered but not verified
    ManuallyRegistered,
    /// Detection failed
    DetectionFailed,
}

/// Report export format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Human-readable text
    Text,
    /// Markdown format
    Markdown,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new() -> Result<Self> {
        let detector = ApplicationDetector::new()?;
        let validator = ConfigValidator::new()?;
        let registry = ManualRegistryManager::new();
        
        Ok(Self {
            detector,
            validator,
            registry,
        })
    }

    /// Generate a comprehensive detection report
    pub async fn generate_report(&mut self) -> Result<DetectionReport> {
        // Get all applications (built-in + manually registered)
        let built_in_apps = self.detector.get_registry().get_all_applications();
        let manual_apps = self.registry.get_all_applications();
        
        let mut all_applications = Vec::new();
        all_applications.extend(built_in_apps.into_iter().cloned());
        all_applications.extend(manual_apps.into_iter().cloned());

        // Detect all applications
        let mut application_reports = Vec::new();
        for app in &all_applications {
            let report = self.generate_application_report(app).await?;
            application_reports.push(report);
        }

        // Generate summary statistics
        let summary = self.generate_summary(&application_reports);

        // Generate overall recommendations
        let recommendations = self.generate_recommendations(&application_reports, &summary);

        // Create report metadata
        let metadata = self.generate_metadata(&all_applications).await?;

        Ok(DetectionReport {
            metadata,
            summary,
            applications: application_reports,
            recommendations,
            generated_at: chrono::Utc::now(),
        })
    }

    /// Generate report for a single application
    async fn generate_application_report(&mut self, app: &ApplicationProfile) -> Result<ApplicationReport> {
        // Detect the application
        let detection = self.detector.detect_application(&app.id).await?;

        // Validate configuration if detected
        let validation = if detection.detected {
            Some(self.validator.validate_application_config(app).await?)
        } else {
            None
        };

        // Determine overall status
        let status = self.determine_application_status(&detection, &validation);

        // Generate recommendations
        let recommendations = self.generate_application_recommendations(&detection, &validation, &status);

        Ok(ApplicationReport {
            profile: app.clone(),
            detection,
            validation,
            status,
            recommendations,
        })
    }

    /// Determine the overall status of an application
    fn determine_application_status(
        &self,
        detection: &DetectionResult,
        validation: &Option<ConfigValidationResult>,
    ) -> ApplicationStatus {
        if !detection.detected {
            return ApplicationStatus::NotInstalled;
        }

        match validation {
            Some(val_result) => {
                if val_result.is_valid && !val_result.mcp_servers.is_empty() {
                    ApplicationStatus::FullyFunctional
                } else if val_result.is_valid && val_result.mcp_servers.is_empty() {
                    ApplicationStatus::InstalledNoConfig
                } else {
                    ApplicationStatus::InstalledInvalidConfig
                }
            }
            None => ApplicationStatus::DetectionFailed,
        }
    }

    /// Generate recommendations for a specific application
    fn generate_application_recommendations(
        &self,
        detection: &DetectionResult,
        validation: &Option<ConfigValidationResult>,
        status: &ApplicationStatus,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        match status {
            ApplicationStatus::FullyFunctional => {
                recommendations.push("Application is fully configured and ready to use".to_string());
            }
            ApplicationStatus::InstalledNoConfig => {
                recommendations.push("Application is installed but has no MCP server configuration".to_string());
                recommendations.push("Consider adding MCP server configurations to enable AI functionality".to_string());
            }
            ApplicationStatus::InstalledInvalidConfig => {
                recommendations.push("Application is installed but has configuration issues".to_string());
                if let Some(val_result) = validation {
                    for message in &val_result.messages {
                        if let Some(suggestion) = &message.suggestion {
                            recommendations.push(suggestion.clone());
                        }
                    }
                }
            }
            ApplicationStatus::NotInstalled => {
                recommendations.push(format!("Consider installing {} to enable MCP functionality", detection.profile.name));
            }
            ApplicationStatus::ManuallyRegistered => {
                recommendations.push("Manually registered application - verify installation and configuration".to_string());
            }
            ApplicationStatus::DetectionFailed => {
                recommendations.push("Detection failed - check application installation and permissions".to_string());
            }
        }

        // Add detection-specific recommendations
        if detection.confidence < 0.8 {
            recommendations.push("Detection confidence is low - consider manual verification".to_string());
        }

        recommendations
    }

    /// Generate summary statistics
    fn generate_summary(&self, reports: &[ApplicationReport]) -> DetectionSummary {
        let total_applications = reports.len();
        let detected_applications = reports.iter().filter(|r| r.detection.detected).count();
        let valid_configurations = reports.iter()
            .filter(|r| r.validation.as_ref().is_some_and(|v| v.is_valid))
            .count();
        let applications_with_servers = reports.iter()
            .filter(|r| r.validation.as_ref().is_some_and(|v| !v.mcp_servers.is_empty()))
            .count();
        let total_mcp_servers = reports.iter()
            .map(|r| r.validation.as_ref().map_or(0, |v| v.mcp_servers.len()))
            .sum();

        let detection_rate = if total_applications > 0 {
            detected_applications as f64 / total_applications as f64
        } else {
            0.0
        };

        let validation_rate = if detected_applications > 0 {
            valid_configurations as f64 / detected_applications as f64
        } else {
            0.0
        };

        // Generate format breakdown
        let mut format_breakdown = HashMap::new();
        for report in reports {
            if let Some(validation) = &report.validation {
                if let Some(format) = &validation.detected_format {
                    let format_name = match format {
                        crate::detection::profiles::ConfigFormat::Json => "JSON",
                        crate::detection::profiles::ConfigFormat::Yaml => "YAML",
                        crate::detection::profiles::ConfigFormat::Toml => "TOML",
                        crate::detection::profiles::ConfigFormat::Plist => "Plist",
                        crate::detection::profiles::ConfigFormat::Custom(name) => name,
                    };
                    *format_breakdown.entry(format_name.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Generate category breakdown
        let mut category_breakdown = HashMap::new();
        for report in reports {
            let category = match &report.profile.metadata.category {
                crate::detection::profiles::ApplicationCategory::ChatClient => "ChatClient",
                crate::detection::profiles::ApplicationCategory::CodeEditor => "CodeEditor",
                crate::detection::profiles::ApplicationCategory::IDE => "IDE",
                crate::detection::profiles::ApplicationCategory::ProductivityTool => "ProductivityTool",
                crate::detection::profiles::ApplicationCategory::Other(name) => name,
            };
            *category_breakdown.entry(category.to_string()).or_insert(0) += 1;
        }

        DetectionSummary {
            total_applications,
            detected_applications,
            valid_configurations,
            applications_with_servers,
            total_mcp_servers,
            detection_rate,
            validation_rate,
            format_breakdown,
            category_breakdown,
        }
    }

    /// Generate overall recommendations
    fn generate_recommendations(&self, reports: &[ApplicationReport], summary: &DetectionSummary) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Detection rate recommendations
        if summary.detection_rate < 0.5 {
            recommendations.push("Low detection rate - consider installing more MCP-enabled applications".to_string());
        }

        // Configuration recommendations
        if summary.validation_rate < 0.7 {
            recommendations.push("Many applications have configuration issues - review MCP server setups".to_string());
        }

        // MCP server recommendations
        if summary.applications_with_servers == 0 {
            recommendations.push("No MCP servers found - configure MCP servers to enable AI functionality".to_string());
        } else if summary.applications_with_servers < summary.detected_applications / 2 {
            recommendations.push("Consider adding MCP server configurations to more applications".to_string());
        }

        // Specific application recommendations
        let not_installed: Vec<_> = reports.iter()
            .filter(|r| matches!(r.status, ApplicationStatus::NotInstalled))
            .map(|r| r.profile.name.as_str())
            .collect();

        if !not_installed.is_empty() && not_installed.len() <= 3 {
            recommendations.push(format!("Consider installing: {}", not_installed.join(", ")));
        }

        recommendations
    }

    /// Generate report metadata
    async fn generate_metadata(&self, _applications: &[ApplicationProfile]) -> Result<ReportMetadata> {
        let system_info = SystemInfo {
            os: std::env::consts::OS.to_string(),
            os_version: None, // Could be enhanced with actual OS version detection
            arch: std::env::consts::ARCH.to_string(),
            home_dir: dirs::home_dir(),
        };

        let manual_count = self.registry.get_all_applications().len();

        let detection_config = DetectionConfig {
            bundle_lookup_enabled: true, // Based on detector configuration
            executable_checks_enabled: true,
            config_checks_enabled: true,
            spotlight_enabled: true,
            manual_applications_count: manual_count,
        };

        Ok(ReportMetadata {
            version: "1.0.0".to_string(),
            system_info,
            detection_config,
        })
    }

    /// Export report in specified format
    pub fn export_report(&self, report: &DetectionReport, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(report)
                    .context("Failed to serialize report to JSON")
            }
            ExportFormat::Yaml => {
                serde_yaml::to_string(report)
                    .context("Failed to serialize report to YAML")
            }
            ExportFormat::Text => {
                Ok(self.format_text_report(report))
            }
            ExportFormat::Markdown => {
                Ok(self.format_markdown_report(report))
            }
        }
    }

    /// Format report as human-readable text
    fn format_text_report(&self, report: &DetectionReport) -> String {
        let mut output = String::new();
        
        output.push_str("=== MCP Control Lite - Application Detection Report ===\n\n");
        output.push_str(&format!("Generated: {}\n", report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str(&format!("System: {} {} ({})\n\n", report.metadata.system_info.os, 
                                report.metadata.system_info.os_version.as_deref().unwrap_or("Unknown"), 
                                report.metadata.system_info.arch));

        // Summary
        output.push_str("=== SUMMARY ===\n");
        output.push_str(&format!("Total Applications: {}\n", report.summary.total_applications));
        output.push_str(&format!("Detected: {} ({:.1}%)\n", 
                                report.summary.detected_applications,
                                report.summary.detection_rate * 100.0));
        output.push_str(&format!("Valid Configurations: {} ({:.1}%)\n", 
                                report.summary.valid_configurations,
                                report.summary.validation_rate * 100.0));
        output.push_str(&format!("Applications with MCP Servers: {}\n", report.summary.applications_with_servers));
        output.push_str(&format!("Total MCP Servers: {}\n\n", report.summary.total_mcp_servers));

        // Applications
        output.push_str("=== APPLICATIONS ===\n");
        for app_report in &report.applications {
            output.push_str(&format!("\n{} ({})\n", app_report.profile.name, app_report.profile.id));
            output.push_str(&format!("  Status: {:?}\n", app_report.status));
            output.push_str(&format!("  Detected: {}\n", app_report.detection.detected));
            
            if let Some(validation) = &app_report.validation {
                output.push_str(&format!("  Config Valid: {}\n", validation.is_valid));
                output.push_str(&format!("  MCP Servers: {}\n", validation.mcp_servers.len()));
            }
            
            if !app_report.recommendations.is_empty() {
                output.push_str("  Recommendations:\n");
                for rec in &app_report.recommendations {
                    output.push_str(&format!("    - {}\n", rec));
                }
            }
        }

        // Overall recommendations
        if !report.recommendations.is_empty() {
            output.push_str("\n=== RECOMMENDATIONS ===\n");
            for rec in &report.recommendations {
                output.push_str(&format!("- {}\n", rec));
            }
        }

        output
    }

    /// Format report as Markdown
    fn format_markdown_report(&self, report: &DetectionReport) -> String {
        let mut output = String::new();
        
        output.push_str("# MCP Control Lite - Application Detection Report\n\n");
        output.push_str(&format!("**Generated:** {}\n", report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str(&format!("**System:** {} {} ({})\n\n", report.metadata.system_info.os, 
                                report.metadata.system_info.os_version.as_deref().unwrap_or("Unknown"), 
                                report.metadata.system_info.arch));

        // Summary
        output.push_str("## Summary\n\n");
        output.push_str("| Metric | Value | Percentage |\n");
        output.push_str("|--------|-------|------------|\n");
        output.push_str(&format!("| Total Applications | {} | 100% |\n", report.summary.total_applications));
        output.push_str(&format!("| Detected | {} | {:.1}% |\n", 
                                report.summary.detected_applications,
                                report.summary.detection_rate * 100.0));
        output.push_str(&format!("| Valid Configurations | {} | {:.1}% |\n", 
                                report.summary.valid_configurations,
                                report.summary.validation_rate * 100.0));
        output.push_str(&format!("| Applications with MCP Servers | {} | - |\n", report.summary.applications_with_servers));
        output.push_str(&format!("| Total MCP Servers | {} | - |\n\n", report.summary.total_mcp_servers));

        // Applications
        output.push_str("## Applications\n\n");
        for app_report in &report.applications {
            output.push_str(&format!("### {} ({})\n\n", app_report.profile.name, app_report.profile.id));
            output.push_str(&format!("- **Status:** {:?}\n", app_report.status));
            output.push_str(&format!("- **Detected:** {}\n", app_report.detection.detected));
            
            if let Some(validation) = &app_report.validation {
                output.push_str(&format!("- **Config Valid:** {}\n", validation.is_valid));
                output.push_str(&format!("- **MCP Servers:** {}\n", validation.mcp_servers.len()));
            }
            
            if !app_report.recommendations.is_empty() {
                output.push_str("\n**Recommendations:**\n");
                for rec in &app_report.recommendations {
                    output.push_str(&format!("- {}\n", rec));
                }
            }
            output.push('\n');
        }

        // Overall recommendations
        if !report.recommendations.is_empty() {
            output.push_str("## Overall Recommendations\n\n");
            for rec in &report.recommendations {
                output.push_str(&format!("- {}\n", rec));
            }
        }

        output
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default ReportGenerator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::profiles::{ApplicationCategory, ApplicationMetadata, DetectionStrategy, DetectionMethod, ConfigFormat};
    use crate::detection::detector::DetectionPaths;
    use crate::detection::validator::{McpServerConfig, ServerType, ServerMetadata, ConfigSource};
    use std::collections::HashMap;

    fn create_test_application() -> ApplicationProfile {
        ApplicationProfile {
            id: "test-app".to_string(),
            name: "Test Application".to_string(),
            bundle_id: "com.test.app".to_string(),
            config_path: "~/test/config.json".to_string(),
            alt_config_paths: vec![],
            config_format: ConfigFormat::Json,
            executable_paths: vec!["/Applications/Test.app".to_string()],
            alt_executable_paths: vec![],
            detection_strategy: DetectionStrategy {
                use_bundle_lookup: true,
                use_executable_check: true,
                use_config_check: true,
                use_spotlight: false,
                priority_order: vec![DetectionMethod::ExecutableCheck],
            },
            metadata: ApplicationMetadata {
                version: Some("1.0.0".to_string()),
                developer: "Test Developer".to_string(),
                category: ApplicationCategory::CodeEditor,
                mcp_version: "1.0".to_string(),
                notes: None,
                requires_permissions: false,
            },
        }
    }

    fn create_test_detection_result(is_detected: bool) -> DetectionResult {
        DetectionResult {
            profile: create_test_application(),
            detected: is_detected,
            detection_method: if is_detected { Some(DetectionMethod::ExecutableCheck) } else { None },
            found_paths: DetectionPaths {
                executable: None,
                config_file: None,
                additional_paths: vec![],
            },
            confidence: if is_detected { 0.95 } else { 0.0 },
            messages: vec![],
            detected_at: chrono::Utc::now(),
        }
    }

    fn create_test_validation_result(is_valid: bool, server_count: usize) -> ConfigValidationResult {
        let mut mcp_servers = Vec::new();
        for i in 0..server_count {
            mcp_servers.push(McpServerConfig {
                name: format!("server-{}", i),
                command: Some("test-command".to_string()),
                args: vec![],
                env: HashMap::new(),
                cwd: None,
                server_type: ServerType::Stdio,
                metadata: ServerMetadata {
                    description: None,
                    version: None,
                    author: None,
                    capabilities: vec![],
                    enabled: true,
                    source: ConfigSource::MainConfig,
                },
            });
        }

        ConfigValidationResult {
            application: create_test_application(),
            is_valid,
            config_path: if is_valid { Some(std::path::PathBuf::from("/test/config.json")) } else { None },
            detected_format: if is_valid { Some(ConfigFormat::Json) } else { None },
            mcp_servers,
            messages: vec![],
            raw_config: None,
            validated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_report_generator_creation() {
        let generator = ReportGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn test_determine_application_status() {
        let generator = ReportGenerator::new().unwrap();

        // Not detected
        let detection = create_test_detection_result(false);
        let status = generator.determine_application_status(&detection, &None);
        assert_eq!(status, ApplicationStatus::NotInstalled);

        // Detected but no validation
        let detection = create_test_detection_result(true);
        let status = generator.determine_application_status(&detection, &None);
        assert_eq!(status, ApplicationStatus::DetectionFailed);

        // Detected with valid config and servers
        let detection = create_test_detection_result(true);
        let validation = create_test_validation_result(true, 2);
        let status = generator.determine_application_status(&detection, &Some(validation));
        assert_eq!(status, ApplicationStatus::FullyFunctional);

        // Detected with valid config but no servers
        let detection = create_test_detection_result(true);
        let validation = create_test_validation_result(true, 0);
        let status = generator.determine_application_status(&detection, &Some(validation));
        assert_eq!(status, ApplicationStatus::InstalledNoConfig);

        // Detected with invalid config
        let detection = create_test_detection_result(true);
        let validation = create_test_validation_result(false, 0);
        let status = generator.determine_application_status(&detection, &Some(validation));
        assert_eq!(status, ApplicationStatus::InstalledInvalidConfig);
    }

    #[test]
    fn test_generate_application_recommendations() {
        let generator = ReportGenerator::new().unwrap();

        // Fully functional
        let detection = create_test_detection_result(true);
        let validation = create_test_validation_result(true, 2);
        let status = ApplicationStatus::FullyFunctional;
        let recommendations = generator.generate_application_recommendations(&detection, &Some(validation), &status);
        assert!(!recommendations.is_empty());
        assert!(recommendations[0].contains("fully configured"));

        // Not installed
        let detection = create_test_detection_result(false);
        let status = ApplicationStatus::NotInstalled;
        let recommendations = generator.generate_application_recommendations(&detection, &None, &status);
        assert!(!recommendations.is_empty());
        assert!(recommendations[0].contains("Consider installing"));

        // Low confidence detection
        let mut detection = create_test_detection_result(true);
        detection.confidence = 0.5;
        let validation = create_test_validation_result(true, 1);
        let status = ApplicationStatus::FullyFunctional;
        let recommendations = generator.generate_application_recommendations(&detection, &Some(validation), &status);
        assert!(recommendations.iter().any(|r| r.contains("confidence is low")));
    }

    #[test]
    fn test_generate_summary() {
        let generator = ReportGenerator::new().unwrap();

        let reports = vec![
            ApplicationReport {
                profile: create_test_application(),
                detection: create_test_detection_result(true),
                validation: Some(create_test_validation_result(true, 2)),
                status: ApplicationStatus::FullyFunctional,
                recommendations: vec![],
            },
            ApplicationReport {
                profile: create_test_application(),
                detection: create_test_detection_result(false),
                validation: None,
                status: ApplicationStatus::NotInstalled,
                recommendations: vec![],
            },
        ];

        let summary = generator.generate_summary(&reports);
        assert_eq!(summary.total_applications, 2);
        assert_eq!(summary.detected_applications, 1);
        assert_eq!(summary.valid_configurations, 1);
        assert_eq!(summary.applications_with_servers, 1);
        assert_eq!(summary.total_mcp_servers, 2);
        assert_eq!(summary.detection_rate, 0.5);
        assert_eq!(summary.validation_rate, 1.0);
    }

    #[test]
    fn test_generate_recommendations() {
        let generator = ReportGenerator::new().unwrap();

        // Low detection rate scenario
        let reports = vec![
            ApplicationReport {
                profile: create_test_application(),
                detection: create_test_detection_result(false),
                validation: None,
                status: ApplicationStatus::NotInstalled,
                recommendations: vec![],
            },
            ApplicationReport {
                profile: create_test_application(),
                detection: create_test_detection_result(false),
                validation: None,
                status: ApplicationStatus::NotInstalled,
                recommendations: vec![],
            },
        ];

        let summary = generator.generate_summary(&reports);
        let recommendations = generator.generate_recommendations(&reports, &summary);
        assert!(recommendations.iter().any(|r| r.contains("Low detection rate")));

        // No MCP servers scenario
        let reports = vec![
            ApplicationReport {
                profile: create_test_application(),
                detection: create_test_detection_result(true),
                validation: Some(create_test_validation_result(true, 0)),
                status: ApplicationStatus::InstalledNoConfig,
                recommendations: vec![],
            },
        ];

        let summary = generator.generate_summary(&reports);
        let recommendations = generator.generate_recommendations(&reports, &summary);
        assert!(recommendations.iter().any(|r| r.contains("No MCP servers found")));
    }

    #[test]
    fn test_export_formats() {
        let generator = ReportGenerator::new().unwrap();
        
        let report = DetectionReport {
            metadata: ReportMetadata {
                version: "1.0.0".to_string(),
                system_info: SystemInfo {
                    os: "test".to_string(),
                    os_version: None,
                    arch: "test".to_string(),
                    home_dir: None,
                },
                detection_config: DetectionConfig {
                    bundle_lookup_enabled: true,
                    executable_checks_enabled: true,
                    config_checks_enabled: true,
                    spotlight_enabled: false,
                    manual_applications_count: 0,
                },
            },
            summary: DetectionSummary {
                total_applications: 1,
                detected_applications: 1,
                valid_configurations: 1,
                applications_with_servers: 1,
                total_mcp_servers: 1,
                detection_rate: 1.0,
                validation_rate: 1.0,
                format_breakdown: HashMap::new(),
                category_breakdown: HashMap::new(),
            },
            applications: vec![],
            recommendations: vec!["Test recommendation".to_string()],
            generated_at: chrono::Utc::now(),
        };

        // Test JSON export
        let json_export = generator.export_report(&report, ExportFormat::Json);
        assert!(json_export.is_ok());
        let json_content = json_export.unwrap();
        assert!(json_content.contains("Test recommendation"));

        // Test YAML export
        let yaml_export = generator.export_report(&report, ExportFormat::Yaml);
        assert!(yaml_export.is_ok());
        let yaml_content = yaml_export.unwrap();
        assert!(yaml_content.contains("Test recommendation"));

        // Test Text export
        let text_export = generator.export_report(&report, ExportFormat::Text);
        assert!(text_export.is_ok());
        let text_content = text_export.unwrap();
        assert!(text_content.contains("MCP Control Lite"));
        assert!(text_content.contains("Test recommendation"));

        // Test Markdown export
        let md_export = generator.export_report(&report, ExportFormat::Markdown);
        assert!(md_export.is_ok());
        let md_content = md_export.unwrap();
        assert!(md_content.contains("# MCP Control Lite"));
        assert!(md_content.contains("Test recommendation"));
    }

    #[test]
    fn test_application_status_serialization() {
        let statuses = vec![
            ApplicationStatus::FullyFunctional,
            ApplicationStatus::InstalledNoConfig,
            ApplicationStatus::InstalledInvalidConfig,
            ApplicationStatus::NotInstalled,
            ApplicationStatus::ManuallyRegistered,
            ApplicationStatus::DetectionFailed,
        ];

        for status in statuses {
            let serialized = serde_json::to_string(&status).unwrap();
            let deserialized: ApplicationStatus = serde_json::from_str(&serialized).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_export_format_options() {
        let formats = [
            ExportFormat::Json,
            ExportFormat::Yaml,
            ExportFormat::Text,
            ExportFormat::Markdown,
        ];

        // Ensure all formats are distinct
        for (i, format1) in formats.iter().enumerate() {
            for (j, format2) in formats.iter().enumerate() {
                if i != j {
                    assert_ne!(format1, format2);
                }
            }
        }
    }

    #[test]
    fn test_detection_summary_calculations() {
        let generator = ReportGenerator::new().unwrap();

        // Test with empty reports
        let empty_reports = vec![];
        let summary = generator.generate_summary(&empty_reports);
        assert_eq!(summary.total_applications, 0);
        assert_eq!(summary.detection_rate, 0.0);
        assert_eq!(summary.validation_rate, 0.0);

        // Test with mixed results
        let mixed_reports = vec![
            ApplicationReport {
                profile: create_test_application(),
                detection: create_test_detection_result(true),
                validation: Some(create_test_validation_result(true, 1)),
                status: ApplicationStatus::FullyFunctional,
                recommendations: vec![],
            },
            ApplicationReport {
                profile: create_test_application(),
                detection: create_test_detection_result(true),
                validation: Some(create_test_validation_result(false, 0)),
                status: ApplicationStatus::InstalledInvalidConfig,
                recommendations: vec![],
            },
            ApplicationReport {
                profile: create_test_application(),
                detection: create_test_detection_result(false),
                validation: None,
                status: ApplicationStatus::NotInstalled,
                recommendations: vec![],
            },
        ];

        let summary = generator.generate_summary(&mixed_reports);
        assert_eq!(summary.total_applications, 3);
        assert_eq!(summary.detected_applications, 2);
        assert_eq!(summary.valid_configurations, 1);
        assert_eq!(summary.applications_with_servers, 1);
        assert_eq!(summary.total_mcp_servers, 1);
        assert!((summary.detection_rate - 2.0/3.0).abs() < 0.01);
        assert_eq!(summary.validation_rate, 0.5);
    }

    #[test]
    fn test_format_breakdown() {
        let generator = ReportGenerator::new().unwrap();

        let mut app1 = create_test_application();
        app1.config_format = ConfigFormat::Json;
        let mut app2 = create_test_application();
        app2.config_format = ConfigFormat::Yaml;

        let reports = vec![
            ApplicationReport {
                profile: app1,
                detection: create_test_detection_result(true),
                validation: Some(create_test_validation_result(true, 1)),
                status: ApplicationStatus::FullyFunctional,
                recommendations: vec![],
            },
            ApplicationReport {
                profile: app2,
                detection: create_test_detection_result(true),
                validation: Some({
                    let mut val = create_test_validation_result(true, 1);
                    val.detected_format = Some(ConfigFormat::Yaml);
                    val
                }),
                status: ApplicationStatus::FullyFunctional,
                recommendations: vec![],
            },
        ];

        let summary = generator.generate_summary(&reports);
        assert_eq!(summary.format_breakdown.get("JSON"), Some(&1));
        assert_eq!(summary.format_breakdown.get("YAML"), Some(&1));
    }

    #[tokio::test]
    async fn test_generate_metadata() {
        let generator = ReportGenerator::new().unwrap();
        let apps = vec![create_test_application()];
        
        let metadata = generator.generate_metadata(&apps).await;
        assert!(metadata.is_ok());
        
        let metadata = metadata.unwrap();
        assert_eq!(metadata.version, "1.0.0");
        assert!(!metadata.system_info.os.is_empty());
        assert!(!metadata.system_info.arch.is_empty());
    }
}
