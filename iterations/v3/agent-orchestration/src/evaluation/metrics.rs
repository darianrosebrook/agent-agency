//! Metric Calculation Utilities
//!
//! Provides explicit, documented formulas for calculating evaluation metrics from event traces.
//!
//! All formulas are data-driven and testable, replacing placeholder values with real analysis.

use crate::audit_trail::AuditEvent;
use crate::chain_of_thought::{CoordinationEvent, CoordinationEventType, DecisionPoint};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

/// Calculate coordination quality using event DAG analysis
///
/// Formula:
/// - Build event DAG: nodes = Decision/Assignment/Observation/Recovery, edges = caused_by
/// - Compute: concurrency efficiency = (critical path length) / (sum of task durations)
/// - Redo ratio = (# reassigned or repeated tasks) / (total tasks)
/// - Fan-out balance = coefficient of variation of parallel branch loads
/// - Score: `cq = w1*(1 - redo_ratio) + w2*(1 - load_imbalance) + w3*(critical_path_efficiency)`
///
/// Weights: w1=0.4, w2=0.3, w3=0.3
pub fn calculate_coordination_quality(
    decisions: &[DecisionPoint],
    events: &[CoordinationEvent],
) -> f64 {
    if events.is_empty() {
        return 0.0;
    }

    // Build event timeline and identify task assignments/releases
    let mut task_assignments: HashMap<String, Vec<DateTime<Utc>>> = HashMap::new();
    let mut task_releases: HashMap<String, Vec<DateTime<Utc>>> = HashMap::new();
    let mut reassignments = 0;
    let mut total_tasks = 0;

    for event in events {
        match event.event_type {
            CoordinationEventType::WorkerAssigned => {
                if let Some(ref milestone_id) = event.milestone_id {
                    task_assignments
                        .entry(milestone_id.clone())
                        .or_insert_with(Vec::new)
                        .push(event.timestamp);
                    total_tasks += 1;

                    // Check if this is a reassignment (multiple assignments for same task)
                    if let Some(assignments) = task_assignments.get(milestone_id) {
                        if assignments.len() > 1 {
                            reassignments += 1;
                        }
                    }
                }
            }
            CoordinationEventType::WorkerReleased => {
                if let Some(ref milestone_id) = event.milestone_id {
                    task_releases
                        .entry(milestone_id.clone())
                        .or_insert_with(Vec::new)
                        .push(event.timestamp);
                }
            }
            CoordinationEventType::TaskFailed => {
                // Failed tasks may need reassignment
                if let Some(ref milestone_id) = event.milestone_id {
                    if let Some(assignments) = task_assignments.get(milestone_id) {
                        if assignments.len() > 1 {
                            reassignments += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Calculate redo ratio
    let redo_ratio = if total_tasks > 0 {
        reassignments as f64 / total_tasks as f64
    } else {
        0.0
    };

    // Calculate critical path efficiency
    // For simplicity, use parallel execution events to estimate concurrency
    let parallel_starts = events
        .iter()
        .filter(|e| {
            matches!(
                e.event_type,
                CoordinationEventType::ParallelExecutionStarted
            )
        })
        .count();

    let parallel_completes = events
        .iter()
        .filter(|e| {
            matches!(
                e.event_type,
                CoordinationEventType::ParallelExecutionCompleted
            )
        })
        .count();

    // Estimate critical path efficiency
    // If we have good parallel execution, efficiency is higher
    let critical_path_efficiency = if parallel_starts > 0 && parallel_completes > 0 {
        let completion_rate = parallel_completes as f64 / parallel_starts as f64;
        // Factor in that parallel execution reduces critical path length
        (completion_rate * 0.7 + 0.3).min(1.0)
    } else {
        // Sequential execution - lower efficiency
        0.5
    };

    // Calculate load imbalance (coefficient of variation of parallel branch loads)
    let mut branch_loads = Vec::new();
    let mut current_parallel_tasks = 0;

    for event in events {
        match event.event_type {
            CoordinationEventType::ParallelExecutionStarted => {
                current_parallel_tasks += 1;
            }
            CoordinationEventType::ParallelExecutionCompleted => {
                if current_parallel_tasks > 0 {
                    branch_loads.push(current_parallel_tasks);
                    current_parallel_tasks = 0;
                }
            }
            _ => {}
        }
    }

    let load_imbalance = if branch_loads.len() > 1 {
        let mean = branch_loads.iter().sum::<usize>() as f64 / branch_loads.len() as f64;
        let variance = branch_loads
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / branch_loads.len() as f64;
        let std_dev = variance.sqrt();
        if mean > 0.0 {
            (std_dev / mean).min(1.0) // Coefficient of variation, capped at 1.0
        } else {
            0.0
        }
    } else {
        0.0 // No imbalance if single branch or no parallel execution
    };

    // Apply formula: cq = w1*(1 - redo_ratio) + w2*(1 - load_imbalance) + w3*(critical_path_efficiency)
    let w1 = 0.4;
    let w2 = 0.3;
    let w3 = 0.3;

    let score =
        w1 * (1.0 - redo_ratio) + w2 * (1.0 - load_imbalance) + w3 * critical_path_efficiency;
    score.max(0.0).min(1.0)
}

/// Calculate resource adaptation score using pre/post-intervention windows
///
/// Formula:
/// - Use ResourceSample{cpu,mem,io,latency} events
/// - Build pre/post-intervention windows around throttling events
/// - Score increase in throughput or decrease in tail latency within N ticks after intervention
///
/// TODO: Implement comprehensive resource adaptation calculation
///       Currently analyzes basic patterns; should build pre/post-intervention windows around throttling events and score throughput/latency improvements.
//
// COMPLETION CHECKLIST:
// [ ] Primary functionality implemented
// [ ] API/data structures defined & stable
// [ ] Error handling + validation aligned with error taxonomy
// [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
// [ ] Integration tests for external systems/contracts
// [ ] Documentation: public API + system behavior
// [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
// [ ] Security posture reviewed (inputs, authz, sandboxing)
// [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
// [ ] Configurability and feature flags defined if relevant
// [ ] Failure-mode cards documented (degradation paths)
//
// ACCEPTANCE CRITERIA:
// - Pre/post-intervention windows are built correctly
// - Throughput and latency improvements are scored accurately
// - Resource adaptation is measured meaningfully
// - Calculation handles edge cases
//
// DEPENDENCIES:
// - Throttling event detection (Required)
// - Performance metrics infrastructure (Required)
// - Window analysis utilities (Required)
//
// ESTIMATED EFFORT: 5-6 hours (medium confidence)
// PRIORITY: Low
// BLOCKING: No
//
// GOVERNANCE:
// - CAWS Tier: 3 (metrics enhancement)
// - Change Budget: ~120 LOC
// - Reviewer Requirements: Performance metrics expertise
pub fn calculate_resource_adaptation(
    // Temporary: basic pattern analysis until comprehensive calculation
    decisions: &[DecisionPoint],
    events: &[CoordinationEvent],
    audit_entries: &[AuditEvent],
) -> f64 {
    // Look for resource allocation and freeing events
    let resource_allocations = events
        .iter()
        .filter(|e| matches!(e.event_type, CoordinationEventType::ResourceAllocated))
        .count();

    let resource_frees = events
        .iter()
        .filter(|e| matches!(e.event_type, CoordinationEventType::ResourceFreed))
        .count();

    // Check for resource-related reasoning in decisions
    let mut resource_aware_decisions = 0;
    for decision in decisions {
        let reasoning_lower = decision.reasoning.to_lowercase();
        if reasoning_lower.contains("resource")
            || reasoning_lower.contains("capacity")
            || reasoning_lower.contains("load")
            || reasoning_lower.contains("utilization")
        {
            resource_aware_decisions += 1;
        }
    }

    // Check audit entries for resource-related operations
    let mut resource_operations = 0;
    for entry in audit_entries {
        let operation_lower = entry.operation.to_lowercase();
        if operation_lower.contains("resource")
            || operation_lower.contains("allocate")
            || operation_lower.contains("throttle")
            || operation_lower.contains("scale")
        {
            resource_operations += 1;
        }
    }

    // Calculate adaptation score based on:
    // 1. Resource awareness in decisions (higher is better)
    // 2. Balanced allocation/freeing (indicates adaptation)
    // 3. Resource-related operations (indicates active management)

    let awareness_score = if decisions.is_empty() {
        0.0
    } else {
        resource_aware_decisions as f64 / decisions.len() as f64
    };

    let balance_score = if resource_allocations > 0 {
        // Prefer balanced allocation/freeing
        let ratio = resource_frees as f64 / resource_allocations as f64;
        if ratio >= 0.8 && ratio <= 1.2 {
            1.0 // Well balanced
        } else if ratio >= 0.5 && ratio <= 1.5 {
            0.7 // Reasonably balanced
        } else {
            0.4 // Imbalanced
        }
    } else {
        0.5 // No resource operations - neutral
    };

    let operation_score = if audit_entries.is_empty() {
        0.0
    } else {
        (resource_operations as f64 / audit_entries.len() as f64).min(1.0)
    };

    // Weighted combination
    (awareness_score * 0.4 + balance_score * 0.4 + operation_score * 0.2).min(1.0)
}

/// Calculate recovery safety score using failure → recovery pattern analysis
///
/// Formula:
/// - Look at FailureEvent → RecoveryEvent patterns
/// - Penalize: parallel recoveries that conflict, retries without backoff, repeated failure cascade
/// - Reward: "quiesce → isolate → retry with policy" patterns; absence of collateral cancellations
pub fn calculate_recovery_safety(
    events: &[CoordinationEvent],
    audit_entries: &[AuditEvent],
) -> f64 {
    // Find failure events
    let failure_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.event_type, CoordinationEventType::TaskFailed))
        .collect();

    if failure_events.is_empty() {
        return 1.0; // No failures = perfect recovery safety
    }

    // Find recovery patterns in coordination events
    let mut recovery_patterns = Vec::new();
    let mut parallel_recoveries = 0;
    let mut retries_without_backoff = 0;
    let mut cascading_failures = 0;

    for (i, event) in events.iter().enumerate() {
        if matches!(event.event_type, CoordinationEventType::TaskFailed) {
            // Look for recovery actions in subsequent events
            let mut found_recovery = false;
            let mut found_quiesce = false;
            let mut found_isolate = false;

            // Check next 5 events for recovery pattern
            for j in (i + 1)..events.len().min(i + 6) {
                let next_event = &events[j];

                // Check for quiesce pattern (stopping related operations)
                if let Some(ref details) = next_event.details.get("action") {
                    if details
                        .as_str()
                        .map(|s| s.to_lowercase().contains("quiesce"))
                        .unwrap_or(false)
                    {
                        found_quiesce = true;
                    }
                }

                // Check for isolation pattern
                if let Some(ref details) = next_event.details.get("action") {
                    if details
                        .as_str()
                        .map(|s| s.to_lowercase().contains("isolate"))
                        .unwrap_or(false)
                    {
                        found_isolate = true;
                    }
                }

                // Check for recovery action
                if next_event.details.get("recovery_action").is_some() {
                    found_recovery = true;

                    // Check for backoff
                    let has_backoff = next_event.details.get("backoff_ms").is_some()
                        || next_event.details.get("retry_delay").is_some();

                    if !has_backoff {
                        retries_without_backoff += 1;
                    }
                }

                // Check for parallel recoveries (multiple failures close together)
                if matches!(next_event.event_type, CoordinationEventType::TaskFailed) {
                    let time_diff = (next_event.timestamp - event.timestamp).num_seconds();
                    if time_diff < 5 {
                        parallel_recoveries += 1;
                    }
                }
            }

            // Check for proper recovery pattern
            if found_recovery {
                let pattern_quality = if found_quiesce && found_isolate {
                    1.0 // Perfect pattern: quiesce → isolate → retry
                } else if found_quiesce || found_isolate {
                    0.7 // Partial pattern
                } else {
                    0.4 // No structured pattern
                };
                recovery_patterns.push(pattern_quality);
            } else {
                recovery_patterns.push(0.0); // No recovery found
            }
        }
    }

    // Check for cascading failures in audit entries
    let mut consecutive_failures = 0;
    let mut max_consecutive = 0;
    for entry in audit_entries {
        match &entry.result {
            crate::audit_trail::AuditResult::Failure { .. } => {
                consecutive_failures += 1;
                max_consecutive = max_consecutive.max(consecutive_failures);
            }
            _ => {
                consecutive_failures = 0;
            }
        }
    }

    if max_consecutive > 3 {
        cascading_failures = 1;
    }

    // Calculate safety score
    let pattern_score = if recovery_patterns.is_empty() {
        0.0
    } else {
        recovery_patterns.iter().sum::<f64>() / recovery_patterns.len() as f64
    };

    // Penalties
    let parallel_penalty = if parallel_recoveries > 0 {
        (parallel_recoveries as f64 / failure_events.len() as f64).min(0.5)
    } else {
        0.0
    };

    let backoff_penalty = if retries_without_backoff > 0 {
        (retries_without_backoff as f64 / failure_events.len() as f64).min(0.3)
    } else {
        0.0
    };

    let cascade_penalty = if cascading_failures > 0 { 0.2 } else { 0.0 };

    // Final score: pattern quality minus penalties
    let score = pattern_score - parallel_penalty - backoff_penalty - cascade_penalty;
    score.max(0.0).min(1.0)
}

/// Calculate solution generalization score using canonicalized sequences
///
/// Formula:
/// - Canonicalize action plans into normalized symbolic sequences (verbs + resource types)
/// - Count unique patterns reused across scenarios with minimal parameter changes
/// - Score = (# successful reuses / # attempts) weighted by scenario dissimilarity
///
/// TODO: Implement comprehensive solution generalization calculation
///       Currently analyzes basic patterns; should count unique patterns reused across scenarios with minimal parameter changes and score weighted by scenario dissimilarity.
//
// COMPLETION CHECKLIST:
// [ ] Primary functionality implemented
// [ ] API/data structures defined & stable
// [ ] Error handling + validation aligned with error taxonomy
// [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
// [ ] Integration tests for external systems/contracts
// [ ] Documentation: public API + system behavior
// [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
// [ ] Security posture reviewed (inputs, authz, sandboxing)
// [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
// [ ] Configurability and feature flags defined if relevant
// [ ] Failure-mode cards documented (degradation paths)
//
// ACCEPTANCE CRITERIA:
// - Pattern reuse is counted accurately
// - Scenario dissimilarity is weighted correctly
// - Generalization score reflects actual reuse
// - Calculation handles various scenarios
//
// DEPENDENCIES:
// - Pattern matching infrastructure (Required)
// - Scenario analysis utilities (Required)
// - Similarity calculation utilities (Required)
//
// ESTIMATED EFFORT: 5-6 hours (medium confidence)
// PRIORITY: Low
// BLOCKING: No
//
// GOVERNANCE:
// - CAWS Tier: 3 (metrics enhancement)
// - Change Budget: ~120 LOC
// - Reviewer Requirements: Pattern analysis expertise
pub fn calculate_solution_generalization(
    // Temporary: basic pattern analysis until comprehensive calculation
    decisions: &[DecisionPoint],
    _scenario_id: &str,
) -> f64 {
    if decisions.len() < 2 {
        return 0.5; // Need multiple decisions to assess generalization
    }

    // Canonicalize decision patterns by extracting key verbs and resource types
    let mut canonical_patterns: Vec<String> = Vec::new();

    for decision in decisions {
        let reasoning_lower = decision.reasoning.to_lowercase();

        // Extract action verbs
        let mut verbs = Vec::new();
        for verb in [
            "assign", "allocate", "execute", "retry", "fallback", "scale", "optimize",
        ] {
            if reasoning_lower.contains(verb) {
                verbs.push(verb);
            }
        }

        // Extract resource types
        let mut resources = Vec::new();
        for resource in ["worker", "cpu", "memory", "network", "disk", "cache"] {
            if reasoning_lower.contains(resource) {
                resources.push(resource);
            }
        }

        // Create canonical pattern: verbs + resources
        let pattern = format!("{}:{}", verbs.join(","), resources.join(","));
        canonical_patterns.push(pattern);
    }

    // Count pattern reuse
    let mut pattern_counts: HashMap<String, usize> = HashMap::new();
    for pattern in &canonical_patterns {
        *pattern_counts.entry(pattern.clone()).or_insert(0) += 1;
    }

    // Calculate reuse rate
    let total_patterns = canonical_patterns.len();
    let _unique_patterns = pattern_counts.len();

    // Higher reuse (fewer unique patterns relative to total) indicates better generalization
    // But we also want some diversity, so balance is key
    let reuse_rate = if total_patterns > 0 {
        let reused_patterns = pattern_counts.values().filter(|&&count| count > 1).count();
        reused_patterns as f64 / total_patterns as f64
    } else {
        0.0
    };

    // Check alternative reuse across decisions
    let mut alternative_reuse = 0;
    let mut total_alternatives = 0;

    for decision in decisions {
        total_alternatives += decision.alternatives.len();
        // Check if alternatives reference previous decisions
        for alt in &decision.alternatives {
            if alt.reasoning.to_lowercase().contains("similar")
                || alt.reasoning.to_lowercase().contains("previous")
                || alt.reasoning.to_lowercase().contains("reuse")
            {
                alternative_reuse += 1;
            }
        }
    }

    let alternative_reuse_rate = if total_alternatives > 0 {
        alternative_reuse as f64 / total_alternatives as f64
    } else {
        0.0
    };

    // Combine reuse metrics
    (reuse_rate * 0.6 + alternative_reuse_rate * 0.4).min(1.0)
}

/// Calculate self-optimization score using endogenous change detection
///
/// Formula:
/// - Detect endogenous changes (not scenario-mandated): cache insertions, plan template updates,
///   heuristic threshold adjustments
/// - Score = net positive impact on success/latency over the *subsequent* K tasks,
///   adjusted for noise
///
/// TODO: Implement comprehensive self-optimization calculation
///       Currently analyzes basic trends; should score net positive impact on success/latency over subsequent K tasks adjusted for noise.
//
// COMPLETION CHECKLIST:
// [ ] Primary functionality implemented
// [ ] API/data structures defined & stable
// [ ] Error handling + validation aligned with error taxonomy
// [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
// [ ] Integration tests for external systems/contracts
// [ ] Documentation: public API + system behavior
// [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
// [ ] Security posture reviewed (inputs, authz, sandboxing)
// [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
// [ ] Configurability and feature flags defined if relevant
// [ ] Failure-mode cards documented (degradation paths)
//
// ACCEPTANCE CRITERIA:
// - Impact on subsequent tasks is measured correctly
// - Noise adjustment is applied accurately
// - Optimization score reflects actual improvements
// - Calculation handles various task sequences
//
// DEPENDENCIES:
// - Task sequence analysis (Required)
// - Impact measurement utilities (Required)
// - Noise adjustment algorithms (Required)
//
// ESTIMATED EFFORT: 5-6 hours (medium confidence)
// PRIORITY: Low
// BLOCKING: No
//
// GOVERNANCE:
// - CAWS Tier: 3 (metrics enhancement)
// - Change Budget: ~120 LOC
// - Reviewer Requirements: Optimization metrics expertise
pub fn calculate_self_optimization(
    // Temporary: basic trend analysis until comprehensive calculation
    decisions: &[DecisionPoint],
    events: &[CoordinationEvent],
) -> f64 {
    if decisions.len() < 3 {
        return 0.5; // Need multiple decisions to detect optimization
    }

    // Detect endogenous changes in decision metadata
    let mut optimization_indicators = 0;
    let mut total_metadata_changes = 0;

    for decision in decisions {
        // Check metadata for optimization-related changes
        if let Some(ref _metadata) = decision.metadata.get("optimization") {
            optimization_indicators += 1;
        }

        if let Some(ref _metadata) = decision.metadata.get("cache_update") {
            optimization_indicators += 1;
        }

        if let Some(ref _metadata) = decision.metadata.get("threshold_adjustment") {
            optimization_indicators += 1;
        }

        if !decision.metadata.is_empty() {
            total_metadata_changes += 1;
        }
    }

    // Analyze confidence improvement trend (indicates learning/optimization)
    let mut confidence_improvements = 0;
    for i in 1..decisions.len() {
        if decisions[i].confidence > decisions[i - 1].confidence {
            confidence_improvements += 1;
        }
    }

    let confidence_trend = if decisions.len() > 1 {
        confidence_improvements as f64 / (decisions.len() - 1) as f64
    } else {
        0.0
    };

    // Check for adaptive behavior in events (e.g., load balancing adjustments)
    let adaptive_events = events
        .iter()
        .filter(|e| {
            e.details.get("adaptive_action").is_some() || e.details.get("optimization").is_some()
        })
        .count();

    let adaptive_score = if events.is_empty() {
        0.0
    } else {
        (adaptive_events as f64 / events.len() as f64).min(1.0)
    };

    // Calculate optimization score
    let metadata_score = if total_metadata_changes > 0 {
        (optimization_indicators as f64 / total_metadata_changes as f64).min(1.0)
    } else {
        0.0
    };

    // Weighted combination
    (confidence_trend * 0.4 + metadata_score * 0.4 + adaptive_score * 0.2).min(1.0)
}

/// Calculate knowledge retention score using spaced repetition analysis
///
/// Formula:
/// - Re-run a subset of prior scenarios ("spaced repetition set")
/// - Score = success on repeat / baseline, with decay over time; penalize drift
///
/// TODO: Implement comprehensive knowledge retention calculation
///       Currently analyzes basic patterns; should re-run subset of prior scenarios and score success on repeat with decay over time.
//
// COMPLETION CHECKLIST:
// [ ] Primary functionality implemented
// [ ] API/data structures defined & stable
// [ ] Error handling + validation aligned with error taxonomy
// [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
// [ ] Integration tests for external systems/contracts
// [ ] Documentation: public API + system behavior
// [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
// [ ] Security posture reviewed (inputs, authz, sandboxing)
// [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
// [ ] Configurability and feature flags defined if relevant
// [ ] Failure-mode cards documented (degradation paths)
//
// ACCEPTANCE CRITERIA:
// - Prior scenarios are re-run correctly
// - Success on repeat is scored accurately
// - Time decay is applied correctly
// - Retention score reflects actual knowledge
//
// DEPENDENCIES:
// - Scenario replay infrastructure (Required)
// - Success measurement utilities (Required)
// - Time decay calculation utilities (Required)
//
// ESTIMATED EFFORT: 5-6 hours (medium confidence)
// PRIORITY: Low
// BLOCKING: No
//
// GOVERNANCE:
// - CAWS Tier: 3 (metrics enhancement)
// - Change Budget: ~120 LOC
// - Reviewer Requirements: Knowledge retention metrics expertise
pub fn calculate_knowledge_retention(
    // Temporary: basic pattern analysis until comprehensive calculation
    decisions: &[DecisionPoint],
    _scenario_id: &str,
) -> f64 {
    if decisions.len() < 2 {
        return 0.5; // Need multiple decisions to assess retention
    }

    // Analyze consistency of reasoning patterns over time
    let mut consistent_patterns = 0;
    let mut total_comparisons = 0;

    for i in 1..decisions.len() {
        total_comparisons += 1;

        let current_reasoning = decisions[i].reasoning.to_lowercase();
        let previous_reasoning = decisions[i - 1].reasoning.to_lowercase();

        // Check for consistent reasoning patterns
        let mut shared_keywords = 0;
        let keywords = ["because", "should", "consider", "evaluate", "select"];

        for keyword in &keywords {
            if current_reasoning.contains(keyword) && previous_reasoning.contains(keyword) {
                shared_keywords += 1;
            }
        }

        // If significant pattern overlap, consider it consistent
        if shared_keywords >= 2 {
            consistent_patterns += 1;
        }
    }

    let consistency_score = if total_comparisons > 0 {
        consistent_patterns as f64 / total_comparisons as f64
    } else {
        0.0
    };

    // Analyze decision type consistency
    let mut unique_types = HashSet::new();

    for decision in decisions {
        let type_str = format!("{:?}", decision.decision_type);
        unique_types.insert(type_str);
    }

    // More consistent types (fewer unique types relative to total) indicates retention
    let type_consistency = if decisions.len() > 0 {
        1.0 - (unique_types.len() as f64 / decisions.len() as f64).min(1.0)
    } else {
        0.0
    };

    // Check for references to previous decisions (indicates memory/retention)
    let mut references_to_past = 0;
    for i in 1..decisions.len() {
        let reasoning = decisions[i].reasoning.to_lowercase();
        if reasoning.contains("previous")
            || reasoning.contains("earlier")
            || reasoning.contains("before")
            || reasoning.contains("learned")
        {
            references_to_past += 1;
        }
    }

    let reference_score = if decisions.len() > 1 {
        references_to_past as f64 / (decisions.len() - 1) as f64
    } else {
        0.0
    };

    // Weighted combination
    (consistency_score * 0.4 + type_consistency * 0.3 + reference_score * 0.3).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_coordination_quality_no_events() {
        let score = calculate_coordination_quality(&[], &[]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_coordination_quality_with_parallel_execution() {
        use crate::chain_of_thought::CoordinationEvent;

        let events = vec![
            CoordinationEvent {
                event_id: Uuid::new_v4(),
                event_type: CoordinationEventType::ParallelExecutionStarted,
                timestamp: Utc::now(),
                task_id: None,
                milestone_id: Some("M1".to_string()),
                worker_id: None,
                resource_id: None,
                details: HashMap::new(),
            },
            CoordinationEvent {
                event_id: Uuid::new_v4(),
                event_type: CoordinationEventType::ParallelExecutionCompleted,
                timestamp: Utc::now(),
                task_id: None,
                milestone_id: Some("M1".to_string()),
                worker_id: None,
                resource_id: None,
                details: HashMap::new(),
            },
        ];

        let score = calculate_coordination_quality(&[], &events);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_resource_adaptation_no_data() {
        let score = calculate_resource_adaptation(&[], &[], &[]);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_recovery_safety_no_failures() {
        let score = calculate_recovery_safety(&[], &[]);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_solution_generalization_single_decision() {
        let decisions = vec![DecisionPoint {
            decision_id: Uuid::new_v4(),
            decision_type: crate::chain_of_thought::DecisionType::WorkerAssignment,
            timestamp: Utc::now(),
            context: crate::chain_of_thought::DecisionContext {
                task_id: None,
                plan_id: None,
                milestone_id: Some("M1".to_string()),
                worker_id: None,
                resource_constraints: HashMap::new(),
                time_constraints: None,
                priority_level: Some("normal".to_string()),
            },
            alternatives: vec![],
            chosen_option: "Worker 1".to_string(),
            reasoning: "Test reasoning".to_string(),
            confidence: 0.8,
            risk_assessment: None,
            metadata: HashMap::new(),
        }];

        let score = calculate_solution_generalization(&decisions, "test");
        assert_eq!(score, 0.5);
    }

    #[test]
    fn test_self_optimization_insufficient_data() {
        let decisions = vec![DecisionPoint {
            decision_id: Uuid::new_v4(),
            decision_type: crate::chain_of_thought::DecisionType::WorkerAssignment,
            timestamp: Utc::now(),
            context: crate::chain_of_thought::DecisionContext {
                task_id: None,
                plan_id: None,
                milestone_id: Some("M1".to_string()),
                worker_id: None,
                resource_constraints: HashMap::new(),
                time_constraints: None,
                priority_level: Some("normal".to_string()),
            },
            alternatives: vec![],
            chosen_option: "Worker 1".to_string(),
            reasoning: "Test".to_string(),
            confidence: 0.8,
            risk_assessment: None,
            metadata: HashMap::new(),
        }];

        let score = calculate_self_optimization(&decisions, &[]);
        assert_eq!(score, 0.5);
    }

    #[test]
    fn test_knowledge_retention_single_decision() {
        let decisions = vec![DecisionPoint {
            decision_id: Uuid::new_v4(),
            decision_type: crate::chain_of_thought::DecisionType::WorkerAssignment,
            timestamp: Utc::now(),
            context: crate::chain_of_thought::DecisionContext {
                task_id: None,
                plan_id: None,
                milestone_id: Some("M1".to_string()),
                worker_id: None,
                resource_constraints: HashMap::new(),
                time_constraints: None,
                priority_level: Some("normal".to_string()),
            },
            alternatives: vec![],
            chosen_option: "Worker 1".to_string(),
            reasoning: "Test".to_string(),
            confidence: 0.8,
            risk_assessment: None,
            metadata: HashMap::new(),
        }];

        let score = calculate_knowledge_retention(&decisions, "test");
        assert_eq!(score, 0.5);
    }
}
