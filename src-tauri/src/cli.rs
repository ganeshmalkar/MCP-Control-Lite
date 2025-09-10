use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::configuration::ConfigurationEngine;
use crate::detection::{ApplicationDetector, ConfigValidator};
use crate::server::ServerManager;

/// MCP Control Lite - Basic CLI for testing backend functionality
#[derive(Parser)]
#[command(name = "mcpctl")]
#[command(about = "A lightweight MCP server management tool")]
#[command(long_about = "MCP Control - Manage Model Context Protocol servers and configurations\n\nTo launch GUI mode: mcpctl --gui")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Detect MCP-enabled applications on the system
    DetectApps,
    /// List configured MCP servers
    ListServers,
    /// Test configuration synchronization
    SyncConfig,
    /// Validate application configurations
    ValidateConfig,
    /// Discover available MCP servers
    DiscoverServers,
    /// List available and installed servers
    ListAllServers,
    /// Install a server from available registry
    InstallServer { name: String },
    /// Remove/uninstall a server
    RemoveServer { name: String },
    /// Start an MCP server
    StartServer { name: String },
    /// Stop a running MCP server
    StopServer { name: String },
    /// Show status of all servers
    ServerStatus,
    /// Test Amazon Q config reading
    TestAmazonQ,
    /// Import configs FROM an application to central store (use 'list-apps' to see available apps)
    ImportFrom { app_name: String },
    /// Export configs TO an application from central store (use 'list-apps' to see available apps)
    ExportTo { app_name: String },
    /// Show central store status
    StoreStatus,
    /// Browse available MCP servers
    Browse { category: Option<String> },
    /// Install an MCP server
    Install { server_name: String, app_name: Option<String> },
    /// Enable an MCP server
    Enable { server_name: String, app_name: Option<String> },
    /// Disable an MCP server
    Disable { server_name: String, app_name: Option<String> },
    /// Search for MCP servers
    Search { query: String },
    /// Create manual backup of all configs
    CreateBackup,
    /// List available backups
    ListBackups,
    /// Restore from backup
    RestoreBackup { backup_name: String },
    /// Show comprehensive system status
    Status,
    /// Show version information
    Version,
    /// List available applications for import/export
    ListApps,
}

pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::DetectApps => detect_apps().await,
        Commands::ListServers => list_servers().await,
        Commands::SyncConfig => sync_config().await,
        Commands::ValidateConfig => validate_config().await,
        Commands::DiscoverServers => discover_servers().await,
        Commands::ListAllServers => list_all_servers().await,
        Commands::InstallServer { name } => install_server(&name, None).await,
        Commands::RemoveServer { name } => remove_server(&name).await,
        Commands::StartServer { name } => start_server(&name).await,
        Commands::StopServer { name } => stop_server(&name).await,
        Commands::ServerStatus => server_status().await,
        Commands::TestAmazonQ => test_amazon_q().await,
        Commands::ImportFrom { app_name } => import_from_app(&app_name).await,
        Commands::ExportTo { app_name } => export_to_app(&app_name).await,
        Commands::StoreStatus => store_status().await,
        Commands::Browse { category } => browse_servers(category.as_deref()).await,
        Commands::Install { server_name, app_name } => install_server(&server_name, app_name.as_deref()).await,
        Commands::Enable { server_name, app_name } => enable_server(&server_name, app_name.as_deref()).await,
        Commands::Disable { server_name, app_name } => disable_server(&server_name, app_name.as_deref()).await,
        Commands::Search { query } => search_servers(&query).await,
        Commands::CreateBackup => create_backup().await,
        Commands::ListBackups => list_backups().await,
        Commands::RestoreBackup { backup_name } => restore_backup(&backup_name).await,
        Commands::Status => show_status().await,
        Commands::Version => show_version().await,
        Commands::ListApps => list_apps().await,
    }
}

async fn detect_apps() -> Result<()> {
    println!("🔍 Detecting MCP-enabled applications...");
    
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    if results.is_empty() {
        println!("❌ No MCP-enabled applications found");
        return Ok(());
    }
    
    println!("✅ Found {} MCP-enabled application(s):", results.len());
    for result in results {
        println!("  📱 {} ({})", result.profile.name, result.profile.id);
        println!("     Config: {}", result.profile.config_path);
        println!("     Status: {}", if result.detected { "✅ Detected" } else { "❌ Not Found" });
        println!("     Confidence: {:.1}%", result.confidence * 100.0);
        
        if let Some(config_path) = &result.found_paths.config_file {
            println!("     Found Config: {}", config_path.display());
        }
        if let Some(exe_path) = &result.found_paths.executable {
            println!("     Found Executable: {}", exe_path.display());
        }
        
        if !result.messages.is_empty() {
            for msg in &result.messages {
                println!("     📝 {}: {}", msg.level, msg.message);
            }
        }
        println!();
    }
    
    Ok(())
}

async fn list_servers() -> Result<()> {
    println!("📋 Listing configured MCP servers...");
    
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    for result in &results {
        if result.detected {
            if let Some(config_path) = &result.found_paths.config_file {
                println!("\n🔍 {} ({})", result.profile.name, config_path.display());
                
                match tokio::fs::read_to_string(config_path).await {
                    Ok(content) => {
                        match serde_json::from_str::<serde_json::Value>(&content) {
                            Ok(config) => {
                                if let Some(servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
                                    if servers.is_empty() {
                                        println!("   📭 No servers configured");
                                    } else {
                                        for (name, server_config) in servers {
                                            let is_disabled = server_config.get("disabled")
                                                .and_then(|d| d.as_bool())
                                                .unwrap_or(false);
                                            
                                            let status = if is_disabled { "🔴 DISABLED" } else { "🟢 ENABLED" };
                                            
                                            println!("   📦 {} - {}", name, status);
                                            
                                            if let Some(command) = server_config.get("command").and_then(|c| c.as_str()) {
                                                println!("      💻 Command: {}", command);
                                            }
                                            
                                            if is_disabled {
                                                println!("      💡 Enable: mcpctl enable {}", name);
                                            } else {
                                                println!("      💡 Disable: mcpctl disable {}", name);
                                            }
                                        }
                                    }
                                } else {
                                    println!("   📭 No mcpServers section found");
                                }
                            }
                            Err(e) => println!("   ❌ Error parsing config: {}", e),
                        }
                    }
                    Err(e) => println!("   ❌ Error reading config: {}", e),
                }
            }
        }
    }
    
    Ok(())
}

async fn sync_config() -> Result<()> {
    println!("🔄 Testing configuration synchronization...");
    
    let temp_dir = std::env::temp_dir();
    let store_path = temp_dir.join("mcp_control_test_store.json");
    let backup_dir = temp_dir.join("mcp_control_backups");
    
    let mut engine = ConfigurationEngine::new(store_path, backup_dir)?;
    
    // Initialize engine (detects apps and imports configs)
    engine.initialize().await?;
    
    // Try to sync with all detected applications
    let results = engine.sync_all_applications().await?;
    
    println!("✅ Synchronization results:");
    for result in results {
        println!("  {}", result);
    }
    
    Ok(())
}

async fn validate_config() -> Result<()> {
    println!("✅ Validating application configurations...");
    
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    let validator = ConfigValidator::new()?;
    
    for result in results {
        println!("📱 Validating {}...", result.profile.name);
        
        if result.detected {
            // Try to validate the configuration if we found it
            if let Some(_config_path) = &result.found_paths.config_file {
                match validator.validate_application_config(&result.profile).await {
                    Ok(validation_result) => {
                        if validation_result.is_valid {
                            println!("  ✅ Configuration is valid");
                            if !validation_result.mcp_servers.is_empty() {
                                println!("  🔧 Found {} MCP server(s):", validation_result.mcp_servers.len());
                                for server in &validation_result.mcp_servers {
                                    println!("    - {}", server.name);
                                }
                            }
                        } else {
                            println!("  ❌ Configuration has issues:");
                            for msg in &validation_result.messages {
                                println!("    📝 {}: {}", msg.level, msg.message);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  ❌ Validation failed: {}", e);
                    }
                }
            } else {
                println!("  ⚠️  Application detected but no config file found");
            }
        } else {
            println!("  ❌ Application not detected");
            for msg in &result.messages {
                println!("    📝 {}: {}", msg.level, msg.message);
            }
        }
        println!();
    }
    
    Ok(())
}
async fn discover_servers() -> Result<()> {
    println!("🔍 Discovering available MCP servers...");
    
    let mut server_manager = ServerManager::new();
    let discovered = server_manager.discover_servers().await?;
    
    println!("✅ Discovered {} server(s)", discovered);
    
    let available = server_manager.get_available_servers();
    if !available.is_empty() {
        println!("📦 Available servers:");
        for server in available {
            println!("  - {}", server);
        }
    }
    
    Ok(())
}

async fn list_all_servers() -> Result<()> {
    println!("📋 Listing all servers...");
    
    let mut server_manager = ServerManager::new();
    let _ = server_manager.discover_servers().await?;
    
    let available = server_manager.get_available_servers();
    let installed = server_manager.get_installed_servers();
    
    println!("📦 Available servers ({}):", available.len());
    for server in available {
        println!("  - {}", server);
    }
    
    println!("✅ Installed servers ({}):", installed.len());
    for server in installed {
        println!("  - {}", server);
    }
    
    Ok(())
}



async fn remove_server(name: &str) -> Result<()> {
    println!("🗑️  Removing server: {}", name);
    
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    let mut removed_count = 0;
    
    for result in &results {
        if result.detected {
            if let Some(config_path) = &result.found_paths.config_file {
                match tokio::fs::read_to_string(config_path).await {
                    Ok(config_content) => {
                        match serde_json::from_str::<serde_json::Value>(&config_content) {
                            Ok(mut config) => {
                                if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
                                    // Find server (try exact match first, then partial)
                                    let server_key = servers.keys()
                                        .find(|k| k.to_lowercase() == name.to_lowercase())
                                        .or_else(|| servers.keys().find(|k| k.to_lowercase().contains(&name.to_lowercase())))
                                        .cloned();
                                    
                                    if let Some(key) = server_key {
                                        servers.remove(&key);
                                        
                                        let updated_content = serde_json::to_string_pretty(&config)?;
                                        tokio::fs::write(config_path, updated_content).await?;
                                        
                                        println!("✅ Removed '{}' from {}", key, result.profile.name);
                                        removed_count += 1;
                                    }
                                }
                            }
                            Err(e) => println!("⚠️  Error parsing {} config: {}", result.profile.name, e),
                        }
                    }
                    Err(e) => println!("⚠️  Error reading {} config: {}", result.profile.name, e),
                }
            }
        }
    }
    
    if removed_count == 0 {
        println!("❌ Server '{}' not found in any application", name);
    } else {
        println!("🎉 Successfully removed server from {} application(s)", removed_count);
    }
    
    Ok(())
}

async fn start_server(name: &str) -> Result<()> {
    println!("🚀 Starting server: {}", name);
    
    let mut server_manager = ServerManager::new();
    
    // Get the server config from installed servers
    if let Some(server_config) = server_manager.get_registry().get_installed_server(name) {
        let result = server_manager.start_server(server_config).await?;
        
        if result.success {
            println!("✅ {}", result.message);
        } else {
            println!("❌ {}", result.message);
            for error in result.errors {
                println!("  📝 {}", error);
            }
        }
    } else {
        println!("❌ Server '{}' not found in installed servers", name);
    }
    
    Ok(())
}

async fn stop_server(name: &str) -> Result<()> {
    println!("🛑 Stopping server: {}", name);
    
    let server_manager = ServerManager::new();
    let result = server_manager.stop_server(name).await?;
    
    if result.success {
        println!("✅ {}", result.message);
    } else {
        println!("❌ {}", result.message);
        for error in result.errors {
            println!("  📝 {}", error);
        }
    }
    
    Ok(())
}

async fn server_status() -> Result<()> {
    println!("📊 Server status:");
    
    let server_manager = ServerManager::new();
    let statuses = server_manager.get_all_server_statuses().await;
    let running = server_manager.get_running_servers().await;
    
    if statuses.is_empty() && running.is_empty() {
        println!("  No servers found");
        return Ok(());
    }
    
    // Show running servers
    if !running.is_empty() {
        println!("🟢 Running servers:");
        for server in running {
            println!("  - {}: Running", server);
        }
    }
    
    // Show all server statuses
    for (server, status) in statuses {
        let icon = match status {
            crate::server::ServerStatus::Running => "🟢",
            crate::server::ServerStatus::Stopped => "🔴",
            crate::server::ServerStatus::Error(_) => "🟡",
            crate::server::ServerStatus::Unknown => "⚪",
        };
        println!("  {} {}: {}", icon, server, status);
    }
    
    Ok(())
}
async fn test_amazon_q() -> Result<()> {
    println!("🧪 Testing Amazon Q config reading...");
    
    use std::path::PathBuf;
    use crate::filesystem::paths::PathUtils;
    
    let config_path = "~/.aws/amazonq/mcp.json";
    let expanded = PathUtils::expand_tilde(config_path)?;
    
    println!("📁 Config path: {} -> {}", config_path, expanded.display());
    println!("📋 File exists: {}", expanded.exists());
    
    if expanded.exists() {
        let content = tokio::fs::read_to_string(&expanded).await?;
        let parsed: serde_json::Value = serde_json::from_str(&content)?;
        
        if let Some(servers) = parsed.get("mcpServers").and_then(|s| s.as_object()) {
            println!("🔧 Found {} MCP servers:", servers.len());
            for (name, _config) in servers {
                println!("  - {}", name);
            }
        }
    }
    
    Ok(())
}
async fn create_backup() -> Result<()> {
    println!("💾 Creating manual backup of all configurations...");
    
    let backup_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".mcp-control-backups");
    
    tokio::fs::create_dir_all(&backup_dir).await?;
    
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    
    let mut backed_up = 0;
    
    for result in results {
        if result.detected {
            if let Some(config_path) = &result.found_paths.config_file {
                let backup_name = format!("{}_{}.backup", 
                    result.profile.id, timestamp);
                let backup_path = backup_dir.join(backup_name);
                
                match tokio::fs::copy(config_path, &backup_path).await {
                    Ok(_) => {
                        println!("  ✅ {}: {}", result.profile.name, backup_path.display());
                        backed_up += 1;
                    }
                    Err(e) => {
                        println!("  ❌ {}: {}", result.profile.name, e);
                    }
                }
            }
        }
    }
    
    println!("📁 Backup complete: {} files backed up to {}", backed_up, backup_dir.display());
    Ok(())
}

async fn list_backups() -> Result<()> {
    println!("📋 Available backups:");
    
    let backup_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".mcp-control-backups");
    
    if !backup_dir.exists() {
        println!("  No backups found");
        return Ok(());
    }
    
    let mut entries = tokio::fs::read_dir(&backup_dir).await?;
    let mut backups = Vec::new();
    
    while let Some(entry) = entries.next_entry().await? {
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(".backup") {
                let metadata = entry.metadata().await?;
                let modified = metadata.modified()?;
                backups.push((name.to_string(), modified));
            }
        }
    }
    
    // Sort by modification time (newest first)
    backups.sort_by(|a, b| b.1.cmp(&a.1));
    
    for (name, modified) in backups {
        let datetime: chrono::DateTime<chrono::Local> = modified.into();
        println!("  📁 {} ({})", name, datetime.format("%Y-%m-%d %H:%M:%S"));
    }
    
    Ok(())
}

async fn restore_backup(backup_name: &str) -> Result<()> {
    println!("🔄 Restoring backup: {}", backup_name);
    
    let backup_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".mcp-control-backups");
    
    let backup_path = backup_dir.join(backup_name);
    
    if !backup_path.exists() {
        println!("❌ Backup not found: {}", backup_path.display());
        return Ok(());
    }
    
    // Extract app ID from backup name (format: appid_timestamp.backup)
    let app_id = backup_name.split('_').next()
        .ok_or_else(|| anyhow::anyhow!("Invalid backup name format"))?;
    
    // Find the application profile
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    for result in results {
        if result.profile.id == app_id && result.detected {
            if let Some(config_path) = &result.found_paths.config_file {
                // Create backup of current state before restore
                let current_backup_name = format!("{}_{}_pre_restore.backup", 
                    app_id, chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                let current_backup_path = backup_dir.join(current_backup_name);
                
                let _ = tokio::fs::copy(config_path, &current_backup_path).await;
                
                // Restore from backup
                match tokio::fs::copy(&backup_path, config_path).await {
                    Ok(_) => {
                        println!("✅ Restored {} from {}", result.profile.name, backup_name);
                        return Ok(());
                    }
                    Err(e) => {
                        println!("❌ Failed to restore {}: {}", result.profile.name, e);
                        return Ok(());
                    }
                }
            }
        }
    }
    
    println!("❌ Could not find application for backup: {}", app_id);
    Ok(())
}

async fn import_from_app(app_name: &str) -> Result<()> {
    println!("📥 Importing MCP servers FROM {} to central store...", app_name);
    
    // Find the application
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    for result in &results {
        if result.profile.name.to_lowercase().contains(&app_name.to_lowercase()) || 
           result.profile.id.to_lowercase().contains(&app_name.to_lowercase()) {
            
            if result.detected {
                if let Some(config_path) = &result.found_paths.config_file {
                    println!("🔍 Found {} config at: {}", result.profile.name, config_path.display());
                    
                    // Read the config file
                    let config_content = tokio::fs::read_to_string(config_path).await?;
                    let config: serde_json::Value = serde_json::from_str(&config_content)?;
                    
                    // Extract servers (basic extraction for Amazon Q)
                    if let Some(mcp_servers) = config.get("mcpServers").and_then(|v| v.as_object()) {
                        println!("✅ Found {} MCP servers in {}", mcp_servers.len(), result.profile.name);
                        
                        // Create central store file
                        let store_path = dirs::home_dir()
                            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                            .join(".mcp-control-store.json");
                        
                        // Create store structure
                        let mut central_store = serde_json::json!({
                            "version": "1.0.0",
                            "created_at": chrono::Utc::now().to_rfc3339(),
                            "source_application": result.profile.id,
                            "servers": {}
                        });
                        
                        // Convert each server to central store format - PRESERVE ALL DATA
                        for (name, server_config) in mcp_servers {
                            println!("  📦 Importing: {} (preserving all config data)", name);
                            
                            // Store the COMPLETE server config without any filtering
                            central_store["servers"][name] = serde_json::json!({
                                "id": name,
                                "name": name,
                                "config": server_config,  // Store COMPLETE original config
                                "source_app": result.profile.id,
                                "imported_at": chrono::Utc::now().to_rfc3339(),
                                "enabled": true
                            });
                        }
                        
                        // Write central store
                        let store_content = serde_json::to_string_pretty(&central_store)?;
                        tokio::fs::write(&store_path, store_content).await?;
                        
                        println!("✅ Successfully imported {} servers to central store", mcp_servers.len());
                        println!("📁 Central store: {}", store_path.display());
                        return Ok(());
                    } else {
                        println!("❌ No MCP servers found in config");
                        return Ok(());
                    }
                } else {
                    println!("❌ Config file not found for {}", result.profile.name);
                    return Ok(());
                }
            } else {
                println!("❌ {} not detected on this system", result.profile.name);
                return Ok(());
            }
        }
    }
    
    println!("❌ Application not found: {}", app_name);
    println!("💡 Available applications:");
    for result in &results {
        if result.detected {
            println!("  - {} ({})", result.profile.name, result.profile.id);
        }
    }
    println!("💡 Use 'mcpctl list-apps' to see all available applications");
    
    Ok(())
}

async fn export_to_app(app_name: &str) -> Result<()> {
    println!("📤 Exporting MCP servers TO {} from central store...", app_name);
    
    // Check if central store exists
    let store_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".mcp-control-store.json");
    
    if !store_path.exists() {
        println!("❌ Central store not found. Run 'import-from' first.");
        return Ok(());
    }
    
    // Read central store
    let store_content = tokio::fs::read_to_string(&store_path).await?;
    let central_store: serde_json::Value = serde_json::from_str(&store_content)?;
    
    let servers = central_store.get("servers").and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Invalid central store format"))?;
    
    println!("📊 Central store has {} servers", servers.len());
    
    // Find target application
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    for result in &results {
        if result.profile.name.to_lowercase().contains(&app_name.to_lowercase()) || 
           result.profile.id.to_lowercase().contains(&app_name.to_lowercase()) {
            
            if result.detected {
                if let Some(config_path) = &result.found_paths.config_file {
                    println!("🔍 Found {} config at: {}", result.profile.name, config_path.display());
                    
                    // Read current config
                    let current_content = tokio::fs::read_to_string(config_path).await?;
                    let mut current_config: serde_json::Value = serde_json::from_str(&current_content)?;
                    
                    // Create/update mcpServers section
                    let mut mcp_servers = serde_json::Map::new();
                    
                    // Add servers from central store
                    for (name, server_data) in servers {
                        if let Some(config) = server_data.get("config") {
                            mcp_servers.insert(name.clone(), config.clone());
                            println!("  📦 Adding: {}", name);
                        }
                    }
                    
                    current_config["mcpServers"] = serde_json::Value::Object(mcp_servers);
                    
                    // Write updated config
                    let updated_content = serde_json::to_string_pretty(&current_config)?;
                    tokio::fs::write(config_path, updated_content).await?;
                    
                    println!("✅ Successfully exported {} servers to {}", servers.len(), result.profile.name);
                    return Ok(());
                } else {
                    println!("❌ Config file not found for {}", result.profile.name);
                    return Ok(());
                }
            } else {
                println!("❌ {} not detected on this system", result.profile.name);
                return Ok(());
            }
        }
    }
    
    println!("❌ Application not found: {}", app_name);
    println!("💡 Available applications:");
    for result in &results {
        if result.detected {
            println!("  - {} ({})", result.profile.name, result.profile.id);
        }
    }
    println!("💡 Use 'mcpctl list-apps' to see all available applications");
    
    Ok(())
}
async fn store_status() -> Result<()> {
    println!("📊 Central Store Status");
    
    let store_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".mcp-control-store.json");
    
    if !store_path.exists() {
        println!("❌ Central store not found");
        println!("💡 Run 'import-from <app>' to create central store");
        return Ok(());
    }
    
    // Read and display store info
    let store_content = tokio::fs::read_to_string(&store_path).await?;
    let central_store: serde_json::Value = serde_json::from_str(&store_content)?;
    
    println!("✅ Central store found: {}", store_path.display());
    
    if let Some(version) = central_store.get("version").and_then(|v| v.as_str()) {
        println!("📋 Version: {}", version);
    }
    
    if let Some(created_at) = central_store.get("created_at").and_then(|v| v.as_str()) {
        println!("📅 Created: {}", created_at);
    }
    
    if let Some(source_app) = central_store.get("source_application").and_then(|v| v.as_str()) {
        println!("🔗 Source: {}", source_app);
    }
    
    if let Some(servers) = central_store.get("servers").and_then(|v| v.as_object()) {
        println!("📦 Servers: {}", servers.len());
        for (name, server_data) in servers {
            let enabled = server_data.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            let status = if enabled { "✅" } else { "❌" };
            println!("  {} {}", status, name);
        }
    }
    
    Ok(())
}
async fn browse_servers(category: Option<&str>) -> Result<()> {
    println!("🔍 Browsing Available MCP Servers");
    
    // Built-in MCP server catalog
    let servers = get_mcp_catalog();
    
    let filtered_servers: Vec<_> = if let Some(cat) = category {
        servers.into_iter().filter(|s| s.category.to_lowercase() == cat.to_lowercase()).collect()
    } else {
        servers
    };
    
    if filtered_servers.is_empty() {
        if let Some(cat) = category {
            println!("❌ No servers found in category: {}", cat);
            println!("💡 Available categories: productivity, development, ai, database, filesystem, git");
        } else {
            println!("❌ No servers available");
        }
        return Ok(());
    }
    
    println!("📦 Found {} servers:", filtered_servers.len());
    for server in &filtered_servers {
        println!("\n📋 {}", server.name);
        println!("   📝 {}", server.description);
        println!("   🏷️  Category: {}", server.category);
        println!("   📦 Package: {}", server.package);
        if !server.env_vars.is_empty() {
            println!("   🔧 Requires: {}", server.env_vars.join(", "));
        }
        println!("   💾 Install: mcpctl install {}", server.name);
    }
    
    Ok(())
}

async fn search_servers(query: &str) -> Result<()> {
    println!("🔍 Searching MCP Servers for: '{}'", query);
    println!("📡 Searching multiple sources...\n");
    
    // Search built-in catalog
    let builtin_servers = get_mcp_catalog();
    let query_lower = query.to_lowercase();
    
    let builtin_matches: Vec<_> = builtin_servers.into_iter()
        .filter(|s| 
            s.name.to_lowercase().contains(&query_lower) ||
            s.description.to_lowercase().contains(&query_lower) ||
            s.category.to_lowercase().contains(&query_lower)
        )
        .collect();
    
    if !builtin_matches.is_empty() {
        println!("📦 Built-in Catalog ({} matches):", builtin_matches.len());
        for server in &builtin_matches {
            println!("  📋 {} - {}", server.name, server.description);
            println!("      💾 mcpctl install {}", server.name);
        }
        println!();
    }
    
    // Search NPM Registry
    let npm_results = match search_npm_registry(query).await {
        Ok(results) => {
            if !results.is_empty() {
                println!("📦 NPM Registry ({} matches):", results.len());
                for result in &results {
                    println!("  📋 {} - {}", result.name, result.description);
                    println!("      💾 mcpctl install {}", result.name);
                }
                println!();
            }
            Some(results)
        }
        Err(e) => {
            println!("⚠️  NPM search failed: {}\n", e);
            None
        }
    };
    
    // Search GitHub
    let github_results = match search_github_repos(query).await {
        Ok(results) => {
            if !results.is_empty() {
                println!("📦 GitHub Repositories ({} matches):", results.len());
                for result in &results {
                    println!("  📋 {} - {}", result.name, result.description);
                    println!("      🔗 {}", result.url);
                    println!("      💾 mcpctl install {}", result.name);
                }
                println!();
            }
            Some(results)
        }
        Err(e) => {
            println!("⚠️  GitHub search failed: {}\n", e);
            None
        }
    };
    
    // Search PulseMCP
    let pulse_results = match search_pulse_mcp(query).await {
        Ok(results) => {
            if !results.is_empty() {
                println!("📦 PulseMCP Registry ({} matches):", results.len());
                for result in &results {
                    println!("  📋 {} - {}", result.name, result.description);
                    println!("      🌐 {}", result.url);
                    if !result.source.is_empty() {
                        // Extract package name from the npm install command
                        let package_name = result.source.replace("npm install ", "");
                        println!("      💾 mcpctl install {}", package_name);
                    }
                }
                println!();
            }
            Some(results)
        }
        Err(e) => {
            println!("⚠️  PulseMCP search failed: {}\n", e);
            None
        }
    };
    
    let total_results = builtin_matches.len() + 
        npm_results.as_ref().map_or(0, |r| r.len()) +
        github_results.as_ref().map_or(0, |r| r.len()) +
        pulse_results.as_ref().map_or(0, |r| r.len());
    
    if total_results == 0 {
        println!("❌ No servers found matching: {}", query);
        println!("💡 Try broader search terms like 'database', 'git', or 'file'");
    } else {
        println!("✅ Found {} total matches across all sources", total_results);
    }
    
    Ok(())
}

async fn install_server(server_name: &str, app_name: Option<&str>) -> Result<()> {
    println!("📦 Installing MCP Server: {}", server_name);
    
    // First check built-in catalog
    let servers = get_mcp_catalog();
    let server = servers.iter().find(|s| s.name.to_lowercase() == server_name.to_lowercase());
    
    if let Some(server) = server {
        println!("📋 Installing: {}", server.name);
        println!("📝 Description: {}", server.description);
        
        // Check for required environment variables
        if !server.env_vars.is_empty() {
            println!("🔧 Required environment variables:");
            for env_var in &server.env_vars {
                println!("   - {}", env_var);
            }
            println!("💡 Make sure to set these before using the server");
        }
        
        // Install from built-in catalog
        install_from_catalog(server, app_name).await
    } else {
        // Try to install as NPM package
        println!("📋 Installing NPM package: {}", server_name);
        install_npm_package(server_name, app_name).await
    }
}

async fn install_from_catalog(server: &McpServerInfo, app_name: Option<&str>) -> Result<()> {
    // Determine target application
    let target_app = app_name.unwrap_or("Amazon Q");
    
    // Find target application config
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    for result in &results {
        if result.profile.name.to_lowercase().contains(&target_app.to_lowercase()) {
            if result.detected {
                if let Some(config_path) = &result.found_paths.config_file {
                    println!("🔍 Found {} config at: {}", result.profile.name, config_path.display());
                    
                    // Read current config
                    let config_content = tokio::fs::read_to_string(config_path).await?;
                    let mut config: serde_json::Value = serde_json::from_str(&config_content)?;
                    
                    // Create server config
                    let mut server_config = serde_json::json!({
                        "command": server.command,
                        "args": server.args
                    });
                    
                    // Add env vars if specified
                    if !server.env_vars.is_empty() {
                        let mut env_obj = serde_json::Map::new();
                        for env_var in &server.env_vars {
                            env_obj.insert(env_var.clone(), serde_json::Value::String("YOUR_VALUE_HERE".to_string()));
                        }
                        server_config["env"] = serde_json::Value::Object(env_obj);
                    }
                    
                    // Add to mcpServers
                    if config.get("mcpServers").is_none() {
                        config["mcpServers"] = serde_json::json!({});
                    }
                    config["mcpServers"][&server.name] = server_config;
                    
                    // Write updated config
                    let updated_content = serde_json::to_string_pretty(&config)?;
                    tokio::fs::write(config_path, updated_content).await?;
                    
                    println!("✅ Successfully installed {} to {}", server.name, result.profile.name);
                    
                    if !server.env_vars.is_empty() {
                        println!("⚠️  Don't forget to set your environment variables!");
                    }
                    
                    return Ok(());
                }
            }
        }
    }
    
    println!("❌ Target application not found: {}", target_app);
    Ok(())
}

async fn enable_server(server_name: &str, app_name: Option<&str>) -> Result<()> {
    println!("🔄 Enabling MCP Server: {}", server_name);
    
    let target_app = app_name.unwrap_or("Amazon Q");
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    for result in &results {
        if result.profile.name.to_lowercase().contains(&target_app.to_lowercase()) {
            if result.detected {
                if let Some(config_path) = &result.found_paths.config_file {
                    let config_content = tokio::fs::read_to_string(config_path).await?;
                    let mut config: serde_json::Value = serde_json::from_str(&config_content)?;
                    
                    if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
                        // Find server (try exact match first, then partial)
                        let server_key = servers.keys()
                            .find(|k| k.to_lowercase() == server_name.to_lowercase())
                            .or_else(|| servers.keys().find(|k| k.to_lowercase().contains(&server_name.to_lowercase())))
                            .cloned();
                        
                        if let Some(key) = server_key {
                            if let Some(server_config) = servers.get_mut(&key).and_then(|s| s.as_object_mut()) {
                                server_config.remove("disabled");
                                
                                let updated_content = serde_json::to_string_pretty(&config)?;
                                tokio::fs::write(config_path, updated_content).await?;
                                
                                println!("✅ Enabled server '{}' in {}", key, result.profile.name);
                                return Ok(());
                            }
                        }
                    }
                    
                    println!("❌ Server '{}' not found in {}", server_name, result.profile.name);
                    return Ok(());
                }
            }
        }
    }
    
    println!("❌ Target application not found: {}", target_app);
    Ok(())
}

async fn disable_server(server_name: &str, app_name: Option<&str>) -> Result<()> {
    println!("⏸️  Disabling MCP Server: {}", server_name);
    
    let target_app = app_name.unwrap_or("Amazon Q");
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    for result in &results {
        if result.profile.name.to_lowercase().contains(&target_app.to_lowercase()) {
            if result.detected {
                if let Some(config_path) = &result.found_paths.config_file {
                    let config_content = tokio::fs::read_to_string(config_path).await?;
                    let mut config: serde_json::Value = serde_json::from_str(&config_content)?;
                    
                    if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
                        // Find server (try exact match first, then partial)
                        let server_key = servers.keys()
                            .find(|k| k.to_lowercase() == server_name.to_lowercase())
                            .or_else(|| servers.keys().find(|k| k.to_lowercase().contains(&server_name.to_lowercase())))
                            .cloned();
                        
                        if let Some(key) = server_key {
                            if let Some(server_config) = servers.get_mut(&key).and_then(|s| s.as_object_mut()) {
                                server_config.insert("disabled".to_string(), serde_json::Value::Bool(true));
                                
                                let updated_content = serde_json::to_string_pretty(&config)?;
                                tokio::fs::write(config_path, updated_content).await?;
                                
                                println!("✅ Disabled server '{}' in {}", key, result.profile.name);
                                println!("💡 Use 'mcpctl enable {}' to re-enable", key);
                                return Ok(());
                            }
                        }
                    }
                    
                    println!("❌ Server '{}' not found in {}", server_name, result.profile.name);
                    return Ok(());
                }
            }
        }
    }
    
    println!("❌ Target application not found: {}", target_app);
    Ok(())
}

async fn show_status() -> Result<()> {
    println!("📊 MCP Control Status\n");
    
    // Application Detection
    println!("🔍 Detected Applications:");
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    let mut total_servers = 0;
    let mut enabled_servers = 0;
    let mut disabled_servers = 0;
    
    for result in &results {
        let status = if result.detected { "🟢 DETECTED" } else { "🔴 NOT FOUND" };
        println!("   {} - {}", result.profile.name, status);
        
        if result.detected {
            if let Some(config_path) = &result.found_paths.config_file {
                match tokio::fs::read_to_string(config_path).await {
                    Ok(content) => {
                        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
                                let app_total = servers.len();
                                let app_enabled = servers.values()
                                    .filter(|s| !s.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false))
                                    .count();
                                let app_disabled = app_total - app_enabled;
                                
                                total_servers += app_total;
                                enabled_servers += app_enabled;
                                disabled_servers += app_disabled;
                                
                                println!("      📦 {} servers ({} enabled, {} disabled)", app_total, app_enabled, app_disabled);
                            }
                        }
                    }
                    Err(_) => println!("      ⚠️  Config file not readable"),
                }
            }
        }
    }
    
    // Summary
    println!("\n📈 Summary:");
    println!("   🎯 Total MCP Servers: {}", total_servers);
    println!("   🟢 Enabled: {}", enabled_servers);
    println!("   🔴 Disabled: {}", disabled_servers);
    
    // Recent backups
    println!("\n💾 Recent Backups:");
    let backup_dir = std::env::temp_dir().join("mcp_control_backups");
    if backup_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(&backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "backup"))
            .collect();
        
        entries.sort_by(|a, b| {
            b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(&a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH))
        });
        
        for (i, entry) in entries.iter().take(3).enumerate() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(datetime) = modified.duration_since(std::time::SystemTime::UNIX_EPOCH) {
                        let dt = chrono::DateTime::from_timestamp(datetime.as_secs() as i64, 0)
                            .unwrap_or_default();
                        println!("   📁 {} ({})", entry.file_name().to_string_lossy(), dt.format("%Y-%m-%d %H:%M:%S"));
                    }
                }
            }
            if i == 2 { break; }
        }
    } else {
        println!("   📭 No backups found");
    }
    
    println!("\n💡 Quick Commands:");
    println!("   mcpctl list-servers    - View all configured servers");
    println!("   mcpctl search <query>  - Find new servers to install");
    println!("   mcpctl browse          - Browse available servers");
    println!("   mcpctl create-backup   - Create a backup");
    
    Ok(())
}

async fn show_version() -> Result<()> {
    println!("🚀 MCP Control (mcpctl) v0.1.0");
    println!("📝 A lightweight MCP server management tool");
    println!("🔗 https://github.com/your-repo/mcp-control-lite");
    println!();
    println!("🛠️  Built with:");
    println!("   • Rust");
    println!("   • Tauri Framework");
    println!("   • Tokio Async Runtime");
    println!();
    println!("📊 Features:");
    println!("   ✅ Multi-application MCP server management");
    println!("   ✅ Live server discovery from 4 sources");
    println!("   ✅ Enable/disable servers to save tokens");
    println!("   ✅ Automatic configuration backup");
    println!("   ✅ Cross-platform application detection");
    
    Ok(())
}

async fn list_apps() -> Result<()> {
    println!("📱 Available Applications for Import/Export:\n");
    
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    for result in &results {
        let status = if result.detected { "🟢 DETECTED" } else { "🔴 NOT FOUND" };
        let app_name = &result.profile.name;
        
        println!("   {} - {}", app_name, status);
        
        if result.detected {
            if let Some(config_path) = &result.found_paths.config_file {
                println!("      📁 Config: {}", config_path.display());
                
                // Show server count if config is readable
                match tokio::fs::read_to_string(config_path).await {
                    Ok(content) => {
                        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
                                println!("      📦 {} MCP servers configured", servers.len());
                            } else {
                                println!("      📦 No MCP servers configured");
                            }
                        }
                    }
                    Err(_) => println!("      ⚠️  Config file not readable"),
                }
            }
            
            println!("      💾 Import: mcpctl import-from \"{}\"", app_name);
            println!("      📤 Export: mcpctl export-to \"{}\"", app_name);
        } else {
            println!("      💡 Install {} to enable import/export", app_name);
        }
        
        println!();
    }
    
    println!("💡 Usage Examples:");
    println!("   mcpctl import-from \"Amazon Q\"     - Import from Amazon Q Developer");
    println!("   mcpctl export-to \"Claude Desktop\" - Export to Claude Desktop");
    println!("   mcpctl import-from \"Cursor\"       - Import from Cursor");
    
    Ok(())
}

async fn install_npm_package(package_name: &str, app_name: Option<&str>) -> Result<()> {
    let target_app = app_name.unwrap_or("Amazon Q");
    let mut detector = ApplicationDetector::new()?;
    let results = detector.detect_all_applications().await?;
    
    for result in &results {
        if result.profile.name.to_lowercase().contains(&target_app.to_lowercase()) {
            if result.detected {
                if let Some(config_path) = &result.found_paths.config_file {
                    println!("🔍 Found {} config at: {}", result.profile.name, config_path.display());
                    
                    let config_content = tokio::fs::read_to_string(config_path).await?;
                    let mut config: serde_json::Value = serde_json::from_str(&config_content)?;
                    
                    let server_config = serde_json::json!({
                        "command": "npx",
                        "args": ["-y", package_name]
                    });
                    
                    let server_name = package_name.replace("@", "").replace("/", "-");
                    
                    if config.get("mcpServers").is_none() {
                        config["mcpServers"] = serde_json::json!({});
                    }
                    config["mcpServers"][&server_name] = server_config;
                    
                    let updated_content = serde_json::to_string_pretty(&config)?;
                    tokio::fs::write(config_path, updated_content).await?;
                    
                    println!("✅ Successfully installed {} to {}", package_name, result.profile.name);
                    println!("📝 Server name in config: {}", server_name);
                    
                    return Ok(());
                }
            }
        }
    }
    
    println!("❌ Target application not found: {}", target_app);
    Ok(())
}

#[derive(Debug)]
struct McpServerInfo {
    name: String,
    description: String,
    category: String,
    package: String,
    command: String,
    args: Vec<String>,
    env_vars: Vec<String>,
}

fn get_mcp_catalog() -> Vec<McpServerInfo> {
    vec![
        McpServerInfo {
            name: "filesystem".to_string(),
            description: "Access and manage local files and directories".to_string(),
            category: "filesystem".to_string(),
            package: "@modelcontextprotocol/server-filesystem".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
            env_vars: vec![],
        },
        McpServerInfo {
            name: "git".to_string(),
            description: "Git repository operations and version control".to_string(),
            category: "git".to_string(),
            package: "@modelcontextprotocol/server-git".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-git".to_string()],
            env_vars: vec![],
        },
        McpServerInfo {
            name: "github".to_string(),
            description: "GitHub API integration for repositories and issues".to_string(),
            category: "development".to_string(),
            package: "@modelcontextprotocol/server-github".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-github".to_string()],
            env_vars: vec!["GITHUB_PERSONAL_ACCESS_TOKEN".to_string()],
        },
        McpServerInfo {
            name: "postgres".to_string(),
            description: "PostgreSQL database operations and queries".to_string(),
            category: "database".to_string(),
            package: "@modelcontextprotocol/server-postgres".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-postgres".to_string()],
            env_vars: vec!["POSTGRES_CONNECTION_STRING".to_string()],
        },
        McpServerInfo {
            name: "sqlite".to_string(),
            description: "SQLite database operations and queries".to_string(),
            category: "database".to_string(),
            package: "@modelcontextprotocol/server-sqlite".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-sqlite".to_string()],
            env_vars: vec![],
        },
        McpServerInfo {
            name: "brave-search".to_string(),
            description: "Web search using Brave Search API".to_string(),
            category: "ai".to_string(),
            package: "@modelcontextprotocol/server-brave-search".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-brave-search".to_string()],
            env_vars: vec!["BRAVE_API_KEY".to_string()],
        },
        McpServerInfo {
            name: "puppeteer".to_string(),
            description: "Web scraping and browser automation".to_string(),
            category: "development".to_string(),
            package: "@modelcontextprotocol/server-puppeteer".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-puppeteer".to_string()],
            env_vars: vec![],
        },
        McpServerInfo {
            name: "slack".to_string(),
            description: "Slack workspace integration and messaging".to_string(),
            category: "productivity".to_string(),
            package: "@modelcontextprotocol/server-slack".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-slack".to_string()],
            env_vars: vec!["SLACK_BOT_TOKEN".to_string()],
        },
        McpServerInfo {
            name: "clickup-intelligence".to_string(),
            description: "ClickUp project management with AI intelligence".to_string(),
            category: "productivity".to_string(),
            package: "@chykalophia/clickup-intelligence-mcp-server".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@chykalophia/clickup-intelligence-mcp-server".to_string()],
            env_vars: vec!["CLICKUP_API_TOKEN".to_string()],
        },
        McpServerInfo {
            name: "task-master".to_string(),
            description: "Advanced task and project management".to_string(),
            category: "productivity".to_string(),
            package: "task-master-ai".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "--package=task-master-ai".to_string(), "task-master-ai".to_string()],
            env_vars: vec![],
        },
    ]
}
#[derive(Debug)]
struct SearchResult {
    name: String,
    description: String,
    url: String,
    source: String,
}

async fn search_npm_registry(query: &str) -> Result<Vec<SearchResult>> {
    let client = reqwest::Client::new();
    // Search for packages that contain both the query AND mcp-related terms
    let search_terms = format!("{} (mcp OR \"model context protocol\" OR \"mcp server\" OR \"mcp-server\")", query);
    let url = format!("https://registry.npmjs.org/-/v1/search?text={}&size=15", urlencoding::encode(&search_terms));
    
    let response = client.get(&url)
        .header("User-Agent", "mcp-control-lite/1.0")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    let mut results = Vec::new();
    
    if let Some(objects) = json.get("objects").and_then(|o| o.as_array()) {
        for obj in objects.iter() {
            if let Some(package) = obj.get("package") {
                let name = package.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                let description = package.get("description").and_then(|d| d.as_str()).unwrap_or("No description");
                let empty_vec = vec![];
                let keywords = package.get("keywords").and_then(|k| k.as_array()).unwrap_or(&empty_vec);
                
                // Simple exclusion of obviously non-MCP packages
                let is_excluded = 
                    name == "@google/gemini-cli" ||
                    name.contains("eslint") ||
                    name.contains("webpack") ||
                    name.contains("babel") ||
                    name.contains("react") ||
                    name.contains("vue") ||
                    name.contains("angular") ||
                    (name.starts_with("@types/") && !name.contains("mcp"));
                
                if !is_excluded {
                    let npm_url = format!("https://www.npmjs.com/package/{}", name);
                    
                    results.push(SearchResult {
                        name: name.to_string(),
                        description: description.to_string(),
                        url: npm_url,
                        source: "npm".to_string(),
                    });
                }
            }
        }
    }
    
    // Limit to top 5 most relevant results
    results.truncate(5);
    Ok(results)
}

async fn search_github_repos(query: &str) -> Result<Vec<SearchResult>> {
    let client = reqwest::Client::new();
    // Search specifically for MCP server repositories
    let search_query = format!("{} (\"mcp server\" OR \"model context protocol\" OR mcp-server) in:name,description,readme", query);
    let url = format!("https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page=10", 
                     urlencoding::encode(&search_query));
    
    let response = client.get(&url)
        .header("User-Agent", "mcp-control-lite/1.0")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    let mut results = Vec::new();
    
    if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
        for item in items.iter() {
            let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
            let description = item.get("description").and_then(|d| d.as_str()).unwrap_or("No description");
            let html_url = item.get("html_url").and_then(|u| u.as_str()).unwrap_or("");
            
            // Filter for MCP server repositories (stricter filtering)
            let is_mcp_server = 
                name.to_lowercase().contains("mcp") ||
                name.contains("-mcp-") ||
                name.ends_with("-mcp") ||
                name.starts_with("mcp-") ||
                name.contains("mcp-server") ||
                description.to_lowercase().contains("mcp server") ||
                description.to_lowercase().contains("model context protocol server");
            
            // Exclude obvious non-MCP server repositories
            let is_excluded = 
                name.to_lowercase().contains("chat") ||
                name.to_lowercase().contains("ui") ||
                name.to_lowercase().contains("frontend") ||
                name.to_lowercase().contains("app") ||
                name.to_lowercase().contains("client") ||
                description.to_lowercase().contains("chat framework") ||
                description.to_lowercase().contains("ai chat") ||
                description.to_lowercase().contains("chat application") ||
                (description.to_lowercase().contains("mcp") && 
                 !description.to_lowercase().contains("mcp server") &&
                 !description.to_lowercase().contains("model context protocol server"));
            
            if is_mcp_server && !is_excluded {
                results.push(SearchResult {
                    name: name.to_string(),
                    description: description.to_string(),
                    url: html_url.to_string(),
                    source: "github".to_string(),
                });
            }
        }
    }
    
    // Limit to top 5 most relevant results
    results.truncate(5);
    Ok(results)
}

async fn search_pulse_mcp(query: &str) -> Result<Vec<SearchResult>> {
    let client = reqwest::Client::new();
    // Use PulseMCP's search URL pattern
    let url = format!("https://www.pulsemcp.com/servers?q={}", urlencoding::encode(query));
    
    let response = client.get(&url)
        .header("User-Agent", "mcp-control-lite/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    let html = response.text().await?;
    let mut results = Vec::new();
    
    // Look for server links and try to extract install commands
    let server_link_pattern = regex::Regex::new(r#"href="/servers/([^"]+)""#).unwrap();
    let mut found_servers = std::collections::HashSet::new();
    
    for cap in server_link_pattern.captures_iter(&html) {
        if let Some(server_name) = cap.get(1) {
            let name = server_name.as_str();
            
            // Skip duplicates and pagination links
            if found_servers.contains(name) || name.contains("page=") || name.contains("?") {
                continue;
            }
            
            found_servers.insert(name.to_string());
            
            // Create a clean display name
            let display_name = name.replace("-", " ").replace("_", " ");
            let description = format!("MCP server from PulseMCP registry");
            let server_url = format!("https://www.pulsemcp.com/servers/{}", name);
            
            // Try to extract install command from the server name pattern
            let install_cmd = if name.contains("-") {
                // Try common patterns: author-package, package-mcp, etc.
                let parts: Vec<&str> = name.split('-').collect();
                if parts.len() >= 2 {
                    // Common patterns for npm packages
                    if parts[0].len() > 2 && parts[1] != "mcp" {
                        format!("npm install @{}/{}", parts[0], parts[1..].join("-"))
                    } else {
                        format!("npm install {}", name)
                    }
                } else {
                    format!("npm install {}", name)
                }
            } else {
                format!("npm install {}", name)
            };
            
            results.push(SearchResult {
                name: display_name,
                description,
                url: server_url,
                source: install_cmd, // Store install command in source field
            });
            
            // Limit results to avoid too many matches
            if results.len() >= 5 {
                break;
            }
        }
    }
    
    Ok(results)
}
