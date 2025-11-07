//! Unified Workspace State Manager
//!
//! Combines file watching, state capture, context generation, and embedding integration
//! into a single cohesive interface for workspace state management.

use super::context_generator::{ContextGenerator, ContextCriteria, WorkspaceContext};
use super::events::{ContextType, WorkspaceStateEvent};
use super::state_manager::WorkspaceStateManager;
use super::state_types::{StateId, WorkspaceConfig, WorkspaceError, WorkspaceResult, WorkspaceState, WorkspaceDiff};
use super::StateStorage;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::info;

/// Configuration for unified workspace state manager
#[derive(Debug, Clone)]
pub struct UnifiedWorkspaceConfig {
    /// Core state management configuration
    pub state_config: WorkspaceConfig,
    
    /// File watching configuration (optional)
    pub watch_config: Option<FileWatchConfig>,
    
    /// Context generation configuration (optional)
    pub context_config: Option<ContextGenerationConfig>,
    
    /// Metrics configuration
    pub metrics_config: MetricsConfig,
}

impl Default for UnifiedWorkspaceConfig {
    fn default() -> Self {
        Self {
            state_config: WorkspaceConfig::default(),
            watch_config: None,
            context_config: None,
            metrics_config: MetricsConfig::default(),
        }
    }
}

/// File watching configuration
#[derive(Debug, Clone)]
pub struct FileWatchConfig {
    /// Enable file watching
    pub enabled: bool,
    
    /// Paths to watch
    pub watch_paths: Vec<PathBuf>,
    
    /// File patterns to match (glob patterns)
    pub file_patterns: Vec<String>,
    
    /// Debounce duration in milliseconds
    pub debounce_ms: u64,
    
    /// Automatically capture state on file changes
    pub auto_capture_state: bool,
    
    /// Generate embeddings for changed files
    pub generate_embeddings: bool,
    
    /// File extensions to embed
    pub embedding_extensions: Vec<String>,
}

impl Default for FileWatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            watch_paths: vec![],
            file_patterns: vec!["**/*".to_string()],
            debounce_ms: 1000,
            auto_capture_state: false,
            generate_embeddings: false,
            embedding_extensions: vec![
                "ts".to_string(), "js".to_string(), "tsx".to_string(), "jsx".to_string(),
                "rs".to_string(), "py".to_string(), "java".to_string(), "cpp".to_string(),
                "c".to_string(), "h".to_string(), "hpp".to_string(),
                "md".to_string(), "txt".to_string(), "json".to_string(),
                "yaml".to_string(), "yml".to_string(), "toml".to_string(),
            ],
        }
    }
}

/// Context generation configuration
#[derive(Debug, Clone)]
pub struct ContextGenerationConfig {
    /// Enable context generation
    pub enabled: bool,
    
    /// Enable code context generation
    pub code_context_enabled: bool,
    
    /// Enable documentation context generation
    pub docs_context_enabled: bool,
    
    /// Enable configuration context generation
    pub config_context_enabled: bool,
    
    /// Maximum files per context
    pub max_files_per_context: usize,
    
    /// Similarity threshold for file selection
    pub similarity_threshold: f32,
    
    /// Language filters for code context
    pub language_filters: Vec<String>,
    
    /// Framework filters for code context
    pub framework_filters: Vec<String>,
}

impl Default for ContextGenerationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            code_context_enabled: true,
            docs_context_enabled: true,
            config_context_enabled: true,
            max_files_per_context: 50,
            similarity_threshold: 0.7,
            language_filters: vec![],
            framework_filters: vec![],
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    
    /// Metrics update interval in seconds
    pub update_interval_secs: u64,
    
    /// Enable detailed metrics
    pub detailed_metrics: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval_secs: 30,
            detailed_metrics: false,
        }
    }
}

/// Workspace metrics
#[derive(Debug, Clone, Default)]
pub struct WorkspaceMetrics {
    pub watcher: WatcherMetrics,
    pub snapshots: SnapshotMetrics,
    pub context: ContextMetrics,
    pub embeddings: EmbeddingMetrics,
    pub memory: MemoryMetrics,
}

/// File watcher metrics
#[derive(Debug, Clone, Default)]
pub struct WatcherMetrics {
    pub files_watched: usize,
    pub events_processed: usize,
    pub events_per_second: f64,
    pub debounce_hits: usize,
    pub errors: usize,
}

/// Snapshot metrics
#[derive(Debug, Clone, Default)]
pub struct SnapshotMetrics {
    pub total_snapshots: usize,
    pub average_snapshot_time_ms: f64,
    pub largest_snapshot_size_bytes: u64,
    pub last_snapshot_time: Option<DateTime<Utc>>,
}

/// Context generation metrics
#[derive(Debug, Clone, Default)]
pub struct ContextMetrics {
    pub requests_processed: usize,
    pub average_context_time_ms: f64,
    pub average_files_selected: f64,
    pub code_context_requests: usize,
    pub docs_context_requests: usize,
    pub config_context_requests: usize,
}

/// Embedding generation metrics
#[derive(Debug, Clone, Default)]
pub struct EmbeddingMetrics {
    pub embeddings_generated: usize,
    pub embeddings_failed: usize,
    pub average_generation_time_ms: f64,
    pub files_embedded: usize,
    pub last_embedding_time: Option<DateTime<Utc>>,
}

/// Memory metrics
#[derive(Debug, Clone, Default)]
pub struct MemoryMetrics {
    pub heap_used_bytes: u64,
    pub heap_total_bytes: u64,
    pub external_bytes: u64,
}

/// Unified workspace state manager
pub struct UnifiedWorkspaceStateManager {
    /// Core state management
    state_manager: Arc<WorkspaceStateManager>,
    
    /// Context generation (optional)
    context_generator: Option<Arc<ContextGenerator>>,
    
    /// File watcher event handler
    file_watcher_handler: Option<Arc<super::file_watcher_trait::FileWatcherEventHandler>>,
    file_watcher_handle: Option<tokio::task::JoinHandle<()>>,
    
    /// Metrics collection
    metrics: Arc<RwLock<WorkspaceMetrics>>,
    metrics_handle: Option<tokio::task::JoinHandle<()>>,
    
    /// Event channels
    event_sender: broadcast::Sender<WorkspaceStateEvent>,
    
    /// Configuration
    config: UnifiedWorkspaceConfig,
    
    /// Workspace root
    workspace_root: PathBuf,
    
    /// Initialization flag
    initialized: bool,
}

impl UnifiedWorkspaceStateManager {
    /// Create a new unified workspace state manager
    pub fn new(
        workspace_root: impl AsRef<Path>,
        config: UnifiedWorkspaceConfig,
        storage: Box<dyn StateStorage>,
    ) -> Self {
        let state_manager = Arc::new(WorkspaceStateManager::new(
            workspace_root.as_ref(),
            config.state_config.clone(),
            storage,
        ));
        
        // Create context generator if enabled
        let context_generator = if config.context_config.as_ref()
            .map(|c| c.enabled)
            .unwrap_or(false) {
            Some(Arc::new(ContextGenerator::new(
                Arc::clone(&state_manager),
                config.context_config.as_ref().unwrap().clone(),
            )))
        } else {
            None
        };
        
        let (event_sender, _) = broadcast::channel(100);
        
        // Create file watcher event handler if enabled
        let file_watcher_handler = if config.watch_config.as_ref()
            .map(|c| c.enabled)
            .unwrap_or(false) {
            Some(Arc::new(super::file_watcher_trait::FileWatcherEventHandler::new(
                event_sender.clone(),
                workspace_root.as_ref(),
            )))
        } else {
            None
        };
        
        Self {
            state_manager,
            context_generator,
            file_watcher_handler,
            file_watcher_handle: None,
            metrics: Arc::new(RwLock::new(WorkspaceMetrics::default())),
            metrics_handle: None,
            event_sender,
            config,
            workspace_root: workspace_root.as_ref().to_path_buf(),
            initialized: false,
        }
    }
    
    /// Initialize all enabled components
    pub async fn initialize(&mut self) -> Result<(), WorkspaceError> {
        if self.initialized {
            return Ok(());
        }
        
        // Start metrics collection if enabled
        if self.config.metrics_config.enabled {
            self.start_metrics_collection();
        }
        
        // Start file watcher if enabled
        // Note: Actual file watcher implementation should be provided externally
        // and connected via the event handler
        if self.file_watcher_handler.is_some() {
            self.start_file_watching_internal().await?;
        }
        
        self.initialized = true;
        info!("Unified workspace state manager initialized");
        
        Ok(())
    }
    
    /// Shutdown all components gracefully
    pub async fn shutdown(&mut self) -> Result<(), WorkspaceError> {
        if !self.initialized {
            return Ok(());
        }
        
        // Stop metrics collection
        if let Some(handle) = self.metrics_handle.take() {
            handle.abort();
        }
        
        // Stop file watcher
        if let Some(handle) = self.file_watcher_handle.take() {
            handle.abort();
        }
        
        // TODO: Flush pending embeddings
        
        self.initialized = false;
        info!("Unified workspace state manager shut down");
        
        Ok(())
    }
    
    /// Capture current workspace state
    pub async fn capture_state(&self) -> Result<WorkspaceResult<StateId>, WorkspaceError> {
        let start_time = std::time::Instant::now();
        let result = self.state_manager.capture_state().await?;
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Get state size for metrics
        let state = self.state_manager.get_state(result.data).await?;
        let size_bytes = state.total_size;
        
        // Record metrics
        self.record_snapshot(duration_ms, size_bytes).await;
        
        // Emit event
        let _ = self.event_sender.send(WorkspaceStateEvent::StateCaptured {
            state_id: result.data,
            duration_ms,
        });
        
        Ok(result)
    }
    
    /// Get workspace state by ID
    pub async fn get_state(&self, id: StateId) -> Result<WorkspaceState, WorkspaceError> {
        self.state_manager.get_state(id).await
    }
    
    /// Compute diff between two states
    pub async fn compute_diff(
        &self,
        from: StateId,
        to: StateId,
    ) -> Result<WorkspaceResult<WorkspaceDiff>, WorkspaceError> {
        self.state_manager.compute_diff(from, to).await
    }
    
    /// Generate code-specific context
    pub async fn generate_code_context(
        &self,
        language: Option<&str>,
        framework: Option<&str>,
    ) -> Result<WorkspaceContext, WorkspaceError> {
        let start_time = std::time::Instant::now();
        
        let context_generator = self.context_generator.as_ref()
            .ok_or_else(|| WorkspaceError::Configuration("Context generation not enabled".to_string()))?;
        
        let context = context_generator.generate_code_context(language, framework).await?;
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Record metrics
        self.record_context_generation(ContextType::Code, duration_ms, context.files.len()).await;
        
        // Emit event
        let _ = self.event_sender.send(WorkspaceStateEvent::ContextGenerated {
            context_type: ContextType::Code,
            files_selected: context.files.len(),
            duration_ms,
        });
        
        Ok(context)
    }
    
    /// Generate documentation context
    pub async fn generate_documentation_context(&self) -> Result<WorkspaceContext, WorkspaceError> {
        let start_time = std::time::Instant::now();
        
        let context_generator = self.context_generator.as_ref()
            .ok_or_else(|| WorkspaceError::Configuration("Context generation not enabled".to_string()))?;
        
        let context = context_generator.generate_documentation_context().await?;
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Record metrics
        self.record_context_generation(ContextType::Documentation, duration_ms, context.files.len()).await;
        
        // Emit event
        let _ = self.event_sender.send(WorkspaceStateEvent::ContextGenerated {
            context_type: ContextType::Documentation,
            files_selected: context.files.len(),
            duration_ms,
        });
        
        Ok(context)
    }
    
    /// Generate configuration context
    pub async fn generate_config_context(&self) -> Result<WorkspaceContext, WorkspaceError> {
        let start_time = std::time::Instant::now();
        
        let context_generator = self.context_generator.as_ref()
            .ok_or_else(|| WorkspaceError::Configuration("Context generation not enabled".to_string()))?;
        
        let context = context_generator.generate_config_context().await?;
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Record metrics
        self.record_context_generation(ContextType::Config, duration_ms, context.files.len()).await;
        
        // Emit event
        let _ = self.event_sender.send(WorkspaceStateEvent::ContextGenerated {
            context_type: ContextType::Config,
            files_selected: context.files.len(),
            duration_ms,
        });
        
        Ok(context)
    }
    
    /// Generate general context with criteria
    pub async fn generate_context(
        &self,
        criteria: ContextCriteria,
    ) -> Result<WorkspaceContext, WorkspaceError> {
        let start_time = std::time::Instant::now();
        
        let context_generator = self.context_generator.as_ref()
            .ok_or_else(|| WorkspaceError::Configuration("Context generation not enabled".to_string()))?;
        
        let context = context_generator.generate_context(criteria).await?;
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Record metrics
        self.record_context_generation(ContextType::General, duration_ms, context.files.len()).await;
        
        // Emit event
        let _ = self.event_sender.send(WorkspaceStateEvent::ContextGenerated {
            context_type: ContextType::General,
            files_selected: context.files.len(),
            duration_ms,
        });
        
        Ok(context)
    }
    
    /// Subscribe to workspace state events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<WorkspaceStateEvent> {
        self.event_sender.subscribe()
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> WorkspaceMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Reset metrics
    pub async fn reset_metrics(&self) {
        *self.metrics.write().await = WorkspaceMetrics::default();
    }
    
    /// Record file event
    async fn record_file_event(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.watcher.events_processed += 1;
    }
    
    /// Record snapshot
    async fn record_snapshot(&self, duration_ms: u64, size_bytes: u64) {
        let mut metrics = self.metrics.write().await;
        let total = metrics.snapshots.total_snapshots;
        metrics.snapshots.total_snapshots += 1;
        
        // Update average snapshot time
        metrics.snapshots.average_snapshot_time_ms = 
            (metrics.snapshots.average_snapshot_time_ms * total as f64 + duration_ms as f64) / 
            (total + 1) as f64;
        
        // Update largest snapshot size
        if size_bytes > metrics.snapshots.largest_snapshot_size_bytes {
            metrics.snapshots.largest_snapshot_size_bytes = size_bytes;
        }
        
        metrics.snapshots.last_snapshot_time = Some(Utc::now());
    }
    
    /// Record context generation
    async fn record_context_generation(&self, context_type: ContextType, duration_ms: u64, files_selected: usize) {
        let mut metrics = self.metrics.write().await;
        let total = metrics.context.requests_processed;
        metrics.context.requests_processed += 1;
        
        match context_type {
            ContextType::Code => metrics.context.code_context_requests += 1,
            ContextType::Documentation => metrics.context.docs_context_requests += 1,
            ContextType::Config => metrics.context.config_context_requests += 1,
            ContextType::General => {}
        }
        
        // Update averages
        metrics.context.average_context_time_ms = 
            (metrics.context.average_context_time_ms * total as f64 + duration_ms as f64) / 
            (total + 1) as f64;
        
        metrics.context.average_files_selected = 
            (metrics.context.average_files_selected * total as f64 + files_selected as f64) / 
            (total + 1) as f64;
    }
    
    /// Record embedding generation
    async fn record_embedding_generation(&self, success: bool, duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        let total = if success {
            metrics.embeddings.embeddings_generated
        } else {
            metrics.embeddings.embeddings_failed
        };
        
        if success {
            metrics.embeddings.embeddings_generated += 1;
            metrics.embeddings.files_embedded += 1;
            metrics.embeddings.last_embedding_time = Some(Utc::now());
            
            // Update average generation time
            metrics.embeddings.average_generation_time_ms = 
                (metrics.embeddings.average_generation_time_ms * total as f64 + duration_ms as f64) / 
                (total + 1) as f64;
        } else {
            metrics.embeddings.embeddings_failed += 1;
        }
    }
    
    /// Start metrics collection background task
    fn start_metrics_collection(&mut self) {
        let metrics = Arc::clone(&self.metrics);
        let interval_secs = self.config.metrics_config.update_interval_secs;
        
        self.metrics_handle = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                
                // Update memory metrics
                // Note: This is a placeholder - actual memory collection would use system APIs
                let mut m = metrics.write().await;
                // TODO: Get actual memory usage from system
                m.memory.heap_used_bytes = 0;
                m.memory.heap_total_bytes = 0;
                m.memory.external_bytes = 0;
            }
        }));
    }
}

impl std::fmt::Debug for UnifiedWorkspaceStateManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnifiedWorkspaceStateManager")
            .field("workspace_root", &self.workspace_root)
            .field("initialized", &self.initialized)
            .field("config", &self.config)
            .finish()
    }
}

