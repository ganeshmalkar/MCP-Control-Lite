use anyhow::{Result, Context};
use std::process::{Child, Command, Stdio};
use std::collections::HashMap;

/// Process management utilities for MCP servers
pub struct ProcessManager;

impl ProcessManager {
    /// Spawn a new MCP server process
    pub fn spawn_server(
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
        working_dir: Option<&str>,
    ) -> Result<Child> {
        let mut cmd = Command::new(command);
        
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set working directory if provided
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Add environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        cmd.spawn()
            .with_context(|| format!("Failed to spawn MCP server process: {}", command))
    }

    /// Check if a process is still running
    pub fn is_process_running(child: &mut Child) -> bool {
        match child.try_wait() {
            Ok(Some(_)) => false, // Process has exited
            Ok(None) => true,     // Process is still running
            Err(_) => false,      // Error checking process, assume not running
        }
    }

    /// Kill a process gracefully
    pub fn kill_process(child: &mut Child) -> Result<()> {
        child.kill()
            .context("Failed to kill process")?;
        
        // Wait for the process to actually exit
        let _ = child.wait();
        
        Ok(())
    }

    /// Get process ID
    pub fn get_process_id(child: &Child) -> u32 {
        child.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_simple_process() {
        let env = HashMap::new();
        let result = ProcessManager::spawn_server("echo", &["hello".to_string()], &env, None);
        assert!(result.is_ok());
        
        if let Ok(mut child) = result {
            let pid = ProcessManager::get_process_id(&child);
            assert!(pid > 0);
            
            // Clean up
            let _ = child.wait();
        }
    }

    #[test]
    fn test_spawn_nonexistent_command() {
        let env = HashMap::new();
        let result = ProcessManager::spawn_server("nonexistent-command-12345", &[], &env, None);
        assert!(result.is_err());
    }
}
