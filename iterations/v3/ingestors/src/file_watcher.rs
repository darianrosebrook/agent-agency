//! @darianrosebrook
//! File watcher for multimodal content with debouncing and size stability

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::mpsc;
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
    pending_events: HashMap<PathBuf, FileWatcherEvent>,
    tx: Option<mpsc::Sender<FileEvent>>,
}

#[derive(Debug, Clone)]
struct FileWatcherEvent {
    path: PathBuf,
    file_size: Option<u64>,
    last_seen: Instant,
}

impl FileWatcher {
    pub fn new(config: Option<FileWatcherConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            pending_events: HashMap::new(),
            tx: None,
        }
    }

    /// Start watching directory and emit file events
    pub async fn watch<F>(
        &mut self,
        root: &Path,
        _callback: F,
    ) -> Result<()>
    where
        F: Fn(FileEvent) + Send + 'static,
    {
        debug!("Starting file watcher on: {:?}", root);

        // TODO: PLACEHOLDER - Integrate with notify crate
        // 1. Set up recursive watcher on root directory
        // 2. Debounce rapid changes
        // 3. Check file size stability
        // 4. Route to appropriate ingestor based on extension
        // 5. Emit file events via callback

        info!("File watcher initialized (placeholder)");

        Ok(())
    }

    /// Check if file should be ignored
    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.config
            .ignore_patterns
            .iter()
            .any(|pattern| self.matches_pattern(&path_str, pattern))
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
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
            "mp4" | "mov" | "avi" | "mkv" => Some(IngestorType::Video),
            "pdf" | "key" | "ppt" | "pptx" => Some(IngestorType::Slides),
            "svg" | "graphml" => Some(IngestorType::Diagrams),
            "srt" | "vtt" => Some(IngestorType::Captions),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestorType {
    Video,
    Slides,
    Diagrams,
    Captions,
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
    }

    #[test]
    fn test_should_ignore() {
        let watcher = FileWatcher::new(None);
        assert!(watcher.should_ignore(Path::new(".gitignore")));
        assert!(watcher.should_ignore(Path::new("file.tmp")));
        assert!(!watcher.should_ignore(Path::new("presentation.pdf")));
    }
}
