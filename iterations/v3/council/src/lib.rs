//! Agent Agency V3 - Council of Judges
//!
//! The Council represents the constitutional authority of the Agent Agency system,
//! implementing a specialized model-based judiciary for evaluating worker outputs
//! against CAWS principles and quality standards.

pub mod advanced_arbitration;
// pub mod claim_extraction;  // Temporarily commented to resolve circular dependency
pub mod claim_extraction;
pub mod claim_extraction_multimodal;
pub mod coordinator;
pub mod debate;
pub mod evidence_enrichment;
pub mod intelligent_edge_case_testing;
pub mod models;
pub mod predictive_learning_system;
pub mod predictive_quality_assessor;
pub mod resilience; // V2 Production Resilience
pub mod semantic;
pub mod tests;
pub mod todo_analyzer;
pub mod types;
pub mod verdicts;
pub use types::*;

// Re-export key components
pub use debate::{ArgumentGenerator, DebateContext, DebateProtocol, MockArgumentGenerator};
pub use intelligent_edge_case_testing::{
    IntelligentEdgeCaseTesting, IntelligentTestInsights, TestSpecification,
};
pub use predictive_learning_system::{LearningInsights, PredictiveLearningSystem, TaskOutcome};
pub use predictive_quality_assessor::{PredictiveQualityAssessor, QualityPrediction};
pub use todo_analyzer::{
    CouncilTodoAnalyzer, TodoAnalysisConfig, TodoAnalysisResult, TodoCategory, TodoDetection,
    TodoSeverity, TrendAnalysis, TrendDirection,
};

#[cfg(test)]
mod advanced_arbitration_tests;

#[cfg(test)]
mod predictive_learning_system_tests;

#[cfg(test)]
mod intelligent_edge_case_testing_tests;

/// Council configuration for judge coordination
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CouncilConfig {
    /// Judge model specifications
    pub judges: JudgeRegistry,
    /// Debate protocol settings
    pub debate: DebateConfig,
    /// Consensus requirements
    pub consensus: ConsensusConfig,
    /// Performance targets
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JudgeRegistry {
    pub constitutional: JudgeSpec,
    pub technical: JudgeSpec,
    pub quality: JudgeSpec,
    pub integration: JudgeSpec,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JudgeSpec {
    pub name: String,
    pub model: String,
    pub endpoint: String,
    pub weight: f32,
    pub timeout_ms: u64,
    pub optimization: OptimizationTarget,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OptimizationTarget {
    ANE, // Apple Neural Engine
    GPU, // Metal GPU
    CPU, // CPU cores
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebateConfig {
    pub max_rounds: u32,
    pub round_timeout_ms: u64,
    pub evidence_required: bool,
    pub research_agent_involvement: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsensusConfig {
    pub tier1_threshold: f32, // 0.8 for 80%
    pub tier2_threshold: f32, // 0.6 for 60%
    pub tier3_threshold: f32, // 0.5 for 50%
    pub constitutional_override: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceConfig {
    pub max_evaluation_time_ms: u64,
    pub parallel_evaluation: bool,
    pub cache_verdicts: bool,
    pub cache_ttl_seconds: u64,
}

impl Default for CouncilConfig {
    fn default() -> Self {
        Self {
            judges: JudgeRegistry {
                constitutional: JudgeSpec {
                    name: "Constitutional Judge".to_string(),
                    model: "llama3.3:3b-constitutional-caws".to_string(),
                    endpoint: "http://localhost:11434".to_string(),
                    weight: 0.4,
                    timeout_ms: 100,
                    optimization: OptimizationTarget::ANE,
                },
                technical: JudgeSpec {
                    name: "Technical Auditor".to_string(),
                    model: "codellama:7b-audit-specialist".to_string(),
                    endpoint: "http://localhost:11434".to_string(),
                    weight: 0.2,
                    timeout_ms: 500,
                    optimization: OptimizationTarget::GPU,
                },
                quality: JudgeSpec {
                    name: "Quality Evaluator".to_string(),
                    model: "gemma2:3b-quality-judge".to_string(),
                    endpoint: "http://localhost:11434".to_string(),
                    weight: 0.2,
                    timeout_ms: 200,
                    optimization: OptimizationTarget::CPU,
                },
                integration: JudgeSpec {
                    name: "Integration Validator".to_string(),
                    model: "mistral:3b-integration-checker".to_string(),
                    endpoint: "http://localhost:11434".to_string(),
                    weight: 0.2,
                    timeout_ms: 150,
                    optimization: OptimizationTarget::CPU,
                },
            },
            debate: DebateConfig {
                max_rounds: 3,
                round_timeout_ms: 5000,
                evidence_required: true,
                research_agent_involvement: true,
            },
            consensus: ConsensusConfig {
                tier1_threshold: 0.8,
                tier2_threshold: 0.6,
                tier3_threshold: 0.5,
                constitutional_override: true,
            },
            performance: PerformanceConfig {
                max_evaluation_time_ms: 10000,
                parallel_evaluation: true,
                cache_verdicts: true,
                cache_ttl_seconds: 3600,
            },
        }
    }
}

pub use claim_extraction::ClaimExtractor;
pub use claim_extraction_multimodal::{
    ClaimWithMultimodalEvidence, CrossModalValidation, ModalityCitation, ModalityEvidence,
    MultimodalEvidence, MultimodalEvidenceEnricher,
};
