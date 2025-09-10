use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

/// Known MCP application configurations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum McpApplication {
    ClaudeDesktop,
    Cursor,
    Zed,
    VSCode,
    Custom(String),
}

impl McpApplication {
    /// Get the display name for the application
    pub fn display_name(&self) -> &str {
        match self {
            McpApplication::ClaudeDesktop => "Claude Desktop",
            McpApplication::Cursor => "Cursor",
            McpApplication::Zed => "Zed",
            McpApplication::VSCode => "VS Code",
            McpApplication::Custom(name) => name,
        }
    }
    
    /// Get the application identifier
    pub fn identifier(&self) -> &str {
        match self {
            McpApplication::ClaudeDesktop => "claude-desktop",
            McpApplication::Cursor => "cursor",
            McpApplication::Zed => "zed",
            McpApplication::VSCode => "vscode",
            McpApplication::Custom(name) => name,
        }
    }
}

/// Configuration path information for an application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationPaths {
    /// The application
    pub application: McpApplication,
    
    /// Primary configuration file path
    pub config_path: PathBuf,
    
    /// Alternative configuration paths (if any)
    pub alt_config_paths: Vec<PathBuf>,
    
    /// Application data directory
    pub data_dir: Option<PathBuf>,
    
    /// Application cache directory
    pub cache_dir: Option<PathBuf>,
    
    /// Application log directory
    pub log_dir: Option<PathBuf>,
    
    /// Whether the application is currently installed/detected
    pub is_installed: bool,
    
    /// Configuration file format
    pub config_format: ConfigFormat,
}

/// Configuration file formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Ini,
    Custom(String),
}

/// Path resolver for finding application configuration files
pub struct PathResolver {
    /// Cached application paths
    cached_paths: HashMap<McpApplication, ApplicationPaths>,
    
    /// Whether to use cache
    use_cache: bool,
}

impl PathResolver {
    /// Create a new path resolver
    pub fn new() -> Self {
        Self {
            cached_paths: HashMap::new(),
            use_cache: true,
        }
    }
    
    /// Disable caching (useful for testing)
    pub fn disable_cache(&mut self) {
        self.use_cache = false;
        self.cached_paths.clear();
    }
    
    /// Get configuration paths for all known applications
    pub fn get_all_application_paths(&mut self) -> Result<Vec<ApplicationPaths>> {
        let applications = vec![
            McpApplication::ClaudeDesktop,
            McpApplication::Cursor,
            McpApplication::Zed,
            McpApplication::VSCode,
        ];
        
        let mut paths = Vec::new();
        for app in applications {
            if let Ok(app_paths) = self.get_application_paths(&app) {
                paths.push(app_paths);
            }
        }
        
        Ok(paths)
    }
    
    /// Get configuration paths for a specific application
    pub fn get_application_paths(&mut self, app: &McpApplication) -> Result<ApplicationPaths> {
        // Check cache first
        if self.use_cache {
            if let Some(cached) = self.cached_paths.get(app) {
                return Ok(cached.clone());
            }
        }
        
        let paths = match app {
            McpApplication::ClaudeDesktop => self.get_claude_desktop_paths()?,
            McpApplication::Cursor => self.get_cursor_paths()?,
            McpApplication::Zed => self.get_zed_paths()?,
            McpApplication::VSCode => self.get_vscode_paths()?,
            McpApplication::Custom(name) => {
                return Err(anyhow::anyhow!("Custom application paths not implemented: {}", name));
            }
        };
        
        // Cache the result
        if self.use_cache {
            self.cached_paths.insert(app.clone(), paths.clone());
        }
        
        Ok(paths)
    }
    
    /// Find all existing configuration files
    pub fn find_existing_configs(&mut self) -> Result<Vec<ApplicationPaths>> {
        let all_paths = self.get_all_application_paths()?;
        
        Ok(all_paths.into_iter()
            .filter(|paths| paths.config_path.exists() || 
                           paths.alt_config_paths.iter().any(|p| p.exists()))
            .collect())
    }
    
    /// Get the primary configuration path for an application
    pub fn get_primary_config_path(&mut self, app: &McpApplication) -> Result<PathBuf> {
        let paths = self.get_application_paths(app)?;
        Ok(paths.config_path)
    }
    
    /// Check if an application is installed
    pub fn is_application_installed(&mut self, app: &McpApplication) -> bool {
        if let Ok(paths) = self.get_application_paths(app) {
            paths.is_installed
        } else {
            false
        }
    }
    
    /// Get the configuration format for an application
    pub fn get_config_format(&mut self, app: &McpApplication) -> Result<ConfigFormat> {
        let paths = self.get_application_paths(app)?;
        Ok(paths.config_format)
    }
    
    /// Create configuration directory if it doesn't exist
    pub fn ensure_config_directory(&mut self, app: &McpApplication) -> Result<PathBuf> {
        let paths = self.get_application_paths(app)?;
        
        if let Some(parent) = paths.config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
            Ok(parent.to_path_buf())
        } else {
            Err(anyhow::anyhow!("Invalid config path for {}", app.display_name()))
        }
    }
    
    // Platform-specific path resolution methods
    
    fn get_claude_desktop_paths(&self) -> Result<ApplicationPaths> {
        let config_path = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                .join("Library/Application Support/Claude/claude_desktop_config.json")
        } else if cfg!(target_os = "windows") {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("Claude\\claude_desktop_config.json")
        } else {
            // Linux
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("claude/claude_desktop_config.json")
        };
        
        let data_dir = if cfg!(target_os = "macos") {
            dirs::home_dir().map(|h| h.join("Library/Application Support/Claude"))
        } else if cfg!(target_os = "windows") {
            dirs::data_dir().map(|d| d.join("Claude"))
        } else {
            dirs::data_dir().map(|d| d.join("claude"))
        };
        
        let is_installed = self.detect_claude_desktop_installation();
        
        Ok(ApplicationPaths {
            application: McpApplication::ClaudeDesktop,
            config_path,
            alt_config_paths: vec![],
            data_dir,
            cache_dir: None,
            log_dir: None,
            is_installed,
            config_format: ConfigFormat::Json,
        })
    }
    
    fn get_cursor_paths(&self) -> Result<ApplicationPaths> {
        let config_path = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                .join("Library/Application Support/Cursor/User/settings.json")
        } else if cfg!(target_os = "windows") {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("Cursor\\User\\settings.json")
        } else {
            // Linux
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("Cursor/User/settings.json")
        };
        
        let data_dir = if cfg!(target_os = "macos") {
            dirs::home_dir().map(|h| h.join("Library/Application Support/Cursor"))
        } else if cfg!(target_os = "windows") {
            dirs::data_dir().map(|d| d.join("Cursor"))
        } else {
            dirs::data_dir().map(|d| d.join("Cursor"))
        };
        
        let is_installed = self.detect_cursor_installation();
        
        Ok(ApplicationPaths {
            application: McpApplication::Cursor,
            config_path,
            alt_config_paths: vec![],
            data_dir,
            cache_dir: None,
            log_dir: None,
            is_installed,
            config_format: ConfigFormat::Json,
        })
    }
    
    fn get_zed_paths(&self) -> Result<ApplicationPaths> {
        let config_path = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                .join("Library/Application Support/Zed/settings.json")
        } else if cfg!(target_os = "windows") {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("Zed\\settings.json")
        } else {
            // Linux
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("zed/settings.json")
        };
        
        let data_dir = if cfg!(target_os = "macos") {
            dirs::home_dir().map(|h| h.join("Library/Application Support/Zed"))
        } else if cfg!(target_os = "windows") {
            dirs::data_dir().map(|d| d.join("Zed"))
        } else {
            dirs::data_dir().map(|d| d.join("zed"))
        };
        
        let is_installed = self.detect_zed_installation();
        
        Ok(ApplicationPaths {
            application: McpApplication::Zed,
            config_path,
            alt_config_paths: vec![],
            data_dir,
            cache_dir: None,
            log_dir: None,
            is_installed,
            config_format: ConfigFormat::Json,
        })
    }
    
    fn get_vscode_paths(&self) -> Result<ApplicationPaths> {
        let config_path = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                .join("Library/Application Support/Code/User/settings.json")
        } else if cfg!(target_os = "windows") {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("Code\\User\\settings.json")
        } else {
            // Linux
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("Code/User/settings.json")
        };
        
        let data_dir = if cfg!(target_os = "macos") {
            dirs::home_dir().map(|h| h.join("Library/Application Support/Code"))
        } else if cfg!(target_os = "windows") {
            dirs::data_dir().map(|d| d.join("Code"))
        } else {
            dirs::data_dir().map(|d| d.join("Code"))
        };
        
        let is_installed = self.detect_vscode_installation();
        
        Ok(ApplicationPaths {
            application: McpApplication::VSCode,
            config_path,
            alt_config_paths: vec![],
            data_dir,
            cache_dir: None,
            log_dir: None,
            is_installed,
            config_format: ConfigFormat::Json,
        })
    }
    
    // Installation detection methods
    
    fn detect_claude_desktop_installation(&self) -> bool {
        if cfg!(target_os = "macos") {
            Path::new("/Applications/Claude.app").exists()
        } else if cfg!(target_os = "windows") {
            // Check common installation paths
            Path::new("C:\\Program Files\\Claude\\Claude.exe").exists() ||
            Path::new("C:\\Program Files (x86)\\Claude\\Claude.exe").exists()
        } else {
            // Linux - check if claude command is available
            std::process::Command::new("which")
                .arg("claude")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
    }
    
    fn detect_cursor_installation(&self) -> bool {
        if cfg!(target_os = "macos") {
            Path::new("/Applications/Cursor.app").exists()
        } else if cfg!(target_os = "windows") {
            Path::new("C:\\Program Files\\Cursor\\Cursor.exe").exists() ||
            Path::new("C:\\Program Files (x86)\\Cursor\\Cursor.exe").exists()
        } else {
            std::process::Command::new("which")
                .arg("cursor")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
    }
    
    fn detect_zed_installation(&self) -> bool {
        if cfg!(target_os = "macos") {
            Path::new("/Applications/Zed.app").exists()
        } else if cfg!(target_os = "windows") {
            Path::new("C:\\Program Files\\Zed\\zed.exe").exists() ||
            Path::new("C:\\Program Files (x86)\\Zed\\zed.exe").exists()
        } else {
            std::process::Command::new("which")
                .arg("zed")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
    }
    
    fn detect_vscode_installation(&self) -> bool {
        if cfg!(target_os = "macos") {
            Path::new("/Applications/Visual Studio Code.app").exists()
        } else if cfg!(target_os = "windows") {
            Path::new("C:\\Program Files\\Microsoft VS Code\\Code.exe").exists() ||
            Path::new("C:\\Program Files (x86)\\Microsoft VS Code\\Code.exe").exists()
        } else {
            std::process::Command::new("which")
                .arg("code")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
    }
}

impl Default for PathResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for path operations
pub struct PathUtils;

impl PathUtils {
    /// Expand tilde (~) in path to home directory
    pub fn expand_tilde<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
        let path = path.as_ref();
        
        if let Some(path_str) = path.to_str() {
            if let Some(stripped) = path_str.strip_prefix("~/") {
                if let Some(home) = dirs::home_dir() {
                    return Ok(home.join(stripped));
                }
            }
        }
        
        Ok(path.to_path_buf())
    }
    
    /// Get relative path from base to target
    pub fn get_relative_path<P: AsRef<Path>, Q: AsRef<Path>>(base: P, target: Q) -> Result<PathBuf> {
        let base = base.as_ref().canonicalize()
            .with_context(|| format!("Failed to canonicalize base path: {}", base.as_ref().display()))?;
        let target = target.as_ref().canonicalize()
            .with_context(|| format!("Failed to canonicalize target path: {}", target.as_ref().display()))?;
        
        target.strip_prefix(&base)
            .map(|p| p.to_path_buf())
            .with_context(|| "Target path is not relative to base path".to_string())
    }
    
    /// Check if a path is safe (no directory traversal)
    pub fn is_safe_path<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();
        
        // Check for directory traversal attempts
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => return false,
                std::path::Component::Normal(name) => {
                    if name.to_string_lossy().contains("..") {
                        return false;
                    }
                }
                _ => {}
            }
        }
        
        true
    }
    
    /// Normalize a path (resolve . and .. components)
    pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
        let path = path.as_ref();
        let mut components = Vec::new();
        
        for component in path.components() {
            match component {
                std::path::Component::CurDir => {
                    // Skip current directory references
                }
                std::path::Component::ParentDir => {
                    // Pop the last component if possible
                    components.pop();
                }
                _ => {
                    components.push(component);
                }
            }
        }
        
        components.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_application_display_names() {
        assert_eq!(McpApplication::ClaudeDesktop.display_name(), "Claude Desktop");
        assert_eq!(McpApplication::Cursor.display_name(), "Cursor");
        assert_eq!(McpApplication::Zed.display_name(), "Zed");
        assert_eq!(McpApplication::VSCode.display_name(), "VS Code");
        assert_eq!(McpApplication::Custom("MyApp".to_string()).display_name(), "MyApp");
    }
    
    #[test]
    fn test_application_identifiers() {
        assert_eq!(McpApplication::ClaudeDesktop.identifier(), "claude-desktop");
        assert_eq!(McpApplication::Cursor.identifier(), "cursor");
        assert_eq!(McpApplication::Zed.identifier(), "zed");
        assert_eq!(McpApplication::VSCode.identifier(), "vscode");
        assert_eq!(McpApplication::Custom("my-app".to_string()).identifier(), "my-app");
    }
    
    #[test]
    fn test_path_resolver_creation() {
        let resolver = PathResolver::new();
        assert!(resolver.use_cache);
        assert!(resolver.cached_paths.is_empty());
    }
    
    #[test]
    fn test_path_utils_expand_tilde() {
        // Test non-tilde path
        let path = PathBuf::from("/absolute/path");
        let expanded = PathUtils::expand_tilde(&path).unwrap();
        assert_eq!(expanded, path);
        
        // Test tilde expansion (if home directory is available)
        if let Some(home) = dirs::home_dir() {
            let tilde_path = PathBuf::from("~/test/path");
            let expanded = PathUtils::expand_tilde(&tilde_path).unwrap();
            assert_eq!(expanded, home.join("test/path"));
        }
    }
    
    #[test]
    fn test_path_utils_is_safe_path() {
        assert!(PathUtils::is_safe_path("safe/path"));
        assert!(PathUtils::is_safe_path("/absolute/safe/path"));
        assert!(!PathUtils::is_safe_path("../unsafe/path"));
        assert!(!PathUtils::is_safe_path("safe/../unsafe"));
        assert!(!PathUtils::is_safe_path("path/with/..hidden"));
    }
    
    #[test]
    fn test_path_utils_normalize_path() {
        let path = PathBuf::from("./test/../normalized/./path");
        let normalized = PathUtils::normalize_path(&path);
        assert_eq!(normalized, PathBuf::from("normalized/path"));
        
        let path = PathBuf::from("/absolute/./test/../path");
        let normalized = PathUtils::normalize_path(&path);
        assert_eq!(normalized, PathBuf::from("/absolute/path"));
    }
    
    #[test]
    fn test_get_application_paths() {
        let mut resolver = PathResolver::new();
        resolver.disable_cache(); // Disable cache for testing
        
        // Test getting paths for each application
        let apps = vec![
            McpApplication::ClaudeDesktop,
            McpApplication::Cursor,
            McpApplication::Zed,
            McpApplication::VSCode,
        ];
        
        for app in apps {
            let result = resolver.get_application_paths(&app);
            assert!(result.is_ok(), "Failed to get paths for {:?}", app);
            
            let paths = result.unwrap();
            assert_eq!(paths.application, app);
            assert!(!paths.config_path.as_os_str().is_empty());
            assert_eq!(paths.config_format, ConfigFormat::Json);
        }
    }
}
