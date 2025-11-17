//! Unified Workspace Setup Helper
//!
//! Helper functions for setting up unified workspace state manager with
//! file watcher and embedding service connections
//! @author @darianrosebrook

use std::path::PathBuf;
use system_resilience::workspace_state::{
    ContextGenerationConfig, FileWatchConfig, MetricsConfig, UnifiedWorkspaceConfig,
    UnifiedWorkspaceStateManagerBuilder, WorkspaceConfig,
};

#[cfg(feature = "memory")]
use crate::workspace_integration::EmbeddingServiceAdapter;
#[cfg(feature = "data-processing")]
use crate::workspace_integration::FileWatcherBridge;

#[cfg(all(feature = "data-processing", feature = "memory"))]
use agent_data_processing::ingestion::FileWatcher as DataProcessingFileWatcher;
#[cfg(feature = "memory")]
use agent_memory::embedding_integration::EmbeddingIntegration;
#[cfg(feature = "memory")]
use std::sync::Arc;

/// Configuration for unified workspace setup
pub struct UnifiedWorkspaceSetupConfig {
    pub workspace_root: PathBuf,
    pub watch_paths: Vec<PathBuf>,
    pub file_patterns: Vec<String>,
    pub embedding_extensions: Vec<String>,
    pub debounce_ms: u64,
    pub auto_capture_state: bool,
    pub generate_embeddings: bool,
}

impl Default for UnifiedWorkspaceSetupConfig {
    fn default() -> Self {
        Self {
            workspace_root: PathBuf::from("."),
            watch_paths: vec![PathBuf::from(".")],
            file_patterns: vec!["**/*".to_string()],
            embedding_extensions: vec![
                "rs".to_string(),
                "ts".to_string(),
                "js".to_string(),
                "py".to_string(),
                "md".to_string(),
                "txt".to_string(),
            ],
            debounce_ms: 500,
            auto_capture_state: true,
            generate_embeddings: true,
        }
    }
}

/// Setup unified workspace state manager with file watcher and embedding service
#[cfg(all(feature = "data-processing", feature = "memory"))]
pub async fn setup_unified_workspace(
    config: UnifiedWorkspaceSetupConfig,
    embedding_integration: Arc<EmbeddingIntegration>,
) -> Result<
    (
        system_resilience::workspace_state::UnifiedWorkspaceStateManager,
        FileWatcherBridge,
    ),
    String,
> {
    use system_resilience::workspace_state::WorkspaceError;

    // Create file watcher
    let file_watcher =
        DataProcessingFileWatcher::new(config.watch_paths.clone(), config.file_patterns.clone())
            .map_err(|e| format!("Failed to create file watcher: {}", e))?;

    // Build unified workspace state manager
    let watch_config = system_resilience::workspace_state::FileWatchConfig {
        enabled: true,
        watch_paths: config.watch_paths.clone(),
        file_patterns: config.file_patterns.clone(),
        debounce_ms: config.debounce_ms,
        auto_capture_state: config.auto_capture_state,
        generate_embeddings: config.generate_embeddings,
        embedding_extensions: config.embedding_extensions.clone(),
    };

    let context_config = ContextGenerationConfig {
        enabled: true,
        code_context_enabled: true,
        docs_context_enabled: true,
        config_context_enabled: true,
        max_files_per_context: 50,
        similarity_threshold: 0.7,
        language_filters: Vec::new(),
        framework_filters: Vec::new(),
    };

    let metrics_config = MetricsConfig {
        enabled: true,
        update_interval_secs: 60,
        detailed_metrics: false,
    };

    let unified_config = UnifiedWorkspaceConfig {
        state_config: WorkspaceConfig::default(),
        watch_config: Some(watch_config),
        context_config: Some(context_config),
        metrics_config,
    };

    let mut manager = UnifiedWorkspaceStateManagerBuilder::new(&config.workspace_root)
        .with_file_watching(unified_config.watch_config.clone().unwrap())
        .with_context_generation(unified_config.context_config.clone().unwrap())
        .with_metrics_config(unified_config.metrics_config.clone())
        .build()
        .map_err(|e| format!("Failed to build unified workspace manager: {}", e))?;

    // Set embedding service
    let embedding_adapter = EmbeddingServiceAdapter::new(embedding_integration);
    manager = manager.with_embedding_service(Box::new(embedding_adapter));

    // Initialize manager
    manager
        .initialize()
        .await
        .map_err(|e| format!("Failed to initialize unified workspace manager: {}", e))?;

    // Get file watcher handler and create bridge
    let event_handler = manager
        .file_watcher_handler()
        .ok_or_else(|| "File watcher handler not available".to_string())?;

    let mut bridge = FileWatcherBridge::new(file_watcher, event_handler)
        .map_err(|e| format!("Failed to create file watcher bridge: {}", e))?;

    // Start file watcher bridge
    bridge
        .start()
        .await
        .map_err(|e| format!("Failed to start file watcher bridge: {}", e))?;

    Ok((manager, bridge))
}

/// Setup unified workspace state manager without file watcher (embedding only)
#[cfg(feature = "memory")]
#[cfg(not(feature = "data-processing"))]
pub async fn setup_unified_workspace_embedding_only(
    config: UnifiedWorkspaceSetupConfig,
    embedding_integration: Arc<EmbeddingIntegration>,
) -> Result<system_resilience::workspace_state::UnifiedWorkspaceStateManager, String> {
    let context_config = ContextGenerationConfig {
        enabled: true,
        code_context_enabled: true,
        docs_context_enabled: true,
        config_context_enabled: true,
        max_files_per_context: 50,
        similarity_threshold: 0.7,
        language_filters: Vec::new(),
        framework_filters: Vec::new(),
    };

    let metrics_config = MetricsConfig {
        enabled: true,
        update_interval_secs: 60,
        detailed_metrics: false,
    };

    let unified_config = UnifiedWorkspaceConfig {
        state_config: WorkspaceConfig::default(),
        watch_config: None,
        context_config: Some(context_config),
        metrics_config,
    };

    let mut manager = UnifiedWorkspaceStateManagerBuilder::new(&config.workspace_root)
        .with_context_generation(unified_config.context_config.clone().unwrap())
        .with_metrics_config(unified_config.metrics_config.clone())
        .build()
        .map_err(|e| format!("Failed to build unified workspace manager: {}", e))?;

    // Set embedding service
    let embedding_adapter = EmbeddingServiceAdapter::new(embedding_integration);
    manager = manager.with_embedding_service(Box::new(embedding_adapter));

    // Initialize manager
    manager
        .initialize()
        .await
        .map_err(|e| format!("Failed to initialize unified workspace manager: {}", e))?;

    Ok(manager)
}

/// Setup unified workspace state manager without embedding service (file watcher only)
#[cfg(feature = "data-processing")]
#[cfg(not(feature = "memory"))]
pub async fn setup_unified_workspace_watcher_only(
    config: UnifiedWorkspaceSetupConfig,
) -> Result<
    (
        system_resilience::workspace_state::UnifiedWorkspaceStateManager,
        FileWatcherBridge,
    ),
    String,
> {
    use system_resilience::workspace_state::WorkspaceError;

    // Create file watcher
    let file_watcher =
        DataProcessingFileWatcher::new(config.watch_paths.clone(), config.file_patterns.clone())
            .map_err(|e| format!("Failed to create file watcher: {}", e))?;

    // Build unified workspace state manager
    let watch_config = system_resilience::workspace_state::FileWatchConfig {
        enabled: true,
        watch_paths: config.watch_paths.clone(),
        file_patterns: config.file_patterns.clone(),
        debounce_ms: config.debounce_ms,
        auto_capture_state: config.auto_capture_state,
        generate_embeddings: false, // No embedding service available
        embedding_extensions: vec![],
    };

    let context_config = ContextGenerationConfig {
        enabled: true,
        code_context_enabled: true,
        docs_context_enabled: true,
        config_context_enabled: true,
        max_files_per_context: 50,
        similarity_threshold: 0.7,
        language_filters: Vec::new(),
        framework_filters: Vec::new(),
    };

    let metrics_config = MetricsConfig {
        enabled: true,
        update_interval_secs: 60,
        detailed_metrics: false,
    };

    let unified_config = UnifiedWorkspaceConfig {
        state_config: WorkspaceConfig::default(),
        watch_config: Some(watch_config),
        context_config: Some(context_config),
        metrics_config,
    };

    let mut manager = UnifiedWorkspaceStateManagerBuilder::new(&config.workspace_root)
        .with_file_watching(unified_config.watch_config.clone().unwrap())
        .with_context_generation(unified_config.context_config.clone().unwrap())
        .with_metrics_config(unified_config.metrics_config.clone())
        .build()
        .map_err(|e| format!("Failed to build unified workspace manager: {}", e))?;

    // Initialize manager
    manager
        .initialize()
        .await
        .map_err(|e| format!("Failed to initialize unified workspace manager: {}", e))?;

    // Get file watcher handler and create bridge
    let event_handler = manager
        .file_watcher_handler()
        .ok_or_else(|| "File watcher handler not available".to_string())?;

    let mut bridge = FileWatcherBridge::new(file_watcher, event_handler)
        .map_err(|e| format!("Failed to create file watcher bridge: {}", e))?;

    // Start file watcher bridge
    bridge
        .start()
        .await
        .map_err(|e| format!("Failed to start file watcher bridge: {}", e))?;

    Ok((manager, bridge))
}
