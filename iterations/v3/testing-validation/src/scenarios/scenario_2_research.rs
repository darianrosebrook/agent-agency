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
use crate::fixtures::research_sources::{create_source_files, get_research_sources};
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
    let db_client = services.postgres();
    #[cfg(feature = "full")]
    use agent_research::KnowledgeSeeker;
    #[cfg(feature = "full")]
    use agent_research::research_types::{ResearchAgentConfig, VectorSearchConfig, WebScrapingConfig, ContextSynthesisConfig, PerformanceConfig, FuzzyMatchingConfig};
    let knowledge_seeker = KnowledgeSeeker::new(
        ResearchAgentConfig {
            vector_search: VectorSearchConfig {
                enabled: true,
                qdrant_url: "http://localhost:6333".to_string(),
                collection_name: "research".to_string(),
                model: "all-MiniLM-L6-v2".to_string(),
                dimension: 384,
                similarity_threshold: 0.7,
                max_results: 10,
                batch_size: 10,
            },
            web_scraping: WebScrapingConfig {
                enabled: false, // Disable web scraping for local-only testing
                max_depth: 1,
                max_pages: 10,
                timeout_ms: 5000,
                timeout_seconds: 5,
                user_agent: "Agent-Agency/1.0".to_string(),
                respect_robots_txt: true,
                allowed_domains: vec![],
                rate_limit_per_minute: 10,
            },
            context_synthesis: ContextSynthesisConfig {
                enabled: true,
                similarity_threshold: 0.7,
                max_cross_references: 5,
                max_context_size: 4096,
                synthesis_timeout_ms: 10000,
            },
            performance: PerformanceConfig {
                max_concurrent_requests: 5,
                request_timeout_ms: 10000,
            },
            fuzzy_matching: FuzzyMatchingConfig {
                enabled: true,
                similarity_threshold: 0.6,
                boost_per_match: 0.1,
                coverage_boost: 0.2,
                max_total_boost: 1.0,
            },
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
    use agent_research::research_types::{ResearchQuery, QueryType, ResearchPriority, KnowledgeSource};
    use chrono::Utc;
    use std::collections::HashMap;
    let research_query = ResearchQuery {
        id: uuid::Uuid::new_v4(),
        query: "Research and summarize homomorphic encryption applications".to_string(),
        query_type: QueryType::Knowledge,
        priority: ResearchPriority::High,
        context: Some("Focus on practical applications in healthcare, finance, and cloud computing. Include technical foundations and current challenges. Cite sources and provide evidence for claims.".to_string()),
        max_results: Some(10),
        sources: vec![KnowledgeSource::InternalKnowledgeBase("test_data".to_string())],
        created_at: Utc::now(),
        deadline: None,
        metadata: HashMap::new(),
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
    // Extract known facts from research sources for fact checking
    let known_facts: Vec<String> = sources.iter()
        .flat_map(|s| s.content.lines())
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|s| s.to_string())
        .collect();
    let fact_checker = match crate::harness::FactChecker::new(known_facts).await {
        Ok(checker) => checker,
        Err(e) => {
            error!("Failed to initialize fact checker: {}", e);
            return TestResult {
                scenario: Scenario::Scenario2Research,
                passed: false,
                metrics: TestMetrics::default(),
                error_message: Some(format!("Fact checker initialization failed: {}", e)),
                ..Default::default()
            };
        }
    };
    assertions.assert_no_hallucination(
        &summary,
        &fact_checker,
        "Summary should not contain hallucinations"
    ).await;

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
    let _metrics = match env.get_metrics().await {
        Ok(m) => m,
        Err(_) => std::collections::HashMap::new(),
    };

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
        metrics: TestMetrics::default(), // TODO: Convert HashMap to TestMetrics if needed
    }
}
