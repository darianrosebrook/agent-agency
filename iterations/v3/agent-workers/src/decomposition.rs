//! Task Decomposition Engine
//!
//! Analyzes complex tasks and decomposes them into parallel subtasks
//! for efficient execution across multiple workers.

use crate::types::*;
use std::collections::HashMap;

/// Task decomposer that analyzes and breaks down complex tasks
pub struct TaskDecomposer {
    complexity_thresholds: HashMap<String, f64>,
}

impl TaskDecomposer {
    /// Create a new task decomposer
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("react-component".to_string(), 50.0);
        thresholds.insert("file-editing".to_string(), 30.0);
        thresholds.insert("research".to_string(), 40.0);
        thresholds.insert("code-generation".to_string(), 60.0);

        Self {
            complexity_thresholds: thresholds,
        }
    }

    /// Analyze a task to determine if it should be decomposed
    pub async fn analyze_task(&self, task: &TaskDefinition) -> Result<TaskAnalysis, DecompositionError> {
        let complexity_score = self.calculate_complexity(task);
        let patterns = self.identify_patterns(task).await?;

        Ok(TaskAnalysis {
            task_id: task.id,
            complexity_score,
            should_decompose: complexity_score > self.get_threshold_for_task(task),
            patterns,
            estimated_subtasks: self.estimate_subtasks(&patterns),
        })
    }

    /// Calculate task complexity score
    fn calculate_complexity(&self, task: &TaskDefinition) -> f64 {
        let mut score = 0.0;

        // Factor in required tools
        score += task.required_tools.len() as f64 * 10.0;

        // Factor in parameter complexity
        score += task.parameters.len() as f64 * 5.0;

        // Factor in task name/description complexity
        let text_complexity = (task.name.len() + task.description.len()) as f64 / 10.0;
        score += text_complexity;

        // Factor in priority (higher priority = more complex)
        score += match task.priority {
            TaskPriority::Low => 0.0,
            TaskPriority::Normal => 5.0,
            TaskPriority::High => 10.0,
            TaskPriority::Critical => 15.0,
        };

        score
    }

    /// Get complexity threshold for a task type
    fn get_threshold_for_task(&self, task: &TaskDefinition) -> f64 {
        // Determine task type from name/description
        if task.name.contains("react") || task.description.contains("component") {
            self.complexity_thresholds.get("react-component").copied().unwrap_or(50.0)
        } else if task.name.contains("file") || task.name.contains("edit") {
            self.complexity_thresholds.get("file-editing").copied().unwrap_or(30.0)
        } else if task.name.contains("research") || task.name.contains("search") {
            self.complexity_thresholds.get("research").copied().unwrap_or(40.0)
        } else {
            self.complexity_thresholds.get("code-generation").copied().unwrap_or(60.0)
        }
    }

    /// Identify decomposition patterns in a task
    async fn identify_patterns(&self, task: &TaskDefinition) -> Result<Vec<TaskPattern>, DecompositionError> {
        let mut patterns = Vec::new();

        // Analyze based on task type
        if task.name.contains("react") {
            patterns.extend(self.analyze_react_patterns(task).await?);
        } else if task.name.contains("file") || task.name.contains("edit") {
            patterns.extend(self.analyze_file_patterns(task).await?);
        } else if task.name.contains("research") {
            patterns.extend(self.analyze_research_patterns(task).await?);
        } else {
            // General decomposition
            patterns.push(TaskPattern {
                pattern_type: "general".to_string(),
                description: "General purpose task".to_string(),
                complexity: 1.0,
                required_tools: task.required_tools.clone(),
            });
        }

        Ok(patterns)
    }

    /// Analyze React component generation patterns
    async fn analyze_react_patterns(&self, task: &TaskDefinition) -> Result<Vec<TaskPattern>, DecompositionError> {
        let mut patterns = Vec::new();

        // Component logic pattern
        patterns.push(TaskPattern {
            pattern_type: "react-component".to_string(),
            description: "Generate React component with TypeScript".to_string(),
            complexity: 3.0,
            required_tools: vec!["react-generator".to_string()],
        });

        // SCSS module pattern
        patterns.push(TaskPattern {
            pattern_type: "scss-module".to_string(),
            description: "Generate SCSS module with styles".to_string(),
            complexity: 2.0,
            required_tools: vec!["react-generator".to_string()],
        });

        // Utils pattern
        patterns.push(TaskPattern {
            pattern_type: "component-utils".to_string(),
            description: "Generate component utility functions".to_string(),
            complexity: 1.5,
            required_tools: vec!["react-generator".to_string()],
        });

        Ok(patterns)
    }

    /// Analyze file editing patterns
    async fn analyze_file_patterns(&self, task: &TaskDefinition) -> Result<Vec<TaskPattern>, DecompositionError> {
        let mut patterns = Vec::new();

        patterns.push(TaskPattern {
            pattern_type: "file-editing".to_string(),
            description: "Edit file with context-aware changes".to_string(),
            complexity: 2.5,
            required_tools: vec!["file-editor".to_string()],
        });

        Ok(patterns)
    }

    /// Analyze research patterns
    async fn analyze_research_patterns(&self, task: &TaskDefinition) -> Result<Vec<TaskPattern>, DecompositionError> {
        let mut patterns = Vec::new();

        patterns.push(TaskPattern {
            pattern_type: "research".to_string(),
            description: "Gather and synthesize research information".to_string(),
            complexity: 3.5,
            required_tools: vec!["research-assistant".to_string()],
        });

        Ok(patterns)
    }

    /// Estimate number of subtasks for given patterns
    fn estimate_subtasks(&self, patterns: &[TaskPattern]) -> usize {
        patterns.len().max(1)
    }
}

/// Analysis result for a task
#[derive(Debug, Clone)]
pub struct TaskAnalysis {
    pub task_id: TaskId,
    pub complexity_score: f64,
    pub should_decompose: bool,
    pub patterns: Vec<TaskPattern>,
    pub estimated_subtasks: usize,
}

/// Pattern identified in a task
#[derive(Debug, Clone)]
pub struct TaskPattern {
    pub pattern_type: String,
    pub description: String,
    pub complexity: f64,
    pub required_tools: Vec<ToolId>,
}

/// Errors from task decomposition
#[derive(Debug, thiserror::Error)]
pub enum DecompositionError {
    #[error("Task analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Pattern identification failed: {0}")]
    PatternIdentificationFailed(String),

    #[error("Complexity calculation failed: {0}")]
    ComplexityCalculationFailed(String),
}
