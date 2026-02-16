//! Built-in Tools
//!
//! Standard tools for common operations.

use std::path::Path;

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use tokio::fs;

use v4_types::operators::{ControlOp, MemorizeOp, OperatorResult, SeekOp};

// ─── Unified Diff Parsing ──────────────────────────────────────────

/// A single hunk from a unified diff.
#[derive(Debug)]
struct DiffHunk {
    /// 1-based start line in the original file.
    old_start: usize,
    /// Number of lines from the original.
    old_count: usize,
    /// Lines in the hunk: ' ' = context, '-' = remove, '+' = add.
    lines: Vec<DiffLine>,
}

#[derive(Debug)]
enum DiffLine {
    Context(String),
    Remove(String),
    Add(String),
}

/// Parse a unified diff string into hunks.
fn parse_unified_diff(diff: &str) -> Result<Vec<DiffHunk>, String> {
    let mut hunks = Vec::new();
    let mut lines = diff.lines().peekable();

    // Skip header lines (--- and +++ lines)
    while let Some(line) = lines.peek() {
        if line.starts_with("---") || line.starts_with("+++") {
            lines.next();
        } else {
            break;
        }
    }

    while let Some(line) = lines.next() {
        if line.starts_with("@@") {
            // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
            let header = line
                .trim_start_matches("@@")
                .split("@@")
                .next()
                .ok_or_else(|| "Invalid hunk header".to_string())?
                .trim();

            let parts: Vec<&str> = header.split_whitespace().collect();
            if parts.is_empty() {
                return Err("Empty hunk header".to_string());
            }

            let old_part = parts[0].trim_start_matches('-');
            let (old_start, old_count) = if let Some(comma) = old_part.find(',') {
                let start: usize = old_part[..comma]
                    .parse()
                    .map_err(|_| "Invalid old_start in hunk header")?;
                let count: usize = old_part[comma + 1..]
                    .parse()
                    .map_err(|_| "Invalid old_count in hunk header")?;
                (start, count)
            } else {
                let start: usize = old_part
                    .parse()
                    .map_err(|_| "Invalid old_start in hunk header")?;
                (start, 1)
            };

            let mut hunk_lines = Vec::new();

            while let Some(hline) = lines.peek() {
                if hline.starts_with("@@") || hline.starts_with("---") || hline.starts_with("+++")
                {
                    break;
                }
                let hline = *hline;
                lines.next();

                if let Some(rest) = hline.strip_prefix('-') {
                    hunk_lines.push(DiffLine::Remove(rest.to_string()));
                } else if let Some(rest) = hline.strip_prefix('+') {
                    hunk_lines.push(DiffLine::Add(rest.to_string()));
                } else if let Some(rest) = hline.strip_prefix(' ') {
                    hunk_lines.push(DiffLine::Context(rest.to_string()));
                } else if hline.is_empty() {
                    // Empty line treated as context with empty content
                    hunk_lines.push(DiffLine::Context(String::new()));
                } else {
                    // Line without prefix — treat as context
                    hunk_lines.push(DiffLine::Context(hline.to_string()));
                }
            }

            hunks.push(DiffHunk {
                old_start,
                old_count,
                lines: hunk_lines,
            });
        }
    }

    if hunks.is_empty() {
        return Err("No hunks found in diff".to_string());
    }

    Ok(hunks)
}

/// Apply parsed hunks to file content. Returns the patched content or an error.
fn apply_hunks(original: &str, hunks: &[DiffHunk]) -> Result<String, String> {
    let orig_lines: Vec<&str> = original.lines().collect();
    let mut result = Vec::new();
    let mut current_line = 0usize; // 0-based index into orig_lines

    for hunk in hunks {
        let hunk_start = hunk.old_start.saturating_sub(1); // Convert 1-based to 0-based

        // Copy lines before this hunk
        if hunk_start > current_line {
            result.extend(orig_lines[current_line..hunk_start].iter().map(|s| s.to_string()));
        } else if hunk_start < current_line {
            return Err(format!(
                "Overlapping hunks at line {} (already processed to line {})",
                hunk.old_start,
                current_line + 1
            ));
        }

        current_line = hunk_start;

        // Verify context lines match and apply changes
        for diff_line in &hunk.lines {
            match diff_line {
                DiffLine::Context(expected) => {
                    if current_line >= orig_lines.len() {
                        return Err(format!(
                            "Context mismatch: expected line {} but file has only {} lines",
                            current_line + 1,
                            orig_lines.len()
                        ));
                    }
                    if orig_lines[current_line] != expected.as_str() {
                        return Err(format!(
                            "Context mismatch at line {}: expected {:?}, found {:?}",
                            current_line + 1,
                            expected,
                            orig_lines[current_line]
                        ));
                    }
                    result.push(orig_lines[current_line].to_string());
                    current_line += 1;
                }
                DiffLine::Remove(expected) => {
                    if current_line >= orig_lines.len() {
                        return Err(format!(
                            "Remove mismatch: expected line {} but file has only {} lines",
                            current_line + 1,
                            orig_lines.len()
                        ));
                    }
                    if orig_lines[current_line] != expected.as_str() {
                        return Err(format!(
                            "Remove mismatch at line {}: expected {:?}, found {:?}",
                            current_line + 1,
                            expected,
                            orig_lines[current_line]
                        ));
                    }
                    // Skip this line (remove it)
                    current_line += 1;
                }
                DiffLine::Add(new_line) => {
                    result.push(new_line.clone());
                    // Don't advance current_line — adds don't consume original lines
                }
            }
        }
    }

    // Copy remaining lines
    if current_line < orig_lines.len() {
        result.extend(orig_lines[current_line..].iter().map(|s| s.to_string()));
    }

    // Preserve trailing newline if original had one
    let mut output = result.join("\n");
    if original.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

// ─── Test Output Parsing ───────────────────────────────────────────

/// Parsed cargo test results.
#[derive(Debug, Clone, serde::Serialize)]
struct TestResults {
    total: usize,
    passed: usize,
    failed: usize,
    ignored: usize,
    failures: Vec<String>,
}

/// Parse cargo test output into structured results.
fn parse_cargo_test_output(output: &str) -> TestResults {
    let mut total = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut ignored = 0usize;
    let mut failures = Vec::new();

    // Collect failure names from individual test lines: "test name ... FAILED"
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("test ") && trimmed.ends_with("... FAILED") {
            let name = trimmed
                .trim_start_matches("test ")
                .trim_end_matches("... FAILED")
                .trim();
            if !name.is_empty() {
                failures.push(name.to_string());
            }
        }
    }

    // Parse summary lines: "test result: ok. X passed; Y failed; Z ignored; ..."
    let result_re = regex::Regex::new(
        r"test result: \w+\.\s+(\d+) passed;\s+(\d+) failed;\s+(\d+) ignored;"
    ).expect("valid regex");

    for line in output.lines() {
        if let Some(caps) = result_re.captures(line) {
            let p: usize = caps[1].parse().unwrap_or(0);
            let f: usize = caps[2].parse().unwrap_or(0);
            let i: usize = caps[3].parse().unwrap_or(0);
            passed += p;
            failed += f;
            ignored += i;
            total += p + f + i;
        }
    }

    TestResults {
        total,
        passed,
        failed,
        ignored,
        failures,
    }
}
use v4_types::OperatorType;

use crate::tool::{Tool, ToolCapability, ToolCategory, ToolError, ToolMetadata};

/// File system read tool
pub struct FileReadTool {
    blocked_paths: Vec<String>,
}

impl FileReadTool {
    pub fn new() -> Self {
        Self {
            blocked_paths: vec![
                "/etc/shadow".to_string(),
                "/etc/passwd".to_string(),
                ".env".to_string(),
            ],
        }
    }

    pub fn with_blocked_paths(paths: Vec<String>) -> Self {
        Self {
            blocked_paths: paths,
        }
    }

    fn is_path_blocked(&self, path: &str) -> bool {
        let path = Path::new(path);
        self.blocked_paths.iter().any(|blocked| {
            path.ends_with(blocked) || path.to_string_lossy().contains(blocked)
        })
    }
}

impl Default for FileReadTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn id(&self) -> &str {
        "builtin:file-read"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "File Read".to_string(),
            description: "Read files from the filesystem".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            capabilities: vec![ToolCapability::ReadOnly, ToolCapability::FileSystemAccess],
            supported_operators: vec!["S".to_string()],
            requires_sandbox: false,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Seek(SeekOp::ReadFile { path }) => {
                if self.is_path_blocked(path) {
                    return Err(ToolError::PermissionDenied(format!(
                        "Access to {} is blocked",
                        path
                    )));
                }

                let content = fs::read_to_string(path)
                    .await
                    .map_err(|e| ToolError::IoError(e.to_string()))?;

                let mut hasher = Sha256::new();
                hasher.update(content.as_bytes());
                let hash = hex::encode(hasher.finalize());

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "content": content,
                        "path": path,
                        "size": content.len(),
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: Some(hash),
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "FileReadTool only handles ReadFile operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Seek(SeekOp::ReadFile { .. }))
    }
}

/// Directory listing tool
pub struct DirectoryListTool {
    max_entries: usize,
}

impl DirectoryListTool {
    pub fn new() -> Self {
        Self { max_entries: 1000 }
    }

    pub fn with_max_entries(max: usize) -> Self {
        Self { max_entries: max }
    }
}

impl Default for DirectoryListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DirectoryListTool {
    fn id(&self) -> &str {
        "builtin:dir-list"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "Directory List".to_string(),
            description: "List directory contents".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            capabilities: vec![ToolCapability::ReadOnly, ToolCapability::FileSystemAccess],
            supported_operators: vec!["S".to_string()],
            requires_sandbox: false,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Seek(SeekOp::ListDirectory { path }) => {
                let mut entries = Vec::new();
                let mut dir = fs::read_dir(path)
                    .await
                    .map_err(|e| ToolError::IoError(e.to_string()))?;

                while let Some(entry) = dir.next_entry().await.map_err(|e| ToolError::IoError(e.to_string()))? {
                    if entries.len() >= self.max_entries {
                        break;
                    }

                    let file_type = entry.file_type().await.ok();
                    let metadata = entry.metadata().await.ok();

                    entries.push(serde_json::json!({
                        "name": entry.file_name().to_string_lossy(),
                        "path": entry.path().to_string_lossy(),
                        "is_dir": file_type.map(|ft| ft.is_dir()).unwrap_or(false),
                        "is_file": file_type.map(|ft| ft.is_file()).unwrap_or(false),
                        "size": metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                    }));
                }

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "path": path,
                        "entries": entries,
                        "count": entries.len(),
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: None,
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "DirectoryListTool only handles ListDirectory operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Seek(SeekOp::ListDirectory { .. }))
    }
}

/// Code search tool - walks directories and searches file contents with regex
pub struct CodeSearchTool {
    max_matches: usize,
}

impl CodeSearchTool {
    pub fn new() -> Self {
        Self { max_matches: 100 }
    }

    pub fn with_max_matches(max: usize) -> Self {
        Self { max_matches: max }
    }

    async fn search_dir(
        path: &std::path::Path,
        regex: &regex::Regex,
        max_matches: usize,
        matches: &mut Vec<serde_json::Value>,
    ) -> Result<(), ToolError> {
        let mut dir = fs::read_dir(path)
            .await
            .map_err(|e| ToolError::IoError(e.to_string()))?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| ToolError::IoError(e.to_string()))?
        {
            if matches.len() >= max_matches {
                return Ok(());
            }

            let entry_path = entry.path();
            let file_name = entry.file_name();

            // Skip hidden entries
            if file_name
                .to_str()
                .is_some_and(|name| name.starts_with('.'))
            {
                continue;
            }

            let file_type = entry
                .file_type()
                .await
                .map_err(|e| ToolError::IoError(e.to_string()))?;

            if file_type.is_dir() {
                Box::pin(Self::search_dir(&entry_path, regex, max_matches, matches)).await?;
            } else if file_type.is_file() {
                let metadata = match fs::metadata(&entry_path).await {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                // Skip files > 1MB
                if metadata.len() > 1_000_000 {
                    continue;
                }

                let bytes = match fs::read(&entry_path).await {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                // Skip binary files (null bytes in first 512 bytes)
                if bytes.iter().take(512).any(|&b| b == 0) {
                    continue;
                }

                let content = match String::from_utf8(bytes) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let file_path_str = entry_path.to_string_lossy().to_string();

                for (line_idx, line) in content.lines().enumerate() {
                    if let Some(m) = regex.find(line) {
                        matches.push(serde_json::json!({
                            "file": file_path_str,
                            "line_number": line_idx + 1,
                            "line": line,
                            "match_text": m.as_str(),
                        }));

                        if matches.len() >= max_matches {
                            return Ok(());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for CodeSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CodeSearchTool {
    fn id(&self) -> &str {
        "builtin:code-search"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "Code Search".to_string(),
            description: "Search code with regex patterns".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::CodeAnalysis,
            capabilities: vec![ToolCapability::ReadOnly, ToolCapability::FileSystemAccess],
            supported_operators: vec!["S".to_string()],
            requires_sandbox: false,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Seek(SeekOp::SearchCode { pattern, path }) => {
                let regex = regex::Regex::new(pattern).map_err(|e| {
                    ToolError::ExecutionFailed(format!("Invalid regex pattern: {}", e))
                })?;

                let search_path = path.as_deref().unwrap_or(".");
                let search_dir = std::path::Path::new(search_path);
                let mut matches = Vec::new();

                Self::search_dir(search_dir, &regex, self.max_matches, &mut matches).await?;

                let truncated = matches.len() >= self.max_matches;

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "pattern": pattern,
                        "search_path": search_path,
                        "matches": matches,
                        "count": matches.len(),
                        "truncated": truncated,
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: None,
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "CodeSearchTool only handles SearchCode operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Seek(SeekOp::SearchCode { .. }))
    }
}

/// File write tool
pub struct FileWriteTool {
    blocked_paths: Vec<String>,
}

impl FileWriteTool {
    pub fn new() -> Self {
        Self {
            blocked_paths: vec![
                "/etc".to_string(),
                "/usr".to_string(),
                "/bin".to_string(),
                "/sbin".to_string(),
            ],
        }
    }

    pub fn with_blocked_paths(paths: Vec<String>) -> Self {
        Self {
            blocked_paths: paths,
        }
    }

    fn is_path_blocked(&self, path: &str) -> bool {
        let path = Path::new(path);
        self.blocked_paths.iter().any(|blocked| {
            path.starts_with(blocked) || path.to_string_lossy().contains(blocked)
        })
    }
}

impl Default for FileWriteTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    fn id(&self) -> &str {
        "builtin:file-write"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "File Write".to_string(),
            description: "Write content to a file, creating parent directories as needed".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            capabilities: vec![ToolCapability::WriteData, ToolCapability::FileSystemAccess],
            supported_operators: vec!["M".to_string()],
            requires_sandbox: true,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Memorize(MemorizeOp::WriteFile { path, content }) => {
                if self.is_path_blocked(path) {
                    return Err(ToolError::PermissionDenied(format!(
                        "Write to {} is blocked",
                        path
                    )));
                }

                let path_obj = Path::new(path);
                if let Some(parent) = path_obj.parent() {
                    if !parent.as_os_str().is_empty() {
                        fs::create_dir_all(parent)
                            .await
                            .map_err(|e| ToolError::IoError(e.to_string()))?;
                    }
                }

                fs::write(path, content)
                    .await
                    .map_err(|e| ToolError::IoError(e.to_string()))?;

                let mut hasher = Sha256::new();
                hasher.update(content.as_bytes());
                let hash = hex::encode(hasher.finalize());

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "path": path,
                        "size": content.len(),
                        "hash": hash,
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: Some(hash.clone()),
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "FileWriteTool only handles WriteFile operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Memorize(MemorizeOp::WriteFile { .. }))
    }
}

/// File edit tool - replace text in files
pub struct FileEditTool {
    blocked_paths: Vec<String>,
}

impl FileEditTool {
    pub fn new() -> Self {
        Self {
            blocked_paths: vec![
                "/etc".to_string(),
                "/usr".to_string(),
                "/bin".to_string(),
                "/sbin".to_string(),
            ],
        }
    }

    pub fn with_blocked_paths(paths: Vec<String>) -> Self {
        Self {
            blocked_paths: paths,
        }
    }

    fn is_path_blocked(&self, path: &str) -> bool {
        let path = Path::new(path);
        self.blocked_paths.iter().any(|blocked| {
            path.starts_with(blocked) || path.to_string_lossy().contains(blocked)
        })
    }
}

impl Default for FileEditTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileEditTool {
    fn id(&self) -> &str {
        "builtin:file-edit"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "File Edit".to_string(),
            description: "Edit a file by replacing text".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            capabilities: vec![ToolCapability::WriteData, ToolCapability::FileSystemAccess],
            supported_operators: vec!["M".to_string()],
            requires_sandbox: true,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Memorize(MemorizeOp::EditFile { path, old_text, new_text, replace_all }) => {
                if self.is_path_blocked(path) {
                    return Err(ToolError::PermissionDenied(format!(
                        "Edit of {} is blocked",
                        path
                    )));
                }

                let content = fs::read_to_string(path)
                    .await
                    .map_err(|e| ToolError::IoError(e.to_string()))?;

                if !content.contains(old_text.as_str()) {
                    return Err(ToolError::ExecutionFailed(
                        "old_text not found in file".to_string(),
                    ));
                }

                let (new_content, replacements_made) = if *replace_all {
                    let count = content.matches(old_text.as_str()).count();
                    (content.replace(old_text.as_str(), new_text.as_str()), count)
                } else {
                    (content.replacen(old_text.as_str(), new_text.as_str(), 1), 1)
                };

                fs::write(path, &new_content)
                    .await
                    .map_err(|e| ToolError::IoError(e.to_string()))?;

                let mut hasher = Sha256::new();
                hasher.update(new_content.as_bytes());
                let hash = hex::encode(hasher.finalize());

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "path": path,
                        "replacements_made": replacements_made,
                        "size": new_content.len(),
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: Some(hash),
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "FileEditTool only handles EditFile operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Memorize(MemorizeOp::EditFile { .. }))
    }
}

/// Shell command execution tool
/// Sandbox profile selection for `sandbox-exec` on macOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellSandboxProfile {
    /// Read/write within working dir, network HTTP/HTTPS, process spawning.
    Standard,
    /// Read-only, no network, no process spawning.
    Restricted,
}

pub struct ShellExecTool {
    blocked_commands: Vec<String>,
    default_timeout_ms: u64,
    /// When set, wraps commands with `sandbox-exec` on macOS.
    sandbox_profile: Option<ShellSandboxProfile>,
}

impl ShellExecTool {
    pub fn new() -> Self {
        Self {
            blocked_commands: vec![
                "rm -rf /".to_string(),
                "dd if=".to_string(),
                "mkfs".to_string(),
                ":(){ :|:& };:".to_string(),
            ],
            default_timeout_ms: 30000,
            sandbox_profile: None,
        }
    }

    /// Enable OS-level sandboxing with the given profile.
    pub fn with_sandbox_profile(mut self, profile: ShellSandboxProfile) -> Self {
        self.sandbox_profile = Some(profile);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.default_timeout_ms = timeout_ms;
        self
    }

    fn is_command_blocked(&self, command: &str) -> bool {
        self.blocked_commands
            .iter()
            .any(|blocked| command.contains(blocked))
    }

    /// Build a `Command` for the given shell command, optionally wrapping with `sandbox-exec`.
    async fn build_command(
        &self,
        command: &str,
        working_dir: Option<&String>,
    ) -> tokio::process::Command {
        if let Some(profile) = self.sandbox_profile {
            if cfg!(target_os = "macos") {
                let allowed_path = working_dir
                    .map(|d| d.as_str())
                    .unwrap_or("/tmp");

                // Embed the profile content and write to a temp file
                let (profile_name, profile_content) = match profile {
                    ShellSandboxProfile::Standard => (
                        "v4-sandbox-standard.sb",
                        include_str!("../../v4-sandbox/profiles/standard.sb"),
                    ),
                    ShellSandboxProfile::Restricted => (
                        "v4-sandbox-restricted.sb",
                        include_str!("../../v4-sandbox/profiles/restricted.sb"),
                    ),
                };

                let profile_path = std::env::temp_dir().join(profile_name);
                // Best-effort write, ignore errors (may already exist)
                let _ = tokio::fs::write(&profile_path, profile_content).await;

                let mut cmd = tokio::process::Command::new("sandbox-exec");
                cmd.arg("-f")
                    .arg(&profile_path)
                    .arg("-D")
                    .arg(format!("ALLOWED_PATH={allowed_path}"))
                    .arg("/bin/sh")
                    .arg("-c")
                    .arg(command);

                if let Some(dir) = working_dir {
                    cmd.current_dir(dir);
                }

                return cmd;
            }
            // Non-macOS: warn and fall back
            tracing::warn!(
                profile = ?profile,
                "sandbox-exec not available on this platform, executing without OS-level sandbox"
            );
        }

        let mut cmd = tokio::process::Command::new("/bin/sh");
        cmd.arg("-c").arg(command);
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        cmd
    }
}

impl Default for ShellExecTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ShellExecTool {
    fn id(&self) -> &str {
        "builtin:shell-exec"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "Shell Execute".to_string(),
            description: "Execute a shell command".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::Control,
            capabilities: vec![ToolCapability::ExecuteCode],
            supported_operators: vec!["C".to_string()],
            requires_sandbox: true,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Control(ControlOp::RunCommand { command, working_dir, timeout_ms }) => {
                if self.is_command_blocked(command) {
                    return Err(ToolError::PermissionDenied(
                        "Command contains blocked pattern".to_string(),
                    ));
                }

                let timeout = timeout_ms.unwrap_or(self.default_timeout_ms);

                let mut cmd = self.build_command(command, working_dir.as_ref()).await;

                cmd.stdout(std::process::Stdio::piped());
                cmd.stderr(std::process::Stdio::piped());

                let output = tokio::time::timeout(
                    std::time::Duration::from_millis(timeout),
                    cmd.output(),
                )
                .await
                .map_err(|_| ToolError::Timeout(timeout))?
                .map_err(|e| ToolError::IoError(e.to_string()))?;

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let max_output = 100 * 1024; // 100KB
                let truncated = stdout.len() + stderr.len() > max_output;

                let combined = format!("{}{}", stdout, stderr);
                let mut hasher = Sha256::new();
                hasher.update(combined.as_bytes());
                let hash = hex::encode(hasher.finalize());

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: output.status.success(),
                    data: Some(serde_json::json!({
                        "command": command,
                        "exit_code": output.status.code(),
                        "stdout": if truncated { &stdout[..stdout.len().min(max_output)] } else { &stdout },
                        "stderr": if truncated { &stderr[..stderr.len().min(max_output)] } else { &stderr },
                        "truncated": truncated,
                    })),
                    error: if output.status.success() { None } else { Some(stderr.clone()) },
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: Some(hash),
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "ShellExecTool only handles RunCommand operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Control(ControlOp::RunCommand { .. }))
    }
}

/// Memory query tool (delegates to memory system)
pub struct MemoryQueryTool;

impl MemoryQueryTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemoryQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for MemoryQueryTool {
    fn id(&self) -> &str {
        "builtin:memory-query"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "Memory Query".to_string(),
            description: "Query the knowledge graph memory".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::Memory,
            capabilities: vec![ToolCapability::ReadOnly],
            supported_operators: vec!["S".to_string()],
            requires_sandbox: false,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Seek(SeekOp::QueryMemory { query, max_hops }) => {
                // This would delegate to the actual memory system
                // For now, return a placeholder indicating the query was received
                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "query": query,
                        "max_hops": max_hops,
                        "results": [],
                        "note": "Memory query requires memory system integration"
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: None,
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "MemoryQueryTool only handles QueryMemory operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Seek(SeekOp::QueryMemory { .. }))
    }
}

/// File patch tool — applies unified diffs
pub struct FilePatchTool {
    blocked_paths: Vec<String>,
}

impl FilePatchTool {
    /// Create a new `FilePatchTool` with default blocked paths.
    pub fn new() -> Self {
        Self {
            blocked_paths: vec![
                "/etc".to_string(),
                "/usr".to_string(),
                "/bin".to_string(),
                "/sbin".to_string(),
            ],
        }
    }

    /// Create with custom blocked paths.
    pub fn with_blocked_paths(paths: Vec<String>) -> Self {
        Self {
            blocked_paths: paths,
        }
    }

    fn is_path_blocked(&self, path: &str) -> bool {
        let path = Path::new(path);
        self.blocked_paths.iter().any(|blocked| {
            path.starts_with(blocked) || path.to_string_lossy().contains(blocked)
        })
    }
}

impl Default for FilePatchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FilePatchTool {
    fn id(&self) -> &str {
        "builtin:file-patch"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "File Patch".to_string(),
            description: "Apply a unified diff patch to a file".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            capabilities: vec![ToolCapability::WriteData, ToolCapability::FileSystemAccess],
            supported_operators: vec!["M".to_string()],
            requires_sandbox: true,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Memorize(MemorizeOp::PatchFile { path, diff }) => {
                if self.is_path_blocked(path) {
                    return Err(ToolError::PermissionDenied(format!(
                        "Patch to {} is blocked",
                        path
                    )));
                }

                // Read the original file
                let original = fs::read_to_string(path)
                    .await
                    .map_err(|e| ToolError::IoError(e.to_string()))?;

                // Create .bak backup
                let bak_path = format!("{path}.bak");
                fs::write(&bak_path, &original)
                    .await
                    .map_err(|e| ToolError::IoError(format!("Failed to write backup: {e}")))?;

                // Parse and apply the diff
                let hunks = parse_unified_diff(diff).map_err(|e| {
                    ToolError::ExecutionFailed(format!("Failed to parse diff: {e}"))
                })?;

                let patched = apply_hunks(&original, &hunks).map_err(|e| {
                    ToolError::ExecutionFailed(format!("Patch failed: {e}"))
                })?;

                // Write the patched file
                fs::write(path, &patched)
                    .await
                    .map_err(|e| ToolError::IoError(e.to_string()))?;

                let mut hasher = Sha256::new();
                hasher.update(patched.as_bytes());
                let hash = hex::encode(hasher.finalize());

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "path": path,
                        "backup": bak_path,
                        "size": patched.len(),
                        "hunks_applied": hunks.len(),
                    })),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: Some(hash),
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "FilePatchTool only handles PatchFile operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Memorize(MemorizeOp::PatchFile { .. }))
    }
}

/// Test runner tool — runs cargo test and parses results
pub struct TestRunnerTool {
    default_timeout_ms: u64,
}

impl TestRunnerTool {
    /// Create a new `TestRunnerTool` with 300s default timeout.
    pub fn new() -> Self {
        Self {
            default_timeout_ms: 300_000,
        }
    }

    /// Set the default timeout.
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.default_timeout_ms = timeout_ms;
        self
    }
}

impl Default for TestRunnerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TestRunnerTool {
    fn id(&self) -> &str {
        "builtin:test-runner"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "Test Runner".to_string(),
            description: "Run cargo test and parse results".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::CodeAnalysis,
            capabilities: vec![ToolCapability::ExecuteCode],
            supported_operators: vec!["C".to_string()],
            requires_sandbox: true,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Control(ControlOp::RunTests {
                crate_name,
                test_filter,
                timeout_ms,
            }) => {
                let mut args = vec!["test".to_string()];

                if let Some(crate_name) = crate_name {
                    args.push("-p".to_string());
                    args.push(crate_name.clone());
                }

                if let Some(filter) = test_filter {
                    args.push("--".to_string());
                    args.push(filter.clone());
                }

                let mut cmd = tokio::process::Command::new("cargo");
                cmd.args(&args);
                cmd.stdout(std::process::Stdio::piped());
                cmd.stderr(std::process::Stdio::piped());

                let timeout = timeout_ms.unwrap_or(self.default_timeout_ms);

                let output = tokio::time::timeout(
                    std::time::Duration::from_millis(timeout),
                    cmd.output(),
                )
                .await
                .map_err(|_| ToolError::Timeout(timeout))?
                .map_err(|e| ToolError::IoError(e.to_string()))?;

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let combined = format!("{stdout}{stderr}");
                let results = parse_cargo_test_output(&combined);

                let mut hasher = Sha256::new();
                hasher.update(combined.as_bytes());
                let hash = hex::encode(hasher.finalize());

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: results.failed == 0 && output.status.success(),
                    data: Some(serde_json::json!({
                        "total": results.total,
                        "passed": results.passed,
                        "failed": results.failed,
                        "ignored": results.ignored,
                        "failures": results.failures,
                        "stdout": stdout,
                        "stderr": stderr,
                    })),
                    error: if results.failed > 0 {
                        Some(format!("{} test(s) failed: {}", results.failed, results.failures.join(", ")))
                    } else {
                        None
                    },
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    content_hash: Some(hash),
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "TestRunnerTool only handles RunTests operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Control(ControlOp::RunTests { .. }))
    }
}

/// Register all built-in tools with a registry
pub fn register_builtin_tools(registry: &crate::registry::ToolRegistry) {
    use std::sync::Arc;

    registry.register(Arc::new(FileReadTool::new()));
    registry.register(Arc::new(FileWriteTool::new()));
    registry.register(Arc::new(FileEditTool::new()));
    registry.register(Arc::new(DirectoryListTool::new()));
    registry.register(Arc::new(CodeSearchTool::new()));
    registry.register(Arc::new(MemoryQueryTool::new()));
    registry.register(Arc::new(ShellExecTool::new()));
    registry.register(Arc::new(FilePatchTool::new()));
    registry.register(Arc::new(TestRunnerTool::new()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ToolRegistry;

    #[test]
    fn test_file_read_tool_metadata() {
        let tool = FileReadTool::new();
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:file-read");
        assert_eq!(meta.category, ToolCategory::FileSystem);
        assert!(meta.capabilities.contains(&ToolCapability::ReadOnly));
    }

    #[test]
    fn test_file_read_tool_blocked_paths() {
        let tool = FileReadTool::new();

        assert!(tool.is_path_blocked("/etc/shadow"));
        assert!(tool.is_path_blocked("/path/to/.env"));
        assert!(!tool.is_path_blocked("/home/user/file.txt"));
    }

    #[test]
    fn test_file_read_can_execute() {
        let tool = FileReadTool::new();

        let read_op = OperatorType::Seek(SeekOp::ReadFile {
            path: "test.txt".to_string(),
        });
        assert!(tool.can_execute(&read_op));

        let list_op = OperatorType::Seek(SeekOp::ListDirectory {
            path: ".".to_string(),
        });
        assert!(!tool.can_execute(&list_op));
    }

    #[test]
    fn test_register_builtin_tools() {
        let registry = ToolRegistry::new();
        register_builtin_tools(&registry);

        assert!(registry.has("builtin:file-read"));
        assert!(registry.has("builtin:file-write"));
        assert!(registry.has("builtin:file-edit"));
        assert!(registry.has("builtin:dir-list"));
        assert!(registry.has("builtin:code-search"));
        assert!(registry.has("builtin:memory-query"));
        assert!(registry.has("builtin:shell-exec"));
        assert!(registry.has("builtin:file-patch"));
        assert!(registry.has("builtin:test-runner"));
        assert_eq!(registry.count(), 9);
    }

    #[test]
    fn test_directory_list_metadata() {
        let tool = DirectoryListTool::new();
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:dir-list");
        assert!(meta.supported_operators.contains(&"S".to_string()));
    }

    #[test]
    fn test_file_write_tool_metadata() {
        let tool = FileWriteTool::new();
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:file-write");
        assert_eq!(meta.category, ToolCategory::FileSystem);
        assert!(meta.capabilities.contains(&ToolCapability::WriteData));
        assert!(meta.requires_sandbox);
    }

    #[test]
    fn test_file_write_tool_blocked_paths() {
        let tool = FileWriteTool::new();

        assert!(tool.is_path_blocked("/etc/passwd"));
        assert!(tool.is_path_blocked("/usr/bin/something"));
        assert!(!tool.is_path_blocked("/tmp/test.txt"));
    }

    #[test]
    fn test_file_write_can_execute() {
        let tool = FileWriteTool::new();

        let write_op = OperatorType::Memorize(MemorizeOp::WriteFile {
            path: "test.txt".to_string(),
            content: "hello".to_string(),
        });
        assert!(tool.can_execute(&write_op));

        let read_op = OperatorType::Seek(SeekOp::ReadFile {
            path: "test.txt".to_string(),
        });
        assert!(!tool.can_execute(&read_op));
    }

    #[test]
    fn test_file_edit_tool_metadata() {
        let tool = FileEditTool::new();
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:file-edit");
        assert_eq!(meta.category, ToolCategory::FileSystem);
        assert!(meta.capabilities.contains(&ToolCapability::WriteData));
    }

    #[test]
    fn test_file_edit_can_execute() {
        let tool = FileEditTool::new();

        let edit_op = OperatorType::Memorize(MemorizeOp::EditFile {
            path: "test.txt".to_string(),
            old_text: "old".to_string(),
            new_text: "new".to_string(),
            replace_all: false,
        });
        assert!(tool.can_execute(&edit_op));

        let write_op = OperatorType::Memorize(MemorizeOp::WriteFile {
            path: "test.txt".to_string(),
            content: "hello".to_string(),
        });
        assert!(!tool.can_execute(&write_op));
    }

    #[test]
    fn test_shell_exec_tool_metadata() {
        let tool = ShellExecTool::new();
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:shell-exec");
        assert_eq!(meta.category, ToolCategory::Control);
        assert!(meta.capabilities.contains(&ToolCapability::ExecuteCode));
        assert!(meta.requires_sandbox);
    }

    #[test]
    fn test_shell_exec_blocked_commands() {
        let tool = ShellExecTool::new();

        assert!(tool.is_command_blocked("rm -rf /"));
        assert!(tool.is_command_blocked("dd if=/dev/zero of=/dev/sda"));
        assert!(tool.is_command_blocked("mkfs.ext4 /dev/sda"));
        assert!(!tool.is_command_blocked("echo hello"));
        assert!(!tool.is_command_blocked("ls -la"));
    }

    #[test]
    fn test_shell_exec_can_execute() {
        let tool = ShellExecTool::new();

        let run_op = OperatorType::Control(ControlOp::RunCommand {
            command: "echo hello".to_string(),
            working_dir: None,
            timeout_ms: None,
        });
        assert!(tool.can_execute(&run_op));

        let term_op = OperatorType::Control(ControlOp::Terminate {
            reason: "done".to_string(),
        });
        assert!(!tool.can_execute(&term_op));
    }

    #[tokio::test]
    async fn test_file_write_execute() {
        let dir = std::env::temp_dir().join("v4-tools-test-write");
        let _ = fs::create_dir_all(&dir).await;
        let file_path = dir.join("test_write.txt");

        let tool = FileWriteTool::with_blocked_paths(vec![]);
        let op = OperatorType::Memorize(MemorizeOp::WriteFile {
            path: file_path.to_string_lossy().to_string(),
            content: "hello world".to_string(),
        });

        let result = tool.execute(&op).await.unwrap();
        assert!(result.success);
        assert!(result.content_hash.is_some());

        let written = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(written, "hello world");

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn test_file_edit_execute() {
        let dir = std::env::temp_dir().join("v4-tools-test-edit");
        let _ = fs::create_dir_all(&dir).await;
        let file_path = dir.join("test_edit.txt");
        fs::write(&file_path, "hello old world, old friend").await.unwrap();

        let tool = FileEditTool::with_blocked_paths(vec![]);

        // Test single replacement
        let op = OperatorType::Memorize(MemorizeOp::EditFile {
            path: file_path.to_string_lossy().to_string(),
            old_text: "old".to_string(),
            new_text: "new".to_string(),
            replace_all: false,
        });
        let result = tool.execute(&op).await.unwrap();
        assert!(result.success);

        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "hello new world, old friend");

        // Test replace_all
        fs::write(&file_path, "aaa bbb aaa bbb aaa").await.unwrap();
        let op = OperatorType::Memorize(MemorizeOp::EditFile {
            path: file_path.to_string_lossy().to_string(),
            old_text: "aaa".to_string(),
            new_text: "ccc".to_string(),
            replace_all: true,
        });
        let result = tool.execute(&op).await.unwrap();
        assert!(result.success);
        let data = result.data.unwrap();
        assert_eq!(data["replacements_made"], 3);

        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "ccc bbb ccc bbb ccc");

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn test_file_edit_not_found() {
        let dir = std::env::temp_dir().join("v4-tools-test-edit-nf");
        let _ = fs::create_dir_all(&dir).await;
        let file_path = dir.join("test_edit_nf.txt");
        fs::write(&file_path, "hello world").await.unwrap();

        let tool = FileEditTool::with_blocked_paths(vec![]);
        let op = OperatorType::Memorize(MemorizeOp::EditFile {
            path: file_path.to_string_lossy().to_string(),
            old_text: "nonexistent".to_string(),
            new_text: "replacement".to_string(),
            replace_all: false,
        });
        let result = tool.execute(&op).await;
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn test_shell_exec_execute() {
        let tool = ShellExecTool::new();
        let op = OperatorType::Control(ControlOp::RunCommand {
            command: "echo hello".to_string(),
            working_dir: None,
            timeout_ms: Some(5000),
        });

        let result = tool.execute(&op).await.unwrap();
        assert!(result.success);
        let data = result.data.unwrap();
        assert_eq!(data["stdout"].as_str().unwrap().trim(), "hello");
        assert_eq!(data["exit_code"], 0);
    }

    #[tokio::test]
    async fn test_shell_exec_blocked() {
        let tool = ShellExecTool::new();
        let op = OperatorType::Control(ControlOp::RunCommand {
            command: "rm -rf /".to_string(),
            working_dir: None,
            timeout_ms: None,
        });

        let result = tool.execute(&op).await;
        assert!(result.is_err());
    }

    // ─── FilePatchTool tests ─────────────────────────────────────────

    #[test]
    fn test_file_patch_tool_metadata() {
        let tool = FilePatchTool::new();
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:file-patch");
        assert_eq!(meta.category, ToolCategory::FileSystem);
        assert!(meta.capabilities.contains(&ToolCapability::WriteData));
        assert!(meta.requires_sandbox);
    }

    #[test]
    fn test_file_patch_can_execute() {
        let tool = FilePatchTool::new();

        let patch_op = OperatorType::Memorize(MemorizeOp::PatchFile {
            path: "test.rs".to_string(),
            diff: "".to_string(),
        });
        assert!(tool.can_execute(&patch_op));

        let write_op = OperatorType::Memorize(MemorizeOp::WriteFile {
            path: "test.rs".to_string(),
            content: "".to_string(),
        });
        assert!(!tool.can_execute(&write_op));
    }

    #[tokio::test]
    async fn test_file_patch_execute() {
        let dir = std::env::temp_dir().join("v4-tools-test-patch");
        let _ = fs::create_dir_all(&dir).await;
        let file_path = dir.join("test_patch.txt");

        // Write original file
        fs::write(&file_path, "line 1\nline 2\nline 3\nline 4\n")
            .await
            .unwrap();

        // Create a unified diff that changes line 2
        let diff = "\
--- a/test_patch.txt
+++ b/test_patch.txt
@@ -1,4 +1,4 @@
 line 1
-line 2
+line TWO
 line 3
 line 4";

        let tool = FilePatchTool::with_blocked_paths(vec![]);
        let op = OperatorType::Memorize(MemorizeOp::PatchFile {
            path: file_path.to_string_lossy().to_string(),
            diff: diff.to_string(),
        });

        let result = tool.execute(&op).await.unwrap();
        assert!(result.success);
        assert!(result.content_hash.is_some());

        let patched = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(patched, "line 1\nline TWO\nline 3\nline 4\n");

        // Verify .bak backup was created
        let bak_path = format!("{}.bak", file_path.to_string_lossy());
        let backup = fs::read_to_string(&bak_path).await.unwrap();
        assert_eq!(backup, "line 1\nline 2\nline 3\nline 4\n");

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn test_file_patch_wrong_context() {
        let dir = std::env::temp_dir().join("v4-tools-test-patch-err");
        let _ = fs::create_dir_all(&dir).await;
        let file_path = dir.join("test_patch_err.txt");

        fs::write(&file_path, "line 1\nline 2\nline 3\n")
            .await
            .unwrap();

        // Diff with wrong context lines
        let diff = "\
@@ -1,3 +1,3 @@
 line 1
-WRONG CONTEXT
+line TWO
 line 3";

        let tool = FilePatchTool::with_blocked_paths(vec![]);
        let op = OperatorType::Memorize(MemorizeOp::PatchFile {
            path: file_path.to_string_lossy().to_string(),
            diff: diff.to_string(),
        });

        let result = tool.execute(&op).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("mismatch"), "Expected mismatch error, got: {err_msg}");

        let _ = fs::remove_dir_all(&dir).await;
    }

    // ─── TestRunnerTool tests ────────────────────────────────────────

    #[test]
    fn test_test_runner_tool_metadata() {
        let tool = TestRunnerTool::new();
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:test-runner");
        assert_eq!(meta.category, ToolCategory::CodeAnalysis);
        assert!(meta.capabilities.contains(&ToolCapability::ExecuteCode));
        assert!(meta.requires_sandbox);
    }

    #[test]
    fn test_test_runner_can_execute() {
        let tool = TestRunnerTool::new();

        let test_op = OperatorType::Control(ControlOp::RunTests {
            crate_name: None,
            test_filter: None,
            timeout_ms: None,
        });
        assert!(tool.can_execute(&test_op));

        let run_op = OperatorType::Control(ControlOp::RunCommand {
            command: "echo".to_string(),
            working_dir: None,
            timeout_ms: None,
        });
        assert!(!tool.can_execute(&run_op));
    }

    #[test]
    fn test_parse_cargo_test_output_success() {
        let output = "\
running 5 tests
test tests::test_one ... ok
test tests::test_two ... ok
test tests::test_three ... ok
test tests::test_four ... ignored
test tests::test_five ... ok

test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s
";
        let results = parse_cargo_test_output(output);
        assert_eq!(results.total, 5);
        assert_eq!(results.passed, 4);
        assert_eq!(results.failed, 0);
        assert_eq!(results.ignored, 1);
        assert!(results.failures.is_empty());
    }

    #[test]
    fn test_parse_cargo_test_output_failures() {
        let output = "\
running 3 tests
test tests::test_one ... ok
test tests::test_two ... FAILED
test tests::test_three ... FAILED

failures:

---- tests::test_two stdout ----
thread 'test_two' panicked at 'assertion failed'

---- tests::test_three stdout ----
thread 'test_three' panicked at 'expected true'

failures:
    tests::test_two
    tests::test_three

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
";
        let results = parse_cargo_test_output(output);
        assert_eq!(results.total, 3);
        assert_eq!(results.passed, 1);
        assert_eq!(results.failed, 2);
        assert_eq!(results.ignored, 0);
        assert_eq!(results.failures.len(), 2);
        assert!(results.failures.contains(&"tests::test_two".to_string()));
        assert!(results.failures.contains(&"tests::test_three".to_string()));
    }

    #[test]
    fn test_parse_cargo_test_output_multiple_crates() {
        let output = "\
running 2 tests
test tests::a ... ok
test tests::b ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 1 test
test tests::c ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
";
        let results = parse_cargo_test_output(output);
        assert_eq!(results.total, 3);
        assert_eq!(results.passed, 2);
        assert_eq!(results.failed, 1);
        assert_eq!(results.failures, vec!["tests::c"]);
    }

    // ─── Unified diff parsing unit tests ─────────────────────────────

    #[test]
    fn test_apply_hunks_basic() {
        let original = "line 1\nline 2\nline 3\n";
        let hunks = vec![DiffHunk {
            old_start: 2,
            old_count: 1,
            lines: vec![
                DiffLine::Remove("line 2".to_string()),
                DiffLine::Add("LINE TWO".to_string()),
            ],
        }];
        let result = apply_hunks(original, &hunks).unwrap();
        assert_eq!(result, "line 1\nLINE TWO\nline 3\n");
    }

    #[test]
    fn test_apply_hunks_context_mismatch() {
        let original = "line 1\nline 2\nline 3\n";
        let hunks = vec![DiffHunk {
            old_start: 1,
            old_count: 2,
            lines: vec![
                DiffLine::Context("WRONG".to_string()),
                DiffLine::Remove("line 2".to_string()),
                DiffLine::Add("new".to_string()),
            ],
        }];
        let result = apply_hunks(original, &hunks);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mismatch"));
    }

    #[test]
    fn test_parse_unified_diff_roundtrip() {
        let diff = "\
--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,3 @@
 alpha
-beta
+BETA
 gamma";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_start, 1);
        assert_eq!(hunks[0].old_count, 3);

        let original = "alpha\nbeta\ngamma\n";
        let patched = apply_hunks(original, &hunks).unwrap();
        assert_eq!(patched, "alpha\nBETA\ngamma\n");
    }
}
