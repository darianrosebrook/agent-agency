//! Demonstration of the Self-Governing Agent System
//!
//! This script shows the core functionality of the self-prompting agent
//! working with mock providers to demonstrate the architecture.

use std::collections::HashMap;
use std::sync::Arc;
use tokio;

#[derive(Debug, Clone)]
pub enum ArtifactType {
    Code,
    Text,
    Test,
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub content: String,
    pub artifact_type: ArtifactType,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub context: Vec<Artifact>,
}

#[derive(Debug)]
pub struct SelfPromptingResult {
    pub success: bool,
    pub iterations: Vec<IterationResult>,
    pub final_quality_score: f64,
    pub final_artifacts: Vec<Artifact>,
}

#[derive(Debug)]
pub struct IterationResult {
    pub iteration: usize,
    pub quality_score: f64,
    pub prompt: String,
    pub response: String,
}

// Mock model provider for demonstration
#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
    async fn infer(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct MockOllamaProvider;

#[async_trait::async_trait]
impl ModelProvider for MockOllamaProvider {
    async fn infer(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        println!(" Mock Ollama processing: {}", prompt.chars().take(50).collect::<String>());

        // Simulate different responses based on prompt content
        if prompt.contains("syntax error") {
            Ok(r#"Here is the fixed code:

```rust
fn main() {
    println!("Hello, world!");
}
```

The syntax error was a missing quote in the string literal. I fixed it by properly closing the string with a quote."#.to_string())
        } else if prompt.contains("documentation") {
            Ok(r#"Here is the code with added documentation:

```rust
/// Calculates the sum of all elements in the provided vector.
///
/// # Arguments
/// * `items` - A vector of i32 values to sum
///
/// # Returns
/// The sum of all elements as an i32
///
/// # Examples
/// ```
/// let result = calculate_total(vec![1, 2, 3]);
/// assert_eq!(result, 6);
/// ```
fn calculate_total(items: Vec<i32>) -> i32 {
    items.iter().sum()
}
```

I added comprehensive documentation including parameter descriptions, return value information, and usage examples."#.to_string())
        } else {
            Ok("I understand your request. Let me provide a helpful response to improve the code.".to_string())
        }
    }
}

// Simple evaluation functions
pub fn evaluate_code_quality(code: &str) -> f64 {
    let mut score = 0.5;

    // Basic heuristics for demonstration
    if code.contains("fn ") { score += 0.1; }
    if code.contains("///") || code.contains("//") { score += 0.1; }
    if code.contains("println!") { score += 0.1; }
    if !code.contains("unclosed") { score += 0.2; }

    score.min(1.0)
}

pub fn extract_code_from_response(response: &str) -> Option<String> {
    // Simple extraction of code blocks from markdown
    if let Some(start) = response.find("```rust") {
        if let Some(end) = response[start + 7..].find("```") {
            return Some(response[start + 7..start + 7 + end].trim().to_string());
        }
    }
    if let Some(start) = response.find("```") {
        if let Some(end) = response[start + 3..].find("```") {
            return Some(response[start + 3..start + 3 + end].trim().to_string());
        }
    }
    None
}

// Core self-prompting agent logic
pub struct SelfPromptingAgent {
    model_provider: Arc<dyn ModelProvider>,
    max_iterations: usize,
}

impl SelfPromptingAgent {
    pub fn new(model_provider: Arc<dyn ModelProvider>, max_iterations: usize) -> Self {
        Self {
            model_provider,
            max_iterations,
        }
    }

    pub async fn execute_task(&self, task: Task) -> Result<SelfPromptingResult, Box<dyn std::error::Error>> {
        println!(" Starting self-prompting execution for task: {}", task.description);
        println!(" Context artifacts: {}", task.context.len());

        let mut iterations = Vec::new();
        let mut current_artifacts = task.context.clone();
        let mut best_quality = 0.0;
        let mut best_artifacts = current_artifacts.clone();

        for iteration in 1..=self.max_iterations {
            println!("\n Iteration {}", iteration);

            // Create prompt for this iteration
            let prompt = self.create_iteration_prompt(&task, &current_artifacts, iteration);

            // Get model response
            let response = self.model_provider.infer(&prompt).await?;
            println!(" Model response length: {} chars", response.len());

            // Extract code from response
            if let Some(improved_code) = extract_code_from_response(&response) {
                println!(" Extracted improved code");

                // Evaluate quality
                let quality_score = evaluate_code_quality(&improved_code);
                println!(" Quality score: {:.2}", quality_score);

                // Update artifacts
                let improved_artifact = Artifact {
                    content: improved_code,
                    artifact_type: ArtifactType::Code,
                };

                current_artifacts = vec![improved_artifact];

                // Track best result
                if quality_score > best_quality {
                    best_quality = quality_score;
                    best_artifacts = current_artifacts.clone();
                }

                iterations.push(IterationResult {
                    iteration,
                    quality_score,
                    prompt: prompt.clone(),
                    response,
                });

                // Check if we should stop (simple satisficing)
                if quality_score >= 0.8 {
                    println!(" Quality threshold reached, stopping early");
                    break;
                }
            } else {
                println!("⚠️  No code extracted from response, continuing...");
                iterations.push(IterationResult {
                    iteration,
                    quality_score: 0.0,
                    prompt,
                    response,
                });
            }
        }

        let success = best_quality >= 0.6; // Minimum success threshold

        Ok(SelfPromptingResult {
            success,
            iterations,
            final_quality_score: best_quality,
            final_artifacts: best_artifacts,
        })
    }

    fn create_iteration_prompt(&self, task: &Task, artifacts: &[Artifact], iteration: usize) -> String {
        let mut prompt = format!("Task: {}\n\n", task.description);

        if iteration == 1 {
            prompt.push_str("Please help me with this coding task. Here is the current code:\n\n");
        } else {
            prompt.push_str("I've made some improvements. Here is the current version:\n\n");
        }

        for artifact in artifacts {
            prompt.push_str(&format!("```\n{}\n```\n\n", artifact.content));
        }

        if iteration > 1 {
            prompt.push_str("Please improve this code further. Focus on:\n");
            prompt.push_str("- Fixing any remaining issues\n");
            prompt.push_str("- Adding documentation\n");
            prompt.push_str("- Improving code quality\n");
            prompt.push_str("- Making it more readable and maintainable\n");
        }

        prompt.push_str("\nProvide the improved code in a code block.");
        prompt
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Self-Governing Agent System Demonstration");
    println!("=============================================");

    // Create mock model provider
    let model_provider = Arc::new(MockOllamaProvider);

    // Create self-prompting agent
    let agent = SelfPromptingAgent::new(model_provider, 3);

    // Test case 1: Fix syntax error
    println!("\n Test Case 1: Fix Syntax Error");
    println!("--------------------------------");

    let broken_code = r#"fn main() {
    println!("unclosed string);
}"#;

    let task1 = Task {
        id: "test-1".to_string(),
        description: "Fix the syntax error in this Rust function".to_string(),
        context: vec![Artifact {
            content: broken_code.to_string(),
            artifact_type: ArtifactType::Code,
        }],
    };

    let result1 = agent.execute_task(task1).await?;
    println!("\n Final Result:");
    println!("- Success: {}", result1.success);
    println!("- Iterations: {}", result1.iterations.len());
    println!("- Final Quality: {:.2}", result1.final_quality_score);

    if let Some(final_artifact) = result1.final_artifacts.first() {
        println!("- Final Code:");
        println!("{}", final_artifact.content);
    }

    // Test case 2: Add documentation
    println!("\n Test Case 2: Add Documentation");
    println!("---------------------------------");

    let undocumented_code = r#"fn calculate_total(items: Vec<i32>) -> i32 {
    items.iter().sum()
}"#;

    let task2 = Task {
        id: "test-2".to_string(),
        description: "Add comprehensive documentation to this Rust function".to_string(),
        context: vec![Artifact {
            content: undocumented_code.to_string(),
            artifact_type: ArtifactType::Code,
        }],
    };

    let result2 = agent.execute_task(task2).await?;
    println!("\n Final Result:");
    println!("- Success: {}", result2.success);
    println!("- Iterations: {}", result2.iterations.len());
    println!("- Final Quality: {:.2}", result2.final_quality_score);

    if let Some(final_artifact) = result2.final_artifacts.first() {
        println!("- Final Code:");
        println!("{}", final_artifact.content);
    }

    println!("\n Demonstration Complete!");
    println!("==========================");
    println!("The self-governing agent successfully demonstrated:");
    println!(" Autonomous task execution through iterative improvement");
    println!(" Quality evaluation and satisficing logic");
    println!(" Code extraction and refinement");
    println!(" Early stopping when quality thresholds are met");

    Ok(())
}
