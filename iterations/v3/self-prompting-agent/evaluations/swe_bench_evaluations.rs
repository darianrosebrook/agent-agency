//! SWE-Bench Style Evaluations for Agent Performance
//!
//! Measures coding task completion rates, tool optimality, and context utilization
//! Target metrics: 60%+ completion, 80%+ tool optimality, 70%+ context relevance

use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// SWE-Bench evaluation configuration
#[derive(Clone, Debug)]
pub struct SWEEvaluationConfig {
    pub max_tasks: usize,
    pub time_limit_per_task: std::time::Duration,
    pub target_completion_rate: f64,
    pub target_tool_optimality: f64,
    pub target_context_relevance: f64,
    pub include_complexity_levels: Vec<u8>, // 1-5 scale
    pub domains: Vec<String>, // "web", "api", "data", "security", etc.
}

/// Evaluation task definition
#[derive(Clone, Debug)]
pub struct SWETask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub domain: String,
    pub complexity: u8,
    pub expected_tools: Vec<String>, // Optimal tool sequence
    pub expected_context: Vec<String>, // Required context segments
    pub test_criteria: Vec<TestCriterion>,
    pub time_estimate_ms: u64,
}

/// Test criteria for task completion
#[derive(Clone, Debug)]
pub struct TestCriterion {
    pub name: String,
    pub description: String,
    pub test_function: Box<dyn Fn(&TaskResult) -> bool + Send + Sync>,
}

/// Evaluation results
#[derive(Clone, Debug)]
pub struct SWEResults {
    pub overall_score: f64,
    pub completion_rate: f64,
    pub tool_optimality_score: f64,
    pub context_relevance_score: f64,
    pub average_execution_time: f64,
    pub task_results: Vec<TaskEvaluation>,
    pub summary: EvaluationSummary,
}

/// Individual task evaluation
#[derive(Clone, Debug)]
pub struct TaskEvaluation {
    pub task_id: String,
    pub completed: bool,
    pub execution_time_ms: u64,
    pub tools_used: Vec<String>,
    pub context_used: Vec<String>,
    pub tool_optimality: f64, // 0.0 to 1.0
    pub context_relevance: f64, // 0.0 to 1.0
    pub test_passed: Vec<bool>,
    pub error_messages: Vec<String>,
}

/// Summary statistics
#[derive(Clone, Debug)]
pub struct EvaluationSummary {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub average_tool_optimality: f64,
    pub average_context_relevance: f64,
    pub performance_by_complexity: HashMap<u8, PerformanceMetrics>,
    pub performance_by_domain: HashMap<String, PerformanceMetrics>,
}

/// Performance metrics for groupings
#[derive(Clone, Debug)]
pub struct PerformanceMetrics {
    pub task_count: usize,
    pub completion_rate: f64,
    pub average_time: f64,
    pub average_optimality: f64,
}

/// SWE-Bench evaluator
pub struct SWEEvaluator {
    config: SWEEvaluationConfig,
    tasks: Vec<SWETask>,
}

impl SWEEvaluator {
    pub fn new(config: SWEEvaluationConfig) -> Self {
        Self {
            config,
            tasks: Self::load_swe_tasks(&config),
        }
    }

    /// Run full evaluation suite
    pub async fn run_evaluation(&self, agent: &dyn EnhancedAgent) -> Result<SWEResults> {
        let mut task_results = Vec::new();
        let mut total_execution_time = 0u64;

        for task in &self.tasks {
            if task_results.len() >= self.config.max_tasks {
                break;
            }

            let result = self.evaluate_single_task(agent, task).await?;
            total_execution_time += result.execution_time_ms;
            task_results.push(result);
        }

        let results = self.compute_results(task_results, total_execution_time);
        self.validate_targets(&results)?;

        Ok(results)
    }

    /// Evaluate a single task
    async fn evaluate_single_task(&self, agent: &dyn EnhancedAgent, task: &SWETask) -> Result<TaskEvaluation> {
        let start_time = std::time::Instant::now();

        // Execute task with timeout
        let execution_result = tokio::time::timeout(
            self.config.time_limit_per_task,
            agent.execute_task_enhanced(Task {
                id: task.id.clone(),
                description: task.description.clone(),
                task_type: task.domain.clone(),
                complexity: task.complexity,
                required_capabilities: vec![], // Will be inferred
                priority: 1,
                created_at: chrono::Utc::now(),
                deadline: None,
            })
        ).await;

        let execution_time = start_time.elapsed().as_millis() as u64;

        match execution_result {
            Ok(Ok(result)) => {
                // Task completed successfully
                let tool_optimality = self.compute_tool_optimality(&task.expected_tools, &result.tools_used);
                let context_relevance = self.compute_context_relevance(&task.expected_context, &result.context_used);
                let test_passed = self.run_test_criteria(task, &result);

                Ok(TaskEvaluation {
                    task_id: task.id.clone(),
                    completed: true,
                    execution_time_ms: execution_time,
                    tools_used: result.tools_used,
                    context_used: result.context_used,
                    tool_optimality,
                    context_relevance,
                    test_passed,
                    error_messages: vec![],
                })
            }
            Ok(Err(e)) => {
                // Task failed
                Ok(TaskEvaluation {
                    task_id: task.id.clone(),
                    completed: false,
                    execution_time_ms: execution_time,
                    tools_used: vec![],
                    context_used: vec![],
                    tool_optimality: 0.0,
                    context_relevance: 0.0,
                    test_passed: vec![false; task.test_criteria.len()],
                    error_messages: vec![e.to_string()],
                })
            }
            Err(_) => {
                // Timeout
                Ok(TaskEvaluation {
                    task_id: task.id.clone(),
                    completed: false,
                    execution_time_ms: execution_time,
                    tools_used: vec![],
                    context_used: vec![],
                    tool_optimality: 0.0,
                    context_relevance: 0.0,
                    test_passed: vec![false; task.test_criteria.len()],
                    error_messages: vec!["Task timed out".to_string()],
                })
            }
        }
    }

    /// Compute tool optimality score
    fn compute_tool_optimality(&self, expected: &[String], actual: &[String]) -> f64 {
        if expected.is_empty() {
            return if actual.is_empty() { 1.0 } else { 0.5 }; // Some tolerance for no expected tools
        }

        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Check if expected tools are used in reasonable order
        for (i, expected_tool) in expected.iter().enumerate() {
            total_weight += 1.0;

            if let Some(actual_pos) = actual.iter().position(|t| t == expected_tool) {
                // Tool is used - score based on order preservation
                let order_penalty = (actual_pos as f64 - i as f64).abs() / expected.len() as f64;
                score += (1.0 - order_penalty.min(0.5)).max(0.1); // Minimum 0.1 for using right tool
            }
            // Not using expected tool hurts score significantly
        }

        // Bonus for not using unnecessary tools
        let unnecessary_penalty = (actual.len().saturating_sub(expected.len())) as f64 * 0.1;
        score -= unnecessary_penalty.min(0.5);

        (score / total_weight).max(0.0).min(1.0)
    }

    /// Compute context relevance score
    fn compute_context_relevance(&self, expected: &[String], actual: &[String]) -> f64 {
        if expected.is_empty() || actual.is_empty() {
            return 0.5; // Neutral score for missing data
        }

        let mut relevant_found = 0;
        let mut total_searched = 0;

        for expected_segment in expected {
            total_searched += 1;
            // Check if any actual segment is relevant to expected
            let is_relevant = actual.iter().any(|actual_segment|
                self.compute_segment_relevance(expected_segment, actual_segment) > 0.7
            );

            if is_relevant {
                relevant_found += 1;
            }
        }

        let precision = relevant_found as f64 / actual.len() as f64;
        let recall = relevant_found as f64 / expected.len() as f64;

        // F1 score
        if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        }
    }

    /// Compute relevance between two text segments
    fn compute_segment_relevance(&self, expected: &str, actual: &str) -> f64 {
        // Simple word overlap similarity (could be enhanced with embeddings)
        let expected_words: std::collections::HashSet<_> = expected
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        let actual_words: std::collections::HashSet<_> = actual
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        let intersection: std::collections::HashSet<_> = expected_words
            .intersection(&actual_words)
            .collect();

        let union_len = expected_words.len() + actual_words.len() - intersection.len();

        if union_len == 0 {
            0.0
        } else {
            intersection.len() as f64 / union_len as f64
        }
    }

    /// Run test criteria against task result
    fn run_test_criteria(&self, task: &SWETask, result: &TaskResult) -> Vec<bool> {
        task.test_criteria.iter()
            .map(|criterion| (criterion.test_function)(result))
            .collect()
    }

    /// Compute overall evaluation results
    fn compute_results(&self, task_results: Vec<TaskEvaluation>, total_time: u64) -> SWEResults {
        let total_tasks = task_results.len();
        let completed_tasks = task_results.iter().filter(|r| r.completed).count();

        let completion_rate = completed_tasks as f64 / total_tasks as f64;

        let tool_optimality_score = task_results.iter()
            .map(|r| r.tool_optimality)
            .sum::<f64>() / total_tasks as f64;

        let context_relevance_score = task_results.iter()
            .map(|r| r.context_relevance)
            .sum::<f64>() / total_tasks as f64;

        let average_execution_time = total_time as f64 / total_tasks as f64;

        let overall_score = (completion_rate * 0.4) +
                           (tool_optimality_score * 0.3) +
                           (context_relevance_score * 0.3);

        let summary = self.compute_summary(&task_results);

        SWEResults {
            overall_score,
            completion_rate,
            tool_optimality_score,
            context_relevance_score,
            average_execution_time,
            task_results,
            summary,
        }
    }

    /// Compute evaluation summary with groupings
    fn compute_summary(&self, results: &[TaskEvaluation]) -> EvaluationSummary {
        let total_tasks = results.len();
        let completed_tasks = results.iter().filter(|r| r.completed).count();
        let failed_tasks = total_tasks - completed_tasks;

        let average_tool_optimality = results.iter()
            .map(|r| r.tool_optimality)
            .sum::<f64>() / total_tasks as f64;

        let average_context_relevance = results.iter()
            .map(|r| r.context_relevance)
            .sum::<f64>() / total_tasks as f64;

        // Group by complexity
        let mut by_complexity: HashMap<u8, Vec<&TaskEvaluation>> = HashMap::new();
        for result in results {
            by_complexity.entry(result.task_id.parse().unwrap_or(1)).or_default().push(result);
        }

        let performance_by_complexity: HashMap<u8, PerformanceMetrics> = by_complexity.into_iter()
            .map(|(complexity, tasks)| {
                let metrics = Self::compute_performance_metrics(&tasks);
                (complexity, metrics)
            })
            .collect();

        // Group by domain (would need domain info in TaskEvaluation)
        let performance_by_domain = HashMap::new(); // Simplified

        EvaluationSummary {
            total_tasks,
            completed_tasks,
            failed_tasks,
            average_tool_optimality,
            average_context_relevance,
            performance_by_complexity,
            performance_by_domain,
        }
    }

    /// Compute performance metrics for a group of tasks
    fn compute_performance_metrics(tasks: &[&TaskEvaluation]) -> PerformanceMetrics {
        let task_count = tasks.len();
        let completion_rate = tasks.iter().filter(|t| t.completed).count() as f64 / task_count as f64;
        let average_time = tasks.iter().map(|t| t.execution_time_ms).sum::<u64>() as f64 / task_count as f64;
        let average_optimality = tasks.iter().map(|t| t.tool_optimality).sum::<f64>() / task_count as f64;

        PerformanceMetrics {
            task_count,
            completion_rate,
            average_time,
            average_optimality,
        }
    }

    /// Validate that targets are met
    fn validate_targets(&self, results: &SWEResults) -> Result<()> {
        if results.completion_rate < self.config.target_completion_rate {
            return Err(anyhow::anyhow!(
                "Completion rate {:.2} below target {:.2}",
                results.completion_rate, self.config.target_completion_rate
            ));
        }

        if results.tool_optimality_score < self.config.target_tool_optimality {
            return Err(anyhow::anyhow!(
                "Tool optimality {:.2} below target {:.2}",
                results.tool_optimality_score, self.config.target_tool_optimality
            ));
        }

        if results.context_relevance_score < self.config.target_context_relevance {
            return Err(anyhow::anyhow!(
                "Context relevance {:.2} below target {:.2}",
                results.context_relevance_score, self.config.target_context_relevance
            ));
        }

        Ok(())
    }

    /// Load SWE tasks based on configuration
    fn load_swe_tasks(config: &SWEEvaluationConfig) -> Vec<SWETask> {
        // This would load from a dataset - here we create sample tasks
        vec![
            SWETask {
                id: "swe_001".to_string(),
                title: "Implement User Authentication".to_string(),
                description: "Implement secure user authentication with password hashing and JWT tokens".to_string(),
                domain: "security".to_string(),
                complexity: 3,
                expected_tools: vec![
                    "password_hasher".to_string(),
                    "jwt_generator".to_string(),
                    "input_validator".to_string(),
                ],
                expected_context: vec![
                    "authentication best practices".to_string(),
                    "JWT security considerations".to_string(),
                    "password hashing algorithms".to_string(),
                ],
                test_criteria: vec![
                    TestCriterion {
                        name: "password_hashing".to_string(),
                        description: "Uses secure password hashing".to_string(),
                        test_function: Box::new(|result| result.implementation.contains("bcrypt") || result.implementation.contains("argon2")),
                    },
                    TestCriterion {
                        name: "jwt_tokens".to_string(),
                        description: "Issues JWT tokens".to_string(),
                        test_function: Box::new(|result| result.implementation.contains("jwt") || result.implementation.contains("token")),
                    },
                ],
                time_estimate_ms: 300000, // 5 minutes
            },
            SWETask {
                id: "swe_002".to_string(),
                title: "Build REST API Endpoint".to_string(),
                description: "Create a REST API endpoint for user management with proper error handling".to_string(),
                domain: "api".to_string(),
                complexity: 2,
                expected_tools: vec![
                    "api_framework".to_string(),
                    "validator".to_string(),
                    "error_handler".to_string(),
                ],
                expected_context: vec![
                    "REST API design".to_string(),
                    "HTTP status codes".to_string(),
                    "input validation".to_string(),
                ],
                test_criteria: vec![
                    TestCriterion {
                        name: "rest_compliant".to_string(),
                        description: "Follows REST principles".to_string(),
                        test_function: Box::new(|result| {
                            result.implementation.contains("GET") ||
                            result.implementation.contains("POST") ||
                            result.implementation.contains("PUT") ||
                            result.implementation.contains("DELETE")
                        }),
                    },
                    TestCriterion {
                        name: "error_handling".to_string(),
                        description: "Handles errors properly".to_string(),
                        test_function: Box::new(|result| result.implementation.contains("try") || result.implementation.contains("catch")),
                    },
                ],
                time_estimate_ms: 180000, // 3 minutes
            },
            // Add more tasks as needed...
        ].into_iter()
         .filter(|task| config.include_complexity_levels.contains(&task.complexity))
         .filter(|task| config.domains.contains(&task.domain))
         .take(config.max_tasks)
         .collect()
    }
}

/// Enhanced agent trait for evaluation
#[async_trait::async_trait]
pub trait EnhancedAgent: Send + Sync {
    async fn execute_task_enhanced(&self, task: Task) -> Result<TaskResult>;
}

/// Task execution result for evaluation
#[derive(Clone, Debug)]
pub struct TaskResult {
    pub tools_used: Vec<String>,
    pub context_used: Vec<String>,
    pub implementation: String,
    pub success: bool,
    pub execution_time_ms: u64,
}

/// Task definition (simplified)
#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub task_type: String,
    pub complexity: u8,
    pub required_capabilities: Vec<String>,
    pub priority: u8,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Run SWE-Bench evaluation
pub async fn run_swe_bench_evaluation(agent: &dyn EnhancedAgent, config: SWEEvaluationConfig) -> Result<SWEResults> {
    let evaluator = SWEEvaluator::new(config);
    evaluator.run_evaluation(agent).await
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEnhancedAgent;

    #[async_trait::async_trait]
    impl EnhancedAgent for MockEnhancedAgent {
        async fn execute_task_enhanced(&self, task: Task) -> Result<TaskResult> {
            // Mock successful execution
            Ok(TaskResult {
                tools_used: vec!["password_hasher".to_string(), "jwt_generator".to_string()],
                context_used: vec!["authentication best practices".to_string()],
                implementation: "bcrypt + JWT implementation".to_string(),
                success: true,
                execution_time_ms: 150000,
            })
        }
    }

    #[tokio::test]
    async fn test_swe_bench_evaluation() {
        let config = SWEEvaluationConfig {
            max_tasks: 2,
            time_limit_per_task: std::time::Duration::from_secs(30),
            target_completion_rate: 0.8,
            target_tool_optimality: 0.7,
            target_context_relevance: 0.6,
            include_complexity_levels: vec![2, 3],
            domains: vec!["security".to_string(), "api".to_string()],
        };

        let agent = MockEnhancedAgent;
        let results = run_swe_bench_evaluation(&agent, config).await.unwrap();

        // Verify basic structure
        assert!(results.task_results.len() <= 2, "Should respect max_tasks");
        assert!(results.completion_rate >= 0.0 && results.completion_rate <= 1.0, "Completion rate should be valid");
        assert!(results.overall_score >= 0.0 && results.overall_score <= 1.0, "Overall score should be valid");
    }

    #[test]
    fn test_tool_optimality_computation() {
        let evaluator = SWEEvaluator::new(SWEEvaluationConfig {
            max_tasks: 1,
            time_limit_per_task: std::time::Duration::from_secs(30),
            target_completion_rate: 0.8,
            target_tool_optimality: 0.7,
            target_context_relevance: 0.6,
            include_complexity_levels: vec![1, 2, 3, 4, 5],
            domains: vec!["test".to_string()],
        });

        // Perfect match
        let score1 = evaluator.compute_tool_optimality(
            &["tool1".to_string(), "tool2".to_string()],
            &["tool1".to_string(), "tool2".to_string()]
        );
        assert!((score1 - 1.0).abs() < 0.1, "Perfect match should score ~1.0");

        // Wrong order
        let score2 = evaluator.compute_tool_optimality(
            &["tool1".to_string(), "tool2".to_string()],
            &["tool2".to_string(), "tool1".to_string()]
        );
        assert!(score2 < 1.0 && score2 > 0.0, "Wrong order should reduce score");

        // Missing tool
        let score3 = evaluator.compute_tool_optimality(
            &["tool1".to_string(), "tool2".to_string()],
            &["tool1".to_string()]
        );
        assert!(score3 < 1.0, "Missing tool should reduce score");
    }

    #[test]
    fn test_context_relevance_computation() {
        let evaluator = SWEEvaluator::new(SWEEvaluationConfig {
            max_tasks: 1,
            time_limit_per_task: std::time::Duration::from_secs(30),
            target_completion_rate: 0.8,
            target_tool_optimality: 0.7,
            target_context_relevance: 0.6,
            include_complexity_levels: vec![1, 2, 3, 4, 5],
            domains: vec!["test".to_string()],
        });

        // Perfect match
        let score1 = evaluator.compute_context_relevance(
            &["authentication security".to_string()],
            &["authentication security best practices".to_string()]
        );
        assert!(score1 > 0.5, "Relevant context should score well");

        // Irrelevant context
        let score2 = evaluator.compute_context_relevance(
            &["authentication security".to_string()],
            &["database optimization".to_string()]
        );
        assert!(score2 < 0.5, "Irrelevant context should score poorly");
    }
}
