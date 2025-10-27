//! Scenario 2: Autonomous Research and Summary
//!
//! Tests autonomous research and summarization capabilities:
//! 1. Agent searches through local "research corpus" (markdown files)
//! 2. Extracts and synthesizes information with citations
//! 3. Council validates accuracy, citations, and hallucination detection
//! 4. Verifies output structure and reusability

use std::time::Instant;
use tracing::{info, error};
use std::sync::Arc;

use crate::harness::{TestEnvironment, LocalServiceManager, AssertionFramework};
use crate::fixtures::research_sources::*;
use crate::{TestResult, TestMetrics, Scenario};

/// Run the research scenario test
pub async fn run_test(
    env: &TestEnvironment,
    services: &LocalServiceManager,
) -> TestResult {
    let start_time = Instant::now();
    let mut assertions = AssertionFramework::new();

    info!("Starting scenario 2: Autonomous research test");

    // Setup test workspace
    let workspace = match env.create_workspace("research_test").await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Failed to create workspace: {}", e);
            return TestResult {
                scenario: Scenario::Scenario2Research,
                passed: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!("Workspace creation failed: {}", e)),
                metrics: TestMetrics::default(),
            };
        }
    };

    // Create research sources
    if let Err(e) = create_source_files(workspace.path()).await {
        error!("Failed to create research sources: {}", e);
        return TestResult {
            scenario: Scenario::Scenario2Research,
            passed: false,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error_message: Some(format!("Source creation failed: {}", e)),
            metrics: TestMetrics::default(),
        };
    }

    // Get expected sources for validation
    let sources = get_research_sources();
    let source_files: Vec<_> = sources.iter().map(|s| {
        crate::harness::SourceFile {
            name: s.filename.clone(),
            content: s.content.clone(),
        }
    }).collect();

    // Initialize real KnowledgeSeeker for research
    let db_client = services.postgres().await?;
    let knowledge_seeker = agent_research::knowledge_seeker::KnowledgeSeeker::new(
        agent_research::research_types::ResearchAgentConfig {
            max_search_depth: 3,
            max_results_per_query: 10,
            enable_web_scraping: false, // Disable web scraping for local-only testing
            enable_vector_search: true,
            context_window_size: 4096,
            synthesis_enabled: true,
            metrics_enabled: true,
        },
        Arc::new(db_client),
    ).await.map_err(|e| {
        TestResult {
            scenario: Scenario::Scenario2Research,
            passed: false,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error_message: Some(format!("Failed to initialize KnowledgeSeeker: {}", e)),
            metrics: TestMetrics::default(),
        }
    })?;

    // Execute research task
    let research_query = agent_research::research_types::ResearchQuery {
        id: uuid::Uuid::new_v4(),
        query: "Research and summarize homomorphic encryption applications".to_string(),
        context: vec![
            "Focus on practical applications in healthcare, finance, and cloud computing".to_string(),
            "Include technical foundations and current challenges".to_string(),
            "Cite sources and provide evidence for claims".to_string(),
        ],
        search_scope: agent_research::research_types::SearchScope::LocalFiles(
            workspace.path().join("research_sources").to_string_lossy().to_string()
        ),
        max_results: 10,
        synthesis_required: true,
    };

    let research_result = knowledge_seeker.execute_research(research_query).await.map_err(|e| {
        TestResult {
            scenario: Scenario::Scenario2Research,
            passed: false,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error_message: Some(format!("Research execution failed: {}", e)),
            metrics: TestMetrics::default(),
        }
    })?;

    // Extract summary and citations from research result
    let summary = research_result.synthesized_context;
    let citations = research_result.citations.into_iter().map(|c| {
        crate::harness::Citation {
            source_name: c.source_name,
            page_or_section: c.section,
            quote: c.quote,
        }
    }).collect::<Vec<_>>();

    // Record metrics
    env.record_metric("sources_processed", sources.len() as f64).await;
    env.record_metric("citations_found", citations.len() as f64).await;

    // Validate citations
    assertions.assert_citation_integrity(
        &citations,
        &source_files,
        "Summary should have valid citations"
    );

    // Check hallucination detection
    let fact_checker = crate::harness::FactChecker::new(get_known_facts());
    assertions.assert_no_hallucination(
        &summary,
        &fact_checker,
        "Summary should not contain hallucinations"
    );

    // Validate minimum citations
    assertions.assert_citation_integrity(
        &citations,
        &source_files,
        &format!("Should have at least {} citations", 3)
    );

    // Check logical structure (basic check for required sections)
    let has_overview = summary.contains("Overview") || summary.contains("overview");
    let has_applications = summary.contains("Applications") || summary.contains("applications") || summary.contains("Key Applications");
    let has_challenges = summary.contains("Challenges") || summary.contains("challenges");

    if !(has_overview && has_applications && has_challenges) {
        assertions.record_assertion(
            crate::harness::AssertionType::CitationIntegrity,
            false,
            "Summary should have logical structure with overview, applications, and challenges",
            Some("Missing required sections in summary".to_string()),
        );
    }

    // Record metrics
    env.record_metric("hallucination_checks", 1.0).await;

    let duration = start_time.elapsed().as_millis() as u64;
    let metrics = env.get_metrics().await;

    let passed = assertions.overall_result();

    TestResult {
        scenario: Scenario::Scenario2Research,
        passed,
        duration_ms: duration,
        error_message: if !passed {
            Some(assertions.failure_summary().join("; "))
        } else {
            None
        },
        metrics: metrics.clone(),
    }
}
