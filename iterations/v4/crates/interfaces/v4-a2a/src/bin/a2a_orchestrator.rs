//! A2A Orchestrator
//!
//! Discovers A2A workers, routes tasks by skill, and delegates work.
//! Proves the full orchestrator→worker vertical slice.
//!
//! ## Usage
//!
//! ```sh
//! # Single task (auto-routes to best skill)
//! cargo run --bin a2a_orchestrator -- "Write a haiku about Rust"
//!
//! # Force a specific skill
//! cargo run --bin a2a_orchestrator -- --skill draft-content "Write a blog post"
//!
//! # Pipeline: chain skills (output of step 1 → input of step 2)
//! cargo run --bin a2a_orchestrator -- --pipeline draft-content,review-text "Write a blog post"
//!
//! # Multiple workers
//! A2A_WORKERS=http://localhost:3001,http://localhost:3002 \
//!   cargo run --bin a2a_orchestrator -- "Convert JSON to YAML"
//! ```

use v4_a2a::{A2AClient, AgentSkill, CostMonitor, Part, Task};

// ============================================================================
// Arg parsing (no dependencies — just std::env)
// ============================================================================

struct Args {
    worker_urls: Vec<String>,
    prompt: String,
    skill: Option<String>,
    pipeline: Option<Vec<String>>,
}

fn parse_args() -> Result<Args, String> {
    let worker_urls = std::env::var("A2A_WORKERS")
        .unwrap_or_else(|_| "http://localhost:3001".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.is_empty() {
        return Err(
            "Usage: a2a_orchestrator [--skill <id>] [--pipeline <s1,s2,...>] \"<prompt>\""
                .to_string(),
        );
    }

    let mut skill = None;
    let mut pipeline = None;
    let mut prompt_parts: Vec<String> = Vec::new();
    let mut i = 0;

    while i < raw.len() {
        match raw[i].as_str() {
            "--skill" => {
                i += 1;
                skill = Some(
                    raw.get(i)
                        .ok_or("--skill requires a value")?
                        .clone(),
                );
            }
            "--pipeline" => {
                i += 1;
                let val = raw.get(i).ok_or("--pipeline requires a value")?;
                pipeline = Some(
                    val.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                );
            }
            other => prompt_parts.push(other.to_string()),
        }
        i += 1;
    }

    let prompt = prompt_parts.join(" ");
    if prompt.is_empty() {
        return Err("No prompt provided".to_string());
    }

    Ok(Args {
        worker_urls,
        prompt,
        skill,
        pipeline,
    })
}

// ============================================================================
// Worker discovery
// ============================================================================

struct Worker {
    url: String,
    client: A2AClient,
}

async fn discover_workers(urls: &[String]) -> Vec<Worker> {
    let mut workers = Vec::new();

    for url in urls {
        match A2AClient::discover(url).await {
            Ok(client) => {
                let card = client.card();
                let skill_ids: Vec<&str> = card.skills.iter().map(|s| s.id.as_str()).collect();
                eprintln!(
                    "  Found: {} ({}) — {} skills {:?}",
                    card.name,
                    url,
                    card.skills.len(),
                    skill_ids
                );
                workers.push(Worker {
                    url: url.clone(),
                    client,
                });
            }
            Err(e) => {
                eprintln!("  Failed: {} — {}", url, e);
            }
        }
    }

    workers
}

// ============================================================================
// Skill routing
// ============================================================================

struct RoutingDecision<'a> {
    worker: &'a Worker,
    skill: AgentSkill,
    score: usize,
}

fn route<'a>(prompt: &str, workers: &'a [Worker], force_skill: Option<&str>) -> Option<RoutingDecision<'a>> {
    // If a skill is forced, find the first worker that has it
    if let Some(skill_id) = force_skill {
        for worker in workers {
            if let Some(skill) = worker.client.card().skills.iter().find(|s| s.id == skill_id) {
                return Some(RoutingDecision {
                    worker,
                    skill: skill.clone(),
                    score: 100,
                });
            }
        }
        return None;
    }

    // Keyword-based scoring
    let keywords = tokenize(prompt);
    let mut best: Option<RoutingDecision<'a>> = None;

    for worker in workers {
        for skill in &worker.client.card().skills {
            let score = score_skill(skill, &keywords);
            if score > best.as_ref().map_or(0, |b| b.score) {
                best = Some(RoutingDecision {
                    worker,
                    skill: skill.clone(),
                    score,
                });
            }
        }
    }

    // Fall back to first worker's first skill if nothing matched
    if best.is_none() || best.as_ref().is_some_and(|b| b.score == 0) {
        if let Some(worker) = workers.first() {
            if let Some(skill) = worker.client.card().skills.first() {
                return Some(RoutingDecision {
                    worker,
                    skill: skill.clone(),
                    score: 0,
                });
            }
        }
    }

    best
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2) // skip short words
        .map(String::from)
        .collect()
}

fn score_skill(skill: &AgentSkill, keywords: &[String]) -> usize {
    let mut score = 0;

    // Skill ID tokens (high signal)
    let id_tokens = tokenize(&skill.id);
    for kw in keywords {
        if id_tokens.contains(kw) {
            score += 10;
        }
    }

    // Skill name tokens
    let name_tokens = tokenize(&skill.name);
    for kw in keywords {
        if name_tokens.contains(kw) {
            score += 5;
        }
    }

    // Description tokens
    let desc_tokens = tokenize(&skill.description);
    for kw in keywords {
        if desc_tokens.contains(kw) {
            score += 2;
        }
    }

    // Tags
    if let Some(tags) = &skill.tags {
        let tag_tokens: Vec<String> = tags.iter().flat_map(|t| tokenize(t)).collect();
        for kw in keywords {
            if tag_tokens.contains(kw) {
                score += 3;
            }
        }
    }

    // Examples
    if let Some(examples) = &skill.examples {
        let ex_tokens: Vec<String> = examples.iter().flat_map(|e| tokenize(e)).collect();
        for kw in keywords {
            if ex_tokens.contains(kw) {
                score += 1;
            }
        }
    }

    score
}

// ============================================================================
// Task execution
// ============================================================================

fn extract_text(task: &Task) -> String {
    task.artifacts
        .as_ref()
        .and_then(|arts| arts.first())
        .and_then(|art| art.parts.first())
        .and_then(|part| match part {
            Part::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_else(|| format!("[No text output — task state: {:?}]", task.status.state))
}

async fn execute_single(worker: &Worker, prompt: &str) -> Result<Task, String> {
    worker
        .client
        .send_message(prompt)
        .await
        .map_err(|e| format!("Task failed: {e}"))
}

async fn execute_pipeline(
    workers: &[Worker],
    initial_prompt: &str,
    skill_ids: &[String],
) -> Result<String, String> {
    let mut current_input = initial_prompt.to_string();
    let total_steps = skill_ids.len();

    for (i, skill_id) in skill_ids.iter().enumerate() {
        let step = i + 1;
        eprintln!("\n[Step {step}/{total_steps}] skill: {skill_id}");

        // Route to a worker with this skill
        let decision = route(&current_input, workers, Some(skill_id))
            .ok_or_else(|| format!("No worker has skill: {skill_id}"))?;

        eprintln!(
            "  Worker: {} ({})",
            decision.worker.client.card().name,
            decision.worker.url
        );

        // For steps after the first, wrap the input with context
        let prompt = if i == 0 {
            current_input.clone()
        } else {
            format!(
                "Here is content from a previous step. Please apply your \"{}\" skill:\n\n{}",
                skill_id, current_input
            )
        };

        let start = std::time::Instant::now();
        let task = execute_single(decision.worker, &prompt).await?;
        let elapsed = start.elapsed();

        let output = extract_text(&task);
        eprintln!("  Done in {:.1}s", elapsed.as_secs_f64());

        // Show intermediate output (truncated)
        let preview = if output.len() > 200 {
            format!("{}...", &output[..200])
        } else {
            output.clone()
        };
        eprintln!("  Preview: {preview}");

        current_input = output;
    }

    Ok(current_input)
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    // Cost tracking
    let cost_monitor = CostMonitor::new();

    // Discover workers
    eprintln!("[Orchestrator] Discovering workers...");
    let workers = discover_workers(&args.worker_urls).await;

    if workers.is_empty() {
        eprintln!("[Orchestrator] No workers found. Is an a2a_worker running?");
        std::process::exit(1);
    }

    eprintln!(
        "[Orchestrator] {} worker(s) ready\n",
        workers.len()
    );

    // Pipeline mode
    if let Some(skill_ids) = &args.pipeline {
        eprintln!(
            "[Orchestrator] Pipeline: {}",
            skill_ids.join(" → ")
        );
        eprintln!("[Orchestrator] Prompt: {}\n", args.prompt);

        match execute_pipeline(&workers, &args.prompt, skill_ids).await {
            Ok(output) => {
                // Estimate tokens: ~4 chars per token
                let input_tokens = (args.prompt.len() as u64) / 4;
                let output_tokens = (output.len() as u64) / 4;
                cost_monitor.record_usage("minimax", input_tokens, output_tokens);

                eprintln!("\n[Final Output]");
                println!("{output}");
            }
            Err(e) => {
                eprintln!("[Orchestrator] Pipeline failed: {e}");
                std::process::exit(1);
            }
        }
        print_cost_summary(&cost_monitor);
        return;
    }

    // Single-step mode
    let decision = match route(&args.prompt, &workers, args.skill.as_deref()) {
        Some(d) => d,
        None => {
            let skill_ref = args.skill.as_deref().unwrap_or("(auto)");
            eprintln!("[Orchestrator] No worker matched skill: {skill_ref}");
            std::process::exit(1);
        }
    };

    eprintln!(
        "[Orchestrator] Routing to: {} → skill \"{}\" (score: {})",
        decision.worker.client.card().name,
        decision.skill.id,
        decision.score
    );
    eprintln!("[Orchestrator] Prompt: {}\n", args.prompt);

    let start = std::time::Instant::now();
    match execute_single(decision.worker, &args.prompt).await {
        Ok(task) => {
            let elapsed = start.elapsed();
            let output = extract_text(&task);

            // Estimate tokens: ~4 chars per token
            // Estimate tokens: ~4 chars per token
            let input_tokens = (args.prompt.len() as u64) / 4;
            let output_tokens = (output.len() as u64) / 4;
            cost_monitor.record_usage("minimax", input_tokens, output_tokens);

            eprintln!(
                "[Orchestrator] Task {} completed in {:.1}s (state: {:?})\n",
                task.id,
                elapsed.as_secs_f64(),
                task.status.state
            );
            println!("{output}");
        }
        Err(e) => {
            eprintln!("[Orchestrator] {e}");
            std::process::exit(1);
        }
    }
    print_cost_summary(&cost_monitor);
}

fn print_cost_summary(monitor: &CostMonitor) {
    let summary = monitor.summary();
    if summary.providers.is_empty() {
        return;
    }
    eprintln!("\n[Cost Summary]");
    for p in &summary.providers {
        eprintln!(
            "  {}: {} in / {} out tokens, ~${:.6}",
            p.provider, p.input_tokens, p.output_tokens, p.estimated_cost_usd
        );
    }
    eprintln!(
        "  Total: {} in / {} out tokens, ~${:.6}",
        summary.total_input_tokens, summary.total_output_tokens, summary.total_cost_usd
    );
}
