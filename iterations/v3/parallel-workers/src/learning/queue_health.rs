//! Queue health monitoring and backpressure mechanisms

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Queue health metrics based on queueing theory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHealth {
    /// Arrival rate (tasks/second)
    pub lambda: f64,
    
    /// Service rate (tasks/second)
    pub mu: f64,
    
    /// Utilization (ρ = λ/μ)
    pub rho: f64,
    
    /// Work in progress (Little's Law: WIP ≈ λ * W)
    pub wip: f64,
    
    /// Average wait time
    pub avg_wait: Duration,
}

impl QueueHealth {
    /// Create new queue health metrics
    pub fn new(lambda: f64, mu: f64, wip: f64, avg_wait: Duration) -> Self {
        let rho = if mu > 0.0 { lambda / mu } else { 0.0 };
        
        Self {
            lambda,
            mu,
            rho,
            wip,
            avg_wait,
        }
    }
    
    /// Check if queue is healthy
    pub fn is_healthy(&self) -> bool {
        self.rho < 0.85
    }
    
    /// Check if backpressure should be applied
    pub fn should_apply_backpressure(&self) -> bool {
        self.rho > 0.85
    }
    
    /// Estimate queue length
    pub fn estimated_queue_length(&self) -> f64 {
        if self.rho < 1.0 {
            self.rho / (1.0 - self.rho)
        } else {
            f64::INFINITY
        }
    }
    
    /// Get utilization percentage
    pub fn utilization_percentage(&self) -> f64 {
        self.rho * 100.0
    }
}

/// Scheduling discipline
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SchedulingDiscipline {
    /// First-in-first-out
    FIFO,
    
    /// Shortest remaining processing time
    SRPT,
    
    /// Priority-based
    Priority,
}

/// Queue manager with backpressure
pub struct QueueManager {
    discipline: SchedulingDiscipline,
    health: QueueHealth,
    backpressure_threshold: f64,
}

impl QueueManager {
    /// Create a new queue manager
    pub fn new(discipline: SchedulingDiscipline, backpressure_threshold: f64) -> Self {
        Self {
            discipline,
            health: QueueHealth::new(0.0, 0.0, 0.0, Duration::from_secs(0)),
            backpressure_threshold,
        }
    }
    
    /// Update queue health metrics
    pub fn update_health(&mut self, health: QueueHealth) {
        self.health = health;
        
        // Switch to SRPT if queue is unhealthy
        if self.health.rho > self.backpressure_threshold {
            self.discipline = SchedulingDiscipline::SRPT;
        } else {
            self.discipline = SchedulingDiscipline::FIFO;
        }
    }
    
    /// Check if a task should be accepted
    pub fn should_accept_task(&self, task_priority: f64) -> bool {
        if self.health.should_apply_backpressure() {
            // Only accept high-priority tasks under backpressure
            task_priority > 0.7
        } else {
            true
        }
    }
    
    /// Get current scheduling discipline
    pub fn current_discipline(&self) -> SchedulingDiscipline {
        self.discipline
    }
    
    /// Get current queue health
    pub fn get_health(&self) -> &QueueHealth {
        &self.health
    }
}

/// Queue health monitor that tracks metrics over time
pub struct QueueHealthMonitor {
    health_history: Vec<(chrono::DateTime<chrono::Utc>, QueueHealth)>,
    max_history_size: usize,
}

impl QueueHealthMonitor {
    /// Create a new queue health monitor
    pub fn new(max_history_size: usize) -> Self {
        Self {
            health_history: Vec::new(),
            max_history_size,
        }
    }
    
    /// Record a health measurement
    pub fn record_health(&mut self, health: QueueHealth) {
        let timestamp = chrono::Utc::now();
        self.health_history.push((timestamp, health));
        
        // Maintain history size
        if self.health_history.len() > self.max_history_size {
            self.health_history.remove(0);
        }
    }
    
    /// Get recent health trends
    pub fn get_health_trend(&self, window_minutes: i64) -> Option<HealthTrend> {
        if self.health_history.len() < 2 {
            return None;
        }
        
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(window_minutes);
        let recent_healths: Vec<_> = self.health_history
            .iter()
            .filter(|(timestamp, _)| *timestamp > cutoff)
            .map(|(_, health)| health)
            .collect();
        
        if recent_healths.is_empty() {
            return None;
        }
        
        let avg_utilization = recent_healths.iter().map(|h| h.rho).sum::<f64>() / recent_healths.len() as f64;
        let max_utilization = recent_healths.iter().map(|h| h.rho).fold(0.0, f64::max);
        let min_utilization = recent_healths.iter().map(|h| h.rho).fold(f64::INFINITY, f64::min);
        
        Some(HealthTrend {
            avg_utilization,
            max_utilization,
            min_utilization,
            sample_count: recent_healths.len(),
            trend_direction: if avg_utilization > 0.8 { TrendDirection::Degrading } else { TrendDirection::Stable },
        })
    }
    
    /// Get current health status
    pub fn get_current_health(&self) -> Option<&QueueHealth> {
        self.health_history.last().map(|(_, health)| health)
    }
}

/// Health trend analysis
#[derive(Debug, Clone)]
pub struct HealthTrend {
    pub avg_utilization: f64,
    pub max_utilization: f64,
    pub min_utilization: f64,
    pub sample_count: usize,
    pub trend_direction: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, Copy)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_health() {
        let health = QueueHealth::new(10.0, 12.0, 5.0, Duration::from_secs(30));
        
        assert!(health.is_healthy());
        assert!(!health.should_apply_backpressure());
        assert!(health.estimated_queue_length() < f64::INFINITY);
        assert_eq!(health.utilization_percentage(), 83.33);
    }
    
    #[test]
    fn test_queue_health_unhealthy() {
        let health = QueueHealth::new(15.0, 10.0, 20.0, Duration::from_secs(60));
        
        assert!(!health.is_healthy());
        assert!(health.should_apply_backpressure());
        assert_eq!(health.utilization_percentage(), 150.0);
    }
    
    #[test]
    fn test_queue_manager() {
        let mut manager = QueueManager::new(SchedulingDiscipline::FIFO, 0.85);
        
        // Healthy queue should accept all tasks
        assert!(manager.should_accept_task(0.5));
        
        // Update to unhealthy state
        let unhealthy_health = QueueHealth::new(15.0, 10.0, 20.0, Duration::from_secs(60));
        manager.update_health(unhealthy_health);
        
        // Should only accept high-priority tasks
        assert!(!manager.should_accept_task(0.5));
        assert!(manager.should_accept_task(0.8));
        
        // Should switch to SRPT discipline
        assert!(matches!(manager.current_discipline(), SchedulingDiscipline::SRPT));
    }
    
    #[test]
    fn test_health_monitor() {
        let mut monitor = QueueHealthMonitor::new(100);
        
        // Record some health measurements
        for i in 0..5 {
            let health = QueueHealth::new(
                10.0 + i as f64,
                12.0,
                5.0 + i as f64,
                Duration::from_secs(30 + i as u64 * 10)
            );
            monitor.record_health(health);
        }
        
        // Should have trend data
        let trend = monitor.get_health_trend(60);
        assert!(trend.is_some());
        
        let trend = trend.unwrap();
        assert_eq!(trend.sample_count, 5);
        assert!(trend.avg_utilization > 0.0);
    }
}
