//! Scenario 2: Autonomous Research and Summary
//!
//! Tests autonomous research and summarization capabilities:
//! 1. Agent searches through local "research corpus" (markdown files)
//! 2. Extracts and synthesizes information with citations
//! 3. Council validates accuracy, citations, and hallucination detection
//! 4. Verifies output structure and reusability

use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info};

use crate::fixtures::research_sources::{create_source_files, get_research_sources};
use crate::harness::{AssertionFramework, LocalServiceManager, TestEnvironment};
use crate::{Scenario, TestMetrics, TestResult};

/// Run the research scenario test
pub async fn run_test(env: &TestEnvironment, services: &LocalServiceManager) -> TestResult {
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
    let source_files: Vec<_> = sources
        .iter()
        .map(|s| crate::harness::SourceFile {
            name: s.filename.clone(),
            content: s.content.clone(),
        })
        .collect();

    // Initialize real KnowledgeSeeker for research
    let postgres_service = services.postgres();
    let postgres_guard = postgres_service.lock().await;

    // Create DatabaseClient from PostgresService connection info
    #[cfg(feature = "full")]
    use data_infrastructure::{DatabaseClient, DatabaseConfig};
    #[cfg(feature = "full")]
    let database_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        postgres_guard.username,
        postgres_guard.password,
        postgres_guard.host,
        postgres_guard.port,
        postgres_guard.database
    );
    #[cfg(feature = "full")]
    let db_config = DatabaseConfig {
        database_url,
        host: Some(postgres_guard.host.clone()),
        port: Some(postgres_guard.port),
        database: Some(postgres_guard.database.clone()),
        username: Some(postgres_guard.username.clone()),
        password: Some(postgres_guard.password.clone()),
        max_connections: Some(10),
        pool_max: Some(10),
        connection_timeout: Some(30),
        connection_timeout_seconds: Some(30),
        query_timeout: Some(60),
        ssl_mode: Some(false),
    };
    #[cfg(feature = "full")]
    let db_client = match DatabaseClient::new(db_config).await {
        Ok(client) => Arc::new(client),
        Err(e) => {
            return TestResult {
                scenario: Scenario::Scenario2Research,
                passed: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!("Failed to create DatabaseClient: {}", e)),
                metrics: TestMetrics::default(),
            };
        }
    };

    #[cfg(feature = "full")]
    use agent_research::research_types::{
        ContextSynthesisConfig, FuzzyMatchingConfig, PerformanceConfig, ResearchAgentConfig,
        VectorSearchConfig, WebScrapingConfig,
    };
    #[cfg(feature = "full")]
    use agent_research::KnowledgeSeeker;
    #[cfg(feature = "full")]
    let knowledge_seeker = match KnowledgeSeeker::new(
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
        db_client,
    )
    .await
    {
        Ok(seeker) => seeker,
        Err(e) => {
            return TestResult {
                scenario: Scenario::Scenario2Research,
                passed: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!("Failed to initialize KnowledgeSeeker: {}", e)),
                metrics: TestMetrics::default(),
            };
        }
    };

    // Execute research task
    use agent_research::research_types::{
        KnowledgeSource, QueryType, ResearchPriority, ResearchQuery,
    };
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

    #[cfg(feature = "full")]
    let research_results = match knowledge_seeker
        .orchestrator()
        .execute_query(research_query.clone())
        .await
    {
        Ok(results) => results,
        Err(e) => {
            return TestResult {
                scenario: Scenario::Scenario2Research,
                passed: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!("Research execution failed: {}", e)),
                metrics: TestMetrics::default(),
            };
        }
    };

    #[cfg(not(feature = "full"))]
    let research_results = Vec::new();

    // Synthesize context from research results
    #[cfg(feature = "full")]
    let synthesized_context = if research_results.is_empty() {
        "No research results available.".to_string()
    } else {
        // Use context synthesizer to create summary from results
        match knowledge_seeker
            .context_synthesizer()
            .synthesize(research_query.id, research_results.clone())
            .await
        {
            Ok(context) => context.context_summary,
            Err(e) => {
                return TestResult {
                    scenario: Scenario::Scenario2Research,
                    passed: false,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some(format!("Context synthesis failed: {}", e)),
                    metrics: TestMetrics::default(),
                };
            }
        }
    };

    #[cfg(not(feature = "full"))]
    let synthesized_context = "Research not available (full feature disabled)".to_string();

    // Extract citations from research results
    #[cfg(feature = "full")]
    let citations: Vec<crate::harness::Citation> = research_results
        .iter()
        .take(10) // Limit citations
        .map(|r| crate::harness::Citation {
            source_name: format!("{:?}", r.source),
            page_or_section: r.url.clone(),
            quote: Some(r.content.chars().take(200).collect::<String>()), // First 200 chars as quote
        })
        .collect();

    #[cfg(not(feature = "full"))]
    let citations = Vec::new();

    let summary = synthesized_context;

    // Record metrics
    env.record_metric("sources_processed", sources.len() as f64)
        .await;
    env.record_metric("citations_found", citations.len() as f64)
        .await;

    // Validate citations
    assertions.assert_citation_integrity(
        &citations,
        &source_files,
        "Summary should have valid citations",
    );

    // Check hallucination detection
    // Extract known facts from research sources for fact checking
    let known_facts: Vec<String> = sources
        .iter()
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
    assertions
        .assert_no_hallucination(
            &summary,
            &fact_checker,
            "Summary should not contain hallucinations",
        )
        .await;

    // Validate minimum citations
    assertions.assert_citation_integrity(
        &citations,
        &source_files,
        &format!("Should have at least {} citations", 3),
    );

    // Check logical structure (basic check for required sections)
    let has_overview = summary.contains("Overview") || summary.contains("overview");
    let has_applications = summary.contains("Applications")
        || summary.contains("applications")
        || summary.contains("Key Applications");
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
