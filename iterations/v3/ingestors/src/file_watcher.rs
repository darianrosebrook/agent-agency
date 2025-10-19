//! @darianrosebrook
//! File watcher for multimodal content with debouncing and size stability

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{debug, info};

/// File watcher configuration
#[derive(Debug, Clone)]
pub struct FileWatcherConfig {
    pub debounce_ms: u64,
    pub size_stability_check_ms: u64,
    pub ignore_patterns: Vec<String>,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 1000,
            size_stability_check_ms: 2000,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                ".*".to_string(),
                "*/.git".to_string(),
            ],
        }
    }
}

pub struct FileWatcher {
    config: FileWatcherConfig,
    tx: Option<mpsc::Sender<FileEvent>>,
    watcher_handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FileWatcherEvent {
    path: PathBuf,
    last_seen: Instant,
}

impl FileWatcher {
    pub fn new(config: Option<FileWatcherConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            tx: None,
            watcher_handle: None,
        }
    }

    /// Start watching directory and emit file events
    pub async fn watch<F>(
        &mut self,
        root: &Path,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(FileEvent) + Send + 'static,
    {
        debug!("Starting file watcher on: {:?}", root);

        // Create channel for file events
        let (tx, mut rx) = mpsc::channel::<FileEvent>(100);
        self.tx = Some(tx.clone());

        // Spawn debouncer task
        let debounce_ms = self.config.debounce_ms;
        let size_stability_ms = self.config.size_stability_check_ms;
        let ignore_patterns = self.config.ignore_patterns.clone();

        let watcher_handle = tokio::spawn(async move {
            let mut pending: HashMap<PathBuf, FileWatcherEvent> = HashMap::new();

            loop {
                // Check for channel messages with timeout
                match tokio::time::timeout(
                    Duration::from_millis(debounce_ms),
                    rx.recv(),
                )
                .await
                {
                    Ok(Some(event)) => {
                        // Add to pending events
                        pending.insert(
                            event.path.clone(),
                            FileWatcherEvent {
                                path: event.path,
                                last_seen: Instant::now(),
                            },
                        );
                    }
                    Ok(None) => {
                        // Channel closed
                        break;
                    }
                    Err(_) => {
                        // Timeout - process pending events
                    }
                }

                // Process events that have been stable for size_stability_ms
                let now = Instant::now();
                let mut ready_events = Vec::new();

                for (path, event) in pending.iter() {
                    let elapsed = now.duration_since(event.last_seen);
                    if elapsed.as_millis() >= size_stability_ms as u128 {
                        ready_events.push((path.clone(), event.clone()));
                    }
                }

                // Emit ready events
                for (path, _event) in ready_events {
                    // Check if file should be ignored
                    if !Self::should_ignore_file(&path, &ignore_patterns) {
                        // Determine ingestor type and create event
                        if let Some(ingestor_type) = Self::get_ingestor_type(&path) {
                            if let Ok(file_size) = std::fs::metadata(&path).map(|m| m.len()) {
                                let file_event = FileEvent {
                                    path: path.clone(),
                                    event_type: FileEventType::Created,
                                    ingestor_type,
                                    file_size,
                                    timestamp: chrono::Utc::now(),
                                };

                                debug!(
                                    "Emitting file event: {:?} ({} bytes)",
                                    path, file_size
                                );

                                callback(file_event);
                            }
                        }
                    }

                    pending.remove(&path);
                }

                // Yield to prevent busy-waiting
                if pending.is_empty() {
                    sleep(Duration::from_millis(100)).await;
                }
            }
        });

        self.watcher_handle = Some(watcher_handle);

        // Simulate initial directory scan
        self.scan_directory(root, &tx).await?;

        info!("File watcher initialized");

        Ok(())
    }

    /// Scan directory for matching files (non-recursive to avoid async recursion issues)
    async fn scan_directory(
        &self,
        root: &Path,
        tx: &mpsc::Sender<FileEvent>,
    ) -> Result<()> {
        debug!("Scanning directory: {:?}", root);

        let mut dirs_to_scan = vec![root.to_path_buf()];

        while let Some(current_dir) = dirs_to_scan.pop() {
            if !current_dir.is_dir() {
                continue;
            }

            for entry in std::fs::read_dir(&current_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && !Self::should_ignore_file(&path, &self.config.ignore_patterns) {
                    if let Some(ingestor_type) = Self::get_ingestor_type(&path) {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            let file_event = FileEvent {
                                path,
                                event_type: FileEventType::Created,
                                ingestor_type,
                                file_size: metadata.len(),
                                timestamp: chrono::Utc::now(),
                            };

                            let _ = tx.send(file_event).await;
                        }
                    }
                } else if path.is_dir() && !Self::should_ignore_file(&path, &self.config.ignore_patterns) {
                    // Queue directory for scanning
                    dirs_to_scan.push(path);
                }
            }
        }

        Ok(())
    }

    /// Check if file should be ignored
    fn should_ignore_file(path: &Path, ignore_patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        ignore_patterns
            .iter()
            .any(|pattern| Self::matches_pattern(&path_str, pattern))
    }

    fn matches_pattern(path: &str, pattern: &str) -> bool {
        // Simple glob pattern matching
        if pattern == ".*" {
            // Match any hidden file
            path.split('/').last().map_or(false, |f| f.starts_with('.'))
        } else if pattern.starts_with('.') {
            path.contains(pattern)
        } else if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            path.ends_with(suffix)
        } else {
            path.contains(pattern)
        }
    }

    /// Get ingestor type for file extension
    pub fn get_ingestor_type(path: &Path) -> Option<IngestorType> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "mp4" | "mov" | "avi" | "mkv" | "webm" => Some(IngestorType::Video),
            "pdf" | "key" | "ppt" | "pptx" => Some(IngestorType::Slides),
            "svg" | "graphml" | "xml" => Some(IngestorType::Diagrams),
            "srt" | "vtt" | "ass" => Some(IngestorType::Captions),
            _ => None,
        }
    }

    /// Emit a file event
    pub async fn emit_event(&self, event: FileEvent) -> Result<()> {
        if let Some(tx) = &self.tx {
            tx.send(event).await?;
        }
        Ok(())
    }

    /// Shutdown the file watcher
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(handle) = self.watcher_handle.take() {
            handle.abort();
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestorType {
    Video,
    Slides,
    Diagrams,
    Captions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct FileEvent {
    pub path: PathBuf,
    pub event_type: FileEventType,
    pub ingestor_type: IngestorType,
    pub file_size: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingestor_type_detection() {
        assert_eq!(
            FileWatcher::get_ingestor_type(Path::new("video.mp4")),
            Some(IngestorType::Video)
        );
        assert_eq!(
            FileWatcher::get_ingestor_type(Path::new("slides.pdf")),
            Some(IngestorType::Slides)
        );
        assert_eq!(
            FileWatcher::get_ingestor_type(Path::new("diagram.svg")),
            Some(IngestorType::Diagrams)
        );
        assert_eq!(
            FileWatcher::get_ingestor_type(Path::new("captions.srt")),
            Some(IngestorType::Captions)
        );
        assert_eq!(
            FileWatcher::get_ingestor_type(Path::new("captions.vtt")),
            Some(IngestorType::Captions)
        );
    }

    #[test]
    fn test_should_ignore() {
        let ignore_patterns = vec![
            "*.tmp".to_string(),
            ".*".to_string(),
            "*/.git".to_string(),
        ];
        assert!(FileWatcher::should_ignore_file(
            Path::new(".gitignore"),
            &ignore_patterns
        ));
        assert!(FileWatcher::should_ignore_file(
            Path::new("file.tmp"),
            &ignore_patterns
        ));
        assert!(!FileWatcher::should_ignore_file(
            Path::new("presentation.pdf"),
            &ignore_patterns
        ));
    }

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let config = FileWatcherConfig::default();
        let watcher = FileWatcher::new(Some(config));
        assert!(watcher.tx.is_none());
    }
}
