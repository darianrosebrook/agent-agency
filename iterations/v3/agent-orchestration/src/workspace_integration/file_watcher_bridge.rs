//! File Watcher Bridge
//!
//! Connects agent-data-processing::FileWatcher to system-resilience workspace state manager
//! @author @darianrosebrook

#[cfg(feature = "data-processing")]
use agent_data_processing::ingestion::FileWatcher as DataProcessingFileWatcher;
#[cfg(feature = "data-processing")]
use agent_data_processing::ingestion_runtime::IngestionCmd;
use std::path::PathBuf;
use std::sync::Arc;
use system_resilience::workspace_state::FileWatcherEventHandler;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Bridge connecting agent-data-processing FileWatcher to workspace state manager
#[cfg(feature = "data-processing")]
pub struct FileWatcherBridge {
    file_watcher: Arc<DataProcessingFileWatcher>,
    event_handler: Arc<FileWatcherEventHandler>,
    cmd_sender: broadcast::Sender<IngestionCmd>,
    watch_handle: Option<tokio::task::JoinHandle<()>>,
}

#[cfg(feature = "data-processing")]
impl FileWatcherBridge {
    /// Create new file watcher bridge
    pub fn new(
        file_watcher: DataProcessingFileWatcher,
        event_handler: Arc<FileWatcherEventHandler>,
    ) -> Result<Self, String> {
        let (cmd_tx, _) = broadcast::channel(100);

        // Bind file watcher to command channel
        let bound_watcher = file_watcher.bind(cmd_tx.clone());

        Ok(Self {
            file_watcher: Arc::new(bound_watcher),
            event_handler,
            cmd_sender: cmd_tx,
            watch_handle: None,
        })
    }

    /// Start watching and processing events
    pub async fn start(&mut self) -> Result<(), String> {
        // Start the file watcher
        self.file_watcher
            .start_watching()
            .await
            .map_err(|e| format!("Failed to start file watcher: {}", e))?;

        // Start event processing task
        let mut cmd_receiver = self.cmd_sender.subscribe();
        let handler = Arc::clone(&self.event_handler);

        self.watch_handle = Some(tokio::spawn(async move {
            while let Ok(cmd) = cmd_receiver.recv().await {
                match cmd {
                    IngestionCmd::FileUpsert { path } => {
                        // Determine if this is a create or modify
                        // Note: FileWatcher sends FileUpsert for both creates and modifies
                        // We'll treat it as modified if file exists, created if it doesn't
                        // However, by the time we process it, the file should exist
                        // So we'll check file metadata to determine
                        let is_create = match std::fs::metadata(&path) {
                            Ok(meta) => {
                                // Check if file is very new (created within last second)
                                meta.modified()
                                    .map(|m| {
                                        let age = std::time::SystemTime::now()
                                            .duration_since(m)
                                            .unwrap_or_default();
                                        age.as_secs() < 1
                                    })
                                    .unwrap_or(false)
                            }
                            Err(_) => true, // File doesn't exist, treat as create
                        };

                        if is_create {
                            debug!("File created: {:?}", path);
                            handler.handle_file_created(path.clone(), None);
                        } else {
                            debug!("File modified: {:?}", path);
                            handler.handle_file_modified(path.clone(), None);
                        }
                    }
                    IngestionCmd::FileRemove { path } => {
                        debug!("File deleted: {:?}", path);
                        handler.handle_file_deleted(path, None);
                    }
                }
            }
        }));

        info!("File watcher bridge started successfully");
        Ok(())
    }

    /// Stop watching
    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(handle) = self.watch_handle.take() {
            handle.abort();
        }
        info!("File watcher bridge stopped");
        Ok(())
    }

    /// Get the file watcher instance
    pub fn file_watcher(&self) -> &Arc<DataProcessingFileWatcher> {
        &self.file_watcher
    }
}

/// TODO: Document placeholder implementation for disabled feature
///       This is an intentional placeholder when data-processing feature is disabled.
///       Methods return errors indicating feature is required. Consider improving error messages.
///
/// Placeholder implementation when data-processing feature is disabled
#[cfg(not(feature = "data-processing"))]
pub struct FileWatcherBridge {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "data-processing"))]
impl FileWatcherBridge {
    pub fn new(
        _file_watcher: (),
        _event_handler: Arc<FileWatcherEventHandler>,
    ) -> Result<Self, String> {
        Err("File watcher bridge requires data-processing feature".to_string())
    }

    pub async fn start(&mut self) -> Result<(), String> {
        Err("File watcher bridge requires data-processing feature".to_string())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
}
