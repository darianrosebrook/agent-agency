//! Simple demonstration of self-governing agent concepts
//! This shows the core iterative improvement loop working

use std::collections::HashMap;

#[derive(Debug, Clone)]
enum ArtifactType {
    Code,
    Text,
}

#[derive(Debug, Clone)]
struct Artifact {
    content: String,
    artifact_type: ArtifactType,
}

#[derive(Debug)]
struct Task {
    id: String,
    description: String,
    context: Vec<Artifact>,
}

#[derive(Debug)]
struct IterationResult {
    iteration: usize,
    quality_score: f64,
    improvement: String,
}

#[derive(Debug)]
struct SelfPromptingResult {
    success: bool,
    iterations: Vec<IterationResult>,
    final_quality_score: f64,
    final_code: String,
}

// Mock model that simulates improvements
struct MockModel;

impl MockModel {
    fn improve_code(&self, code: &str, iteration: usize) -> (String, String) {
        match iteration {
            1 => {
                // Fix syntax error
                let improved = code.replace("unclosed string);", "\"Hello, world!\"");
                (improved, "Fixed syntax error by properly closing string literal".to_string())
            },
            2 => {
                // Add documentation
                let improved = format!("/// Prints a greeting to the console\n{}", code);
                (improved, "Added documentation comment".to_string())
            },
            3 => {
                // Improve formatting
                let improved = code.replace("println!", "println! ");
                (improved, "Improved code formatting".to_string())
            },
            _ => (code.to_string(), "No further improvements needed".to_string())
        }
    }
}

// Simple quality evaluation
fn evaluate_quality(code: &str) -> f64 {
    let mut score = 0.0;

    // Basic quality heuristics
    if code.contains("///") { score += 0.3; }  // Has documentation
    if !code.contains("unclosed") { score += 0.3; }  // No syntax errors
    if code.contains("println!") { score += 0.2; }  // Has functionality
    if code.contains("fn main") { score += 0.2; }   // Has main function

    score.min(1.0)
}

struct SelfPromptingAgent {
    model: MockModel,
    max_iterations: usize,
    quality_threshold: f64,
}

impl SelfPromptingAgent {
    fn new(max_iterations: usize, quality_threshold: f64) -> Self {
        Self {
            model: MockModel,
            max_iterations,
            quality_threshold,
        }
    }

    fn execute_task(&self, task: Task) -> SelfPromptingResult {
        println!("🎯 Executing task: {}", task.description);
        println!("📋 Initial code quality: {:.2}", evaluate_quality(&task.context[0].content));

        let mut current_code = task.context[0].content.clone();
        let mut iterations = Vec::new();

        for iteration in 1..=self.max_iterations {
            println!("\n🔄 Iteration {}", iteration);

            let (improved_code, improvement) = self.model.improve_code(&current_code, iteration);
            let quality_score = evaluate_quality(&improved_code);

            println!("📈 Quality score: {:.2}", quality_score);
            println!("💡 Improvement: {}", improvement);

            iterations.push(IterationResult {
                iteration,
                quality_score,
                improvement,
            });

            current_code = improved_code;

            // Check satisficing condition
            if quality_score >= self.quality_threshold {
                println!("✅ Quality threshold reached!");
                break;
            }
        }

        let final_quality = evaluate_quality(&current_code);
        let success = final_quality >= 0.8;

        SelfPromptingResult {
            success,
            iterations,
            final_quality_score: final_quality,
            final_code: current_code,
        }
    }
}

fn main() {
    println!("🚀 Self-Governing Agent - Simple Demonstration");
    println!("==============================================");

    let agent = SelfPromptingAgent::new(5, 0.8);

    // Test case: Fix broken Rust code
    let broken_code = r#"fn main() {
    println!("unclosed string);
}"#;

    let task = Task {
        id: "demo-task".to_string(),
        description: "Fix syntax errors and improve code quality".to_string(),
        context: vec![Artifact {
            content: broken_code.to_string(),
            artifact_type: ArtifactType::Code,
        }],
    };

    let result = agent.execute_task(task);

    println!("\n🏁 Final Results:");
    println!("=================");
    println!("Success: {}", result.success);
    println!("Iterations: {}", result.iterations.len());
    println!("Final Quality Score: {:.2}", result.final_quality_score);
    println!("\n📝 Final Code:");
    println!("==============");
    println!("{}", result.final_code);

    println!("\n📊 Iteration Summary:");
    println!("====================");
    for iter in &result.iterations {
        println!("Iteration {}: {:.2} - {}",
                iter.iteration, iter.quality_score, iter.improvement);
    }

    println!("\n🎉 Demonstration shows:");
    println!("✅ Iterative improvement loop");
    println!("✅ Quality evaluation and scoring");
    println!("✅ Satisficing logic (early stopping)");
    println!("✅ Progressive code enhancement");
    println!("✅ Autonomous decision making");

    if result.success {
        println!("\n🎯 SUCCESS: Self-governing agent achieved quality threshold!");
    } else {
        println!("\n⚠️  PARTIAL: Agent made improvements but didn't reach threshold");
    }
}
