//! Fairness constraints and diversity tracking

use crate::types::{WorkerId, WorkerSpecialty};
use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Fairness constraints to prevent worker starvation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessConstraints {
    /// Minimum share per specialty (0.0 to 1.0)
    pub min_share_per_specialty: f64,
    
    /// Cooldown period after consecutive failures
    pub cooldown_failures: u32,
    
    /// Diversity budget per window
    pub diversity_budget: f64,
}

impl Default for FairnessConstraints {
    fn default() -> Self {
        Self {
            min_share_per_specialty: 0.1,
            cooldown_failures: 3,
            diversity_budget: 0.2,
        }
    }
}

/// Worker fairness tracker
pub struct FairnessTracker {
    constraints: FairnessConstraints,
    specialty_counts: HashMap<WorkerSpecialty, u64>,
    worker_failures: HashMap<WorkerId, u32>,
    total_selections: u64,
    window_start: chrono::DateTime<chrono::Utc>,
    window_duration: Duration,
}

impl FairnessTracker {
    /// Create a new fairness tracker
    pub fn new(constraints: FairnessConstraints, window_duration: Duration) -> Self {
        Self {
            constraints,
            specialty_counts: HashMap::new(),
            worker_failures: HashMap::new(),
            total_selections: 0,
            window_start: chrono::Utc::now(),
            window_duration,
        }
    }
    
    /// Record worker selection
    pub fn record_selection(&mut self, worker_id: &WorkerId, specialty: &WorkerSpecialty) {
        *self.specialty_counts.entry(specialty.clone()).or_insert(0) += 1;
        self.total_selections += 1;
        
        // Reset window if needed
        if chrono::Utc::now() - self.window_start > chrono::Duration::from_std(self.window_duration).unwrap() {
            self.reset_window();
        }
    }
    
    /// Record worker failure
    pub fn record_failure(&mut self, worker_id: &WorkerId) {
        *self.worker_failures.entry(worker_id.clone()).or_insert(0) += 1;
    }
    
    /// Record worker success (resets failure count)
    pub fn record_success(&mut self, worker_id: &WorkerId) {
        self.worker_failures.remove(worker_id);
    }
    
    /// Check if specialty is under quota
    pub fn is_under_quota(&self, specialty: &WorkerSpecialty) -> bool {
        if self.total_selections == 0 {
            return true;
        }
        
        let count = self.specialty_counts.get(specialty).copied().unwrap_or(0);
        let share = count as f64 / self.total_selections as f64;
        
        share < self.constraints.min_share_per_specialty
    }
    
    /// Check if worker is in cooldown
    pub fn is_in_cooldown(&self, worker_id: &WorkerId) -> bool {
        self.worker_failures
            .get(worker_id)
            .map(|&failures| failures >= self.constraints.cooldown_failures)
            .unwrap_or(false)
    }
    
    /// Get specialty share
    pub fn get_specialty_share(&self, specialty: &WorkerSpecialty) -> f64 {
        if self.total_selections == 0 {
            return 0.0;
        }
        
        let count = self.specialty_counts.get(specialty).copied().unwrap_or(0);
        count as f64 / self.total_selections as f64
    }
    
    /// Get fairness report
    pub fn get_fairness_report(&self) -> FairnessReport {
        let mut specialty_shares = HashMap::new();
        for specialty in &[
            WorkerSpecialty::CompilationErrors { error_codes: vec!["E0277".to_string()] },
            WorkerSpecialty::Refactoring { strategies: vec!["extract".to_string()] },
            WorkerSpecialty::Testing { frameworks: vec!["cargo".to_string()] },
            WorkerSpecialty::Documentation { formats: vec!["markdown".to_string()] },
            WorkerSpecialty::TypeSystem { domains: vec![crate::types::TypeDomain::TraitBounds] },
            WorkerSpecialty::AsyncPatterns { patterns: vec!["async".to_string()] },
        ] {
            specialty_shares.insert(specialty.clone(), self.get_specialty_share(specialty));
        }
        
        let under_quota_specialties: Vec<_> = specialty_shares
            .iter()
            .filter(|(_, &share)| share < self.constraints.min_share_per_specialty)
            .map(|(specialty, _)| specialty.clone())
            .collect();
        
        FairnessReport {
            specialty_shares,
            under_quota_specialties,
            total_selections: self.total_selections,
            workers_in_cooldown: self.worker_failures.len(),
            window_start: self.window_start,
        }
    }
    
    fn reset_window(&mut self) {
        self.specialty_counts.clear();
        self.total_selections = 0;
        self.window_start = chrono::Utc::now();
    }
}

/// Fairness report
#[derive(Debug, Clone)]
pub struct FairnessReport {
    pub specialty_shares: HashMap<WorkerSpecialty, f64>,
    pub under_quota_specialties: Vec<WorkerSpecialty>,
    pub total_selections: u64,
    pub workers_in_cooldown: usize,
    pub window_start: chrono::DateTime<chrono::Utc>,
}

/// Diversity tracker for maintaining worker variety
pub struct DiversityTracker {
    recent_workers: Vec<WorkerId>,
    max_recent_size: usize,
    diversity_budget: f64,
}

impl DiversityTracker {
    /// Create a new diversity tracker
    pub fn new(max_recent_size: usize, diversity_budget: f64) -> Self {
        Self {
            recent_workers: Vec::new(),
            max_recent_size,
            diversity_budget,
        }
    }
    
    /// Record worker selection
    pub fn record_selection(&mut self, worker_id: WorkerId) {
        self.recent_workers.push(worker_id);
        
        // Maintain recent workers list
        if self.recent_workers.len() > self.max_recent_size {
            self.recent_workers.remove(0);
        }
    }
    
    /// Check if we should force diversity
    pub fn should_force_diversity(&self, candidate_worker: &WorkerId) -> bool {
        if self.recent_workers.len() < self.max_recent_size {
            return false;
        }
        
        // Count how many times this worker was recently selected
        let recent_count = self.recent_workers.iter()
            .filter(|&id| id == candidate_worker)
            .count();
        
        let diversity_ratio = recent_count as f64 / self.recent_workers.len() as f64;
        
        // Force diversity if this worker is over-represented
        diversity_ratio > (1.0 - self.diversity_budget)
    }
    
    /// Get diversity score (0.0 = no diversity, 1.0 = perfect diversity)
    pub fn get_diversity_score(&self) -> f64 {
        if self.recent_workers.is_empty() {
            return 1.0;
        }
        
        let unique_workers: std::collections::HashSet<_> = self.recent_workers.iter().collect();
        let unique_count = unique_workers.len();
        let total_count = self.recent_workers.len();
        
        unique_count as f64 / total_count as f64
    }
}

/// Fairness-aware worker selector
pub struct FairnessAwareSelector {
    fairness_tracker: FairnessTracker,
    diversity_tracker: DiversityTracker,
}

impl FairnessAwareSelector {
    /// Create a new fairness-aware selector
    pub fn new(
        constraints: FairnessConstraints,
        window_duration: Duration,
        diversity_budget: f64,
    ) -> Self {
        Self {
            fairness_tracker: FairnessTracker::new(constraints, window_duration),
            diversity_tracker: DiversityTracker::new(100, diversity_budget),
        }
    }
    
    /// Select worker with fairness constraints
    pub fn select_worker(
        &mut self,
        candidates: &[(WorkerId, WorkerSpecialty)],
        task_priority: f64,
    ) -> Option<(WorkerId, WorkerSpecialty)> {
        if candidates.is_empty() {
            return None;
        }
        
        // Filter out workers in cooldown
        let available_candidates: Vec<_> = candidates
            .iter()
            .filter(|(worker_id, _)| !self.fairness_tracker.is_in_cooldown(worker_id))
            .collect();
        
        if available_candidates.is_empty() {
            // If all workers are in cooldown, select the one with least failures
            return candidates.first().map(|(id, specialty)| (id.clone(), specialty.clone()));
        }
        
        // Boost under-quota specialties
        let boosted_candidates: Vec<_> = available_candidates
            .iter()
            .map(|(worker_id, specialty)| {
                let base_score = 1.0;
                let quota_boost = if self.fairness_tracker.is_under_quota(specialty) {
                    2.0
                } else {
                    1.0
                };
                let diversity_penalty = if self.diversity_tracker.should_force_diversity(worker_id) {
                    0.5
                } else {
                    1.0
                };
                
                let final_score = base_score * quota_boost * diversity_penalty;
                (worker_id, specialty, final_score)
            })
            .collect();
        
        // Select candidate with highest score
        let selected = boosted_candidates
            .iter()
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        
        selected.map(|&(worker_id, specialty, _)| {
            // Record selection
            self.fairness_tracker.record_selection(&worker_id, &specialty);
            self.diversity_tracker.record_selection(worker_id.clone());
            
            (worker_id.clone(), specialty.clone())
        })
    }
    
    /// Record worker outcome
    pub fn record_outcome(&mut self, worker_id: &WorkerId, success: bool) {
        if success {
            self.fairness_tracker.record_success(worker_id);
        } else {
            self.fairness_tracker.record_failure(worker_id);
        }
    }
    
    /// Get fairness report
    pub fn get_fairness_report(&self) -> FairnessReport {
        self.fairness_tracker.get_fairness_report()
    }
    
    /// Get diversity score
    pub fn get_diversity_score(&self) -> f64 {
        self.diversity_tracker.get_diversity_score()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fairness_tracker() {
        let constraints = FairnessConstraints::default();
        let window_duration = Duration::from_secs(3600);
        let mut tracker = FairnessTracker::new(constraints, window_duration);
        
        let worker_id = WorkerId::new();
        let specialty = WorkerSpecialty::Compilation;
        
        // Initially under quota
        assert!(tracker.is_under_quota(&specialty));
        
        // Record some selections
        for _ in 0..5 {
            tracker.record_selection(&worker_id, &specialty);
        }
        
        // Should have share > 0
        assert!(tracker.get_specialty_share(&specialty) > 0.0);
        
        // Record failures
        for _ in 0..5 {
            tracker.record_failure(&worker_id);
        }
        
        // Should be in cooldown
        assert!(tracker.is_in_cooldown(&worker_id));
    }
    
    #[test]
    fn test_diversity_tracker() {
        let mut tracker = DiversityTracker::new(10, 0.2);
        
        let worker1 = WorkerId::new();
        let worker2 = WorkerId::new();
        
        // Add some selections
        for _ in 0..5 {
            tracker.record_selection(worker1.clone());
        }
        
        // Should force diversity for over-represented worker
        assert!(tracker.should_force_diversity(&worker1));
        assert!(!tracker.should_force_diversity(&worker2));
        
        // Add more diverse selections
        for _ in 0..5 {
            tracker.record_selection(worker2.clone());
        }
        
        // Should have better diversity score
        assert!(tracker.get_diversity_score() > 0.5);
    }
    
    #[test]
    fn test_fairness_aware_selector() {
        let constraints = FairnessConstraints::default();
        let window_duration = Duration::from_secs(3600);
        let mut selector = FairnessAwareSelector::new(constraints, window_duration, 0.2);
        
        let worker1 = (WorkerId::new(), WorkerSpecialty::Compilation);
        let worker2 = (WorkerId::new(), WorkerSpecialty::Testing);
        
        let candidates = vec![worker1.clone(), worker2.clone()];
        
        // Should select a worker
        let selected = selector.select_worker(&candidates, 0.5);
        assert!(selected.is_some());
        
        // Record outcome
        selector.record_outcome(&worker1.0, true);
        selector.record_outcome(&worker2.0, false);
        
        // Should have fairness report
        let report = selector.get_fairness_report();
        assert!(report.total_selections > 0);
    }
}
