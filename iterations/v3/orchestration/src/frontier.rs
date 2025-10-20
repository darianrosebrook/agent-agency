//! Frontier Queue for Autonomous Task Generation
//!
//! Manages spawned tasks from autonomous agents with deduplication,
//! rate limiting, and scope enforcement to prevent task explosion.

use std::collections::{BinaryHeap, HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::planning::types::Task;

/// Entry in the frontier priority queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEntry {
    /// The task to be executed
    pub task: Task,
    /// Fingerprint for deduplication (hash of task properties)
    pub fingerprint: String,
    /// ID of the parent task that spawned this one
    pub parent_id: String,
    /// Priority score (higher = more important)
    pub priority: f64,
    /// When this task was added to the frontier
    pub timestamp: DateTime<Utc>,
}

impl PartialEq for TaskEntry {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Eq for TaskEntry {}

impl PartialOrd for TaskEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier timestamp for FIFO within priority
        match self.priority.partial_cmp(&other.priority).unwrap_or(std::cmp::Ordering::Equal) {
            std::cmp::Ordering::Equal => other.timestamp.cmp(&self.timestamp), // Earlier timestamp first
            ord => ord.reverse(), // Higher priority first
        }
    }
}

/// Configuration for the frontier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierConfig {
    /// Maximum number of tasks in the frontier
    pub max_size: usize,
    /// Maximum tasks per parent per hour
    pub max_per_parent_per_hour: usize,
    /// Global maximum tasks per hour
    pub max_global_per_hour: usize,
    /// How long to keep rate limit data (in seconds)
    pub rate_limit_window_seconds: u64,
}

impl Default for FrontierConfig {
    fn default() -> Self {
        Self {
            max_size: 100,
            max_per_parent_per_hour: 5,
            max_global_per_hour: 20,
            rate_limit_window_seconds: 3600, // 1 hour
        }
    }
}

/// Errors that can occur in frontier operations
#[derive(Debug, thiserror::Error)]
pub enum FrontierError {
    #[error("Frontier is at maximum capacity ({0})")]
    AtCapacity(usize),

    #[error("Task fingerprint already exists: {0}")]
    DuplicateFingerprint(String),

    #[error("Rate limit exceeded for parent: {0}")]
    ParentRateLimitExceeded(String),

    #[error("Global rate limit exceeded")]
    GlobalRateLimitExceeded,

    #[error("Task violates scope envelope")]
    ScopeViolation,

    #[error("Invalid task configuration")]
    InvalidTask,
}

/// Rate limit tracking entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: usize,
    window_start: DateTime<Utc>,
}

/// Frontier queue for managing spawned tasks
#[derive(Debug)]
pub struct Frontier {
    /// Priority queue of tasks (max-heap by priority)
    queue: BinaryHeap<TaskEntry>,
    /// Set of fingerprints to prevent duplicates
    fingerprints: HashSet<String>,
    /// Rate limiting per parent
    parent_rate_limits: HashMap<String, RateLimitEntry>,
    /// Global rate limiting
    global_rate_limit: RateLimitEntry,
    /// Configuration
    config: FrontierConfig,
    /// Scope envelope (paths that spawned tasks must be within)
    scope_envelope: Vec<String>,
}

impl Frontier {
    /// Create a new frontier with default configuration
    pub fn new() -> Self {
        Self::with_config(FrontierConfig::default())
    }

    /// Create a new frontier with custom configuration
    pub fn with_config(config: FrontierConfig) -> Self {
        Self {
            queue: BinaryHeap::new(),
            fingerprints: HashSet::new(),
            parent_rate_limits: HashMap::new(),
            global_rate_limit: RateLimitEntry {
                count: 0,
                window_start: Utc::now(),
            },
            config,
            scope_envelope: Vec::new(),
        }
    }

    /// Set the scope envelope for this frontier
    pub fn with_scope_envelope(mut self, scope: Vec<String>) -> Self {
        self.scope_envelope = scope;
        self
    }

    /// Add a task to the frontier with automatic priority calculation
    pub fn push(&mut self, task: Task, parent_id: &str) -> Result<(), FrontierError> {
        self.push_with_priority(task, parent_id, self.calculate_priority(&task, parent_id))
    }

    /// Add a task with explicit priority
    pub fn push_with_priority(
        &mut self,
        task: Task,
        parent_id: &str,
        priority: f64,
    ) -> Result<(), FrontierError> {
        // Validate scope envelope
        if !self.check_scope_envelope(&task) {
            return Err(FrontierError::ScopeViolation);
        }

        // Check rate limits
        self.check_rate_limits(parent_id)?;

        // Generate fingerprint and check for duplicates
        let fingerprint = self.generate_fingerprint(&task);
        if self.fingerprints.contains(&fingerprint) {
            return Err(FrontierError::DuplicateFingerprint(fingerprint));
        }

        // Check capacity
        if self.queue.len() >= self.config.max_size {
            // Try to evict lowest priority task if we're at capacity
            if !self.evict_lowest_priority() {
                return Err(FrontierError::AtCapacity(self.config.max_size));
            }
        }

        // Add the task
        let entry = TaskEntry {
            task,
            fingerprint: fingerprint.clone(),
            parent_id: parent_id.to_string(),
            priority,
            timestamp: Utc::now(),
        };

        self.queue.push(entry);
        self.fingerprints.insert(fingerprint);

        // Update rate limits
        self.update_rate_limits(parent_id);

        Ok(())
    }

    /// Remove and return the highest priority task
    pub fn pop(&mut self) -> Option<Task> {
        if let Some(entry) = self.queue.pop() {
            self.fingerprints.remove(&entry.fingerprint);
            Some(entry.task)
        } else {
            None
        }
    }

    /// Peek at the highest priority task without removing it
    pub fn peek(&self) -> Option<&Task> {
        self.queue.peek().map(|entry| &entry.task)
    }

    /// Check if the frontier is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get the current size of the frontier
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if a fingerprint already exists
    pub fn contains_fingerprint(&self, fingerprint: &str) -> bool {
        self.fingerprints.contains(fingerprint)
    }

    /// Get statistics about the frontier
    pub fn stats(&self) -> FrontierStats {
        let mut priorities = Vec::new();
        let mut parent_counts = HashMap::new();

        for entry in &self.queue {
            priorities.push(entry.priority);
            *parent_counts.entry(entry.parent_id.clone()).or_insert(0) += 1;
        }

        FrontierStats {
            size: self.queue.len(),
            priorities,
            parent_counts,
        }
    }

    /// Calculate priority for a task (internal method)
    fn calculate_priority(&self, task: &Task, parent_id: &str) -> f64 {
        // Base priority from task type
        let base_priority = match task.task_type {
            crate::planning::types::TaskType::CriticalFix => 1.0,
            crate::planning::types::TaskType::SecurityIssue => 0.9,
            crate::planning::types::TaskType::Performance => 0.8,
            crate::planning::types::TaskType::Feature => 0.6,
            crate::planning::types::TaskType::Refactor => 0.4,
            crate::planning::types::TaskType::Documentation => 0.2,
        };

        // Boost for tasks with fewer target files (more focused)
        let focus_boost = if task.target_files.len() <= 2 { 0.2 } else { 0.0 };

        // Penalty for tasks from parents that have many spawned tasks
        let parent_penalty = if let Some(&count) = self.parent_rate_limits.get(parent_id) {
            (count as f64 * 0.05).min(0.3) // Max 30% penalty
        } else {
            0.0
        };

        (base_priority + focus_boost - parent_penalty).max(0.0).min(1.0)
    }

    /// Generate a fingerprint for deduplication
    fn generate_fingerprint(&self, task: &Task) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        task.description.hash(&mut hasher);
        task.task_type.hash(&mut hasher);
        task.target_files.hash(&mut hasher);

        // Include constraints in fingerprint (order shouldn't matter)
        let mut sorted_constraints: Vec<_> = task.constraints.iter().collect();
        sorted_constraints.sort_by_key(|(k, _)| *k);
        sorted_constraints.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    /// Check if task is within scope envelope
    fn check_scope_envelope(&self, task: &Task) -> bool {
        if self.scope_envelope.is_empty() {
            return true; // No scope restrictions
        }

        // Check if all target files are within the scope envelope
        for file in &task.target_files {
            let mut allowed = false;
            for scope_pattern in &self.scope_envelope {
                if self.matches_scope_pattern(file, scope_pattern) {
                    allowed = true;
                    break;
                }
            }
            if !allowed {
                return false;
            }
        }

        true
    }

    /// Simple scope pattern matching (can be enhanced with glob patterns)
    fn matches_scope_pattern(&self, file: &str, pattern: &str) -> bool {
        if pattern.ends_with("/**") {
            // Directory prefix match
            let prefix = &pattern[..pattern.len() - 3];
            file.starts_with(prefix)
        } else if pattern.ends_with("/*") {
            // File in directory
            let dir = &pattern[..pattern.len() - 2];
            file.starts_with(dir) && !file[dir.len()..].contains('/')
        } else {
            // Exact match
            file == pattern
        }
    }

    /// Check rate limits before adding task
    fn check_rate_limits(&self, parent_id: &str) -> Result<(), FrontierError> {
        let now = Utc::now();

        // Check parent rate limit
        if let Some(entry) = self.parent_rate_limits.get(parent_id) {
            if !self.is_rate_limit_expired(entry, now) && entry.count >= self.config.max_per_parent_per_hour {
                return Err(FrontierError::ParentRateLimitExceeded(parent_id.to_string()));
            }
        }

        // Check global rate limit
        if !self.is_rate_limit_expired(&self.global_rate_limit, now) &&
           self.global_rate_limit.count >= self.config.max_global_per_hour {
            return Err(FrontierError::GlobalRateLimitExceeded);
        }

        Ok(())
    }

    /// Update rate limits after adding task
    fn update_rate_limits(&mut self, parent_id: &str) {
        let now = Utc::now();

        // Update parent rate limit
        let parent_entry = self.parent_rate_limits.entry(parent_id.to_string())
            .or_insert_with(|| RateLimitEntry {
                count: 0,
                window_start: now,
            });

        if self.is_rate_limit_expired(parent_entry, now) {
            parent_entry.count = 1;
            parent_entry.window_start = now;
        } else {
            parent_entry.count += 1;
        }

        // Update global rate limit
        if self.is_rate_limit_expired(&self.global_rate_limit, now) {
            self.global_rate_limit.count = 1;
            self.global_rate_limit.window_start = now;
        } else {
            self.global_rate_limit.count += 1;
        }
    }

    /// Check if rate limit window has expired
    fn is_rate_limit_expired(&self, entry: &RateLimitEntry, now: DateTime<Utc>) -> bool {
        let elapsed = now.signed_duration_since(entry.window_start);
        elapsed.num_seconds() as u64 >= self.config.rate_limit_window_seconds
    }

    /// Try to evict the lowest priority task when at capacity
    fn evict_lowest_priority(&mut self) -> bool {
        // BinaryHeap doesn't support efficient removal of arbitrary elements
        // For now, we'll just reject if at capacity
        // In a production implementation, you might want a different data structure
        false
    }
}

/// Statistics about the frontier state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierStats {
    pub size: usize,
    pub priorities: Vec<f64>,
    pub parent_counts: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::types::{TaskType};

    fn create_test_task(description: &str, task_type: TaskType, files: Vec<String>) -> Task {
        Task {
            id: uuid::Uuid::new_v4(),
            description: description.to_string(),
            task_type,
            target_files: files,
            constraints: HashMap::new(),
            refinement_context: vec![],
        }
    }

    #[test]
    fn test_frontier_push_pop() {
        let mut frontier = Frontier::new();

        let task1 = create_test_task("task1", TaskType::CriticalFix, vec!["src/main.rs".to_string()]);
        let task2 = create_test_task("task2", TaskType::Feature, vec!["src/utils.rs".to_string()]);

        frontier.push(task1.clone(), "parent1").unwrap();
        frontier.push(task2.clone(), "parent2").unwrap();

        assert_eq!(frontier.len(), 2);

        // Should pop higher priority task first (CriticalFix > Feature)
        let popped = frontier.pop().unwrap();
        assert_eq!(popped.description, "task1");

        let popped2 = frontier.pop().unwrap();
        assert_eq!(popped2.description, "task2");

        assert!(frontier.is_empty());
    }

    #[test]
    fn test_fingerprint_deduplication() {
        let mut frontier = Frontier::new();

        let task1 = create_test_task("duplicate", TaskType::Feature, vec!["src/test.rs".to_string()]);
        let task2 = create_test_task("duplicate", TaskType::Feature, vec!["src/test.rs".to_string()]);

        frontier.push(task1, "parent1").unwrap();

        // Second push with same fingerprint should fail
        assert!(matches!(
            frontier.push(task2, "parent2"),
            Err(FrontierError::DuplicateFingerprint(_))
        ));
    }

    #[test]
    fn test_scope_envelope() {
        let scope = vec!["src/".to_string(), "tests/".to_string()];
        let mut frontier = Frontier::new().with_scope_envelope(scope);

        let allowed_task = create_test_task("allowed", TaskType::Feature, vec!["src/main.rs".to_string()]);
        let blocked_task = create_test_task("blocked", TaskType::Feature, vec!["external/lib.rs".to_string()]);

        assert!(frontier.push(allowed_task, "parent1").is_ok());
        assert!(matches!(
            frontier.push(blocked_task, "parent2"),
            Err(FrontierError::ScopeViolation)
        ));
    }

    #[test]
    fn test_rate_limiting() {
        let config = FrontierConfig {
            max_per_parent_per_hour: 2,
            max_global_per_hour: 3,
            ..Default::default()
        };
        let mut frontier = Frontier::with_config(config);

        // Add tasks up to parent limit
        for i in 0..2 {
            let task = create_test_task(&format!("task{}", i), TaskType::Feature, vec![format!("file{}.rs", i)]);
            frontier.push(task, "parent1").unwrap();
        }

        // Third task from same parent should be rate limited
        let task3 = create_test_task("task3", TaskType::Feature, vec!["file3.rs".to_string()]);
        assert!(matches!(
            frontier.push(task3, "parent1"),
            Err(FrontierError::ParentRateLimitExceeded(_))
        ));
    }

    #[test]
    fn test_capacity_limits() {
        let config = FrontierConfig {
            max_size: 2,
            ..Default::default()
        };
        let mut frontier = Frontier::with_config(config);

        // Fill to capacity
        for i in 0..2 {
            let task = create_test_task(&format!("task{}", i), TaskType::Feature, vec![format!("file{}.rs", i)]);
            frontier.push(task, "parent1").unwrap();
        }

        // Third task should be rejected (eviction not implemented yet)
        let task3 = create_test_task("task3", TaskType::Feature, vec!["file3.rs".to_string()]);
        assert!(matches!(
            frontier.push(task3, "parent1"),
            Err(FrontierError::AtCapacity(2))
        ));
    }

    #[test]
    fn test_priority_ordering() {
        let mut frontier = Frontier::new();

        // Add tasks with different priorities
        let critical = create_test_task("critical", TaskType::CriticalFix, vec!["src/main.rs".to_string()]);
        let feature = create_test_task("feature", TaskType::Feature, vec!["src/utils.rs".to_string()]);
        let refactor = create_test_task("refactor", TaskType::Refactor, vec!["src/old.rs".to_string()]);

        frontier.push(critical, "parent1").unwrap();
        frontier.push(feature, "parent2").unwrap();
        frontier.push(refactor, "parent3").unwrap();

        // Should pop in priority order: CriticalFix > Feature > Refactor
        assert_eq!(frontier.pop().unwrap().description, "critical");
        assert_eq!(frontier.pop().unwrap().description, "feature");
        assert_eq!(frontier.pop().unwrap().description, "refactor");
    }
}
