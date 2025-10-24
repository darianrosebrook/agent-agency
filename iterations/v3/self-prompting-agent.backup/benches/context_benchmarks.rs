//! Performance benchmarks for Context Management components
//!
//! Uses Criterion for accurate latency measurements and regression detection.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;

use context::{
    manager::HierarchicalContextManager,
    budget::{ContextAllocator, ContextBudget},
    retriever::CalibratedRetriever,
    compressor::ContextCompressor,
};

/// Benchmark context budget allocation
fn bench_context_allocation(c: &mut Criterion) {
    let budget = ContextBudget {
        max_tokens: 2000,
        headroom: 0.2,
    };

    let costs = ContextCosts {
        utility: ContextUtility {
            working: 0.4,
            episodic: 0.4,
            semantic: 0.2,
        },
        tokens: ContextTokens {
            working: 500,
            episodic: 800,
            semantic: 700,
        },
    };

    c.bench_function("context_budget_allocation", |b| {
        b.iter(|| {
            let allocation = ContextAllocator::allocate(black_box(&budget), black_box(&costs));
            black_box(allocation);
        });
    });
}

/// Benchmark context compression
fn bench_context_compression(c: &mut Criterion) {
    let compressor = ContextCompressor::new(0.7); // 70% compression target

    let working_memory = vec![
        "Current task: implement user authentication".to_string(),
        "Local variables: userId, password, token".to_string(),
        "Recent changes: added password hashing".to_string(),
    ];

    let episodic_memory = vec![
        "Previous login implementation used bcrypt".to_string(),
        "Security audit recommended argon2".to_string(),
    ];

    let semantic_memory = vec![
        "Authentication best practices".to_string(),
        "Password security guidelines".to_string(),
        "JWT token standards".to_string(),
    ];

    c.bench_function("context_compression", |b| {
        b.iter(|| {
            let compressed = compressor.compress_with_attribution(
                black_box(&working_memory),
                black_box(&episodic_memory),
                black_box(&semantic_memory),
                black_box(&ContextBudget { max_tokens: 1000, headroom: 0.1 }),
            );
            black_box(compressed);
        });
    });
}

/// Benchmark calibrated retrieval
fn bench_calibrated_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let retriever = rt.block_on(async {
        CalibratedRetriever::new(
            Arc::new(MockVectorSearchEngine::new()),
            MinHashDeduper::new(0.8),
            CalibrationTracker::new(),
        ).await
    });

    let query = "implement secure password authentication";
    let budget = ContextBudget {
        max_tokens: 1500,
        headroom: 0.1,
    };

    c.bench_function("calibrated_retrieval", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(retriever.retrieve_with_calibration(
                black_box(query),
                black_box(10), // k=10
                black_box(&budget),
            ));
            black_box(result);
        });
    });
}

/// Benchmark hierarchical context building
fn bench_hierarchical_context_building(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = rt.block_on(async {
        HierarchicalContextManager::new(
            ContextAllocator,
            Arc::new(CalibratedRetriever::new(
                Arc::new(MockVectorSearchEngine::new()),
                MinHashDeduper::new(0.8),
                CalibrationTracker::new(),
            ).await),
            ContextCompressor::new(0.7),
            Arc::new(MockWorkingMemory::new()),
            Arc::new(MockEpisodicMemory::new()),
            Arc::new(MockSemanticMemory::new()),
        )
    });

    let task = Task {
        id: "auth_implementation".to_string(),
        description: "Implement secure user authentication with password hashing".to_string(),
        task_type: "coding".to_string(),
        complexity: 3,
        required_capabilities: vec!["security".to_string(), "authentication".to_string()],
        priority: 1,
        created_at: chrono::Utc::now(),
        deadline: None,
    };

    let budget = ContextBudget {
        max_tokens: 2000,
        headroom: 0.15,
    };

    c.bench_function("hierarchical_context_building", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(manager.build_context(
                black_box(&task),
                black_box(&budget),
            ));
            black_box(result);
        });
    });
}

/// Benchmark deduplication performance
fn bench_deduplication(c: &mut Criterion) {
    let deduper = MinHashDeduper::new(0.8);

    let segments = vec![
        "The quick brown fox jumps over the lazy dog.".to_string(),
        "A quick brown fox jumps over a lazy dog.".to_string(),
        "The weather is nice today.".to_string(),
        "Programming requires attention to detail.".to_string(),
        "The weather is beautiful today.".to_string(),
        "Software development involves problem solving.".to_string(),
    ];

    c.bench_function("minhash_deduplication", |b| {
        b.iter(|| {
            let deduped = deduper.deduplicate(black_box(segments.clone()));
            black_box(deduped);
        });
    });
}

/// Benchmark utility estimation
fn bench_utility_estimation(c: &mut Criterion) {
    let task = Task {
        id: "complex_task".to_string(),
        description: "Build a distributed system with consensus and fault tolerance".to_string(),
        task_type: "architecture".to_string(),
        complexity: 5,
        required_capabilities: vec![
            "distributed_systems".to_string(),
            "consensus".to_string(),
            "fault_tolerance".to_string(),
        ],
        priority: 2,
        created_at: chrono::Utc::now(),
        deadline: None,
    };

    c.bench_function("context_utility_estimation", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async {
                let working_utility = MockWorkingMemory::new().estimate_utility(black_box(&task));
                let episodic_utility = MockEpisodicMemory::new().estimate_utility(black_box(&task)).await;
                let semantic_utility = MockVectorSearchEngine::new().estimate_utility(black_box(&task.description.as_str())).await;

                ContextCosts {
                    utility: ContextUtility {
                        working: working_utility,
                        episodic: episodic_utility,
                        semantic: semantic_utility,
                    },
                    tokens: ContextTokens {
                        working: 300,
                        episodic: 500,
                        semantic: 700,
                    },
                }
            });
            black_box(result);
        });
    });
}

/// Benchmark memory leak detection
fn bench_memory_usage_tracking(c: &mut Criterion) {
    let mut memory_tracker = MemoryTracker::new();

    c.bench_function("memory_usage_tracking", |b| {
        b.iter(|| {
            // Simulate memory operations
            memory_tracker.allocate(black_box(1024)); // 1KB
            memory_tracker.allocate(black_box(2048)); // 2KB
            let usage = memory_tracker.current_usage();
            memory_tracker.deallocate(black_box(512)); // Free some
            black_box(usage);
        });
    });
}

/// Benchmark attribution tracking
fn bench_attribution_tracking(c: &mut Criterion) {
    let mut attribution_tracker = AttributionTracker::new();

    c.bench_function("attribution_tracking", |b| {
        b.iter(|| {
            let segment = AttributedSegment {
                content: "Security best practice: use argon2 for password hashing".to_string(),
                score: 0.85,
                evidence_id: generate_evidence_id(),
                source_citation: "source:security_guidelines".to_string(),
                timestamp: chrono::Utc::now(),
            };

            attribution_tracker.add_segment(black_box(segment));
            let segments = attribution_tracker.get_segments();
            black_box(segments);
        });
    });
}

// Mock implementations for benchmarking
struct MockVectorSearchEngine;

impl MockVectorSearchEngine {
    fn new() -> Self {
        Self
    }

    async fn search(&self, _query: &str, k: usize) -> Vec<ScoredSegment> {
        (0..k).map(|i| ScoredSegment {
            content: format!("Mock result {}", i),
            score: 0.9 - (i as f64 * 0.05),
            source_id: format!("source_{}", i),
        }).collect()
    }

    async fn estimate_utility(&self, _query: &str) -> f64 {
        0.75 // Mock utility
    }
}

struct MockWorkingMemory;

impl MockWorkingMemory {
    fn new() -> Self {
        Self
    }

    fn estimate_utility(&self, _task: &Task) -> f64 {
        0.8 // High utility for working memory
    }

    fn get_within_budget(&self, _budget: usize) -> Vec<String> {
        vec![
            "Current implementation details".to_string(),
            "Local state and variables".to_string(),
        ]
    }

    fn estimate_tokens(&self) -> usize {
        200
    }
}

struct MockEpisodicMemory;

impl MockEpisodicMemory {
    fn new() -> Self {
        Self
    }

    async fn estimate_utility(&self, _task: &Task) -> f64 {
        0.6 // Medium utility for episodic
    }

    async fn retrieve_similar(&self, _task: &Task, _budget: usize) -> Vec<String> {
        vec![
            "Previous similar implementation".to_string(),
            "Related debugging experience".to_string(),
        ]
    }

    fn estimate_tokens(&self) -> usize {
        300
    }
}

struct MockSemanticMemory;

impl MockSemanticMemory {
    fn new() -> Self {
        Self
    }
}

// Additional mock structures
struct MinHashDeduper {
    threshold: f64,
}

impl MinHashDeduper {
    fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    fn deduplicate(&self, segments: Vec<String>) -> DeduplicationResult {
        // Simple mock deduplication - remove duplicates
        let mut unique = std::collections::HashSet::new();
        let mut deduped = vec![];

        for segment in segments {
            if unique.insert(segment.clone()) {
                deduped.push(segment);
            }
        }

        DeduplicationResult {
            segments: deduped,
            stats: DeduplicationStats {
                original_count: segments.len(),
                deduped_count: unique.len(),
                compression_ratio: unique.len() as f64 / segments.len() as f64,
            },
        }
    }
}

struct CalibrationTracker;

impl CalibrationTracker {
    fn new() -> Self {
        Self
    }

    fn rank_by_calibration(&self, segments: &[ScoredSegment]) -> Vec<ScoredSegment> {
        // Mock calibration - just return as-is with slight adjustment
        segments.iter().cloned().collect()
    }

    fn estimate_recall(&self, _query: &str) -> f64 {
        0.85 // Mock recall estimate
    }
}

struct MemoryTracker {
    allocated: usize,
}

impl MemoryTracker {
    fn new() -> Self {
        Self { allocated: 0 }
    }

    fn allocate(&mut self, bytes: usize) {
        self.allocated += bytes;
    }

    fn deallocate(&mut self, bytes: usize) {
        self.allocated = self.allocated.saturating_sub(bytes);
    }

    fn current_usage(&self) -> usize {
        self.allocated
    }
}

struct AttributionTracker {
    segments: Vec<AttributedSegment>,
}

impl AttributionTracker {
    fn new() -> Self {
        Self { segments: vec![] }
    }

    fn add_segment(&mut self, segment: AttributedSegment) {
        self.segments.push(segment);
    }

    fn get_segments(&self) -> &[AttributedSegment] {
        &self.segments
    }
}

fn generate_evidence_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("evidence_{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

// Import required types (these would be defined in the actual modules)
#[derive(Clone, Debug)]
struct Task {
    id: String,
    description: String,
    task_type: String,
    complexity: u8,
    required_capabilities: Vec<String>,
    priority: u8,
    created_at: chrono::DateTime<chrono::Utc>,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug)]
struct ScoredSegment {
    content: String,
    score: f64,
    source_id: String,
}

#[derive(Clone, Debug)]
struct AttributedSegment {
    content: String,
    score: f64,
    evidence_id: String,
    source_citation: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug)]
struct ContextCosts {
    utility: ContextUtility,
    tokens: ContextTokens,
}

#[derive(Clone, Debug)]
struct ContextUtility {
    working: f64,
    episodic: f64,
    semantic: f64,
}

#[derive(Clone, Debug)]
struct ContextTokens {
    working: usize,
    episodic: usize,
    semantic: usize,
}

struct DeduplicationResult {
    segments: Vec<String>,
    stats: DeduplicationStats,
}

#[derive(Clone, Debug)]
struct DeduplicationStats {
    original_count: usize,
    deduped_count: usize,
    compression_ratio: f64,
}

criterion_group!(
    benches,
    bench_context_allocation,
    bench_context_compression,
    bench_calibrated_retrieval,
    bench_hierarchical_context_building,
    bench_deduplication,
    bench_utility_estimation,
    bench_memory_usage_tracking,
    bench_attribution_tracking,
);
criterion_main!(benches);
