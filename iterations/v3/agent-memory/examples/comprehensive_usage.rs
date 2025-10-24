//! Comprehensive example of the Agent Memory System
//!
//! This example demonstrates:
//! - Setting up the memory system
//! - Storing agent experiences
//! - Retrieving contextual memories
//! - Performing multi-hop reasoning
//! - Analyzing temporal patterns
//! - Managing memory decay and importance

use agent_memory::*;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Agent Memory System - Comprehensive Example");
    println!("==============================================\n");

    // 1. Initialize the memory system
    println!("1. Initializing Memory System...");
    let memory_config = MemoryConfig::default();
    let memory_system = Arc::new(MemorySystem::init(memory_config).await?);
    println!("✅ Memory system initialized\n");

    // 2. Create some sample agent experiences
    println!("2. Storing Agent Experiences...");

    let agent_id = "agent-001";
    let task_id_1 = "task-001";
    let task_id_2 = "task-002";

    // Experience 1: Successful code review task
    let experience_1 = AgentExperience {
        id: MemoryId::new_v4(),
        agent_id: agent_id.to_string(),
        task_id: task_id_1.to_string(),
        context: TaskContext {
            task_id: task_id_1.to_string(),
            task_type: "code_review".to_string(),
            description: "Review pull request for authentication middleware implementation".to_string(),
            domain: vec!["security".to_string(), "authentication".to_string()],
            entities: vec!["middleware".to_string(), "JWT".to_string()],
            temporal_context: Some(TemporalContext {
                start_time: Utc::now() - Duration::hours(2),
                deadline: Some(Utc::now() - Duration::hours(1)),
                priority: TaskPriority::High,
                recurrence_pattern: None,
            }),
            metadata: HashMap::new(),
        },
        input: serde_json::json!({
            "files": ["auth_middleware.rs", "tests.rs"],
            "lines_of_code": 150
        }),
        output: serde_json::json!({
            "issues_found": 2,
            "suggestions": ["Add input validation", "Improve error handling"],
            "approved": true
        }),
        outcome: ExperienceOutcome {
            success: true,
            performance_score: Some(0.9),
            learned_capabilities: vec!["security_review".to_string(), "middleware_patterns".to_string()],
            failure_reasons: vec![],
            success_factors: vec!["thorough_analysis".to_string(), "clear_feedback".to_string()],
            execution_time_ms: Some(4500),
            tokens_used: Some(1200),
            feedback: Some(AgentFeedback {
                quality_score: Some(0.95),
                relevance_score: Some(0.9),
                accuracy_score: Some(0.95),
                comments: vec!["Excellent attention to security details".to_string()],
                evaluator_id: Some("senior-dev-001".to_string()),
            }),
        },
        memory_type: MemoryType::Episodic,
        timestamp: Utc::now() - Duration::hours(2),
        metadata: HashMap::new(),
    };

    // Experience 2: Database optimization task
    let experience_2 = AgentExperience {
        id: MemoryId::new_v4(),
        agent_id: agent_id.to_string(),
        task_id: task_id_2.to_string(),
        context: TaskContext {
            task_id: task_id_2.to_string(),
            task_type: "database_optimization".to_string(),
            description: "Optimize slow database queries in user management system".to_string(),
            domain: vec!["database".to_string(), "performance".to_string()],
            entities: vec!["PostgreSQL".to_string(), "indexes".to_string()],
            temporal_context: Some(TemporalContext {
                start_time: Utc::now() - Duration::hours(1),
                deadline: Some(Utc::now() - Duration::minutes(30)),
                priority: TaskPriority::Medium,
                recurrence_pattern: None,
            }),
            metadata: HashMap::new(),
        },
        input: serde_json::json!({
            "query_count": 15,
            "slowest_query_time": "2.3s"
        }),
        output: serde_json::json!({
            "indexes_added": 3,
            "queries_optimized": 12,
            "performance_improvement": "75%"
        }),
        outcome: ExperienceOutcome {
            success: true,
            performance_score: Some(0.85),
            learned_capabilities: vec!["query_optimization".to_string(), "index_strategy".to_string()],
            failure_reasons: vec![],
            success_factors: vec!["systematic_analysis".to_string(), "effective_indexing".to_string()],
            execution_time_ms: Some(3200),
            tokens_used: Some(800),
            feedback: Some(AgentFeedback {
                quality_score: Some(0.88),
                relevance_score: Some(0.85),
                accuracy_score: Some(0.9),
                comments: vec!["Good optimization strategy".to_string()],
                evaluator_id: Some("dba-001".to_string()),
            }),
        },
        memory_type: MemoryType::Episodic,
        timestamp: Utc::now() - Duration::hours(1),
        metadata: HashMap::new(),
    };

    // Store experiences
    let memory_id_1 = memory_system.store_experience(experience_1.clone()).await?;
    let memory_id_2 = memory_system.store_experience(experience_2.clone()).await?;

    println!("✅ Stored 2 agent experiences");
    println!("   - Memory 1: {} ({})", memory_id_1, experience_1.context.task_type);
    println!("   - Memory 2: {} ({})\n", memory_id_2, experience_2.context.task_type);

    // 3. Retrieve contextual memories
    println!("3. Retrieving Contextual Memories...");

    let current_context = TaskContext {
        task_id: "task-003".to_string(),
        task_type: "security_audit".to_string(),
        description: "Perform security audit on authentication system with JWT tokens".to_string(),
        domain: vec!["security".to_string(), "authentication".to_string()],
        entities: vec!["JWT".to_string(), "authentication".to_string()],
        temporal_context: Some(TemporalContext {
            start_time: Utc::now(),
            deadline: Some(Utc::now() + Duration::hours(4)),
            priority: TaskPriority::High,
            recurrence_pattern: None,
        }),
        metadata: HashMap::new(),
    };

    let contextual_memories = memory_system.retrieve_contextual_memories(&current_context, 5).await?;

    println!("✅ Found {} contextual memories:", contextual_memories.len());
    for (i, memory) in contextual_memories.iter().enumerate() {
        println!("   {}. {} - Relevance: {:.3}, Match: {:?}",
                i + 1,
                memory.memory.context.description.chars().take(50).collect::<String>(),
                memory.relevance_score,
                memory.context_match);
    }
    println!();

    // 4. Perform multi-hop reasoning
    println!("4. Performing Multi-Hop Reasoning...");

    let reasoning_query = ReasoningQuery {
        start_entities: vec!["agent:agent-001".to_string()],
        target_entities: vec!["capability:security_review".to_string()],
        relationship_types: vec![RelationshipType::LearnsFrom, RelationshipType::Performs],
        max_hops: 2,
        min_confidence: 0.5,
        time_range: Some(TimeRange {
            start: Utc::now() - Duration::days(7),
            end: Utc::now(),
        }),
    };

    let reasoning_result = memory_system.perform_reasoning(reasoning_query).await?;

    println!("✅ Reasoning analysis complete:");
    println!("   - Paths found: {}", reasoning_result.paths.len());
    println!("   - Confidence: {:.3}", reasoning_result.confidence_score);
    println!("   - Entities discovered: {}", reasoning_result.entities_discovered.len());

    for (i, path) in reasoning_result.paths.iter().take(2).enumerate() {
        println!("   Path {}: {} -> {} (confidence: {:.3}, hops: {})",
                i + 1,
                path.entities.first().unwrap_or(&"unknown".to_string()),
                path.entities.last().unwrap_or(&"unknown".to_string()),
                path.confidence,
                path.hops);
    }
    println!();

    // 5. Analyze temporal patterns
    println!("5. Analyzing Temporal Patterns...");

    let time_range = TimeRange {
        start: Utc::now() - Duration::days(7),
        end: Utc::now(),
    };

    let temporal_analysis = memory_system.analyze_temporal_patterns(agent_id, &time_range).await?;

    println!("✅ Temporal analysis for agent {}:", agent_id);
    println!("   - Time range: {} to {}", time_range.start.format("%Y-%m-%d"), time_range.end.format("%Y-%m-%d"));
    println!("   - Performance trends: {}", temporal_analysis.trends.len());
    println!("   - Change points detected: {}", temporal_analysis.change_points.len());
    println!("   - Causality links: {}", temporal_analysis.causality_links.len());

    println!("   Performance Summary:");
    println!("     - Average score: {:.3}", temporal_analysis.performance_summary.average_score);
    println!("     - Best score: {:.3}", temporal_analysis.performance_summary.best_score);
    println!("     - Improvement rate: {:.3}", temporal_analysis.performance_summary.improvement_rate);
    println!("     - Consistency: {:.3}", temporal_analysis.performance_summary.consistency_score);
    println!();

    // 6. Run memory maintenance
    println!("6. Running Memory Maintenance...");

    let maintenance_result = memory_system.run_maintenance().await?;

    println!("✅ Memory maintenance completed:");
    println!("   - Memories decayed: {}", maintenance_result.decayed_memories);
    println!("   - Memories consolidated: {}", maintenance_result.consolidated_memories);
    println!("   - Memories cleaned up: {}\n", maintenance_result.cleaned_memories);

    // 7. Get memory system statistics
    println!("7. Memory System Statistics...");

    let memory_stats = memory_system.manager().get_memory_stats().await?;
    let decay_stats = memory_system.decay_engine().get_decay_stats().await?;
    let embedding_stats = memory_system.embedding_integration().get_embedding_stats().await?;
    let graph_stats = memory_system.graph_engine().get_graph_stats().await?;

    println!("📊 Memory System Health:");
    println!("   Total Memories: {}", memory_stats.total_memories);
    println!("   Unique Agents: {}", memory_stats.unique_agents);
    println!("   Knowledge Graph: {} entities, {} relationships", graph_stats.entity_count, graph_stats.relationship_count);
    println!("   Embeddings: {} stored", embedding_stats.total_embeddings);
    println!("   Average Importance: {:.3}", decay_stats.avg_importance);
    println!("   Average Decay: {:.3}", decay_stats.avg_decay);
    println!("   Heavily Decayed: {}", decay_stats.heavily_decayed);
    println!("   Highly Important: {}\n", decay_stats.highly_important);

    // 8. Demonstrate memory importance boosting
    println!("8. Demonstrating Memory Importance Management...");

    // Boost the importance of the security-related memory
    memory_system.decay_engine().boost_memory_importance(memory_id_1, 1.5).await?;
    println!("✅ Boosted importance of security review memory");

    // Protect highly important memories from decay
    let protected = memory_system.decay_engine().protect_important_memories(1.2).await?;
    println!("✅ Protected {} important memories from decay\n", protected);

    println!("🎉 Agent Memory System demonstration complete!");
    println!("===============================================");
    println!("The memory system now contains:");
    println!("- Episodic memories of agent experiences");
    println!("- Semantic knowledge graph of entities and relationships");
    println!("- Vector embeddings for semantic similarity search");
    println!("- Temporal analysis capabilities");
    println!("- Automatic decay and importance management");
    println!("- Multi-hop reasoning for complex queries");
    println!();
    println!("This forms the foundation for truly intelligent, learning agents");
    println!("that can build upon their experiences and share knowledge effectively.");

    Ok(())
}
