//! Technical Feasibility Assessment System

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::planning_cache::CachedLLMClient;

/// Technical feasibility assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeasibilityAssessment {
    /// Overall feasibility score (0.0 = impossible, 1.0 = highly feasible)
    pub feasibility_score: f32,
    /// Specific feasibility concerns identified
    pub feasibility_concerns: Vec<FeasibilityConcern>,
    /// Domain expertise requirements
    pub domain_expertise: Vec<DomainExpertise>,
    /// Resource requirements assessment
    pub resource_requirements: ResourceRequirements,
    /// Technical complexity metrics
    pub technical_complexity: TechnicalComplexityMetrics,
    /// Timeline estimation
    pub estimated_timeline_days: Option<u32>,
    /// Risk level assessment
    pub risk_level: FeasibilityRiskLevel,
}

/// Types of feasibility concerns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeasibilityConcern {
    /// Insufficient technical expertise available
    ExpertiseGap(String),
    /// Timeline too aggressive for complexity
    TimelinePressure(String),
    /// Resource constraints identified
    ResourceLimitation(String),
    /// Technical dependencies not available
    DependencyIssue(String),
    /// Architectural constraints
    ArchitectureConstraint(String),
    /// Integration complexity too high
    IntegrationComplexity(String),
}

/// Domain expertise requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainExpertise {
    /// Domain area (e.g., "machine learning", "security", "performance")
    pub domain: String,
    /// Required expertise level
    pub required_level: ExpertiseLevel,
    /// Currently available expertise level
    pub available_level: Option<ExpertiseLevel>,
    /// Gap analysis
    pub gap_description: Option<String>,
}

/// Expertise level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExpertiseLevel {
    /// Basic understanding
    Beginner,
    /// Working knowledge
    Intermediate,
    /// Deep expertise
    Advanced,
    /// Subject matter expert
    Expert,
}

/// Resource requirements for implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Developer time in person-hours
    pub developer_hours: Option<u32>,
    /// Required computational resources
    pub computational_resources: Option<String>,
    /// Special hardware requirements
    pub hardware_requirements: Option<String>,
    /// Third-party service dependencies
    pub external_dependencies: Vec<String>,
    /// Team size requirements
    pub team_size: Option<u32>,
}

/// Technical complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalComplexityMetrics {
    /// Algorithmic complexity score (0-10)
    pub algorithmic_complexity: u8,
    /// Integration complexity score (0-10)
    pub integration_complexity: u8,
    /// Testing complexity score (0-10)
    pub testing_complexity: u8,
    /// Maintenance complexity score (0-10)
    pub maintenance_complexity: u8,
    /// Overall complexity score (0-10)
    pub overall_complexity: u8,
}

/// Risk level for feasibility assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeasibilityRiskLevel {
    /// Low risk, high confidence in success
    Low,
    /// Moderate risk, manageable challenges expected
    Moderate,
    /// High risk, significant challenges anticipated
    High,
    /// Very high risk, success unlikely
    Critical,
}

/// Feasibility assessment service
pub struct FeasibilityAssessor {
    expertise_database: HashMap<String, ExpertiseLevel>,
}

impl FeasibilityAssessor {
    pub fn new() -> Self {
        let mut expertise_database = HashMap::new();

        // Initialize with common domain expertise levels
        // In a real system, this would be loaded from configuration
        expertise_database.insert("rust".to_string(), ExpertiseLevel::Advanced);
        expertise_database.insert("machine learning".to_string(), ExpertiseLevel::Intermediate);
        expertise_database.insert("web development".to_string(), ExpertiseLevel::Expert);
        expertise_database.insert("security".to_string(), ExpertiseLevel::Intermediate);
        expertise_database.insert("performance optimization".to_string(), ExpertiseLevel::Advanced);

        Self { expertise_database }
    }

    /// Assess technical feasibility of a task
    pub async fn assess_feasibility(
        &self,
        task_description: &str,
        llm_client: &CachedLLMClient,
    ) -> Result<FeasibilityAssessment, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Assessing technical feasibility for: {}", task_description);

        // Analyze task for technical requirements
        let technical_analysis = self.analyze_technical_requirements(task_description, llm_client).await?;

        // Assess domain expertise requirements
        let domain_expertise = self.assess_domain_expertise(&technical_analysis)?;

        // Evaluate resource requirements
        let resource_requirements = self.evaluate_resource_requirements(&technical_analysis)?;

        // Calculate technical complexity
        let technical_complexity = self.calculate_technical_complexity(&technical_analysis)?;

        // Identify feasibility concerns
        let feasibility_concerns = self.identify_feasibility_concerns(
            &domain_expertise,
            &resource_requirements,
            &technical_complexity,
        )?;

        // Calculate overall feasibility score
        let feasibility_score = self.calculate_feasibility_score(
            &domain_expertise,
            &resource_requirements,
            &technical_complexity,
            &feasibility_concerns,
        );

        // Estimate timeline
        let estimated_timeline_days = self.estimate_timeline(
            &technical_complexity,
            &resource_requirements,
            feasibility_score,
        );

        // Determine risk level
        let risk_level = self.determine_risk_level(feasibility_score, &feasibility_concerns);

        Ok(FeasibilityAssessment {
            feasibility_score,
            feasibility_concerns,
            domain_expertise,
            resource_requirements,
            technical_complexity,
            estimated_timeline_days,
            risk_level,
        })
    }

    /// Analyze technical requirements using LLM
    async fn analyze_technical_requirements(
        &self,
        task_description: &str,
        llm_client: &CachedLLMClient,
    ) -> Result<TechnicalAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Analyze the following task description and extract technical requirements. \
             Provide analysis in JSON format with: \
             - domains: array of technical domains required \
             - technologies: array of specific technologies mentioned \
             - complexity_indicators: array of complexity indicators \
             - dependencies: array of external dependencies \
             - expertise_areas: array of expertise areas needed\n\n\
             Task: {}",
            task_description
        );

        let response = llm_client.generate_cached(&prompt).await?;
        let analysis: TechnicalAnalysis = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse technical analysis: {}", e))?;

        Ok(analysis)
    }

    /// Assess domain expertise requirements
    fn assess_domain_expertise(&self, analysis: &TechnicalAnalysis) -> Result<Vec<DomainExpertise>, Box<dyn std::error::Error + Send + Sync>> {
        let mut expertise_requirements = Vec::new();

        for domain in &analysis.expertise_areas {
            let required_level = self.infer_required_expertise_level(domain);
            let available_level = self.expertise_database.get(domain).cloned();

            let gap_description = match (&required_level, &available_level) {
                (ExpertiseLevel::Expert, Some(ExpertiseLevel::Advanced)) |
                (ExpertiseLevel::Expert, Some(ExpertiseLevel::Intermediate)) |
                (ExpertiseLevel::Expert, Some(ExpertiseLevel::Beginner)) |
                (ExpertiseLevel::Advanced, Some(ExpertiseLevel::Intermediate)) |
                (ExpertiseLevel::Advanced, Some(ExpertiseLevel::Beginner)) |
                (ExpertiseLevel::Intermediate, Some(ExpertiseLevel::Beginner)) => {
                    Some(format!("Required: {:?}, Available: {:?}", required_level, available_level.unwrap()))
                },
                _ => None,
            };

            expertise_requirements.push(DomainExpertise {
                domain: domain.clone(),
                required_level,
                available_level,
                gap_description,
            });
        }

        Ok(expertise_requirements)
    }

    /// Infer required expertise level for a domain
    fn infer_required_expertise_level(&self, domain: &str) -> ExpertiseLevel {
        match domain.to_lowercase().as_str() {
            "machine learning" | "ai" | "neural networks" => ExpertiseLevel::Advanced,
            "security" | "cryptography" | "performance optimization" => ExpertiseLevel::Expert,
            "web development" | "databases" | "apis" => ExpertiseLevel::Intermediate,
            _ => ExpertiseLevel::Intermediate,
        }
    }

    /// Evaluate resource requirements
    fn evaluate_resource_requirements(&self, analysis: &TechnicalAnalysis) -> Result<ResourceRequirements, Box<dyn std::error::Error + Send + Sync>> {
        // Estimate developer hours based on complexity indicators
        let developer_hours = self.estimate_developer_hours(&analysis.complexity_indicators);

        // Identify computational requirements
        let computational_resources = if analysis.technologies.iter().any(|t| t.to_lowercase().contains("ml") || t.to_lowercase().contains("ai")) {
            Some("GPU acceleration recommended".to_string())
        } else {
            None
        };

        // Hardware requirements
        let hardware_requirements = if analysis.technologies.iter().any(|t| t.to_lowercase().contains("mobile") || t.to_lowercase().contains("ios")) {
            Some("macOS development environment".to_string())
        } else {
            None
        };

        Ok(ResourceRequirements {
            developer_hours,
            computational_resources,
            hardware_requirements,
            external_dependencies: analysis.dependencies.clone(),
            team_size: self.estimate_team_size(&analysis.complexity_indicators),
        })
    }

    /// Estimate developer hours based on complexity
    fn estimate_developer_hours(&self, complexity_indicators: &[String]) -> Option<u32> {
        let base_hours = 40; // Minimum 1 week
        let complexity_multiplier = complexity_indicators.len() as f32 * 0.5 + 1.0;
        Some((base_hours as f32 * complexity_multiplier) as u32)
    }

    /// Estimate team size
    fn estimate_team_size(&self, complexity_indicators: &[String]) -> Option<u32> {
        if complexity_indicators.len() > 5 {
            Some(3) // Complex tasks need multiple developers
        } else if complexity_indicators.len() > 2 {
            Some(2) // Moderate complexity
        } else {
            Some(1) // Simple tasks
        }
    }

    /// Calculate technical complexity metrics
    fn calculate_technical_complexity(&self, analysis: &TechnicalAnalysis) -> Result<TechnicalComplexityMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let algorithmic_complexity = if analysis.complexity_indicators.iter().any(|c| c.to_lowercase().contains("algorithm")) {
            8
        } else {
            5
        };

        let integration_complexity = (analysis.dependencies.len() * 2).min(10) as u8;

        let testing_complexity = if analysis.domains.iter().any(|d| d.to_lowercase().contains("security") || d.to_lowercase().contains("network")) {
            8
        } else {
            6
        };

        let maintenance_complexity = (analysis.technologies.len() * 1).min(10) as u8;

        let overall_complexity = ((algorithmic_complexity + integration_complexity + testing_complexity + maintenance_complexity) / 4) as u8;

        Ok(TechnicalComplexityMetrics {
            algorithmic_complexity,
            integration_complexity,
            testing_complexity,
            maintenance_complexity,
            overall_complexity,
        })
    }

    /// Identify feasibility concerns
    fn identify_feasibility_concerns(
        &self,
        domain_expertise: &[DomainExpertise],
        resource_requirements: &ResourceRequirements,
        technical_complexity: &TechnicalComplexityMetrics,
    ) -> Result<Vec<FeasibilityConcern>, Box<dyn std::error::Error + Send + Sync>> {
        let mut concerns = Vec::new();

        // Check for expertise gaps
        for expertise in domain_expertise {
            if let Some(gap) = &expertise.gap_description {
                concerns.push(FeasibilityConcern::ExpertiseGap(gap.clone()));
            }
        }

        // Check for high complexity
        if technical_complexity.overall_complexity > 8 {
            concerns.push(FeasibilityConcern::IntegrationComplexity(
                "High technical complexity may impact timeline".to_string()
            ));
        }

        // Check resource constraints
        if resource_requirements.external_dependencies.len() > 3 {
            concerns.push(FeasibilityConcern::DependencyIssue(
                "Multiple external dependencies may cause integration challenges".to_string()
            ));
        }

        Ok(concerns)
    }

    /// Calculate overall feasibility score
    fn calculate_feasibility_score(
        &self,
        domain_expertise: &[DomainExpertise],
        resource_requirements: &ResourceRequirements,
        technical_complexity: &TechnicalComplexityMetrics,
        concerns: &[FeasibilityConcern],
    ) -> f32 {
        let mut score = 1.0; // Start with perfect feasibility

        // Reduce score for expertise gaps
        let expertise_gaps = domain_expertise.iter().filter(|e| e.gap_description.is_some()).count();
        score -= expertise_gaps as f32 * 0.2;

        // Reduce score for resource constraints
        if resource_requirements.external_dependencies.len() > 2 {
            score -= 0.1;
        }

        // Reduce score for high complexity
        if technical_complexity.overall_complexity > 7 {
            score -= 0.2;
        }

        // Reduce score for feasibility concerns
        score -= concerns.len() as f32 * 0.1;

        score.max(0.0).min(1.0)
    }

    /// Estimate timeline in days
    fn estimate_timeline(
        &self,
        technical_complexity: &TechnicalComplexityMetrics,
        resource_requirements: &ResourceRequirements,
        feasibility_score: f32,
    ) -> Option<u32> {
        let base_days = 14; // 2 weeks minimum
        let complexity_factor = technical_complexity.overall_complexity as f32 / 10.0;
        let team_factor = resource_requirements.team_size.unwrap_or(1) as f32;
        let feasibility_factor = 1.0 / feasibility_score.max(0.1); // Lower feasibility = longer timeline

        let estimated_days = base_days as f32 * complexity_factor * feasibility_factor / team_factor;
        Some(estimated_days as u32)
    }

    /// Determine risk level
    fn determine_risk_level(&self, feasibility_score: f32, concerns: &[FeasibilityConcern]) -> FeasibilityRiskLevel {
        let concern_count = concerns.len();

        if feasibility_score > 0.8 && concern_count == 0 {
            FeasibilityRiskLevel::Low
        } else if feasibility_score > 0.6 && concern_count <= 2 {
            FeasibilityRiskLevel::Moderate
        } else if feasibility_score > 0.4 || concern_count <= 4 {
            FeasibilityRiskLevel::High
        } else {
            FeasibilityRiskLevel::Critical
        }
    }
}

impl Default for FeasibilityAssessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal technical analysis structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TechnicalAnalysis {
    domains: Vec<String>,
    technologies: Vec<String>,
    complexity_indicators: Vec<String>,
    dependencies: Vec<String>,
    expertise_areas: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feasibility_assessment_creation() {
        let assessor = FeasibilityAssessor::new();
        // Test that assessor can be created
        assert!(assessor.expertise_database.contains_key("rust"));
    }

    #[test]
    fn test_expertise_level_inference() {
        let assessor = FeasibilityAssessor::new();

        assert_eq!(assessor.infer_required_expertise_level("machine learning"), ExpertiseLevel::Advanced);
        assert_eq!(assessor.infer_required_expertise_level("security"), ExpertiseLevel::Expert);
        assert_eq!(assessor.infer_required_expertise_level("web development"), ExpertiseLevel::Intermediate);
    }

    #[test]
    fn test_risk_level_determination() {
        let assessor = FeasibilityAssessor::new();

        assert_eq!(assessor.determine_risk_level(0.9, &[]), FeasibilityRiskLevel::Low);
        assert_eq!(assessor.determine_risk_level(0.7, &vec![FeasibilityConcern::ExpertiseGap("test".to_string())]), FeasibilityRiskLevel::Moderate);
        assert_eq!(assessor.determine_risk_level(0.3, &[]), FeasibilityRiskLevel::High);
        assert_eq!(assessor.determine_risk_level(0.1, &[]), FeasibilityRiskLevel::Critical);
    }
}
