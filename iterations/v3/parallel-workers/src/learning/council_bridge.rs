//! Bridge between parallel worker metrics and council learning system

use crate::types::{TaskId, WorkerId, WorkerSpecialty, TaskPattern};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Parallel worker-specific learning signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParallelWorkerSignal {
    WorkerPerformance {
        worker_id: WorkerId,
        specialty: WorkerSpecialty,
        task_pattern: TaskPattern,
        success: bool,
        execution_time: Duration,
        quality_score: f32,
        resource_usage: ResourceUsageMetrics,
    },
    
    DecompositionEffectiveness {
        task_id: TaskId,
        strategy: String,
        subtask_count: usize,
        parallel_efficiency: f32,
        speedup_factor: f32,
    },
    
    CoordinationOverhead {
        task_id: TaskId,
        worker_count: usize,
        communication_cost: Duration,
        coordination_efficiency: f32,
    },
    
    QualityGateResult {
        task_id: TaskId,
        gate_name: String,
        passed: bool,
        score: f32,
        execution_time: Duration,
    },
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    pub cpu_percent: f32,
    pub memory_mb: f32,
    pub disk_io_mb: f32,
    pub network_io_mb: f32,
}

/// Council learning bridge
pub struct CouncilLearningBridge {
    signal_buffer: Arc<RwLock<Vec<ParallelWorkerSignal>>>,
    buffer_size: usize,
    flush_interval: Duration,
}

impl CouncilLearningBridge {
    /// Create a new council learning bridge
    pub fn new(buffer_size: usize, flush_interval: Duration) -> Self {
        Self {
            signal_buffer: Arc::new(RwLock::new(Vec::new())),
            buffer_size,
            flush_interval,
        }
    }
    
    /// Publish signals to council learning system
    pub async fn publish_signals(&self, signals: Vec<ParallelWorkerSignal>) -> anyhow::Result<()> {
        let mut buffer = self.signal_buffer.write().await;
        
        // Add signals to buffer
        buffer.extend(signals);
        
        // Flush if buffer is full
        if buffer.len() >= self.buffer_size {
            self.flush_signals_internal(&mut buffer).await?;
        }
        
        Ok(())
    }
    
    /// Flush buffered signals to council learning system
    pub async fn flush_signals(&self) -> anyhow::Result<()> {
        let mut buffer = self.signal_buffer.write().await;
        self.flush_signals_internal(&mut buffer).await
    }
    
    /// Internal flush implementation
    async fn flush_signals_internal(&self, buffer: &mut Vec<ParallelWorkerSignal>) -> anyhow::Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }
        
        // Convert parallel worker signals to council format
        let council_signals: Vec<CouncilLearningSignal> = buffer
            .drain(..)
            .map(|signal| self.convert_to_council_signal(signal))
            .collect();
        
        // In a real implementation, you would send these to the council learning system
        // For now, we'll just log them
        tracing::info!("Publishing {} signals to council learning system", council_signals.len());
        
        // TODO: Implement actual council learning system integration
        // self.council_learning.process_signals(council_signals).await?;
        
        Ok(())
    }
    
    /// Convert parallel worker signal to council format
    fn convert_to_council_signal(&self, signal: ParallelWorkerSignal) -> CouncilLearningSignal {
        match signal {
            ParallelWorkerSignal::WorkerPerformance {
                worker_id,
                specialty,
                task_pattern,
                success,
                execution_time,
                quality_score,
                resource_usage,
            } => CouncilLearningSignal::WorkerPerformance {
                worker_id: worker_id.to_string(),
                specialty: format!("{:?}", specialty),
                task_pattern: format!("{:?}", task_pattern),
                success,
                execution_time_ms: execution_time.as_millis() as u64,
                quality_score,
                resource_usage: ResourceUsageData {
                    cpu_percent: resource_usage.cpu_percent,
                    memory_mb: resource_usage.memory_mb,
                    disk_io_mb: resource_usage.disk_io_mb,
                    network_io_mb: resource_usage.network_io_mb,
                },
                timestamp: Utc::now(),
            },
            
            ParallelWorkerSignal::DecompositionEffectiveness {
                task_id,
                strategy,
                subtask_count,
                parallel_efficiency,
                speedup_factor,
            } => CouncilLearningSignal::DecompositionEffectiveness {
                task_id: task_id.to_string(),
                strategy,
                subtask_count,
                parallel_efficiency,
                speedup_factor,
                timestamp: Utc::now(),
            },
            
            ParallelWorkerSignal::CoordinationOverhead {
                task_id,
                worker_count,
                communication_cost,
                coordination_efficiency,
            } => CouncilLearningSignal::CoordinationOverhead {
                task_id: task_id.to_string(),
                worker_count,
                communication_cost_ms: communication_cost.as_millis() as u64,
                coordination_efficiency,
                timestamp: Utc::now(),
            },
            
            ParallelWorkerSignal::QualityGateResult {
                task_id,
                gate_name,
                passed,
                score,
                execution_time,
            } => CouncilLearningSignal::QualityGateResult {
                task_id: task_id.to_string(),
                gate_name,
                passed,
                score,
                execution_time_ms: execution_time.as_millis() as u64,
                timestamp: Utc::now(),
            },
        }
    }
    
    /// Start background flush task
    pub async fn start_background_flush(&self) -> anyhow::Result<()> {
        let buffer = self.signal_buffer.clone();
        let flush_interval = self.flush_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(flush_interval);
            
            loop {
                interval.tick().await;
                
                let mut buffer_guard = buffer.write().await;
                if !buffer_guard.is_empty() {
                    // In a real implementation, you would flush to council learning system
                    tracing::debug!("Background flush: {} signals", buffer_guard.len());
                    buffer_guard.clear();
                }
            }
        });
        
        Ok(())
    }
}

/// Council learning signal format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilLearningSignal {
    WorkerPerformance {
        worker_id: String,
        specialty: String,
        task_pattern: String,
        success: bool,
        execution_time_ms: u64,
        quality_score: f32,
        resource_usage: ResourceUsageData,
        timestamp: DateTime<Utc>,
    },
    
    DecompositionEffectiveness {
        task_id: String,
        strategy: String,
        subtask_count: usize,
        parallel_efficiency: f32,
        speedup_factor: f32,
        timestamp: DateTime<Utc>,
    },
    
    CoordinationOverhead {
        task_id: String,
        worker_count: usize,
        communication_cost_ms: u64,
        coordination_efficiency: f32,
        timestamp: DateTime<Utc>,
    },
    
    QualityGateResult {
        task_id: String,
        gate_name: String,
        passed: bool,
        score: f32,
        execution_time_ms: u64,
        timestamp: DateTime<Utc>,
    },
}

/// Resource usage data for council signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageData {
    pub cpu_percent: f32,
    pub memory_mb: f32,
    pub disk_io_mb: f32,
    pub network_io_mb: f32,
}

/// Signal aggregator for batch processing
pub struct SignalAggregator {
    signals: Vec<ParallelWorkerSignal>,
    max_batch_size: usize,
    max_batch_age: Duration,
    last_batch_time: DateTime<Utc>,
}

impl SignalAggregator {
    /// Create a new signal aggregator
    pub fn new(max_batch_size: usize, max_batch_age: Duration) -> Self {
        Self {
            signals: Vec::new(),
            max_batch_size,
            max_batch_age,
            last_batch_time: Utc::now(),
        }
    }
    
    /// Add signal to batch
    pub fn add_signal(&mut self, signal: ParallelWorkerSignal) -> bool {
        self.signals.push(signal);
        
        // Check if batch should be flushed
        self.should_flush()
    }
    
    /// Check if batch should be flushed
    fn should_flush(&self) -> bool {
        self.signals.len() >= self.max_batch_size ||
        Utc::now() - self.last_batch_time > chrono::Duration::from_std(self.max_batch_age).unwrap()
    }
    
    /// Get and clear current batch
    pub fn take_batch(&mut self) -> Vec<ParallelWorkerSignal> {
        let batch = std::mem::take(&mut self.signals);
        self.last_batch_time = Utc::now();
        batch
    }
    
    /// Get current batch size
    pub fn batch_size(&self) -> usize {
        self.signals.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskId, WorkerId, WorkerSpecialty, TaskPattern};

    #[tokio::test]
    async fn test_council_bridge() {
        let bridge = CouncilLearningBridge::new(10, Duration::from_secs(60));
        
        let signal = ParallelWorkerSignal::WorkerPerformance {
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Compilation,
            task_pattern: TaskPattern::Compilation,
            success: true,
            execution_time: Duration::from_secs(5),
            quality_score: 0.9,
            resource_usage: ResourceUsageMetrics {
                cpu_percent: 50.0,
                memory_mb: 100.0,
                disk_io_mb: 10.0,
                network_io_mb: 1.0,
            },
        };
        
        let result = bridge.publish_signals(vec![signal]).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_signal_aggregator() {
        let mut aggregator = SignalAggregator::new(5, Duration::from_secs(30));
        
        let signal = ParallelWorkerSignal::WorkerPerformance {
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Compilation,
            task_pattern: TaskPattern::Compilation,
            success: true,
            execution_time: Duration::from_secs(5),
            quality_score: 0.9,
            resource_usage: ResourceUsageMetrics {
                cpu_percent: 50.0,
                memory_mb: 100.0,
                disk_io_mb: 10.0,
                network_io_mb: 1.0,
            },
        };
        
        // Add signals until batch is ready
        for _ in 0..5 {
            let should_flush = aggregator.add_signal(signal.clone());
            if should_flush {
                break;
            }
        }
        
        let batch = aggregator.take_batch();
        assert_eq!(batch.len(), 5);
    }
    
    #[test]
    fn test_signal_conversion() {
        let bridge = CouncilLearningBridge::new(10, Duration::from_secs(60));
        
        let signal = ParallelWorkerSignal::DecompositionEffectiveness {
            task_id: TaskId::new(),
            strategy: "domain_based".to_string(),
            subtask_count: 3,
            parallel_efficiency: 0.8,
            speedup_factor: 2.5,
        };
        
        let council_signal = bridge.convert_to_council_signal(signal);
        
        match council_signal {
            CouncilLearningSignal::DecompositionEffectiveness {
                strategy,
                subtask_count,
                parallel_efficiency,
                speedup_factor,
                ..
            } => {
                assert_eq!(strategy, "domain_based");
                assert_eq!(subtask_count, 3);
                assert_eq!(parallel_efficiency, 0.8);
                assert_eq!(speedup_factor, 2.5);
            }
            _ => panic!("Expected DecompositionEffectiveness signal"),
        }
    }
}
