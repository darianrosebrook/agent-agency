//! Failure taxonomy with RCA classifier

use crate::types::{TaskId, WorkerId, WorkerSpecialty};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Failure category classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FailureCategory {
    /// Compilation errors
    CompilationError,
    
    /// Test failures
    TestFailure,
    
    /// Timeout errors
    Timeout,
    
    /// Resource exhaustion
    ResourceExhaustion,
    
    /// Configuration issues
    ConfigurationError,
    
    /// Dependency issues
    DependencyError,
    
    /// Worker communication failure
    CommunicationError,
    
    /// Unknown/other
    Unknown,
}

/// Root cause analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remediation {
    pub category: FailureCategory,
    pub confidence: f64,
    pub suggested_actions: Vec<String>,
    pub prevention_strategies: Vec<String>,
}

/// Execution context for failure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub specialty: WorkerSpecialty,
    pub execution_time_ms: u64,
    pub resource_usage: ResourceUsage,
    pub error_message: String,
    pub stack_trace: Option<String>,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub disk_io_mb: f64,
    pub network_io_mb: f64,
}

/// RCA classifier for failure analysis
pub struct RCAClassifier {
    /// Pattern matching rules for different failure types
    patterns: HashMap<FailureCategory, Vec<String>>,
    
    /// Historical failure data
    failure_history: HashMap<FailureCategory, u64>,
}

impl RCAClassifier {
    /// Create a new RCA classifier
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Compilation error patterns
        patterns.insert(FailureCategory::CompilationError, vec![
            "error:".to_string(),
            "failed to compile".to_string(),
            "compilation error".to_string(),
            "syntax error".to_string(),
            "type error".to_string(),
        ]);
        
        // Test failure patterns
        patterns.insert(FailureCategory::TestFailure, vec![
            "test failed".to_string(),
            "assertion failed".to_string(),
            "test timeout".to_string(),
            "test error".to_string(),
        ]);
        
        // Timeout patterns
        patterns.insert(FailureCategory::Timeout, vec![
            "timeout".to_string(),
            "timed out".to_string(),
            "execution timeout".to_string(),
        ]);
        
        // Resource exhaustion patterns
        patterns.insert(FailureCategory::ResourceExhaustion, vec![
            "out of memory".to_string(),
            "disk full".to_string(),
            "resource limit".to_string(),
            "memory allocation failed".to_string(),
        ]);
        
        // Configuration error patterns
        patterns.insert(FailureCategory::ConfigurationError, vec![
            "configuration error".to_string(),
            "invalid config".to_string(),
            "missing configuration".to_string(),
        ]);
        
        // Dependency error patterns
        patterns.insert(FailureCategory::DependencyError, vec![
            "dependency not found".to_string(),
            "package not found".to_string(),
            "import error".to_string(),
            "module not found".to_string(),
        ]);
        
        // Communication error patterns
        patterns.insert(FailureCategory::CommunicationError, vec![
            "connection failed".to_string(),
            "network error".to_string(),
            "communication timeout".to_string(),
        ]);
        
        Self {
            patterns,
            failure_history: HashMap::new(),
        }
    }
    
    /// Classify failure and provide remediation
    pub fn classify_failure(&mut self, context: &ExecutionContext) -> Remediation {
        let category = self.determine_category(context);
        let confidence = self.calculate_confidence(context, &category);
        let suggested_actions = self.get_suggested_actions(&category, context);
        let prevention_strategies = self.get_prevention_strategies(&category);
        
        // Update failure history
        *self.failure_history.entry(category.clone()).or_insert(0) += 1;
        
        Remediation {
            category,
            confidence,
            suggested_actions,
            prevention_strategies,
        }
    }
    
    /// Determine failure category based on context
    fn determine_category(&self, context: &ExecutionContext) -> FailureCategory {
        let error_lower = context.error_message.to_lowercase();
        
        // Check patterns for each category
        for (category, patterns) in &self.patterns {
            for pattern in patterns {
                if error_lower.contains(pattern) {
                    return category.clone();
                }
            }
        }
        
        // Check resource usage for resource exhaustion
        if context.resource_usage.memory_mb > 1000.0 || context.resource_usage.cpu_percent > 90.0 {
            return FailureCategory::ResourceExhaustion;
        }
        
        // Check execution time for timeout
        if context.execution_time_ms > 300000 { // 5 minutes
            return FailureCategory::Timeout;
        }
        
        FailureCategory::Unknown
    }
    
    /// Calculate confidence in classification
    fn calculate_confidence(&self, context: &ExecutionContext, category: &FailureCategory) -> f64 {
        let mut confidence: f64 = 0.5; // Base confidence
        
        // Boost confidence if error message matches patterns
        if let Some(patterns) = self.patterns.get(category) {
            let error_lower = context.error_message.to_lowercase();
            for pattern in patterns {
                if error_lower.contains(pattern) {
                    confidence += 0.2;
                }
            }
        }
        
        // Boost confidence based on resource usage
        match category {
            FailureCategory::ResourceExhaustion => {
                if context.resource_usage.memory_mb > 1000.0 {
                    confidence += 0.3;
                }
                if context.resource_usage.cpu_percent > 90.0 {
                    confidence += 0.2;
                }
            }
            FailureCategory::Timeout => {
                if context.execution_time_ms > 300000 {
                    confidence += 0.3;
                }
            }
            _ => {}
        }
        
        confidence.min(1.0_f64)
    }
    
    /// Get suggested actions for remediation
    fn get_suggested_actions(&self, category: &FailureCategory, context: &ExecutionContext) -> Vec<String> {
        match category {
            FailureCategory::CompilationError => vec![
                "Check syntax and type errors".to_string(),
                "Update dependencies".to_string(),
                "Review compiler warnings".to_string(),
                "Check import statements".to_string(),
            ],
            FailureCategory::TestFailure => vec![
                "Review test cases".to_string(),
                "Check test data".to_string(),
                "Update test expectations".to_string(),
                "Debug test environment".to_string(),
            ],
            FailureCategory::Timeout => vec![
                "Increase timeout limits".to_string(),
                "Optimize algorithm complexity".to_string(),
                "Check for infinite loops".to_string(),
                "Profile performance bottlenecks".to_string(),
            ],
            FailureCategory::ResourceExhaustion => vec![
                "Increase memory limits".to_string(),
                "Optimize memory usage".to_string(),
                "Check for memory leaks".to_string(),
                "Scale resources".to_string(),
            ],
            FailureCategory::ConfigurationError => vec![
                "Validate configuration files".to_string(),
                "Check environment variables".to_string(),
                "Update configuration schema".to_string(),
                "Verify configuration format".to_string(),
            ],
            FailureCategory::DependencyError => vec![
                "Install missing dependencies".to_string(),
                "Update dependency versions".to_string(),
                "Check dependency compatibility".to_string(),
                "Resolve version conflicts".to_string(),
            ],
            FailureCategory::CommunicationError => vec![
                "Check network connectivity".to_string(),
                "Verify service endpoints".to_string(),
                "Increase timeout values".to_string(),
                "Check firewall settings".to_string(),
            ],
            FailureCategory::Unknown => vec![
                "Review error logs".to_string(),
                "Check system status".to_string(),
                "Contact support".to_string(),
            ],
        }
    }
    
    /// Get prevention strategies
    fn get_prevention_strategies(&self, category: &FailureCategory) -> Vec<String> {
        match category {
            FailureCategory::CompilationError => vec![
                "Implement pre-commit hooks".to_string(),
                "Use static analysis tools".to_string(),
                "Regular code reviews".to_string(),
                "Automated linting".to_string(),
            ],
            FailureCategory::TestFailure => vec![
                "Comprehensive test coverage".to_string(),
                "Regular test maintenance".to_string(),
                "Test data validation".to_string(),
                "Automated test execution".to_string(),
            ],
            FailureCategory::Timeout => vec![
                "Performance monitoring".to_string(),
                "Resource usage alerts".to_string(),
                "Timeout configuration".to_string(),
                "Load testing".to_string(),
            ],
            FailureCategory::ResourceExhaustion => vec![
                "Resource monitoring".to_string(),
                "Memory profiling".to_string(),
                "Resource limits".to_string(),
                "Capacity planning".to_string(),
            ],
            FailureCategory::ConfigurationError => vec![
                "Configuration validation".to_string(),
                "Schema enforcement".to_string(),
                "Environment management".to_string(),
                "Configuration testing".to_string(),
            ],
            FailureCategory::DependencyError => vec![
                "Dependency management".to_string(),
                "Version pinning".to_string(),
                "Dependency scanning".to_string(),
                "Regular updates".to_string(),
            ],
            FailureCategory::CommunicationError => vec![
                "Network monitoring".to_string(),
                "Service health checks".to_string(),
                "Circuit breakers".to_string(),
                "Retry mechanisms".to_string(),
            ],
            FailureCategory::Unknown => vec![
                "Comprehensive logging".to_string(),
                "Error tracking".to_string(),
                "Monitoring systems".to_string(),
                "Incident response".to_string(),
            ],
        }
    }
    
    /// Get failure statistics
    pub fn get_failure_stats(&self) -> HashMap<FailureCategory, u64> {
        self.failure_history.clone()
    }
    
    /// Get most common failure category
    pub fn get_most_common_failure(&self) -> Option<FailureCategory> {
        self.failure_history
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(category, _)| category.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_error_classification() {
        let mut classifier = RCAClassifier::new();
        
        let context = ExecutionContext {
            task_id: TaskId::new(),
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Compilation,
            execution_time_ms: 5000,
            resource_usage: ResourceUsage {
                cpu_percent: 50.0,
                memory_mb: 100.0,
                disk_io_mb: 10.0,
                network_io_mb: 1.0,
            },
            error_message: "error: expected `;` found `}`".to_string(),
            stack_trace: None,
        };
        
        let remediation = classifier.classify_failure(&context);
        
        assert_eq!(remediation.category, FailureCategory::CompilationError);
        assert!(remediation.confidence > 0.5);
        assert!(!remediation.suggested_actions.is_empty());
        assert!(!remediation.prevention_strategies.is_empty());
    }
    
    #[test]
    fn test_timeout_classification() {
        let mut classifier = RCAClassifier::new();
        
        let context = ExecutionContext {
            task_id: TaskId::new(),
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Testing,
            execution_time_ms: 400000, // 6+ minutes
            resource_usage: ResourceUsage {
                cpu_percent: 30.0,
                memory_mb: 200.0,
                disk_io_mb: 5.0,
                network_io_mb: 0.5,
            },
            error_message: "execution timeout".to_string(),
            stack_trace: None,
        };
        
        let remediation = classifier.classify_failure(&context);
        
        assert_eq!(remediation.category, FailureCategory::Timeout);
        assert!(remediation.confidence > 0.7);
    }
    
    #[test]
    fn test_resource_exhaustion_classification() {
        let mut classifier = RCAClassifier::new();
        
        let context = ExecutionContext {
            task_id: TaskId::new(),
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Refactoring,
            execution_time_ms: 10000,
            resource_usage: ResourceUsage {
                cpu_percent: 95.0,
                memory_mb: 2000.0, // High memory usage
                disk_io_mb: 100.0,
                network_io_mb: 10.0,
            },
            error_message: "out of memory".to_string(),
            stack_trace: None,
        };
        
        let remediation = classifier.classify_failure(&context);
        
        assert_eq!(remediation.category, FailureCategory::ResourceExhaustion);
        assert!(remediation.confidence > 0.8);
    }
    
    #[test]
    fn test_failure_statistics() {
        let mut classifier = RCAClassifier::new();
        
        // Classify a few failures
        let context1 = ExecutionContext {
            task_id: TaskId::new(),
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Compilation,
            execution_time_ms: 5000,
            resource_usage: ResourceUsage {
                cpu_percent: 50.0,
                memory_mb: 100.0,
                disk_io_mb: 10.0,
                network_io_mb: 1.0,
            },
            error_message: "compilation error".to_string(),
            stack_trace: None,
        };
        
        let context2 = ExecutionContext {
            task_id: TaskId::new(),
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Testing,
            execution_time_ms: 5000,
            resource_usage: ResourceUsage {
                cpu_percent: 50.0,
                memory_mb: 100.0,
                disk_io_mb: 10.0,
                network_io_mb: 1.0,
            },
            error_message: "test failed".to_string(),
            stack_trace: None,
        };
        
        classifier.classify_failure(&context1);
        classifier.classify_failure(&context2);
        
        let stats = classifier.get_failure_stats();
        assert_eq!(stats.get(&FailureCategory::CompilationError), Some(&1));
        assert_eq!(stats.get(&FailureCategory::TestFailure), Some(&1));
    }
}
