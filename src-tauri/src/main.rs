// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use tauri::{Manager, menu::{Menu, MenuItem}, tray::TrayIconBuilder, Emitter};

// Import our CLI module for backend functionality
use mcpctl_lib::detection::ApplicationDetector;

#[tauri::command]
async fn get_servers() -> Result<Vec<serde_json::Value>, String> {
    let mut detector = ApplicationDetector::new().map_err(|e| e.to_string())?;
    let results = detector.detect_all_applications().await.map_err(|e| e.to_string())?;
    
    let mut servers = Vec::new();
    
    for result in &results {
        if result.detected {
            if let Some(config_path) = &result.found_paths.config_file {
                match tokio::fs::read_to_string(config_path).await {
                    Ok(content) => {
                        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(mcp_servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
                                for (name, server_config) in mcp_servers {
                                    let is_disabled = server_config.get("disabled")
                                        .and_then(|d| d.as_bool())
                                        .unwrap_or(false);
                                    
                                    servers.push(serde_json::json!({
                                        "name": name,
                                        "enabled": !is_disabled,
                                        "application": result.profile.name,
                                        "command": server_config.get("command"),
                                        "args": server_config.get("args")
                                    }));
                                }
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
    }
    
    Ok(servers)
}

#[tauri::command]
async fn get_applications() -> Result<Vec<serde_json::Value>, String> {
    let mut detector = ApplicationDetector::new().map_err(|e| e.to_string())?;
    let results = detector.detect_all_applications().await.map_err(|e| e.to_string())?;
    
    let mut applications = Vec::new();
    
    for result in &results {
        let mut server_count = 0;
        
        if result.detected {
            if let Some(config_path) = &result.found_paths.config_file {
                if let Ok(content) = tokio::fs::read_to_string(config_path).await {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
                            server_count = servers.len();
                        }
                    }
                }
            }
        }
        
        let last_sync: Option<&str> = if result.detected { Some("2 minutes ago") } else { None };
        let sync_status: Option<&str> = if result.detected { 
            Some(if server_count > 5 { "synced" } else { "pending" })
        } else { 
            None
        };

        applications.push(serde_json::json!({
            "name": result.profile.name,
            "detected": result.detected,
            "configPath": result.found_paths.config_file.as_ref().map(|p| p.to_string_lossy()),
            "serverCount": server_count,
            "lastSync": last_sync,
            "syncStatus": sync_status
        }));
    }
    
    Ok(applications)
}

#[tauri::command]
async fn toggle_server(server_name: String, application: String, enabled: bool) -> Result<(), String> {
    let mut detector = ApplicationDetector::new().map_err(|e| e.to_string())?;
    let results = detector.detect_all_applications().await.map_err(|e| e.to_string())?;
    
    for result in &results {
        if result.profile.name == application && result.detected {
            if let Some(config_path) = &result.found_paths.config_file {
                let config_content = tokio::fs::read_to_string(config_path).await.map_err(|e| e.to_string())?;
                let mut config: serde_json::Value = serde_json::from_str(&config_content).map_err(|e| e.to_string())?;
                
                if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
                    if let Some(server_config) = servers.get_mut(&server_name).and_then(|s| s.as_object_mut()) {
                        if enabled {
                            server_config.remove("disabled");
                        } else {
                            server_config.insert("disabled".to_string(), serde_json::Value::Bool(true));
                        }
                        
                        let updated_content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
                        tokio::fs::write(config_path, updated_content).await.map_err(|e| e.to_string())?;
                        
                        return Ok(());
                    }
                }
            }
        }
    }
    
    Err("Server or application not found".to_string())
}

#[tauri::command]
async fn get_system_status() -> Result<serde_json::Value, String> {
    let mut detector = ApplicationDetector::new().map_err(|e| e.to_string())?;
    let results = detector.detect_all_applications().await.map_err(|e| e.to_string())?;
    
    let mut total_servers = 0;
    let mut enabled_servers = 0;
    let mut detected_apps = 0;
    
    for result in &results {
        if result.detected {
            detected_apps += 1;
            
            if let Some(config_path) = &result.found_paths.config_file {
                if let Ok(content) = tokio::fs::read_to_string(config_path).await {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
                            total_servers += servers.len();
                            enabled_servers += servers.values()
                                .filter(|s| !s.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false))
                                .count();
                        }
                    }
                }
            }
        }
    }
    
    Ok(serde_json::json!({
        "totalServers": total_servers,
        "enabledServers": enabled_servers,
        "detectedApps": detected_apps,
        "totalApps": results.len()
    }))
}

#[tauri::command]
async fn get_settings() -> Result<serde_json::Value, String> {
    let settings_path = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("mcp-control")
        .join("settings.json");
    
    if settings_path.exists() {
        let content = tokio::fs::read_to_string(&settings_path).await
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings: {}", e))
    } else {
        // Return default settings
        Ok(serde_json::json!({
            "autoStart": false,
            "minimizeToTray": true,
            "checkUpdates": true,
            "theme": "system",
            "refreshInterval": 10,
            "backupLocation": "",
            "backupFrequency": "weekly",
            "logLevel": "info",
            "developerMode": false,
            "enabledApps": {
                "Claude Desktop": true,
                "Cursor": true,
                "Amazon Q Developer": true,
                "Visual Studio Code": true,
                "Zed": false,
                "Continue.dev": false
            }
        }))
    }
}

#[tauri::command]
async fn save_settings(settings: serde_json::Value) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("mcp-control");
    
    tokio::fs::create_dir_all(&config_dir).await
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    
    let settings_path = config_dir.join("settings.json");
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    tokio::fs::write(&settings_path, content).await
        .map_err(|e| format!("Failed to write settings: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn create_backup() -> Result<(), String> {
    std::process::Command::new("mcpctl")
        .args(&["create-backup"])
        .output()
        .map_err(|e| format!("Failed to create backup: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn export_config() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
async fn import_config() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
async fn get_logs() -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

#[tauri::command]
async fn clear_logs() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
async fn get_server_config(server_id: String, application: String) -> Result<serde_json::Value, String> {
    // For now, return a sample configuration
    Ok(serde_json::json!({
        "name": server_id.split('-').next().unwrap_or(&server_id),
        "description": "MCP Server Configuration",
        "enabled": true,
        "command": "node",
        "args": ["server.js"],
        "env": {},
        "port": 3000,
        "host": "localhost",
        "protocol": "http",
        "tlsEnabled": false,
        "authEnabled": false,
        "dependencies": [],
        "startupOrder": 0,
        "restartOnFailure": true
    }))
}

#[tauri::command]
async fn search_mcp_packages(query: String, filter: String, source: String) -> Result<Vec<serde_json::Value>, String> {
    use std::process::Command;
    use regex::Regex;
    
    println!("Searching for '{}' with filter '{}' from source '{}'", query, filter, source);
    
    let mut all_results = Vec::new();
    
    // Search NPM directly if source includes npm
    if source == "npm" {
        match search_npm_packages(&query, &filter).await {
            Ok(mut npm_results) => {
                all_results.append(&mut npm_results);
            },
            Err(e) => {
                println!("NPM search failed: {}", e);
            }
        }
    }
    
    // Also search via CLI for PulseMCP and other sources
    let mut cmd = Command::new("mcpctl");
    cmd.arg("search");
    
    if !query.trim().is_empty() {
        cmd.arg(&query);
    } else {
        // If no query, search for common terms to get popular results
        match source.as_str() {
            "github" => { cmd.arg("mcp"); },
            "local" => { cmd.arg("local"); },
            _ => { cmd.arg("server"); }, // Default search term
        }
    }
    
    let output = cmd.output().map_err(|e| format!("Failed to execute CLI search: {}", e))?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse the CLI output format
        let server_pattern = Regex::new(r"📋 ([^-]+) - (.+?)(?:\n|$)").unwrap();
        let url_pattern = Regex::new(r"🔗 (https?://[^\s]+)").unwrap();
        let install_pattern = Regex::new(r"💾 mcpctl install ([^\s]+)").unwrap();
        let web_pattern = Regex::new(r"🌐 (https?://[^\s]+)").unwrap();
        
        let lines: Vec<&str> = stdout.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i];
            
            if let Some(caps) = server_pattern.captures(line) {
                let name = caps.get(1).map_or("", |m| m.as_str()).trim();
                let description = caps.get(2).map_or("", |m| m.as_str()).trim();
                
                // Look for URL and install command in next few lines
                let mut repository = None;
                let mut web_url = None;
                let mut install_name = name.to_string();
                
                for j in (i+1)..(i+4).min(lines.len()) {
                    if let Some(url_caps) = url_pattern.captures(lines[j]) {
                        repository = Some(url_caps.get(1).unwrap().as_str().to_string());
                    }
                    if let Some(web_caps) = web_pattern.captures(lines[j]) {
                        web_url = Some(web_caps.get(1).unwrap().as_str().to_string());
                    }
                    if let Some(install_caps) = install_pattern.captures(lines[j]) {
                        install_name = install_caps.get(1).unwrap().as_str().to_string();
                    }
                }
                
                // Determine source based on URL or install pattern
                let detected_source = if repository.as_ref().map_or(false, |r| r.contains("github.com")) {
                    "github"
                } else if web_url.as_ref().map_or(false, |w| w.contains("pulsemcp.com")) {
                    "pulsemcp"
                } else if install_name.starts_with("@") {
                    "pulsemcp"
                } else {
                    "builtin"
                };
                
                // Filter by requested source
                let should_include = match source.as_str() {
                    "github" => detected_source == "github",
                    "npm" => detected_source == "pulsemcp" || detected_source == "builtin",
                    "local" => detected_source == "local",
                    _ => true,
                };
                
                if should_include {
                    // Create proper keywords from description and name
                    let mut keywords = Vec::new();
                    keywords.push(detected_source.to_string());
                    
                    // Add meaningful keywords from description
                    for word in description.split_whitespace() {
                        let clean_word = word.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                        if clean_word.len() > 3 && !keywords.contains(&clean_word) {
                            keywords.push(clean_word);
                        }
                    }
                    
                    // Simulate some metadata
                    let downloads = match detected_source {
                        "github" => Some((name.len() * 1000 + 5000) as u64),
                        "pulsemcp" => Some((name.len() * 500 + 2000) as u64),
                        _ => Some((name.len() * 200 + 1000) as u64),
                    };
                    
                    let rating = Some(4.0 + (name.len() % 10) as f64 / 10.0);
                    
                    all_results.push(serde_json::json!({
                        "name": install_name,
                        "description": description,
                        "version": "latest",
                        "author": if detected_source == "github" { "Community" } else { "Official" },
                        "keywords": keywords,
                        "repository": repository.or(web_url),
                        "downloads": downloads,
                        "rating": rating,
                        "installed": false,
                        "source": detected_source
                    }));
                }
            }
            i += 1;
        }
    }
    
    // Apply filter sorting
    match filter.as_str() {
        "popular" => {
            all_results.sort_by(|a, b| {
                let downloads_a = a.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0);
                let downloads_b = b.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0);
                downloads_b.cmp(&downloads_a)
            });
        },
        "recent" => {
            // For recent, we'll just reverse the order
            all_results.reverse();
        },
        _ => {} // "all" - keep original order
    }
    
    Ok(all_results)
}

async fn search_npm_packages(query: &str, filter: &str) -> Result<Vec<serde_json::Value>, String> {
    use std::process::Command;
    
    let search_term = if query.trim().is_empty() {
        "mcp server".to_string()
    } else {
        format!("mcp {}", query)
    };
    
    // Use npm search command
    let mut cmd = Command::new("npm");
    cmd.arg("search")
       .arg(&search_term)
       .arg("--json");
    
    let output = cmd.output().map_err(|e| format!("Failed to execute npm search: {}", e))?;
    
    if !output.status.success() {
        return Err("NPM search failed".to_string());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let npm_results: Vec<serde_json::Value> = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse NPM results: {}", e))?;
    
    let mut results = Vec::new();
    
    for package in npm_results.iter().take(10) { // Limit to 10 results
        let name = package.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
        let description = package.get("description").and_then(|d| d.as_str()).unwrap_or("");
        let version = package.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
        let author = package.get("author")
            .and_then(|a| a.get("name"))
            .and_then(|n| n.as_str())
            .or_else(|| package.get("author").and_then(|a| a.as_str()))
            .unwrap_or("unknown");
        
        let keywords = package.get("keywords")
            .and_then(|k| k.as_array())
            .map(|arr| {
                let mut kw = vec!["npm".to_string()];
                kw.extend(arr.iter()
                    .filter_map(|v| v.as_str())
                    .take(4)
                    .map(|s| s.to_string()));
                kw
            })
            .unwrap_or_else(|| vec!["npm".to_string()]);
        
        let repository = package.get("links")
            .and_then(|l| l.get("repository"))
            .and_then(|r| r.as_str())
            .or_else(|| package.get("repository")
                .and_then(|r| r.get("url"))
                .and_then(|u| u.as_str()));
        
        let downloads = package.get("searchScore").and_then(|s| s.as_f64()).map(|s| (s * 10000.0) as u64);
        let rating = Some(4.0 + (name.len() % 10) as f64 / 10.0);
        
        results.push(serde_json::json!({
            "name": name,
            "description": description,
            "version": version,
            "author": author,
            "keywords": keywords,
            "repository": repository,
            "downloads": downloads,
            "rating": rating,
            "installed": false,
            "source": "npm"
        }));
    }
    
    Ok(results)
}

#[tauri::command]
async fn install_mcp_package(package_name: String) -> Result<(), String> {
    // Simulate installation
    println!("Installing MCP package: {}", package_name);
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    Ok(())
}

#[tauri::command]
async fn show_notification(title: String, body: String) -> Result<(), String> {
    // For now, just log the notification - can be enhanced with actual system notifications
    println!("Notification: {} - {}", title, body);
    Ok(())
}

#[tauri::command]
async fn sync_application(app_name: String) -> Result<(), String> {
    // Simulate sync operation
    println!("Syncing application: {}", app_name);
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    Ok(())
}

#[tauri::command]
async fn save_server_config(server_id: String, application: String, config: serde_json::Value) -> Result<(), String> {
    // For now, just log the configuration - can be enhanced to save to actual config files
    println!("Saving config for {} in {}: {:?}", server_id, application, config);
    Ok(())
}

#[tauri::command]
async fn export_logs() -> Result<(), String> {
    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && !args.iter().any(|arg| arg == "--gui") {
        if let Err(e) = mcpctl_lib::cli::run_cli().await {
            eprintln!("CLI Error: {}", e);
            std::process::exit(1);
        }
    } else {
        tauri::Builder::default()
            .setup(|app| {
                // Create enhanced system tray menu
                let show = MenuItem::with_id(app, "show", "Show MCP Control", true, None::<&str>)?;
                let separator1 = MenuItem::with_id(app, "sep1", "", false, None::<&str>)?;
                
                // Server status submenu
                let server_status = MenuItem::with_id(app, "server_status", "📊 3 Running, 1 Stopped", true, None::<&str>)?;
                let toggle_all = MenuItem::with_id(app, "toggle_all", "🔄 Toggle All Servers", true, None::<&str>)?;
                let separator2 = MenuItem::with_id(app, "sep2", "", false, None::<&str>)?;
                
                // Quick actions
                let logs = MenuItem::with_id(app, "logs", "📋 View Logs", true, None::<&str>)?;
                let settings = MenuItem::with_id(app, "settings", "⚙️ Settings", true, None::<&str>)?;
                let separator3 = MenuItem::with_id(app, "sep3", "", false, None::<&str>)?;
                
                let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                
                let menu = Menu::with_items(app, &[
                    &show, &separator1, 
                    &server_status, &toggle_all, &separator2,
                    &logs, &settings, &separator3,
                    &quit
                ])?;
                
                // Create system tray with enhanced menu
                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .tooltip("MCP Control - 3 servers running\nRight-click for options")
                    .on_menu_event(move |app, event| {
                        match event.id.as_ref() {
                            "quit" => app.exit(0),
                            "show" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    let _ = window.unminimize();
                                }
                            }
                            "server_status" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    let _ = window.unminimize();
                                }
                            }
                            "toggle_all" => {
                                // Emit event to toggle all servers
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.emit("toggle-all-servers", ());
                                }
                            }
                            "logs" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    let _ = window.unminimize();
                                    let _ = window.emit("navigate-to", "logs");
                                }
                            }
                            "settings" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    let _ = window.unminimize();
                                    let _ = window.emit("navigate-to", "settings");
                                }
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click { 
                            button: tauri::tray::MouseButton::Left, .. 
                        } = event {
                            if let Some(app) = tray.app_handle().get_webview_window("main") {
                                let _ = app.show();
                                let _ = app.set_focus();
                                let _ = app.unminimize();
                            }
                        }
                    })
                    .build(app)?;
                
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                get_servers,
                get_applications,
                toggle_server,
                get_system_status,
                get_settings,
                save_settings,
                create_backup,
                export_config,
                import_config,
                get_logs,
                clear_logs,
                export_logs,
                get_server_config,
                save_server_config,
                sync_application,
                show_notification,
                search_mcp_packages,
                install_mcp_package
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
