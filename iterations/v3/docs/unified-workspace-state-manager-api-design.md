# Unified Workspace State Manager API Design

**Author:** @darianrosebrook  
**Date:** January 2025  
**Status:** API Design Specification

## Overview

This document provides the complete API design for the unified workspace state manager that combines file watching, state capture, context generation, and embedding integration into a single cohesive interface.

## Architecture

```
UnifiedWorkspaceStateManager
├── WorkspaceStateManager (core state capture)
├── FileWatcher (optional, from agent-data-processing)
├── EmbeddingIntegration (optional, from agent-memory)
├── ContextGenerator (new, workspace-aware)
└── MetricsCollector (new, comprehensive metrics)
```

## Core Types

### UnifiedWorkspaceStateManager

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Unified workspace state manager combining all capabilities
pub struct UnifiedWorkspaceStateManager {
    // Core state management
    state_manager: system_resilience::workspace_state::WorkspaceStateManager,
    
    // File watching (optional)
    file_watcher: Option<Arc<agent_data_processing::ingestion::FileWatcher>>,
    file_watcher_handle: Option<tokio::task::JoinHandle<()>>,
    file_event_receiver: Option<broadcast::Receiver<FileEvent>>,
    
    // Embedding integration (optional)
    embedding_integration: Option<Arc<agent_memory::embedding_integration::EmbeddingIntegration>>,
    embedding_debounce_map: Arc<RwLock<std::collections::HashMap<PathBuf, tokio::time::Instant>>>,
    
    // Context generation (optional)
    context_generator: Option<Arc<ContextGenerator>>,
    
    // Metrics collection
    metrics: Arc<RwLock<WorkspaceMetrics>>,
    metrics_handle: Option<tokio::task::JoinHandle<()>>,
    
    // Event channels
    event_sender: broadcast::Sender<WorkspaceStateEvent>,
    
    // Configuration
    config: UnifiedWorkspaceConfig,
    
    // Workspace root
    workspace_root: PathBuf,
}
```

### Configuration Types

```rust
/// Unified configuration for workspace state manager
#[derive(Debug, Clone)]
pub struct UnifiedWorkspaceConfig {
    /// Core state management configuration
    pub state_config: system_resilience::workspace_state::WorkspaceConfig,
    
    /// File watching configuration (optional)
    pub watch_config: Option<FileWatchConfig>,
    
    /// Embedding configuration (optional)
    pub embedding_config: Option<agent_memory::memory_types::EmbeddingConfig>,
    
    /// Context generation configuration (optional)
    pub context_config: Option<ContextGenerationConfig>,
    
    /// Metrics configuration
    pub metrics_config: MetricsConfig,
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
```

### Event Types

```rust
/// Workspace state events
#[derive(Debug, Clone)]
pub enum WorkspaceStateEvent {
    /// File was created
    FileCreated {
        path: PathBuf,
        state_id: Option<system_resilience::workspace_state::StateId>,
    },
    
    /// File was modified
    FileModified {
        path: PathBuf,
        state_id: Option<system_resilience::workspace_state::StateId>,
    },
    
    /// File was deleted
    FileDeleted {
        path: PathBuf,
        state_id: Option<system_resilience::workspace_state::StateId>,
    },
    
    /// State was captured
    StateCaptured {
        state_id: system_resilience::workspace_state::StateId,
        duration_ms: u64,
    },
    
    /// Embedding was generated
    EmbeddingGenerated {
        path: PathBuf,
        success: bool,
        duration_ms: u64,
    },
    
    /// Context was generated
    ContextGenerated {
        context_type: ContextType,
        files_selected: usize,
        duration_ms: u64,
    },
}

/// Context type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    Code,
    Documentation,
    Config,
    General,
}
```

### Context Types

```rust
/// Workspace context with file information
#[derive(Debug, Clone)]
pub struct WorkspaceContext {
    /// Context type
    pub context_type: ContextType,
    
    /// Selected files with their content
    pub files: Vec<ContextFile>,
    
    /// Generated timestamp
    pub generated_at: DateTime<Utc>,
    
    /// Metadata about the context
    pub metadata: ContextMetadata,
}

/// File in context
#[derive(Debug, Clone)]
pub struct ContextFile {
    /// File path relative to workspace root
    pub path: PathBuf,
    
    /// File content (may be truncated)
    pub content: String,
    
    /// File metadata
    pub metadata: FileMetadata,
    
    /// Relevance score
    pub relevance_score: f32,
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,
    
    /// Last modified time
    pub modified_at: DateTime<Utc>,
    
    /// File extension
    pub extension: Option<String>,
    
    /// Language (if code file)
    pub language: Option<String>,
    
    /// Framework (if applicable)
    pub framework: Option<String>,
}

/// Context metadata
#[derive(Debug, Clone)]
pub struct ContextMetadata {
    /// Number of files considered
    pub files_considered: usize,
    
    /// Number of files selected
    pub files_selected: usize,
    
    /// Generation duration in milliseconds
    pub generation_duration_ms: u64,
    
    /// Criteria used for generation
    pub criteria: ContextCriteria,
}

/// Context generation criteria
#[derive(Debug, Clone)]
pub struct ContextCriteria {
    /// Include code files
    pub include_code: bool,
    
    /// Include documentation files
    pub include_docs: bool,
    
    /// Include configuration files
    pub include_config: bool,
    
    /// Language filters
    pub languages: Vec<String>,
    
    /// Framework filters
    pub frameworks: Vec<String>,
    
    /// Maximum files to include
    pub max_files: usize,
    
    /// Similarity threshold
    pub similarity_threshold: f32,
}
```

## Builder API

```rust
/// Builder for unified workspace state manager
pub struct UnifiedWorkspaceStateManagerBuilder {
    workspace_root: PathBuf,
    config: UnifiedWorkspaceConfig,
    state_storage: Option<Box<dyn system_resilience::workspace_state::StateStorage>>,
    embedding_integration: Option<Arc<agent_memory::embedding_integration::EmbeddingIntegration>>,
}

impl UnifiedWorkspaceStateManagerBuilder {
    /// Create new builder
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
            config: UnifiedWorkspaceConfig::default(),
            state_storage: None,
            embedding_integration: None,
        }
    }
    
    /// Set state management configuration
    pub fn with_state_config(mut self, config: system_resilience::workspace_state::WorkspaceConfig) -> Self {
        self.config.state_config = config;
        self
    }
    
    /// Enable and configure file watching
    pub fn with_file_watching(mut self, config: FileWatchConfig) -> Self {
        self.config.watch_config = Some(config);
        self
    }
    
    /// Set embedding integration
    pub fn with_embedding_integration(
        mut self,
        integration: Arc<agent_memory::embedding_integration::EmbeddingIntegration>,
    ) -> Self {
        self.embedding_integration = Some(integration);
        self
    }
    
    /// Enable and configure context generation
    pub fn with_context_generation(mut self, config: ContextGenerationConfig) -> Self {
        self.config.context_config = Some(config);
        self
    }
    
    /// Set storage backend
    pub fn with_storage(mut self, storage: Box<dyn system_resilience::workspace_state::StateStorage>) -> Self {
        self.state_storage = Some(storage);
        self
    }
    
    /// Set metrics configuration
    pub fn with_metrics_config(mut self, config: MetricsConfig) -> Self {
        self.config.metrics_config = config;
        self
    }
    
    /// Build the unified workspace state manager
    pub async fn build(self) -> Result<UnifiedWorkspaceStateManager, WorkspaceError> {
        // Implementation details
        // 1. Create core WorkspaceStateManager
        // 2. Initialize file watcher if enabled
        // 3. Initialize context generator if enabled
        // 4. Initialize metrics collection
        // 5. Set up event channels
    }
}
```

## Core API Methods

### Initialization and Lifecycle

```rust
impl UnifiedWorkspaceStateManager {
    /// Initialize all enabled components
    pub async fn initialize(&mut self) -> Result<(), WorkspaceError> {
        // 1. Initialize file watcher if enabled
        // 2. Start file watching if enabled
        // 3. Start metrics collection if enabled
        // 4. Create initial state snapshot
    }
    
    /// Shutdown all components gracefully
    pub async fn shutdown(&mut self) -> Result<(), WorkspaceError> {
        // 1. Stop file watcher
        // 2. Stop metrics collection
        // 3. Flush pending embeddings
        // 4. Save final state
    }
}
```

### State Management (Delegated)

```rust
impl UnifiedWorkspaceStateManager {
    /// Capture current workspace state
    pub async fn capture_state(&self) -> Result<system_resilience::workspace_state::WorkspaceResult<system_resilience::workspace_state::StateId>, WorkspaceError> {
        let start_time = std::time::Instant::now();
        let result = self.state_manager.capture_state().await?;
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Record metrics
        self.record_snapshot(duration_ms, 0).await;
        
        // Emit event
        let _ = self.event_sender.send(WorkspaceStateEvent::StateCaptured {
            state_id: result.data,
            duration_ms,
        });
        
        Ok(result)
    }
    
    /// Get workspace state by ID
    pub async fn get_state(
        &self,
        id: system_resilience::workspace_state::StateId,
    ) -> Result<system_resilience::workspace_state::WorkspaceState, WorkspaceError> {
        self.state_manager.get_state(id).await
    }
    
    /// Compute diff between two states
    pub async fn compute_diff(
        &self,
        from: system_resilience::workspace_state::StateId,
        to: system_resilience::workspace_state::StateId,
    ) -> Result<system_resilience::workspace_state::WorkspaceResult<system_resilience::workspace_state::WorkspaceDiff>, WorkspaceError> {
        self.state_manager.compute_diff(from, to).await
    }
}
```

### File Watching Integration

```rust
impl UnifiedWorkspaceStateManager {
    /// Start file watching
    pub async fn start_file_watching(&mut self) -> Result<(), WorkspaceError> {
        if let Some(ref watcher) = self.file_watcher {
            watcher.start_watching().await
                .map_err(|e| WorkspaceError::Configuration(format!("Failed to start file watcher: {}", e)))?;
            
            // Start event processing task
            self.start_file_event_processor().await?;
        }
        Ok(())
    }
    
    /// Stop file watching
    pub async fn stop_file_watching(&mut self) -> Result<(), WorkspaceError> {
        if let Some(handle) = self.file_watcher_handle.take() {
            handle.abort();
        }
        Ok(())
    }
    
    /// Get list of watched files
    pub fn get_watched_files(&self) -> Vec<PathBuf> {
        // Return list of files being watched
        // This would require tracking in FileWatcher
        vec![]
    }
    
    /// Subscribe to file events
    pub fn subscribe_to_file_events(&self) -> broadcast::Receiver<WorkspaceStateEvent> {
        self.event_sender.subscribe()
    }
    
    /// Process file events from watcher
    async fn start_file_event_processor(&mut self) -> Result<(), WorkspaceError> {
        let event_sender = self.event_sender.clone();
        let state_manager = Arc::new(self.state_manager.clone());
        let embedding_integration = self.embedding_integration.clone();
        let watch_config = self.config.watch_config.clone();
        let debounce_map = Arc::clone(&self.embedding_debounce_map);
        
        self.file_watcher_handle = Some(tokio::spawn(async move {
            // Process file events
            // Handle debouncing
            // Trigger state capture if enabled
            // Trigger embedding generation if enabled
        }));
        
        Ok(())
    }
}
```

### Context Generation

```rust
impl UnifiedWorkspaceStateManager {
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
        // Similar implementation
    }
    
    /// Generate configuration context
    pub async fn generate_config_context(&self) -> Result<WorkspaceContext, WorkspaceError> {
        // Similar implementation
    }
    
    /// Generate general context with criteria
    pub async fn generate_context(
        &self,
        criteria: ContextCriteria,
    ) -> Result<WorkspaceContext, WorkspaceError> {
        // Similar implementation
    }
}
```

### Embedding Integration

```rust
impl UnifiedWorkspaceStateManager {
    /// Generate embedding for a file
    pub async fn generate_file_embedding(&self, file_path: &Path) -> Result<Vec<f32>, WorkspaceError> {
        let embedding_integration = self.embedding_integration.as_ref()
            .ok_or_else(|| WorkspaceError::Configuration("Embedding integration not enabled".to_string()))?;
        
        // Read file content
        let content = tokio::fs::read_to_string(file_path).await
            .map_err(|e| WorkspaceError::Io(e))?;
        
        // Generate embedding
        let start_time = std::time::Instant::now();
        let embedding = embedding_integration.generate_file_embedding(file_path, &content).await
            .map_err(|e| WorkspaceError::Embedding(e.to_string()))?;
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Record metrics
        self.record_embedding_generation(true, duration_ms).await;
        
        // Emit event
        let _ = self.event_sender.send(WorkspaceStateEvent::EmbeddingGenerated {
            path: file_path.to_path_buf(),
            success: true,
            duration_ms,
        });
        
        Ok(embedding)
    }
    
    /// Update embedding for a file (called on file changes)
    pub async fn update_file_embedding(&self, file_path: &Path) -> Result<(), WorkspaceError> {
        // Check debounce
        // Generate embedding
        // Store embedding
    }
    
    /// Search files by semantic similarity
    pub async fn search_files_by_similarity(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(PathBuf, f32)>, WorkspaceError> {
        // Use embedding service to search
    }
}
```

### Metrics

```rust
impl UnifiedWorkspaceStateManager {
    /// Get current metrics
    pub fn get_metrics(&self) -> WorkspaceMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Reset metrics
    pub fn reset_metrics(&self) {
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
        metrics.snapshots.total_snapshots += 1;
        // Update averages
    }
    
    /// Record context generation
    async fn record_context_generation(&self, context_type: ContextType, duration_ms: u64, files_selected: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.context.requests_processed += 1;
        // Update averages
    }
    
    /// Record embedding generation
    async fn record_embedding_generation(&self, success: bool, duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        if success {
            metrics.embeddings.embeddings_generated += 1;
        } else {
            metrics.embeddings.embeddings_failed += 1;
        }
        // Update averages
    }
}
```

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Embedding error: {0}")]
    Embedding(String),
    
    #[error("State management error: {0}")]
    StateManagement(#[from] system_resilience::workspace_state::WorkspaceError),
    
    #[error("File watcher error: {0}")]
    FileWatcher(String),
    
    #[error("Context generation error: {0}")]
    ContextGeneration(String),
}
```

## Usage Examples

### Basic Usage

```rust
use unified_workspace_state_manager::UnifiedWorkspaceStateManagerBuilder;

// Create unified manager
let mut manager = UnifiedWorkspaceStateManagerBuilder::new("/path/to/workspace")
    .with_file_watching(FileWatchConfig {
        enabled: true,
        watch_paths: vec!["/path/to/workspace".into()],
        file_patterns: vec!["**/*.rs".to_string(), "**/*.ts".to_string()],
        debounce_ms: 1000,
        auto_capture_state: true,
        generate_embeddings: true,
        embedding_extensions: vec!["rs".to_string(), "ts".to_string()],
    })
    .with_context_generation(ContextGenerationConfig {
        enabled: true,
        code_context_enabled: true,
        docs_context_enabled: true,
        config_context_enabled: true,
        max_files_per_context: 50,
        similarity_threshold: 0.7,
        language_filters: vec!["rust".to_string(), "typescript".to_string()],
        framework_filters: vec![],
    })
    .build()
    .await?;

// Initialize
manager.initialize().await?;

// Generate code context
let context = manager.generate_code_context(Some("rust"), None).await?;
println!("Generated context with {} files", context.files.len());

// Capture state
let state_id = manager.capture_state().await?;
println!("Captured state: {:?}", state_id);

// Get metrics
let metrics = manager.get_metrics();
println!("Events processed: {}", metrics.watcher.events_processed);

// Shutdown
manager.shutdown().await?;
```

### With Embedding Integration

```rust
// Create embedding integration
let embedding_integration = Arc::new(
    agent_memory::embedding_integration::EmbeddingIntegration::new(&embedding_config).await?
);

// Create unified manager with embedding
let mut manager = UnifiedWorkspaceStateManagerBuilder::new("/path/to/workspace")
    .with_embedding_integration(embedding_integration)
    .with_file_watching(FileWatchConfig {
        enabled: true,
        generate_embeddings: true,
        // ... other config
    })
    .build()
    .await?;

// Generate file embedding
let embedding = manager.generate_file_embedding(Path::new("src/main.rs")).await?;
println!("Generated embedding with {} dimensions", embedding.len());

// Search files by similarity
let results = manager.search_files_by_similarity("authentication", 10).await?;
for (path, similarity) in results {
    println!("{}: {:.2}", path.display(), similarity);
}
```

## Implementation Notes

1. **Thread Safety**: All shared state uses `Arc<RwLock<T>>` for thread-safe access
2. **Async**: All I/O operations are async using `tokio`
3. **Error Handling**: Uses `thiserror` for error types
4. **Events**: Uses `tokio::sync::broadcast` for event distribution
5. **Debouncing**: Uses `tokio::time` for debouncing file events
6. **Metrics**: Metrics collection runs in background task

## Next Steps

1. Implement `UnifiedWorkspaceStateManager` struct
2. Implement builder pattern
3. Integrate file watcher
4. Integrate embedding service
5. Implement context generators
6. Implement metrics collection
7. Add comprehensive tests
8. Add documentation and examples

