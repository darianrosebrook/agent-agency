<!-- d29127fc-ead1-48a7-8681-bf1ce0839887 44092e78-c4d6-45b6-97e5-9796db1c7b1a -->
# Learning System Integration for Parallel Workers

## Overview

Implement a complete learning and feedback loop that captures parallel worker execution metrics, analyzes performance patterns, and feeds insights back into worker selection and configuration. This will integrate with the existing `council/src/learning.rs` infrastructure while maintaining the parallel-workers crate's independence.

## Architecture Decision

**Centralized Learning with Distributed Collection**: Use the existing `council::learning` module as the centralized learning system, with parallel-workers providing a specialized adapter/bridge for its domain-specific metrics.

## Implementation Components

### 1. Performance Data Collection Layer

**File**: `iterations/v3/parallel-workers/src/learning/metrics_collector.rs`

```rust
/// Collects and aggregates performance metrics from parallel worker execution
pub struct ParallelWorkerMetricsCollector {
    execution_history: DashMap<TaskId, Vec<ExecutionRecord>>,
    worker_profiles: DashMap<WorkerId, WorkerPerformanceProfile>,
    pattern_cache: Arc<RwLock<PatternCache>>,
}

pub struct ExecutionRecord {
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub specialty: WorkerSpecialty,
    pub subtask_id: SubTaskId,
    pub metrics: ExecutionMetrics,
    pub outcome: ExecutionOutcome,
    pub timestamp: DateTime<Utc>,
}

pub struct WorkerPerformanceProfile {
    pub worker_id: WorkerId,
    pub specialty: WorkerSpecialty,
    pub total_executions: usize,
    pub success_rate: f32,
    pub average_execution_time: Duration,
    pub average_quality_score: f32,
    pub resource_efficiency: ResourceEfficiencyScore,
    pub specialization_strength: HashMap<TaskPattern, f32>,
    pub last_updated: DateTime<Utc>,
}

impl ParallelWorkerMetricsCollector {
    /// Record execution completion
    pub fn record_execution(&self, record: ExecutionRecord) {
        // Store in execution history
        // Update worker performance profile
        // Trigger pattern analysis if threshold reached
    }
    
    /// Get worker performance profile
    pub fn get_worker_profile(&self, worker_id: &WorkerId) -> Option<WorkerPerformanceProfile> {
        self.worker_profiles.get(worker_id).map(|p| p.clone())
    }
    
    /// Get all workers with specialty
    pub fn get_workers_by_specialty(&self, specialty: &WorkerSpecialty) -> Vec<WorkerPerformanceProfile> {
        // Filter and return workers matching specialty
    }
}
```

### 2. Council Learning Bridge

**File**: `iterations/v3/parallel-workers/src/learning/council_bridge.rs`

```rust
use agent_agency_council::learning::{LearningSignal as CouncilLearningSignal, LearningSystem};

/// Bridge between parallel worker metrics and council learning system
pub struct CouncilLearningBridge {
    council_learning: Arc<LearningSystem>,
    metrics_collector: Arc<ParallelWorkerMetricsCollector>,
    signal_buffer: Arc<RwLock<Vec<ParallelWorkerSignal>>>,
}

/// Parallel worker-specific learning signals
#[derive(Debug, Clone)]
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
        strategy: DecompositionStrategy,
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

impl CouncilLearningBridge {
    /// Convert parallel worker signals to council learning signals
    pub async fn publish_signals(&self, signals: Vec<ParallelWorkerSignal>) -> Result<()> {
        let council_signals: Vec<CouncilLearningSignal> = signals
            .into_iter()
            .map(|s| self.convert_to_council_signal(s))
            .collect();
        
        self.council_learning.process_signals(council_signals).await
    }
    
    /// Convert parallel worker signal to council format
    fn convert_to_council_signal(&self, signal: ParallelWorkerSignal) -> CouncilLearningSignal {
        // Map parallel worker signals to council learning signal format
        // Include parallel-specific context in signal data
    }
    
    /// Flush buffered signals to council learning system
    pub async fn flush_signals(&self) -> Result<()> {
        let signals = {
            let mut buffer = self.signal_buffer.write().await;
            std::mem::take(&mut *buffer)
        };
        
        if !signals.is_empty() {
            self.publish_signals(signals).await?;
        }
        
        Ok(())
    }
}
```

### 3. Pattern Recognition Engine

**File**: `iterations/v3/parallel-workers/src/learning/pattern_analyzer.rs`

```rust
/// Analyzes execution patterns to identify optimization opportunities
pub struct PatternAnalyzer {
    metrics_collector: Arc<ParallelWorkerMetricsCollector>,
    pattern_cache: Arc<RwLock<PatternCache>>,
}

pub struct PatternCache {
    pub successful_patterns: HashMap<TaskPattern, Vec<SuccessPattern>>,
    pub failure_patterns: HashMap<TaskPattern, Vec<FailurePattern>>,
    pub optimal_configurations: HashMap<TaskPattern, OptimalConfig>,
}

pub struct SuccessPattern {
    pub task_pattern: TaskPattern,
    pub worker_specialty: WorkerSpecialty,
    pub decomposition_strategy: DecompositionStrategy,
    pub average_speedup: f32,
    pub success_rate: f32,
    pub sample_size: usize,
}

pub struct FailurePattern {
    pub task_pattern: TaskPattern,
    pub failure_mode: FailureMode,
    pub frequency: usize,
    pub common_causes: Vec<String>,
}

pub struct OptimalConfig {
    pub task_pattern: TaskPattern,
    pub recommended_workers: Vec<WorkerSpecialty>,
    pub optimal_subtask_count: usize,
    pub timeout_multiplier: f32,
    pub quality_thresholds: QualityThresholds,
    pub confidence: f32,
}

impl PatternAnalyzer {
    /// Analyze recent executions to identify patterns
    pub async fn analyze_patterns(&self) -> Result<PatternAnalysisReport> {
        // Analyze execution history
        // Identify successful patterns
        // Identify failure patterns
        // Generate optimal configurations
        // Update pattern cache
    }
    
    /// Get optimal configuration for task pattern
    pub fn get_optimal_config(&self, pattern: &TaskPattern) -> Option<OptimalConfig> {
        self.pattern_cache.read().unwrap()
            .optimal_configurations
            .get(pattern)
            .cloned()
    }
}
```

### 4. Adaptive Worker Selector

**File**: `iterations/v3/parallel-workers/src/learning/adaptive_selector.rs`

```rust
/// Selects optimal workers based on learned performance patterns
pub struct AdaptiveWorkerSelector {
    metrics_collector: Arc<ParallelWorkerMetricsCollector>,
    pattern_analyzer: Arc<PatternAnalyzer>,
    selection_strategy: SelectionStrategy,
}

pub enum SelectionStrategy {
    /// Always select best performing worker
    BestPerformer,
    
    /// Balance between performance and load
    LoadBalanced { performance_weight: f32 },
    
    /// Epsilon-greedy exploration
    EpsilonGreedy { epsilon: f32 },
    
    /// Thompson sampling for exploration/exploitation
    ThompsonSampling,
}

pub struct WorkerRecommendation {
    pub worker_id: Option<WorkerId>,
    pub specialty: WorkerSpecialty,
    pub confidence: f32,
    pub expected_performance: PerformanceEstimate,
    pub alternatives: Vec<AlternativeRecommendation>,
}

pub struct PerformanceEstimate {
    pub expected_execution_time: Duration,
    pub expected_quality_score: f32,
    pub success_probability: f32,
}

impl AdaptiveWorkerSelector {
    /// Recommend optimal worker for subtask
    pub async fn recommend_worker(&self, subtask: &SubTask) -> Result<WorkerRecommendation> {
        // Extract task pattern from subtask
        let pattern = self.extract_pattern(subtask);
        
        // Get workers with matching specialty
        let candidates = self.metrics_collector
            .get_workers_by_specialty(&subtask.specialty);
        
        // Get optimal configuration if available
        let optimal_config = self.pattern_analyzer
            .get_optimal_config(&pattern);
        
        // Score candidates based on performance history
        let scored_candidates = self.score_candidates(&candidates, &pattern, &optimal_config);
        
        // Apply selection strategy
        let selected = self.apply_selection_strategy(&scored_candidates);
        
        Ok(WorkerRecommendation {
            worker_id: selected.worker_id,
            specialty: subtask.specialty.clone(),
            confidence: selected.confidence,
            expected_performance: selected.performance_estimate,
            alternatives: scored_candidates.into_iter()
                .filter(|c| c.worker_id != selected.worker_id)
                .take(3)
                .collect(),
        })
    }
    
    /// Score candidate workers based on historical performance
    fn score_candidates(
        &self,
        candidates: &[WorkerPerformanceProfile],
        pattern: &TaskPattern,
        optimal_config: &Option<OptimalConfig>,
    ) -> Vec<ScoredCandidate> {
        // Score based on success rate
        // Score based on execution time
        // Score based on specialization strength for this pattern
        // Boost score if matches optimal config
        // Apply recency weighting
    }
}
```

### 5. Configuration Optimizer

**File**: `iterations/v3/parallel-workers/src/learning/config_optimizer.rs`

```rust
/// Optimizes parallel coordinator configuration based on learned patterns
pub struct ConfigurationOptimizer {
    pattern_analyzer: Arc<PatternAnalyzer>,
    metrics_collector: Arc<ParallelWorkerMetricsCollector>,
    optimization_history: Arc<RwLock<Vec<OptimizationEvent>>>,
}

pub struct OptimizationEvent {
    pub timestamp: DateTime<Utc>,
    pub optimization_type: OptimizationType,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub rationale: String,
}

pub enum OptimizationType {
    TimeoutAdjustment,
    ConcurrencyLimit,
    QualityThreshold,
    ComplexityThreshold,
    DecompositionStrategy,
}

pub struct ConfigurationRecommendations {
    pub recommendations: Vec<ConfigRecommendation>,
    pub confidence: f32,
    pub expected_improvement: f32,
}

pub struct ConfigRecommendation {
    pub parameter: String,
    pub current_value: serde_json::Value,
    pub recommended_value: serde_json::Value,
    pub rationale: String,
    pub confidence: f32,
    pub expected_impact: ExpectedImpact,
}

pub struct ExpectedImpact {
    pub throughput_change: f32,
    pub quality_change: f32,
    pub resource_efficiency_change: f32,
}

impl ConfigurationOptimizer {
    /// Generate configuration recommendations based on learned patterns
    pub async fn generate_recommendations(&self) -> Result<ConfigurationRecommendations> {
        // Analyze recent performance trends
        // Identify configuration bottlenecks
        // Generate recommendations for each parameter
        // Estimate impact of each recommendation
        // Rank recommendations by expected improvement
    }
    
    /// Apply configuration optimization
    pub async fn apply_optimization(
        &self,
        recommendation: &ConfigRecommendation,
    ) -> Result<OptimizationEvent> {
        // Validate recommendation
        // Apply configuration change
        // Record optimization event
        // Schedule performance validation
    }
    
    /// Analyze timeout patterns and recommend adjustments
    fn analyze_timeout_patterns(&self) -> Vec<ConfigRecommendation> {
        // Group executions by specialty
        // Calculate P95 execution times
        // Identify timeouts vs successful completions
        // Recommend timeout multipliers per specialty
    }
    
    /// Analyze concurrency patterns and recommend limits
    fn analyze_concurrency_patterns(&self) -> Vec<ConfigRecommendation> {
        // Analyze coordination overhead vs worker count
        // Identify optimal concurrency levels per task complexity
        // Recommend dynamic concurrency limits
    }
}
```

### 6. Database Persistence Layer

**File**: `iterations/v3/parallel-workers/src/learning/persistence.rs`

```rust
use agent_agency_database::DatabaseClient;

/// Persists learning data to database for long-term analysis
pub struct LearningPersistence {
    db_client: Arc<DatabaseClient>,
    batch_buffer: Arc<RwLock<BatchBuffer>>,
    flush_interval: Duration,
}

struct BatchBuffer {
    execution_records: Vec<ExecutionRecord>,
    performance_profiles: Vec<WorkerPerformanceProfile>,
    pattern_updates: Vec<PatternUpdate>,
}

impl LearningPersistence {
    /// Persist execution record
    pub async fn persist_execution(&self, record: ExecutionRecord) -> Result<()> {
        // Add to batch buffer
        // Flush if buffer size threshold reached
    }
    
    /// Persist worker performance profile
    pub async fn persist_profile(&self, profile: WorkerPerformanceProfile) -> Result<()> {
        // Upsert worker profile in database
        // Update performance history
    }
    
    /// Flush batch buffer to database
    pub async fn flush(&self) -> Result<()> {
        let batch = {
            let mut buffer = self.batch_buffer.write().await;
            std::mem::take(&mut *buffer)
        };
        
        // Batch insert execution records
        // Batch upsert performance profiles
        // Batch update pattern cache
    }
    
    /// Query historical performance data
    pub async fn query_performance_history(
        &self,
        filters: PerformanceQueryFilters,
    ) -> Result<Vec<ExecutionRecord>> {
        // Query database with filters
        // Return matching execution records
    }
}
```

### 7. Integration with ParallelCoordinator

**File**: `iterations/v3/parallel-workers/src/coordinator.rs` (modifications)

```rust
// Add to existing ParallelCoordinator struct
pub struct ParallelCoordinator {
    // ... existing fields ...
    
    // Learning system components
    metrics_collector: Arc<ParallelWorkerMetricsCollector>,
    council_bridge: Arc<CouncilLearningBridge>,
    adaptive_selector: Arc<AdaptiveWorkerSelector>,
    config_optimizer: Arc<ConfigurationOptimizer>,
    learning_persistence: Arc<LearningPersistence>,
}

impl ParallelCoordinator {
    /// Execute parallel with learning integration
    pub async fn execute_parallel(
        &mut self,
        task: ComplexTask,
    ) -> Result<TaskResult, ParallelError> {
        let start_time = Utc::now();
        
        // 1. Analyze task complexity and decompose
        let analysis = self.analyze_task(&task).await?;
        
        // 2. Get adaptive configuration recommendations
        let config_recommendations = self.config_optimizer
            .generate_recommendations()
            .await?;
        
        // Apply high-confidence recommendations
        self.apply_config_recommendations(&config_recommendations).await?;
        
        if !analysis.should_parallelize() {
            return self.orchestrator_handle.execute_sequential(task).await;
        }
        
        // 3. Create optimized subtasks
        let subtasks = self.decomposition_engine.decompose(analysis.clone())?;
        
        // 4. Use adaptive selector to assign workers
        let worker_assignments = self.assign_workers_adaptively(&subtasks).await?;
        
        // 5. Spawn workers with assignments
        let workers = self.worker_manager.spawn_workers_with_assignments(worker_assignments).await?;
        
        // 6. Monitor and coordinate execution
        let results = self.coordinate_workers(workers).await?;
        
        // 7. Validate quality gates
        self.validator.validate_all(&results).await?;
        
        // 8. Synthesize final result
        let final_result = self.progress_tracker.synthesize_results(results.clone())?;
        
        // 9. Record execution metrics for learning
        self.record_execution_metrics(
            &task,
            &analysis,
            &subtasks,
            &results,
            start_time,
            Utc::now(),
        ).await?;
        
        // 10. Publish learning signals to council
        self.publish_learning_signals(&task, &results).await?;
        
        Ok(final_result)
    }
    
    /// Assign workers using adaptive selector
    async fn assign_workers_adaptively(
        &self,
        subtasks: &[SubTask],
    ) -> Result<Vec<WorkerAssignment>> {
        let mut assignments = Vec::new();
        
        for subtask in subtasks {
            let recommendation = self.adaptive_selector
                .recommend_worker(subtask)
                .await?;
            
            assignments.push(WorkerAssignment {
                subtask: subtask.clone(),
                worker_id: recommendation.worker_id,
                expected_performance: recommendation.expected_performance,
            });
        }
        
        Ok(assignments)
    }
    
    /// Record execution metrics for learning
    async fn record_execution_metrics(
        &self,
        task: &ComplexTask,
        analysis: &TaskAnalysis,
        subtasks: &[SubTask],
        results: &[WorkerResult],
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<()> {
        // Record individual worker executions
        for result in results {
            let record = ExecutionRecord {
                task_id: task.id.clone(),
                worker_id: result.worker_id.clone(),
                specialty: result.specialty.clone(),
                subtask_id: result.subtask_id.clone(),
                metrics: result.metrics.clone(),
                outcome: self.classify_outcome(result),
                timestamp: result.metrics.end_time,
            };
            
            self.metrics_collector.record_execution(record.clone());
            self.learning_persistence.persist_execution(record).await?;
        }
        
        // Record decomposition effectiveness
        let speedup_factor = self.calculate_speedup_factor(results);
        let decomposition_signal = ParallelWorkerSignal::DecompositionEffectiveness {
            task_id: task.id.clone(),
            strategy: analysis.decomposition_strategy.clone(),
            subtask_count: subtasks.len(),
            parallel_efficiency: self.calculate_parallel_efficiency(results),
            speedup_factor,
        };
        
        self.council_bridge.publish_signals(vec![decomposition_signal]).await?;
        
        Ok(())
    }
    
    /// Publish learning signals to council
    async fn publish_learning_signals(
        &self,
        task: &ComplexTask,
        results: &[WorkerResult],
    ) -> Result<()> {
        let mut signals = Vec::new();
        
        // Worker performance signals
        for result in results {
            signals.push(ParallelWorkerSignal::WorkerPerformance {
                worker_id: result.worker_id.clone(),
                specialty: result.specialty.clone(),
                task_pattern: self.extract_pattern(&result.subtask),
                success: result.outcome.is_success(),
                execution_time: result.metrics.end_time
                    .signed_duration_since(result.metrics.start_time)
                    .to_std()
                    .unwrap_or_default(),
                quality_score: result.metrics.quality_score,
                resource_usage: result.metrics.resource_usage.clone(),
            });
        }
        
        self.council_bridge.publish_signals(signals).await?;
        
        Ok(())
    }
}
```

### 8. Database Schema

**File**: `iterations/v3/database/migrations/YYYYMMDD_parallel_worker_learning.sql`

```sql
-- Worker performance profiles
CREATE TABLE parallel_worker_profiles (
    worker_id UUID PRIMARY KEY,
    specialty VARCHAR(255) NOT NULL,
    total_executions INTEGER NOT NULL DEFAULT 0,
    success_rate REAL NOT NULL DEFAULT 0.0,
    avg_execution_time_ms BIGINT NOT NULL DEFAULT 0,
    avg_quality_score REAL NOT NULL DEFAULT 0.0,
    resource_efficiency_score REAL NOT NULL DEFAULT 0.0,
    specialization_strengths JSONB NOT NULL DEFAULT '{}',
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Execution history
CREATE TABLE parallel_execution_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    worker_id UUID NOT NULL,
    specialty VARCHAR(255) NOT NULL,
    subtask_id UUID NOT NULL,
    execution_time_ms BIGINT NOT NULL,
    quality_score REAL NOT NULL,
    outcome VARCHAR(50) NOT NULL,
    metrics JSONB NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Pattern cache
CREATE TABLE parallel_pattern_cache (
    pattern_hash VARCHAR(64) PRIMARY KEY,
    task_pattern JSONB NOT NULL,
    optimal_config JSONB NOT NULL,
    success_rate REAL NOT NULL,
    sample_size INTEGER NOT NULL,
    confidence REAL NOT NULL,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Configuration optimization history
CREATE TABLE parallel_config_optimizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    optimization_type VARCHAR(100) NOT NULL,
    old_value JSONB NOT NULL,
    new_value JSONB NOT NULL,
    rationale TEXT NOT NULL,
    expected_improvement REAL,
    actual_improvement REAL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_execution_records_task_id ON parallel_execution_records(task_id);
CREATE INDEX idx_execution_records_worker_id ON parallel_execution_records(worker_id);
CREATE INDEX idx_execution_records_timestamp ON parallel_execution_records(timestamp DESC);
CREATE INDEX idx_worker_profiles_specialty ON parallel_worker_profiles(specialty);
CREATE INDEX idx_pattern_cache_updated ON parallel_pattern_cache(last_updated DESC);
```

### 9. Configuration Updates

**File**: `iterations/v3/config/production.yaml` (additions)

```yaml
parallel_workers:
  # ... existing config ...
  
  learning:
    enabled: true
    
    # Metrics collection
    metrics_collection:
      enabled: true
      buffer_size: 1000
      flush_interval_seconds: 60
      
    # Pattern analysis
    pattern_analysis:
      enabled: true
      analysis_interval_seconds: 300
      min_sample_size: 10
      confidence_threshold: 0.7
      
    # Adaptive selection
    adaptive_selection:
      enabled: true
      strategy: "epsilon_greedy"  # best_performer | load_balanced | epsilon_greedy | thompson_sampling
      epsilon: 0.1  # For epsilon_greedy
      performance_weight: 0.7  # For load_balanced
      
    # Configuration optimization
    config_optimization:
      enabled: true
      auto_apply: false  # Require manual approval
      optimization_interval_seconds: 3600
      min_confidence_for_auto_apply: 0.9
      
    # Database persistence
    persistence:
      enabled: true
      batch_size: 100
      flush_interval_seconds: 30
      retention_days: 90
      
    # Council learning bridge
    council_bridge:
      enabled: true
      signal_buffer_size: 500
      flush_interval_seconds: 60
```

### 10. Testing

**File**: `iterations/v3/parallel-workers/tests/learning_integration_tests.rs`

```rust
#[tokio::test]
async fn test_metrics_collection() {
    // Create coordinator with learning enabled
    // Execute parallel task
    // Verify metrics collected
    // Verify worker profiles updated
}

#[tokio::test]
async fn test_adaptive_worker_selection() {
    // Create coordinator with learning
    // Execute multiple similar tasks
    // Verify worker selection improves over time
    // Verify best performers selected more frequently
}

#[tokio::test]
async fn test_pattern_recognition() {
    // Execute diverse set of tasks
    // Verify patterns identified
    // Verify optimal configurations generated
}

#[tokio::test]
async fn test_configuration_optimization() {
    // Execute tasks with suboptimal config
    // Verify optimizer generates recommendations
    // Apply recommendations
    // Verify performance improvement
}

#[tokio::test]
async fn test_council_bridge_integration() {
    // Create coordinator with council bridge
    // Execute parallel tasks
    // Verify signals published to council
    // Verify council learning system receives signals
}

#[tokio::test]
async fn test_learning_persistence() {
    // Execute tasks with persistence enabled
    // Verify data persisted to database
    // Query historical data
    // Verify data integrity
}
```

## Integration Points

1. **Council Learning System** (`iterations/v3/council/src/learning.rs`)

   - Consume parallel worker signals via `CouncilLearningBridge`
   - Integrate parallel worker performance into global learning
   - Provide feedback for optimization

2. **Database Layer** (`iterations/v3/database/`)

   - Add migration for learning tables
   - Implement queries for performance history
   - Support batch operations for efficiency

3. **Worker Pool** (`iterations/v3/workers/`)

   - Expose worker performance metrics
   - Support adaptive worker selection
   - Track worker specialization effectiveness

4. **Configuration** (`iterations/v3/config/`)

   - Add learning configuration section
   - Support dynamic configuration updates
   - Provide optimization controls

## Success Criteria

1. **Metrics Collection**

   - All worker executions recorded with complete metrics
   - Worker performance profiles maintained and updated
   - Execution history persisted to database

2. **Pattern Recognition**

   - Successful patterns identified with >70% confidence
   - Optimal configurations generated for common task patterns
   - Failure patterns detected and analyzed

3. **Adaptive Selection**

   - Worker selection improves over time (measured by success rate)
   - Best performers selected more frequently
   - Exploration/exploitation balance maintained

4. **Configuration Optimization**

   - Optimizer generates actionable recommendations
   - Recommendations improve performance when applied
   - Optimization history tracked and validated

5. **Council Integration**

   - Signals successfully published to council learning
   - Parallel worker data integrated into global learning
   - Feedback loop functional and effective

6. **Performance**

   - Learning overhead <5% of execution time
   - Database queries optimized and indexed
   - Batch operations efficient and reliable

## Documentation

Create comprehensive documentation:

1. **Learning System Guide** (`docs/parallel-workers/LEARNING_SYSTEM.md`)

   - Architecture overview
   - Component descriptions
   - Integration points
   - Configuration options

2. **Optimization Guide** (`docs/parallel-workers/OPTIMIZATION.md`)

   - How to interpret recommendations
   - How to apply optimizations
   - Performance tuning strategies
   - Troubleshooting common issues

3. **Developer Guide** (`docs/parallel-workers/LEARNING_EXTENSION.md`)

   - How to add new metrics
   - How to extend pattern recognition
   - How to implement custom selection strategies
   - How to integrate with other learning systems