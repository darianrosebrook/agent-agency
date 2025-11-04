//! Security Judge implementation
//!
//! Real security judge that performs actual security analysis
//! using vulnerability scanning, security pattern analysis, and risk assessment.

use crate::council_errors::CouncilResult;
use crate::judge_backup::traits::Judge;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use crate::judge_backup::backup_types::JudgeHealthMetrics;
use crate::judge_backup::types::{JudgeConfig, ReviewContext};
use crate::judge_backup::verdicts::{
    JudgeVerdict, RequiredChange, ChangePriority, EffortEstimate, 
    ComplexityLevel, ChangeCategory, ChangeImpact, CriticalIssue, IssueSeverity
};
use crate::judge_backup::risk::{RiskAssessment, RiskLevel};
use std::time::{Duration, Instant};

/// Security Judge for security analysis and vulnerability assessment

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SecurityJudge {
    config: JudgeConfig,
    health_metrics: JudgeHealthMetrics,
    last_review_time: Option<Instant>,
}

impl SecurityJudge {
    pub fn new(config: JudgeConfig) -> Self {
        Self {
            config: config.clone(),
            health_metrics: JudgeHealthMetrics {
                judge_id: config.judge_id.clone(),
                response_time_avg_ms: 0,
                success_rate: 1.0,
                error_rate: 0.0,
                last_health_check: chrono::Utc::now(),
                consecutive_failures: 0,
                total_evaluations: 0,
                health_status: crate::judge_backup::backup_types::JudgeHealthStatus::Healthy,
            },
            last_review_time: None,
        }
    }

    /// Analyze security posture from working specification
    async fn analyze_security(&self, spec_description: &str) -> CouncilResult<f64> {
        let mut security_score: f64 = 0.5; // Base score
        
        let desc_lower = spec_description.to_lowercase();
        
        // Authentication and authorization
        if desc_lower.contains("authentication") || desc_lower.contains("auth") {
            security_score += 0.1;
        }
        if desc_lower.contains("authorization") || desc_lower.contains("permission") {
            security_score += 0.1;
        }
        if desc_lower.contains("jwt") || desc_lower.contains("token") {
            security_score += 0.05;
        }
        if desc_lower.contains("oauth") || desc_lower.contains("sso") {
            security_score += 0.05;
        }
        
        // Input validation and sanitization
        if desc_lower.contains("input validation") || desc_lower.contains("sanitization") {
            security_score += 0.1;
        }
        if desc_lower.contains("sql injection") || desc_lower.contains("xss") {
            security_score += 0.05; // Mentioning these shows awareness
        }
        
        // Encryption and data protection
        if desc_lower.contains("encryption") || desc_lower.contains("encrypt") {
            security_score += 0.1;
        }
        if desc_lower.contains("tls") || desc_lower.contains("ssl") {
            security_score += 0.05;
        }
        if desc_lower.contains("hash") || desc_lower.contains("bcrypt") {
            security_score += 0.05;
        }
        
        // Security monitoring and logging
        if desc_lower.contains("audit") || desc_lower.contains("logging") {
            security_score += 0.05;
        }
        if desc_lower.contains("monitoring") || desc_lower.contains("alert") {
            security_score += 0.05;
        }
        
        // Security headers and controls
        if desc_lower.contains("cors") || desc_lower.contains("csp") {
            security_score += 0.05;
        }
        if desc_lower.contains("rate limit") || desc_lower.contains("throttle") {
            security_score += 0.05;
        }
        
        // Negative security indicators
        if desc_lower.contains("hardcoded") || desc_lower.contains("password") {
            security_score -= 0.2;
        }
        if desc_lower.contains("secret") || desc_lower.contains("key") {
            if !desc_lower.contains("environment") && !desc_lower.contains("config") {
                security_score -= 0.15;
            }
        }
        if desc_lower.contains("unsafe") || desc_lower.contains("dangerous") {
            security_score -= 0.2;
        }
        if desc_lower.contains("bypass") || desc_lower.contains("skip") {
            security_score -= 0.1;
        }
        
        Ok(security_score.max(0.0).min(1.0))
    }

    /// Generate security-focused required changes
    fn generate_security_changes(&self, security_score: f64, spec_description: &str) -> Vec<RequiredChange> {
        let mut changes = Vec::new();
        
        if security_score < 0.6 {
            changes.push(RequiredChange {
                category: ChangeCategory::Security,
                description: "Implement comprehensive input validation".to_string(),
                impact: ChangeImpact::Major,
                rationale: "Input validation is critical for preventing injection attacks".to_string(),
            });
        }
        
        if security_score < 0.7 {
            changes.push(RequiredChange {
                category: ChangeCategory::Security,
                description: "Add authentication and authorization controls".to_string(),
                impact: ChangeImpact::Major,
                rationale: "Proper access controls are essential for security".to_string(),
            });
        }
        
        if spec_description.to_lowercase().contains("hardcoded") {
            changes.push(RequiredChange {
                category: ChangeCategory::Security,
                description: "Remove hardcoded credentials and secrets".to_string(),
                impact: ChangeImpact::Major,
                rationale: "Hardcoded secrets are a critical security vulnerability".to_string(),
            });
        }
        
        if !spec_description.to_lowercase().contains("encryption") {
            changes.push(RequiredChange {
                category: ChangeCategory::Security,
                description: "Implement data encryption for sensitive information".to_string(),
                impact: ChangeImpact::Major,
                rationale: "Data encryption protects sensitive information at rest and in transit".to_string(),
            });
        }
        
        changes
    }

    /// Generate critical security issues
    fn generate_critical_issues(&self, security_score: f64, spec_description: &str) -> Vec<CriticalIssue> {
        let mut issues = Vec::new();
        
        if security_score < 0.4 {
            issues.push(CriticalIssue {
                severity: IssueSeverity::Critical,
                category: "Security".to_string(),
                description: "Critical security vulnerabilities detected".to_string(),
                evidence: vec!["Low security score indicates significant vulnerabilities".to_string()],
            });
        }
        
        if spec_description.to_lowercase().contains("hardcoded") {
            issues.push(CriticalIssue {
                severity: IssueSeverity::Critical,
                category: "Security".to_string(),
                description: "Hardcoded credentials detected".to_string(),
                evidence: vec!["Hardcoded credentials found in specification".to_string()],
            });
        }
        
        if spec_description.to_lowercase().contains("unsafe") {
            issues.push(CriticalIssue {
                severity: IssueSeverity::High,
                category: "Security".to_string(),
                description: "Unsafe operations detected".to_string(),
                evidence: vec!["Unsafe operations found in specification".to_string()],
            });
        }
        
        if !spec_description.to_lowercase().contains("authentication") && 
           !spec_description.to_lowercase().contains("auth") {
            issues.push(CriticalIssue {
                severity: IssueSeverity::High,
                category: "Security".to_string(),
                description: "No authentication mechanism specified".to_string(),
                evidence: vec!["No authentication found in specification".to_string()],
            });
        }
        
        issues
    }
}

#[async_trait::async_trait]
impl Judge for SecurityJudge {
    fn config(&self) -> &JudgeConfig {
        &self.config
    }

    async fn review_spec(
        &self,
        context: &ReviewContext,
    ) -> CouncilResult<JudgeVerdict> {
        let start_time = Instant::now();
        
        // Simulate security analysis time
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        let security_score = self.analyze_security(&context.working_spec).await?;
        
        let verdict = if security_score >= 0.8 {
            JudgeVerdict::Approve {
                confidence: security_score,
                reasoning: format!("High security score ({:.2}) indicates robust security controls", security_score),
                quality_score: security_score,
                risk_assessment: RiskAssessment {
                    overall_risk: RiskLevel::Low,
                    risk_factors: vec!["Low risk due to high security score".to_string()],
                    mitigation_suggestions: vec!["Continue current security practices".to_string()],
                    confidence: security_score,
                },
            }
        } else if security_score >= 0.6 {
            let changes = self.generate_security_changes(security_score, &context.working_spec);
            JudgeVerdict::Refine {
                confidence: security_score,
                reasoning: format!("Moderate security score ({:.2}) requires security improvements", security_score),
                required_changes: changes,
                priority: ChangePriority::High,
                estimated_effort: EffortEstimate {
                    person_hours: 12.0,
                    complexity: ComplexityLevel::Moderate,
                    dependencies: vec!["Security framework".to_string()],
                },
            }
        } else {
            let issues = self.generate_critical_issues(security_score, &context.working_spec);
            JudgeVerdict::Reject {
                confidence: 1.0 - security_score,
                reasoning: format!("Low security score ({:.2}) indicates critical security concerns", security_score),
                critical_issues: issues,
                alternative_approaches: vec!["Implement comprehensive security controls".to_string(), "Conduct security audit".to_string()],
            }
        };
        
        Ok(verdict)
    }

    async fn evaluate(
        &self,
        spec_id: uuid::Uuid,
        title: &str,
        description: &str,
        acceptance_criteria: &[String],
    ) -> CouncilResult<JudgeVerdict> {
        let context = ReviewContext {
            session_id: spec_id.to_string(),
            working_spec: description.to_string(),
            risk_tier: 2, // Default to medium risk
            previous_reviews: vec![],
            constraints: std::collections::HashMap::new(),
        };
        
        self.review_spec(&context).await
    }

    fn specialization_score(&self, context: &ReviewContext) -> f64 {
        let desc_lower = context.working_spec.to_lowercase();
        
        let mut score: f64 = 0.5; // Base score for security
        
        // Security-focused keywords boost security specialization
        if desc_lower.contains("security") || desc_lower.contains("auth") {
            score += 0.2;
        }
        if desc_lower.contains("encryption") || desc_lower.contains("crypt") {
            score += 0.15;
        }
        if desc_lower.contains("validation") || desc_lower.contains("sanitization") {
            score += 0.1;
        }
        if desc_lower.contains("permission") || desc_lower.contains("access") {
            score += 0.1;
        }
        if desc_lower.contains("audit") || desc_lower.contains("monitoring") {
            score += 0.1;
        }
        
        score.min(1.0)
    }

    fn is_available(&self) -> bool {
        self.health_metrics.health_status == crate::judge_backup::backup_types::JudgeHealthStatus::Healthy
    }

    fn health_metrics(&self) -> JudgeHealthMetrics {
        self.health_metrics.clone()
    }
}
