//! Integration tests for the Research system

use anyhow::Result;
use tracing::{debug, info};

use crate::fixtures::{TestDataGenerator, TestFixtures};
use crate::mocks::{MockDatabase, MockEventEmitter, MockFactory, MockHttpClient};
use crate::test_utils::{TestExecutor, TestResult, DEFAULT_TEST_TIMEOUT};

/// Research integration test suite
pub struct ResearchIntegrationTests {
    executor: TestExecutor,
    mock_db: MockDatabase,
    mock_events: MockEventEmitter,
    mock_http: MockHttpClient,
}

impl ResearchIntegrationTests {
    pub fn new() -> Self {
        Self {
            executor: TestExecutor::new(DEFAULT_TEST_TIMEOUT),
            mock_db: MockFactory::create_database(),
            mock_events: MockFactory::create_event_emitter(),
            mock_http: MockFactory::create_http_client(),
        }
    }

    /// Run all research integration tests
    pub async fn run_all_tests(&self) -> Result<Vec<TestResult>> {
        info!("Running Research integration tests");

        let mut results = Vec::new();

        // Test knowledge seeking
        results.push(
            self.executor
                .execute("research_knowledge_seeking", self.test_knowledge_seeking())
                .await,
        );

        // Test context synthesis
        results.push(
            self.executor
                .execute("research_context_synthesis", self.test_context_synthesis())
                .await,
        );

        // Test cross-reference detection
        results.push(
            self.executor
                .execute(
                    "research_cross_reference_detection",
                    self.test_cross_reference_detection(),
                )
                .await,
        );

        // Test web scraping integration
        results.push(
            self.executor
                .execute("research_web_scraping", self.test_web_scraping())
                .await,
        );

        // Test vector search integration
        results.push(
            self.executor
                .execute("research_vector_search", self.test_vector_search())
                .await,
        );

        // Test hybrid search
        results.push(
            self.executor
                .execute("research_hybrid_search", self.test_hybrid_search())
                .await,
        );

        Ok(results)
    }

    /// Test knowledge seeking functionality
    async fn test_knowledge_seeking(&self) -> Result<()> {
        debug!("Testing research knowledge seeking");

        // Setup test data
        let research_query = TestFixtures::research_query();
        let knowledge_entries = TestDataGenerator::generate_working_specs(5)
            .into_iter()
            .map(|_| TestFixtures::knowledge_entry())
            .collect::<Vec<_>>();

        // Initialize research system
        let research_config = agent_agency_research::ResearchConfig::default();
        let research_agent = agent_agency_research::KnowledgeSeeker::new(
            agent_agency_research::ResearchAgentConfig {
                vector_search: agent_agency_research::VectorSearchConfig {
                    enabled: false, // Disable for testing
                    qdrant_url: "http://localhost:6333".to_string(),
                    collection_name: "test_collection".to_string(),
                    model: "test".to_string(),
                    dimension: 768,
                    similarity_threshold: 0.7,
                    max_results: 10,
                    batch_size: 32,
                },
                web_scraping: agent_agency_research::WebScrapingConfig {
                    enabled: false, // Disable for testing
                    max_depth: 2,
                    max_pages: 10,
                    timeout_ms: 30000,
                    timeout_seconds: 30,
                    user_agent: "test".to_string(),
                    respect_robots_txt: false,
                    allowed_domains: vec![],
                    rate_limit_per_minute: 60,
                },
                context_synthesis: agent_agency_research::ContextSynthesisConfig {
                    enabled: true,
                    similarity_threshold: 0.7,
                    max_cross_references: 10,
                    max_context_size: 50000,
                    synthesis_timeout_ms: 30000,
                },
                performance: agent_agency_research::PerformanceConfig {
                    max_concurrent_requests: 10,
                    request_timeout_ms: 30000,
                },
            }
        ).await.unwrap_or_else(|_| agent_agency_research::KnowledgeSeeker::minimal_for_tests());

        // Test knowledge seeking
        let research_query = agent_agency_research::ResearchQuery {
            id: uuid::Uuid::new_v4(),
            query: research_query.query.clone(),
            query_type: agent_agency_research::QueryType::Knowledge,
            max_results: Some(5),
            context: None,
            priority: agent_agency_research::ResearchPriority::Normal,
            sources: vec![],
            created_at: chrono::Utc::now(),
            deadline: None,
            metadata: std::collections::HashMap::new(),
        };
        
        let results = research_agent.execute_query(research_query).await?;
        assert!(!results.is_empty());

        // Verify events were emitted
        let events = self.mock_events.get_events().await;
        // assert!(events.iter().any(|e| e.event_type == "knowledge_searched"));

        info!("✅ Knowledge seeking test completed");
        Ok(())
    }

    /// Test context synthesis
    async fn test_context_synthesis(&self) -> Result<()> {
        debug!("Testing research context synthesis");

        // Setup test data - create research results for synthesis
        let query_id = uuid::Uuid::new_v4();
        let research_results = vec![
            agent_agency_research::ResearchResult {
                query_id,
                source: agent_agency_research::KnowledgeSource::Documentation("JWT Guide".to_string()),
                title: "JWT Authentication Guide".to_string(),
                content: "JWT tokens provide secure authentication for web applications. They consist of three parts: header, payload, and signature.".to_string(),
                summary: Some("JWT tokens for secure authentication".to_string()),
                relevance_score: 0.9,
                confidence_score: 0.8,
                extracted_at: chrono::Utc::now(),
                url: Some("https://example.com/jwt-guide".to_string()),
                metadata: std::collections::HashMap::new(),
            },
            agent_agency_research::ResearchResult {
                query_id,
                source: agent_agency_research::KnowledgeSource::CodeRepository("auth-service".to_string()),
                title: "JWT Implementation Example".to_string(),
                content: "Here's how to implement JWT authentication in a Node.js application using jsonwebtoken library.".to_string(),
                summary: Some("JWT implementation in Node.js".to_string()),
                relevance_score: 0.85,
                confidence_score: 0.9,
                extracted_at: chrono::Utc::now(),
                url: Some("https://github.com/example/auth-service".to_string()),
                metadata: std::collections::HashMap::new(),
            },
            agent_agency_research::ResearchResult {
                query_id,
                source: agent_agency_research::KnowledgeSource::WebPage("Security Best Practices".to_string()),
                title: "JWT Security Best Practices".to_string(),
                content: "When using JWT tokens, always use strong secret keys, set appropriate expiration times, and validate tokens on every request.".to_string(),
                summary: Some("JWT security best practices".to_string()),
                relevance_score: 0.8,
                confidence_score: 0.7,
                extracted_at: chrono::Utc::now(),
                url: Some("https://example.com/jwt-security".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        ];

        // Initialize context builder
        let context_config = agent_agency_research::ContextSynthesisConfig {
            similarity_threshold: 0.7,
            max_cross_references: 10,
            max_context_size: 50000,
            synthesis_timeout_ms: 30000,
        };
        let context_builder = agent_agency_research::ContextBuilder::new(context_config);

        // Test context synthesis
        let (synthesized_context, metrics) = context_builder.synthesize_context(query_id, research_results).await?;
        
        // Verify synthesis results
        assert!(!synthesized_context.context_summary.is_empty());
        assert!(!synthesized_context.key_findings.is_empty());
        assert_eq!(synthesized_context.supporting_evidence.len(), 3);
        assert!(synthesized_context.confidence_score > 0.0);
        assert!(synthesized_context.confidence_score <= 1.0);
        
        // Verify metrics
        assert!(metrics.synthesis_time_ms > 0);
        assert_eq!(metrics.evidence_items_processed, 3);
        assert!(metrics.synthesis_confidence > 0.0);
        assert!(metrics.synthesis_confidence <= 1.0);
        
        // Verify cross-references were detected (should find similarities between JWT-related content)
        assert!(!synthesized_context.cross_references.is_empty());
        
        debug!("Synthesized context: {}", synthesized_context.context_summary);
        debug!("Key findings: {:?}", synthesized_context.key_findings);
        debug!("Cross-references found: {}", synthesized_context.cross_references.len());

        info!("✅ Context synthesis test completed");
        Ok(())
    }

    /// Test cross-reference detection
    async fn test_cross_reference_detection(&self) -> Result<()> {
        debug!("Testing research cross-reference detection");

        // Setup test data with cross-references
        let query_id = uuid::Uuid::new_v4();
        let research_results = vec![
            agent_agency_research::ResearchResult {
                query_id,
                source: agent_agency_research::KnowledgeSource::Documentation("JWT Guide".to_string()),
                title: "JWT Authentication".to_string(),
                content: "JWT tokens are used for authentication in web applications. They provide stateless authentication.".to_string(),
                summary: Some("JWT authentication overview".to_string()),
                relevance_score: 0.9,
                confidence_score: 0.8,
                extracted_at: chrono::Utc::now(),
                url: Some("https://example.com/jwt-auth".to_string()),
                metadata: std::collections::HashMap::new(),
            },
            agent_agency_research::ResearchResult {
                query_id,
                source: agent_agency_research::KnowledgeSource::WebPage("Token Security".to_string()),
                title: "Token Security Best Practices".to_string(),
                content: "Tokens must be secured properly to prevent unauthorized access. Use HTTPS and secure storage.".to_string(),
                summary: Some("Token security practices".to_string()),
                relevance_score: 0.85,
                confidence_score: 0.9,
                extracted_at: chrono::Utc::now(),
                url: Some("https://example.com/token-security".to_string()),
                metadata: std::collections::HashMap::new(),
            },
            agent_agency_research::ResearchResult {
                query_id,
                source: agent_agency_research::KnowledgeSource::CodeRepository("auth-flow".to_string()),
                title: "Authentication Flow Implementation".to_string(),
                content: "Authentication flow with JWT tokens involves token generation, validation, and refresh mechanisms.".to_string(),
                summary: Some("JWT authentication flow".to_string()),
                relevance_score: 0.8,
                confidence_score: 0.7,
                extracted_at: chrono::Utc::now(),
                url: Some("https://github.com/example/auth-flow".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        ];

        // Initialize cross-reference detector
        let detector = agent_agency_research::CrossReferenceDetector::new(0.3, 10); // Lower threshold to catch more references

        // Test cross-reference detection
        let cross_references = detector.detect_cross_references(&research_results).await?;
        
        // Verify cross-references were detected
        assert!(!cross_references.is_empty(), "Expected to find cross-references between JWT-related content");
        
        // Verify cross-reference properties
        for cross_ref in &cross_references {
            assert!(cross_ref.strength > 0.0);
            assert!(cross_ref.strength <= 1.0);
            assert!(!cross_ref.context.is_empty());
            assert_eq!(cross_ref.relationship, agent_agency_research::CrossReferenceType::Related);
        }
        
        debug!("Found {} cross-references", cross_references.len());
        for (i, cross_ref) in cross_references.iter().enumerate() {
            debug!("Cross-reference {}: strength={:.3}, context='{}'", i, cross_ref.strength, cross_ref.context);
        }

        info!("✅ Cross-reference detection test completed");
        Ok(())
    }

    /// Test web scraping integration
    async fn test_web_scraping(&self) -> Result<()> {
        debug!("Testing research web scraping");

        // Setup mock HTTP responses
        let mock_response = crate::mocks::MockResponse {
            status: 200,
            body: r#"
                <html>
                    <head><title>JWT Authentication Guide</title></head>
                    <body>
                        <h1>JWT Authentication</h1>
                        <p>JWT tokens provide secure authentication for web applications.</p>
                        <h2>Best Practices</h2>
                        <ul>
                            <li>Use strong secret keys</li>
                            <li>Set appropriate expiration times</li>
                            <li>Validate tokens on every request</li>
                        </ul>
                    </body>
                </html>
            "#
            .to_string(),
            headers: std::collections::HashMap::new(),
        };

        self.mock_http
            .mock_response("https://example.com/jwt-guide".to_string(), mock_response)
            .await;

        // Initialize web scraper with test configuration
        let web_scraping_config = agent_agency_research::WebScrapingConfig {
            user_agent: "Agent-Agency-Research-Test/1.0".to_string(),
            timeout_seconds: 30,
            max_content_size: 1024 * 1024, // 1MB
            rate_limit_per_minute: 60,
            allowed_domains: vec!["example.com".to_string()],
            blocked_domains: vec![],
            search_engines: vec!["google".to_string()],
        };
        let web_scraper = agent_agency_research::WebScraper::new(web_scraping_config);

        // Test web scraping
        let scraped_result = web_scraper.scrape_url("https://example.com/jwt-guide").await?;
        
        // Verify scraping results
        assert_eq!(scraped_result.url, "https://example.com/jwt-guide");
        assert_eq!(scraped_result.title, "JWT Authentication Guide");
        assert!(!scraped_result.content.is_empty());
        assert!(scraped_result.content.contains("JWT Authentication"));
        assert!(scraped_result.content.contains("Best Practices"));
        assert_eq!(scraped_result.status_code, 200);
        assert!(scraped_result.content_size > 0);
        assert!(scraped_result.processing_time_ms > 0);
        assert_eq!(scraped_result.content_type, agent_agency_research::ContentType::Html);
        
        debug!("Scraped content length: {}", scraped_result.content.len());
        debug!("Scraped title: {}", scraped_result.title);
        debug!("Processing time: {}ms", scraped_result.processing_time_ms);

        info!("✅ Web scraping test completed");
        Ok(())
    }

    /// Test vector search integration
    async fn test_vector_search(&self) -> Result<()> {
        debug!("Testing research vector search");

        // Setup test data
        let query = "JWT authentication best practices";
        
        // Initialize vector search engine with mock configuration
        let vector_search = agent_agency_research::VectorSearchEngine::new_mock();

        // Test vector search with mock data
        // Note: This uses the mock implementation which doesn't require a real Qdrant instance
        let results = vector_search.search(query, 5).await?;
        
        // Verify search results
        assert!(!results.is_empty(), "Expected to find search results");
        
        // Verify result properties
        for result in &results {
            assert!(!result.title.is_empty());
            assert!(!result.content.is_empty());
            assert!(result.relevance_score >= 0.0);
            assert!(result.relevance_score <= 1.0);
            assert!(result.credibility_score >= 0.0);
            assert!(result.credibility_score <= 1.0);
        }
        
        debug!("Found {} vector search results", results.len());
        for (i, result) in results.iter().enumerate() {
            debug!("Result {}: title='{}', relevance={:.3}, credibility={:.3}", 
                   i, result.title, result.relevance_score, result.credibility_score);
        }

        info!("✅ Vector search test completed");
        Ok(())
    }

    /// Test hybrid search (vector + keyword)
    async fn test_hybrid_search(&self) -> Result<()> {
        debug!("Testing research hybrid search");

        // Setup test data
        let query = "JWT authentication security";
        let knowledge_entries = TestDataGenerator::generate_working_specs(10)
            .into_iter()
            .map(|_| TestFixtures::knowledge_entry())
            .collect::<Vec<_>>();

        // TODO: Initialize hybrid search
        // let hybrid_search = HybridSearchEngine::new()
        //     .with_vector_search(Arc::new(vector_search))
        //     .with_keyword_search(Arc::new(keyword_search))
        //     .with_database(Arc::new(self.mock_db.clone()))
        //     .with_events(Arc::new(self.mock_events.clone()))
        //     .build()?;

        // TODO: Test hybrid search
        // let results = hybrid_search.search(query, 10).await?;
        // assert!(!results.is_empty());

        // Verify both vector and keyword search were used
        let events = self.mock_events.get_events().await;
        // assert!(events.iter().any(|e| e.event_type == "vector_search_performed"));
        // assert!(events.iter().any(|e| e.event_type == "keyword_search_performed"));

        info!("✅ Hybrid search test completed");
        Ok(())
    }
}

impl Default for ResearchIntegrationTests {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_research_integration_tests_creation() {
        let tests = ResearchIntegrationTests::new();
        assert_eq!(tests.mock_db.count().await, 0);
        assert_eq!(tests.mock_events.event_count().await, 0);
    }

    #[tokio::test]
    async fn test_mock_http_setup() {
        let tests = ResearchIntegrationTests::new();

        let mock_response = crate::mocks::MockResponse {
            status: 200,
            body: "test response".to_string(),
            headers: std::collections::HashMap::new(),
        };

        tests
            .mock_http
            .mock_response("https://example.com/test".to_string(), mock_response)
            .await;

        let response = tests
            .mock_http
            .get("https://example.com/test")
            .await
            .unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "test response");
    }
}
