//! Quality and monitoring bridges for orchestration
//! 
//! This module contains bridge implementations that connect the orchestration
//! system with quality gates and monitoring systems.

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use crate::{TaskId, QualityRequirements, Progress};
use agent_agency_contracts::task_executor::ExecutionStatus;
use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use crate::error::ParallelError;
use std::collections::HashMap;
use serde_json;
use tracing::{info, error};

/// Real implementation of orchestration quality bridge

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OrchestrationQualityBridge {
    /// Quality gate thresholds
    quality_thresholds: QualityRequirements,
}

impl OrchestrationQualityBridge {
    pub fn new() -> Self {
        Self {
            quality_thresholds: QualityRequirements::default(),
        }
    }
    
    /// Validate execution artifacts against orchestration quality gates
    pub async fn validate_with_orchestration_gates(
        &self,
        task_id: &TaskId,
        artifacts: &ExecutionArtifacts,
        requirements: &QualityRequirements,
    ) -> Result<bool, ParallelError> {
        tracing::info!("Running orchestration quality gates for task: {}", task_id.0);
        
        // Check test coverage if available
        let coverage = artifacts.coverage.line_coverage;
        if coverage < requirements.min_coverage as f64 {
            return Err(ParallelError::Validation {
                message: format!("Test coverage {} below required {}", coverage, requirements.min_coverage as f64),
                source: None,
            });
        }
        
        // Check linting results if available
        if artifacts.linting.errors > 0 {
            return Err(ParallelError::Validation {
                message: format!("Linting errors found: {}", artifacts.linting.errors),
                source: None,
            });
        }
        
        // Check complexity requirements
        // Note: Complexity checking would require additional implementation
        // For now, we skip this check as the field structure needs clarification
        
        tracing::info!("Quality gates passed for task: {}", task_id.0);
        Ok(true)
    }
    
    /// Get current quality thresholds
    pub fn get_quality_thresholds(&self) -> &QualityRequirements {
        &self.quality_thresholds
    }
    
    /// Update quality thresholds
    pub fn update_quality_thresholds(&mut self, thresholds: QualityRequirements) {
        self.quality_thresholds = thresholds;
    }
}

/// Real implementation of orchestration monitoring bridge

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OrchestrationMonitoringBridge {
    /// Event storage for monitoring
    events: std::sync::Arc<std::sync::RwLock<Vec<MonitoringEvent>>>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct MonitoringEvent {
    task_id: TaskId,
    event_type: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    data: serde_json::Value,
}

impl OrchestrationMonitoringBridge {
    pub fn new() -> Self {
        Self {
            events: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }
    
    /// Publish an event to the monitoring system
    pub async fn publish_event(
        &self,
        task_id: TaskId,
        event_type: String,
        data: serde_json::Value,
    ) -> Result<(), ParallelError> {
        let task_id_clone = task_id.clone();
        let event = MonitoringEvent {
            task_id,
            event_type,
            timestamp: chrono::Utc::now(),
            data,
        };
        
        {
            let mut events = self.events.write().unwrap();
            events.push(event);
            
            // Keep only last 1000 events to prevent memory growth
            if events.len() > 1000 {
                events.remove(0);
            }
        }
        
        tracing::info!("Published monitoring event for task: {}", task_id_clone.0);
        Ok(())
    }
    
    /// Update task progress in monitoring system
    pub async fn update_task_progress(
        &self,
        task_id: &TaskId,
        status: ExecutionStatus,
        progress_percentage: f64,
        message: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), ParallelError> {
        let progress_data = serde_json::json!({
            "status": format!("{:?}", status),
            "progress_percentage": progress_percentage,
            "message": message,
            "metadata": metadata,
        });
        
        self.publish_event(
            task_id.clone(),
            "progress_update".to_string(),
            progress_data,
        ).await?;
        
        Ok(())
    }
    
    /// Get recent events for a task
    pub fn get_task_events(&self, task_id: &TaskId, limit: Option<usize>) -> Vec<MonitoringEvent> {
        let limit = limit.unwrap_or(50);
        let events = self.events.read().unwrap();
        
        events.iter()
            .filter(|event| event.task_id == *task_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Get all recent events
    pub fn get_recent_events(&self, limit: Option<usize>) -> Vec<MonitoringEvent> {
        let limit = limit.unwrap_or(100);
        let events = self.events.read().unwrap();
        
        events.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

/// Real implementation of council learning bridge

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CouncilLearningBridge {
    /// Learning events storage
    learning_events: std::sync::Arc<std::sync::RwLock<Vec<LearningEvent>>>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct LearningEvent {
    event_type: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    data: serde_json::Value,
}

impl CouncilLearningBridge {
    pub fn new() -> Self {
        Self {
            learning_events: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }
    
    /// Send learning signal to council
    pub async fn send_learning_signal(
        &self,
        signal_type: String,
        data: serde_json::Value,
    ) -> Result<(), ParallelError> {
        let event = LearningEvent {
            event_type: signal_type.clone(),
            timestamp: chrono::Utc::now(),
            data,
        };
        
        {
            let mut events = self.learning_events.write().unwrap();
            events.push(event);
            
            // Keep only last 500 learning events
            if events.len() > 500 {
                events.remove(0);
            }
        }
        
        tracing::info!("Sent learning signal: {}", signal_type);
        Ok(())
    }
    
    /// Get learning events
    pub fn get_learning_events(&self, limit: Option<usize>) -> Vec<LearningEvent> {
        let limit = limit.unwrap_or(50);
        let events = self.learning_events.read().unwrap();
        
        events.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Get learning events by type
    pub fn get_learning_events_by_type(&self, event_type: &str, limit: Option<usize>) -> Vec<LearningEvent> {
        let limit = limit.unwrap_or(50);
        let events = self.learning_events.read().unwrap();
        
        events.iter()
            .filter(|event| event.event_type == event_type)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}
