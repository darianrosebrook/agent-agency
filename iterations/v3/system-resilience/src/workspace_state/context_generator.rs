//! Context Generator for workspace-aware context generation
//!
//! Provides code-specific, documentation-specific, and configuration-specific
//! context generation from workspace files.

use super::state_manager::WorkspaceStateManager;
use super::state_types::WorkspaceError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::Utc;

/// Workspace context with file information
#[derive(Debug, Clone)]
pub struct WorkspaceContext {
    /// Context type
    pub context_type: super::events::ContextType,
    
    /// Selected files with their content
    pub files: Vec<ContextFile>,
    
    /// Generated timestamp
    pub generated_at: chrono::DateTime<Utc>,
    
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
    pub modified_at: chrono::DateTime<Utc>,
    
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

impl Default for ContextCriteria {
    fn default() -> Self {
        Self {
            include_code: true,
            include_docs: true,
            include_config: true,
            languages: vec![],
            frameworks: vec![],
            max_files: 50,
            similarity_threshold: 0.7,
        }
    }
}

/// Context generator for workspace-aware context generation
pub struct ContextGenerator {
    state_manager: Arc<WorkspaceStateManager>,
    config: super::unified::ContextGenerationConfig,
}

impl ContextGenerator {
    /// Create new context generator
    pub fn new(
        state_manager: Arc<WorkspaceStateManager>,
        config: super::unified::ContextGenerationConfig,
    ) -> Self {
        Self {
            state_manager,
            config,
        }
    }
    
    /// Generate code-specific context
    pub async fn generate_code_context(
        &self,
        language: Option<&str>,
        framework: Option<&str>,
    ) -> Result<WorkspaceContext, WorkspaceError> {
        let start_time = std::time::Instant::now();
        
        // Get current workspace state
        let state_result = self.state_manager.capture_state().await?;
        let state = self.state_manager.get_state(state_result.data).await?;
        
        // Filter files by language and framework
        let mut files = Vec::new();
        let mut files_considered = 0;
        
        for (path, file_state) in &state.files {
            files_considered += 1;
            
            // Check if file matches language filter
            if let Some(lang) = language {
                if !self.matches_language(path, lang) {
                    continue;
                }
            }
            
            // Check if file matches framework filter
            if let Some(fw) = framework {
                if !self.matches_framework(path, fw) {
                    continue;
                }
            }
            
            // Check if file is a code file
            if !self.is_code_file(path) {
                continue;
            }
            
            // Read file content
            let full_path = state.workspace_root.join(path);
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue, // Skip files we can't read
            };
            
            // Truncate content if needed
            let max_content_length = 8000;
            let truncated_content = if content.len() > max_content_length {
                format!("{}...", &content[..max_content_length])
            } else {
                content
            };
            
            // Determine language and framework
            let detected_language = self.detect_language(path);
            let detected_framework = self.detect_framework(path, &truncated_content);
            
            files.push(ContextFile {
                path: path.clone(),
                content: truncated_content,
                metadata: FileMetadata {
                    size: file_state.size,
                    modified_at: file_state.modified_at,
                    extension: path.extension().and_then(|e| e.to_str().map(|s| s.to_string())),
                    language: detected_language,
                    framework: detected_framework,
                },
                relevance_score: 1.0, // TODO: Calculate relevance score
            });
            
            // Limit to max files
            let files_count = files.len();
            if files_count >= self.config.max_files_per_context {
                break;
            }
        }
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let files_selected = files.len();
        
        Ok(WorkspaceContext {
            context_type: super::events::ContextType::Code,
            files,
            generated_at: Utc::now(),
            metadata: ContextMetadata {
                files_considered,
                files_selected,
                generation_duration_ms: duration_ms,
                criteria: ContextCriteria {
                    include_code: true,
                    include_docs: false,
                    include_config: false,
                    languages: language.map(|l| vec![l.to_string()]).unwrap_or_default(),
                    frameworks: framework.map(|f| vec![f.to_string()]).unwrap_or_default(),
                    max_files: self.config.max_files_per_context,
                    similarity_threshold: self.config.similarity_threshold,
                },
            },
        })
    }
    
    /// Generate documentation context
    pub async fn generate_documentation_context(&self) -> Result<WorkspaceContext, WorkspaceError> {
        let start_time = std::time::Instant::now();
        
        // Get current workspace state
        let state_result = Arc::clone(&self.state_manager).capture_state().await?;
        let state = Arc::clone(&self.state_manager).get_state(state_result.data).await?;
        
        // Filter files matching documentation patterns
        let mut files = Vec::new();
        let mut files_considered = 0;
        
        let doc_patterns = vec!["*.md", "*.rst", "*.txt", "docs/**", "*.mdx"];
        
        for (path, file_state) in &state.files {
            files_considered += 1;
            
            // Check if file matches documentation patterns
            if !self.matches_patterns(path, &doc_patterns) {
                continue;
            }
            
            // Read file content
            let full_path = state.workspace_root.join(path);
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            
            // Prioritize README, CHANGELOG, API docs
            let priority = if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_uppercase().starts_with("README") || 
                          n.to_uppercase().starts_with("CHANGELOG") ||
                          n.to_uppercase().contains("API"))
                .unwrap_or(false) {
                2.0
            } else {
                1.0
            };
            
            files.push(ContextFile {
                path: path.clone(),
                content,
                metadata: FileMetadata {
                    size: file_state.size,
                    modified_at: file_state.modified_at,
                    extension: path.extension().and_then(|e| e.to_str().map(|s| s.to_string())),
                    language: None,
                    framework: None,
                },
                relevance_score: priority,
            });
            
            let files_count = files.len();
            if files_count >= self.config.max_files_per_context {
                break;
            }
        }
        
        // Sort by priority
        files.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let files_selected = files.len();
        
        Ok(WorkspaceContext {
            context_type: super::events::ContextType::Documentation,
            files,
            generated_at: Utc::now(),
            metadata: ContextMetadata {
                files_considered,
                files_selected,
                generation_duration_ms: duration_ms,
                criteria: ContextCriteria {
                    include_code: false,
                    include_docs: true,
                    include_config: false,
                    languages: vec![],
                    frameworks: vec![],
                    max_files: self.config.max_files_per_context,
                    similarity_threshold: self.config.similarity_threshold,
                },
            },
        })
    }
    
    /// Generate configuration context
    pub async fn generate_config_context(&self) -> Result<WorkspaceContext, WorkspaceError> {
        let start_time = std::time::Instant::now();
        
        // Get current workspace state
        let state_result = Arc::clone(&self.state_manager).capture_state().await?;
        let state = Arc::clone(&self.state_manager).get_state(state_result.data).await?;
        
        // Filter files matching configuration patterns
        let mut files = Vec::new();
        let mut files_considered = 0;
        
        let config_patterns = vec!["*.toml", "*.yaml", "*.yml", "*.json", "package.json", "Cargo.toml", "*.config.*"];
        
        for (path, file_state) in &state.files {
            files_considered += 1;
            
            // Check if file matches configuration patterns
            if !self.matches_patterns(path, &config_patterns) {
                continue;
            }
            
            // Read file content
            let full_path = state.workspace_root.join(path);
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            
            files.push(ContextFile {
                path: path.clone(),
                content,
                metadata: FileMetadata {
                    size: file_state.size,
                    modified_at: file_state.modified_at,
                    extension: path.extension().and_then(|e| e.to_str().map(|s| s.to_string())),
                    language: None,
                    framework: None,
                },
                relevance_score: 1.0,
            });
            
            let files_count = files.len();
            if files_count >= self.config.max_files_per_context {
                break;
            }
        }
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let files_selected = files.len();
        
        Ok(WorkspaceContext {
            context_type: super::events::ContextType::Config,
            files,
            generated_at: Utc::now(),
            metadata: ContextMetadata {
                files_considered,
                files_selected,
                generation_duration_ms: duration_ms,
                criteria: ContextCriteria {
                    include_code: false,
                    include_docs: false,
                    include_config: true,
                    languages: vec![],
                    frameworks: vec![],
                    max_files: self.config.max_files_per_context,
                    similarity_threshold: self.config.similarity_threshold,
                },
            },
        })
    }
    
    /// Generate general context with criteria
    pub async fn generate_context(
        &self,
        criteria: ContextCriteria,
    ) -> Result<WorkspaceContext, WorkspaceError> {
        let start_time = std::time::Instant::now();
        
        // Get current workspace state
        let state_result = Arc::clone(&self.state_manager).capture_state().await?;
        let state = Arc::clone(&self.state_manager).get_state(state_result.data).await?;
        
        let mut files = Vec::new();
        let mut files_considered = 0;
        
        for (path, file_state) in &state.files {
            files_considered += 1;
            
            // Check criteria
            let is_code = self.is_code_file(path);
            let is_doc = self.matches_patterns(path, &vec!["*.md", "*.rst", "*.txt"]);
            let is_config = self.matches_patterns(path, &vec!["*.toml", "*.yaml", "*.yml", "*.json"]);
            
            if !((criteria.include_code && is_code) ||
                 (criteria.include_docs && is_doc) ||
                 (criteria.include_config && is_config)) {
                continue;
            }
            
            // Language filter
            if !criteria.languages.is_empty() {
                let matches = criteria.languages.iter().any(|lang| self.matches_language(path, lang));
                if !matches {
                    continue;
                }
            }
            
            // Read file content
            let full_path = state.workspace_root.join(path);
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            
            let truncated_content = if content.len() > 8000 {
                format!("{}...", &content[..8000])
            } else {
                content
            };
            
            files.push(ContextFile {
                path: path.clone(),
                content: truncated_content,
                metadata: FileMetadata {
                    size: file_state.size,
                    modified_at: file_state.modified_at,
                    extension: path.extension().and_then(|e| e.to_str().map(|s| s.to_string())),
                    language: self.detect_language(path),
                    framework: None,
                },
                relevance_score: 1.0,
            });
            
            let files_count = files.len();
            if files_count >= criteria.max_files {
                break;
            }
        }
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let files_selected = files.len();
        
        Ok(WorkspaceContext {
            context_type: super::events::ContextType::General,
            files,
            generated_at: Utc::now(),
            metadata: ContextMetadata {
                files_considered,
                files_selected,
                generation_duration_ms: duration_ms,
                criteria,
            },
        })
    }
    
    // Helper methods
    
    fn is_code_file(&self, path: &Path) -> bool {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        matches!(extension, "ts" | "js" | "tsx" | "jsx" | "rs" | "py" | "java" | "cpp" | "c" | "h" | "hpp" | "go" | "rb" | "php")
    }
    
    fn matches_language(&self, path: &Path, language: &str) -> bool {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        match language.to_lowercase().as_str() {
            "rust" => extension == "rs",
            "typescript" => extension == "ts" || extension == "tsx",
            "javascript" => extension == "js" || extension == "jsx",
            "python" => extension == "py",
            "java" => extension == "java",
            "cpp" | "c++" => extension == "cpp" || extension == "hpp",
            "c" => extension == "c" || extension == "h",
            "go" => extension == "go",
            "ruby" => extension == "rb",
            "php" => extension == "php",
            _ => false,
        }
    }
    
    fn matches_framework(&self, path: &Path, framework: &str) -> bool {
        // Simple framework detection based on file patterns
        // TODO: Improve with content analysis
        match framework.to_lowercase().as_str() {
            "react" => path.to_string_lossy().contains("react") || 
                       path.extension().and_then(|e| e.to_str()) == Some("tsx") ||
                       path.extension().and_then(|e| e.to_str()) == Some("jsx"),
            "actix" => path.to_string_lossy().contains("actix"),
            "fastapi" => path.to_string_lossy().contains("fastapi"),
            _ => false,
        }
    }
    
    fn matches_patterns(&self, path: &Path, patterns: &[&str]) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        patterns.iter().any(|pattern| {
            if pattern.starts_with("*.") {
                let ext = &pattern[2..];
                path.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase() == ext)
                    .unwrap_or(false)
            } else if pattern.ends_with("/**") {
                let prefix = &pattern[..pattern.len() - 3];
                path_str.starts_with(&prefix.to_lowercase())
            } else {
                path_str.contains(pattern) || file_name.contains(pattern)
            }
        })
    }
    
    fn detect_language(&self, path: &Path) -> Option<String> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        match extension {
            "rs" => Some("rust".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "js" | "jsx" => Some("javascript".to_string()),
            "py" => Some("python".to_string()),
            "java" => Some("java".to_string()),
            "cpp" | "hpp" => Some("cpp".to_string()),
            "c" | "h" => Some("c".to_string()),
            "go" => Some("go".to_string()),
            "rb" => Some("ruby".to_string()),
            "php" => Some("php".to_string()),
            _ => None,
        }
    }
    
    fn detect_framework(&self, path: &Path, _content: &str) -> Option<String> {
        // Simple framework detection based on file patterns
        // TODO: Improve with content analysis (imports, dependencies)
        let path_str = path.to_string_lossy().to_lowercase();
        
        if path_str.contains("react") || path.extension().and_then(|e| e.to_str()) == Some("tsx") {
            Some("react".to_string())
        } else if path_str.contains("actix") {
            Some("actix".to_string())
        } else if path_str.contains("fastapi") {
            Some("fastapi".to_string())
        } else {
            None
        }
    }
}

