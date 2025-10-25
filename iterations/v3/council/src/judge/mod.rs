//! Judge module - Main orchestrator for judge functionality

// Re-export types from judge_types module
pub use judge_types::*;

// Re-export cache functionality
pub use cache::*;

// Re-export ethics judge
pub use ethics::*;

// Re-export mock judge
pub use mock_judge::*;

// Re-export Mistral judge
pub use mistral::*;

// Import sub-modules
pub mod judge_types;
pub mod cache;
pub mod ethics;
pub mod mock_judge;
pub mod mistral;

/// Create a mock judge panel for testing
pub fn create_mock_judge_panel() -> Vec<Box<dyn Judge>> {
    vec![
        Box::new(MockJudge::new("quality_judge".to_string(), 0.85)),
        Box::new(MockJudge::new("security_judge".to_string(), 0.90)),
        Box::new(MockJudge::new("performance_judge".to_string(), 0.75)),
    ]
}

/// Judge orchestrator for coordinating multiple judges
pub struct JudgeOrchestrator {
    judges: Vec<Box<dyn Judge>>,
    cache: ResponseCache,
    config: JudgePanelConfig,
}

impl JudgeOrchestrator {
    pub fn new(judges: Vec<Box<dyn Judge>>, config: JudgePanelConfig) -> Self {
        Self {
            judges,
            cache: ResponseCache::default(),
            config,
        }
    }

    /// Evaluate specification across all judges with caching
    pub async fn evaluate_spec(
        &self,
        spec_id: uuid::Uuid,
        title: &str,
        description: &str,
        acceptance_criteria: &[String],
    ) -> Result<Vec<JudgeVerdictSummary>, Box<dyn std::error::Error + Send + Sync>> {
        let mut verdicts = Vec::new();

        for judge in &self.judges {
            // Check cache first
            if let Some(cached_verdict) = self.cache.get(spec_id, title, description).await {
                let summary = JudgeVerdictSummary {
                    judge_id: judge.id(),
                    judge_type: judge.judge_type().to_string(),
                    verdict: cached_verdict,
                    processing_time_ms: 0, // Cached response
                    timestamp: chrono::Utc::now(),
                };
                verdicts.push(summary);
                continue;
            }

            // Evaluate with judge
            let start_time = std::time::Instant::now();
            let verdict = judge.evaluate(spec_id, title, description, acceptance_criteria).await?;
            let processing_time = start_time.elapsed().as_millis() as u64;

            // Cache the result
            self.cache.put(spec_id, title, description, verdict.clone(), None).await;

            let summary = JudgeVerdictSummary {
                judge_id: judge.id(),
                judge_type: judge.judge_type().to_string(),
                verdict,
                processing_time_ms: processing_time,
                timestamp: chrono::Utc::now(),
            };

            verdicts.push(summary);
        }

        Ok(verdicts)
    }

    /// Get consensus verdict from judge panel
    pub fn get_consensus_verdict(&self, verdicts: &[JudgeVerdictSummary]) -> ConsensusResult {
        if verdicts.is_empty() {
            return ConsensusResult::InsufficientData;
        }

        let mut approve_count = 0;
        let mut refine_count = 0;
        let mut reject_count = 0;
        let mut total_confidence = 0.0;

        for verdict_summary in verdicts {
            total_confidence += match &verdict_summary.verdict {
                JudgeVerdict::Approve { confidence, .. } => {
                    approve_count += 1;
                    *confidence
                }
                JudgeVerdict::Refine { confidence, .. } => {
                    refine_count += 1;
                    *confidence
                }
                JudgeVerdict::Reject { confidence, .. } => {
                    reject_count += 1;
                    *confidence
                }
            };
        }

        let average_confidence = total_confidence / verdicts.len() as f64;

        if approve_count as f64 >= verdicts.len() as f64 * self.config.consensus_threshold {
            ConsensusResult::Approve {
                confidence: average_confidence,
                judge_count: approve_count,
            }
        } else if refine_count > reject_count {
            ConsensusResult::Refine {
                confidence: average_confidence,
                judge_count: refine_count,
            }
        } else {
            ConsensusResult::Reject {
                confidence: average_confidence,
                judge_count: reject_count,
            }
        }
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        self.cache.stats().await
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }
}

/// Consensus result from judge panel
#[derive(Debug, Clone)]
pub enum ConsensusResult {
    Approve { confidence: f64, judge_count: usize },
    Refine { confidence: f64, judge_count: usize },
    Reject { confidence: f64, judge_count: usize },
    InsufficientData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_judge_orchestrator_basic() {
        let judges = create_mock_judge_panel();
        let config = JudgePanelConfig::default();
        let orchestrator = JudgeOrchestrator::new(judges, config);

        let spec_id = uuid::Uuid::new_v4();
        let verdicts = orchestrator.evaluate_spec(
            spec_id,
            "Test Spec",
            "Test description",
            &["Should work".to_string()],
        ).await.unwrap();

        assert!(!verdicts.is_empty());
        assert_eq!(verdicts.len(), 3); // 3 mock judges
    }

    #[tokio::test]
    async fn test_consensus_calculation() {
        let judges = create_mock_judge_panel();
        let config = JudgePanelConfig {
            consensus_threshold: 0.6, // 60% threshold
            ..Default::default()
        };
        let orchestrator = JudgeOrchestrator::new(judges, config);

        let spec_id = uuid::Uuid::new_v4();
        let verdicts = orchestrator.evaluate_spec(
            spec_id,
            "Test Spec",
            "Test description",
            &["Should work".to_string()],
        ).await.unwrap();

        let consensus = orchestrator.get_consensus_verdict(&verdicts);

        // With mock judges returning Approve, should get Approve consensus
        match consensus {
            ConsensusResult::Approve { judge_count, .. } => assert_eq!(judge_count, 3),
            _ => panic!("Expected Approve consensus"),
        }
    }
}
