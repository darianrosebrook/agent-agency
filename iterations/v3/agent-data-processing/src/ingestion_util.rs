//! Utilities for constructing DataInput from file paths and operation identity

use std::path::{Path, PathBuf};

use crate::data_processing_types::*;
use crate::{DataProcessingResult, DataProcessingError};
use infer;
use tokio::fs;

/// Generate a stable operation identity for deduplication based on path, size, and mtime
pub async fn op_identity(path: &Path) -> DataProcessingResult<String> {
    let meta = fs::metadata(path).await.map_err(DataProcessingError::Io)?;

    let mtime = meta.modified().ok()
        .and_then(|t| t.elapsed().ok())
        .map(|e| format!("{}", e.as_millis()))
        .unwrap_or_default();

    Ok(format!("{}:{}:{}", path.to_string_lossy(), meta.len(), mtime))
}

/// Construct a DataInput from a file path with proper MIME type inference
pub async fn data_input_from_path(path: PathBuf) -> DataProcessingResult<DataInput> {
    let meta = fs::metadata(&path).await.map_err(DataProcessingError::Io)?;
    let bytes = fs::read(&path).await.map_err(DataProcessingError::Io)?;

    // MIME type inference from file content
    let kind = infer::get(&bytes);
    let mime_str = kind.map(|k| k.mime_type()).unwrap_or("application/octet-stream");
    let ct = ContentType::from_mime_type(mime_str);

    // Select canonical DataSource::File
    let src = DataSource::File(FileSource {
        path: path.clone(),
        content_type: ct.clone(),
        size_bytes: meta.len(),
        last_modified: chrono::Utc::now(), // optional: real mtime via filetime
    });

    // Don't parse here; hand raw file to FileIngestor to reuse existing logic
    let content = DataContent::File(path.clone());

    Ok(DataInput {
        id: ProcessingId::new(),
        source: src,
        content,
        metadata: std::collections::HashMap::new(),
        processing_context: ProcessingContext {
            request_id: uuid::Uuid::new_v4().to_string(),
            user_id: None,
            project_scope: None,
            priority: ProcessingPriority::Normal,
            deadline: None,
            tags: vec!["filewatch".into()],
        },
    })
}