use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// File system event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileEvent {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

/// File system watch event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    /// Event ID
    pub id: Uuid,
    
    /// File path that triggered the event
    pub path: PathBuf,
    
    /// Type of event
    pub event_type: FileEvent,
    
    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,
    
    /// File size at time of event (if available)
    pub file_size: Option<u64>,
    
    /// File hash at time of event (if available)
    pub file_hash: Option<String>,
}

impl WatchEvent {
    pub fn new(path: PathBuf, event_type: FileEvent) -> Self {
        Self {
            id: Uuid::new_v4(),
            path,
            event_type,
            timestamp: Utc::now(),
            file_size: None,
            file_hash: None,
        }
    }
    
    pub fn with_metadata(mut self, size: Option<u64>, hash: Option<String>) -> Self {
        self.file_size = size;
        self.file_hash = hash;
        self
    }
}

/// Configuration file watcher
pub struct ConfigWatcher {
    /// Watched paths and their handlers
    watched_paths: Arc<Mutex<Vec<WatchedPath>>>,
    
    /// Event sender
    event_sender: Option<Sender<WatchEvent>>,
    
    /// Whether the watcher is running
    is_running: Arc<Mutex<bool>>,
    
    /// Polling interval for file changes
    poll_interval: Duration,
}

/// Internal structure for tracking watched paths
#[derive(Debug, Clone)]
struct WatchedPath {
    path: PathBuf,
    last_modified: Option<DateTime<Utc>>,
    last_size: Option<u64>,
    last_hash: Option<String>,
}

impl Default for ConfigWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigWatcher {
    /// Create a new configuration file watcher
    pub fn new() -> Self {
        Self {
            watched_paths: Arc::new(Mutex::new(Vec::new())),
            event_sender: None,
            is_running: Arc::new(Mutex::new(false)),
            poll_interval: Duration::from_secs(1), // Default 1 second polling
        }
    }
    
    /// Set the polling interval
    pub fn set_poll_interval(&mut self, interval: Duration) {
        self.poll_interval = interval;
    }
    
    /// Start watching files and return a receiver for events
    pub fn start_watching(&mut self) -> Result<Receiver<WatchEvent>> {
        let (sender, receiver) = mpsc::channel();
        self.event_sender = Some(sender.clone());
        
        let watched_paths = Arc::clone(&self.watched_paths);
        let is_running = Arc::clone(&self.is_running);
        let poll_interval = self.poll_interval;
        
        // Set running flag
        {
            let mut running = is_running.lock().unwrap();
            *running = true;
        }
        
        // Start the watcher thread
        thread::spawn(move || {
            Self::watch_loop(watched_paths, sender, is_running, poll_interval);
        });
        
        Ok(receiver)
    }
    
    /// Stop watching files
    pub fn stop_watching(&mut self) {
        if let Ok(mut running) = self.is_running.lock() {
            *running = false;
        }
        self.event_sender = None;
    }
    
    /// Add a path to watch
    pub fn watch_path<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
        }
        
        let metadata = std::fs::metadata(&path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
        
        let last_modified = metadata.modified()
            .map(DateTime::<Utc>::from)
            .ok();
        
        let last_size = if metadata.is_file() {
            Some(metadata.len())
        } else {
            None
        };
        
        let last_hash = if metadata.is_file() {
            Self::calculate_file_hash(&path).ok()
        } else {
            None
        };
        
        let watched_path = WatchedPath {
            path: path.clone(),
            last_modified,
            last_size,
            last_hash,
        };
        
        if let Ok(mut paths) = self.watched_paths.lock() {
            // Remove existing entry for this path if it exists
            paths.retain(|p| p.path != path);
            paths.push(watched_path);
        }
        
        Ok(())
    }
    
    /// Remove a path from watching
    pub fn unwatch_path<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        
        if let Ok(mut paths) = self.watched_paths.lock() {
            paths.retain(|p| p.path != path);
        }
        
        Ok(())
    }
    
    /// Get list of currently watched paths
    pub fn get_watched_paths(&self) -> Vec<PathBuf> {
        if let Ok(paths) = self.watched_paths.lock() {
            paths.iter().map(|p| p.path.clone()).collect()
        } else {
            Vec::new()
        }
    }
    
    /// Check if a path is being watched
    pub fn is_watching<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        if let Ok(paths) = self.watched_paths.lock() {
            paths.iter().any(|p| p.path == path)
        } else {
            false
        }
    }
    
    /// Main watch loop that runs in a separate thread
    fn watch_loop(
        watched_paths: Arc<Mutex<Vec<WatchedPath>>>,
        sender: Sender<WatchEvent>,
        is_running: Arc<Mutex<bool>>,
        poll_interval: Duration,
    ) {
        while {
            let running = is_running.lock().unwrap();
            *running
        } {
            // Check each watched path for changes
            if let Ok(mut paths) = watched_paths.lock() {
                for watched_path in paths.iter_mut() {
                    if let Err(e) = Self::check_path_for_changes(watched_path, &sender) {
                        eprintln!("Error checking path {}: {}", watched_path.path.display(), e);
                    }
                }
            }
            
            thread::sleep(poll_interval);
        }
    }
    
    /// Check a single path for changes
    fn check_path_for_changes(
        watched_path: &mut WatchedPath,
        sender: &Sender<WatchEvent>,
    ) -> Result<()> {
        let path = &watched_path.path;
        
        if !path.exists() {
            // File was deleted
            let event = WatchEvent::new(path.clone(), FileEvent::Deleted);
            let _ = sender.send(event);
            return Ok(());
        }
        
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
        
        let current_modified = metadata.modified()
            .map(DateTime::<Utc>::from)
            .ok();
        
        let current_size = if metadata.is_file() {
            Some(metadata.len())
        } else {
            None
        };
        
        // Check if file was modified
        let was_modified = match (&watched_path.last_modified, &current_modified) {
            (Some(last), Some(current)) => last != current,
            (None, Some(_)) => true, // File was created
            _ => false,
        };
        
        let size_changed = watched_path.last_size != current_size;
        
        if was_modified || size_changed {
            let current_hash = if metadata.is_file() {
                Self::calculate_file_hash(path).ok()
            } else {
                None
            };
            
            // Determine event type
            let event_type = if watched_path.last_modified.is_none() {
                FileEvent::Created
            } else {
                FileEvent::Modified
            };
            
            let event = WatchEvent::new(path.clone(), event_type)
                .with_metadata(current_size, current_hash.clone());
            
            let _ = sender.send(event);
            
            // Update watched path state
            watched_path.last_modified = current_modified;
            watched_path.last_size = current_size;
            watched_path.last_hash = current_hash;
        }
        
        Ok(())
    }
    
    /// Calculate SHA-256 hash of a file
    fn calculate_file_hash(path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let content = std::fs::read(path)
            .with_context(|| format!("Failed to read file for hashing: {}", path.display()))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = hasher.finalize();
        
        Ok(format!("{:x}", hash))
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        self.stop_watching();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_watch_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        let mut watcher = ConfigWatcher::new();
        watcher.set_poll_interval(Duration::from_millis(100));
        
        // Create the file first
        fs::write(&test_file, "test content").unwrap();
        
        // Start watching the file
        watcher.watch_path(&test_file).unwrap();
        let receiver = watcher.start_watching().unwrap();
        
        // Wait a bit to establish baseline
        thread::sleep(Duration::from_millis(150));
        
        // Modify the file to trigger an event
        fs::write(&test_file, "modified content").unwrap();
        
        // Wait for the watcher to detect the change
        thread::sleep(Duration::from_millis(200));
        
        // Check if we received any events
        let events: Vec<_> = receiver.try_iter().collect();
        assert!(!events.is_empty(), "Expected to receive file modification events");
        
        watcher.stop_watching();
    }
    
    #[test]
    fn test_watch_file_modification() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Create initial file
        fs::write(&test_file, "initial content").unwrap();
        
        let mut watcher = ConfigWatcher::new();
        watcher.set_poll_interval(Duration::from_millis(100));
        watcher.watch_path(&test_file).unwrap();
        
        let receiver = watcher.start_watching().unwrap();
        
        // Wait a bit to establish baseline
        thread::sleep(Duration::from_millis(150));
        
        // Modify the file
        fs::write(&test_file, "modified content").unwrap();
        
        // Wait for the watcher to detect the change
        thread::sleep(Duration::from_millis(200));
        
        // Check if we received modification events
        let events: Vec<_> = receiver.try_iter().collect();
        let has_modified_event = events.iter().any(|e| matches!(e.event_type, FileEvent::Modified));
        assert!(has_modified_event);
        
        watcher.stop_watching();
    }
    
    #[test]
    fn test_watch_file_deletion() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Create initial file
        fs::write(&test_file, "content").unwrap();
        
        let mut watcher = ConfigWatcher::new();
        watcher.set_poll_interval(Duration::from_millis(100));
        watcher.watch_path(&test_file).unwrap();
        
        let receiver = watcher.start_watching().unwrap();
        
        // Wait a bit to establish baseline
        thread::sleep(Duration::from_millis(150));
        
        // Delete the file
        fs::remove_file(&test_file).unwrap();
        
        // Wait for the watcher to detect the deletion
        thread::sleep(Duration::from_millis(200));
        
        // Check if we received deletion events
        let events: Vec<_> = receiver.try_iter().collect();
        let has_deleted_event = events.iter().any(|e| matches!(e.event_type, FileEvent::Deleted));
        assert!(has_deleted_event);
        
        watcher.stop_watching();
    }
    
    #[test]
    fn test_multiple_watched_paths() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        
        // Create initial files
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();
        
        let mut watcher = ConfigWatcher::new();
        watcher.set_poll_interval(Duration::from_millis(100));
        watcher.watch_path(&file1).unwrap();
        watcher.watch_path(&file2).unwrap();
        
        assert_eq!(watcher.get_watched_paths().len(), 2);
        assert!(watcher.is_watching(&file1));
        assert!(watcher.is_watching(&file2));
        
        watcher.unwatch_path(&file1).unwrap();
        assert_eq!(watcher.get_watched_paths().len(), 1);
        assert!(!watcher.is_watching(&file1));
        assert!(watcher.is_watching(&file2));
    }
}
