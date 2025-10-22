<!-- 73200485-ed97-4951-aaa8-3b4158669149 7dca0712-795b-47a7-92e7-18c5956c9bde -->
# <!-- 73200485-ed97-4951-aaa8-3b4158669149 7dca0712-795b-47a7-92e7-18c5956c9bde -->

# Kimi K2-Inspired Agent Enhancements

## High-level Critiques & Architectural Improvements

### 1. Tool Chains as Typed DAGs, Not Lists

Replace linear `ToolChain { steps: Vec<ToolStep> }` with **typed DAG** using `petgraph` for fan-out/fan-in, joins, retries. Add explicit **data contracts** between steps via **Schema Registry** for type safety and autoconversion.

### 2. Constrained Contextual Bandits for Tool Selection

Replace vanilla Q-learning with **LinUCB/Thompson sampling** using contextual features (task type, tool descriptors). Enforce **hard constraints** (latency, token/cost budgets, compliance) inside the chooser for production safety.

### 3. Tool Schema Registry + Adapter Layer

Tool signatures become **JSON Schema**-backed and versioned. Enables better validation, autoconversion (string→url, csv→table), and safer planning over **typed I/O** vs `serde_json::Value` blobs.

### 4. Context System with Budgets, Not Just Features

Treat context as **budgeted allocator**: allocate tokens across working/episodic/semantic memories using knapsack optimization. Back with **document-level dedup**, **attribution tracking**, and **calibrated retrieval** (recall@k targets).

### 5. MoE Router with Latency/Cost Awareness

Router objective: `R = wq·quality − wl·latency_norm − wt·tokens_norm`. Use **calibrated confidence** (Platt/Isotonic) and enforce **sparse activation** with per-task budgets. Build **shadow router** for offline evaluation.

### 6. Observability and Rollout Discipline First

Bake in Request IDs, plan hashes, propensities, OpenTelemetry spans. **Shadow→Canary→Guarded** with **offline IPS/DR evaluation** gates; provide **auto-rollback** and **circuit breakers** (tower-style).

### 7. Concurrency, Backpressure, Cancellation

Put chain executor on `tokio` with **bounded queues**, **Semaphore** limits per tool, and **CancellationToken** per plan. Keep system healthy under bursty load.

## Phase 1: Enhanced Tool Reasoning (Typed DAG Planning + Constrained Bandits)

### 1.1 Implement Typed DAG Tool Chain Planner

**Location**: `iterations/v3/tool-ecosystem/src/tool_chain_planner.rs`

Replace linear chains with typed DAG using `petgraph` for fan-out/fan-in and joins:

```rust
use petgraph::graph::{Graph, NodeIndex};
use serde_json::Value;

pub type ToolId = String;
pub type PortName = String;

#[derive(Clone, Debug)]
pub struct PortSchemaRef {
    pub registry_key: String,      // e.g. "web.search.Query@v1"
    pub optional: bool,
}

#[derive(Clone, Debug)]
pub struct ToolPort {
    pub name: PortName,
    pub schema: PortSchemaRef,
}

#[derive(Clone, Debug)]
pub struct ToolNode {
    pub tool_id: ToolId,
    pub inputs: Vec<ToolPort>,
    pub outputs: Vec<ToolPort>,
    pub params: serde_json::Value,     // static params (validated)
    pub fallback: Option<ToolId>,
    pub sla_ms: u64,                   // per-step SLO target
    pub cost_hint: f64,                // $ estimate
}

#[derive(Clone, Debug)]
pub struct ToolEdge {
    pub from_port: PortName,
    pub to_port: PortName,
    pub codec: Option<String>,         // optional converter key
}

pub struct ToolChain {
    pub dag: Graph<ToolNode, ToolEdge>,
    pub roots: Vec<NodeIndex>,
    pub sinks: Vec<NodeIndex>,
    pub estimated_cost: f64,
    pub estimated_time_ms: u64,
    pub confidence: f64,
    pub plan_hash: u64,                // blake3 of canonical plan
}
```

**Planning Algorithm**:

- Start with rule-based templates + A* search over tool graph (cost = latency+cost; heuristic from historical latency)
- Use Schema Registry to ensure edges connect only when types match; allow codecs for conversions
- Compile to deterministic execution graph with cancellation, retries, idempotency

**Key Features**:

- Fan-out/fan-in support via DAG structure
- Explicit data contracts between steps
- Schema-validated port connections
- Parallel execution via topological scheduling

### 1.2 Implement Constrained Contextual Bandits for Tool Selection

**Location**: `iterations/v3/tool-ecosystem/src/tool_bandits.rs`

Replace vanilla Q-learning with LinUCB/Thompson sampling using contextual features:

```rust
pub struct ToolBandit {
    alpha: f64,                        // exploration parameter
    theta: HashMap<ToolId, Vec<f64>>,  // weights per tool
    cov:   HashMap<ToolId, Matrix>,    // A^-1 per tool (LinUCB)
}

pub struct ToolContextFeatures {
    pub task_type: String,
    pub prompt_len: usize,
    pub retrieval_k: usize,
    pub is_code_task: bool,
    pub expected_latency_ms: u64,
    pub cost_budget_cents: u32,
    pub risk_tier: u8,
}

pub struct ToolConstraints {
    pub max_latency_ms: u64,
    pub max_cost_cents: u32,
    pub require_caws: bool,
}

impl ToolBandit {
    pub fn select_tool(&self,
        ctx: &ToolContextFeatures,
        tools: &[ToolId],
        constraints: &ToolConstraints
    ) -> (ToolId, f64 /*propensity*/, f64 /*confidence*/) {
        // Score = θᵀx + α·sqrt(xᵀ A⁻¹ x) for LinUCB
        // Reject infeasible arms before scoring
        // Return propensity for IPS/DR evaluation
    }

    pub fn update(&mut self,
        ctx: &ToolContextFeatures,
        tool: &ToolId,
        reward: f64
    ) {
        // Update weights and covariance matrix
    }
}
```

**Key Features**:

- LinUCB/Thompson sampling with contextual features
- Hard constraint enforcement (latency, cost, compliance)
- Propensity logging for offline policy evaluation
- Sample-efficient learning vs vanilla Q-learning

### 1.3 Implement Chain Executor with Concurrency & Cancellation

**Location**: `iterations/v3/tool-ecosystem/src/executor.rs`

Tokio-based executor with bounded queues, semaphores, and cancellation:

```rust
use tokio_util::sync::CancellationToken;
use petgraph::visit::Topo;

pub struct ChainExecutor {
    concurrency: usize,
    limiter: Arc<tokio::sync::Semaphore>,
}

impl ChainExecutor {
    pub async fn execute(
        &self,
        chain: &ToolChain,
        registry: &dyn SchemaRegistry,
        toolbox: &dyn ToolRuntime,
        cancel: CancellationToken,
    ) -> anyhow::Result<HashMap<NodeIndex, serde_json::Value>> {
        let mut results = HashMap::new();
        let mut topo = Topo::new(&chain.dag);

        while let Some(nx) = topo.next(&chain.dag) {
            if cancel.is_cancelled() { anyhow::bail!("cancelled") }

            // Gather inputs by following incoming edges
            let mut inputs = serde_json::Map::new();
            for edge in chain.dag.edges_directed(nx, petgraph::Direction::Incoming) {
                let (from, _to) = (edge.source(), edge.target());
                let from_node = &chain.dag[from];
                let edge_meta = edge.weight();
                let payload = results.get(&from)
                    .ok_or_else(|| anyhow::anyhow!("missing upstream result"))?;
                let v = if let Some(codec) = &edge_meta.codec {
                    registry.convert(codec, &chain.dag[nx].inputs[0].schema.registry_key, payload.clone())?
                } else { payload.clone() };
                inputs.insert(edge_meta.to_port.clone(), v);
            }

            // Run node with per-tool concurrency control
            let _permit = self.limiter.acquire().await?;
            let node = &chain.dag[nx];
            let r = toolbox.call(&node.tool_id, serde_json::Value::Object(inputs)).await?;
            registry.validate(&node.outputs[0].schema.registry_key, &r)?;
            results.insert(nx, r);
        }
        Ok(results)
    }
}
```

**Key Features**:

- Topological scheduling for parallel execution
- Per-tool rate limits and bulkheads
- Circuit breaker per tool with automatic fallback
- Cancellation token propagation

### 1.4 Create Tool Schema Registry

**Location**: `iterations/v3/tool-ecosystem/src/schema_registry.rs`

JSON Schema-backed registry for tool I/O validation and conversion:

```rust
pub trait SchemaRegistry: Send + Sync {
    fn get(&self, key: &str) -> Option<serde_json::Value>;       // JSON Schema
    fn validate(&self, key: &str, value: &serde_json::Value) -> Result<()>;
    fn convert(&self, from: &str, to: &str, v: serde_json::Value) -> Result<serde_json::Value>;
}

pub struct JsonSchemaRegistry {
    schemas: HashMap<String, serde_json::Value>,
    converters: HashMap<String, Box<dyn Converter>>,
}
```

**Integration Points**:

- Connect to existing MCP tool ecosystem (`docs/MCP/README.md`)
- Enable autoconversion between compatible types (HTML→Markdown, CSV→Table)
- Validate all tool inputs/outputs against schemas

**Testing Requirements** (CAWS-compliant):

- Unit tests: Schema validation, type conversion, registry operations
- Integration tests: Full DAG execution with schema validation
- Fault injection tests: Timeouts, partial failures, cancellation
- Coverage: 80%+ line coverage, 90%+ branch coverage

---

## Phase 2: Budgeted Context Management (Knapsack Optimization + Calibration)

### 2.1 Implement Budgeted Context Allocator

**Location**: `iterations/v3/self-prompting-agent/src/context/budget.rs`

Treat context as budgeted allocator with knapsack optimization:

```rust
pub struct ContextBudget {
    pub max_tokens: usize,
    pub headroom: f32, // keep 20% slack
}

pub struct Allocation {
    pub working: usize,
    pub episodic: usize,
    pub semantic: usize,
    pub citations: bool,
}

pub struct ContextCosts {
    pub utility: ContextUtility,
    pub estimated_tokens: ContextTokens,
}

pub struct ContextAllocator;
impl ContextAllocator {
    pub fn allocate(budget: &ContextBudget, est_costs: &ContextCosts) -> Allocation {
        // Lightweight knapsack: maximize predicted utility under token budget
        let cap = (budget.max_tokens as f32 * (1.0 - budget.headroom)).round() as usize;
        let u = est_costs.utility;
        let total = u.working + u.episodic + u.semantic + 1e-6;
        Allocation {
            working: ((u.working / total) * cap as f32) as usize,
            episodic: ((u.episodic / total) * cap as f32) as usize,
            semantic: ((u.semantic / total) * cap as f32) as usize,
            citations: true, // Enable for critical tasks
        }
    }
}
```

**Key Features**:

- Knapsack optimization for token allocation across memories
- Predicted utility weighting for each memory type
- Citations tracking for explainability
- Headroom buffer for safety

### 2.2 Implement Calibrated Retrieval with Deduplication

**Location**: `iterations/v3/self-prompting-agent/src/context/retrieval.rs`

Calibrated retrieval with MinHash deduplication and attribution tracking:

```rust
pub struct CalibratedRetriever {
    vector_search: Arc<VectorSearchEngine>,
    minhash_deduper: MinHashDeduper,
    calibration_tracker: CalibrationTracker,
}

impl CalibratedRetriever {
    pub async fn retrieve_with_calibration(&self,
        query: &str,
        k: usize,
        context_budget: &ContextBudget
    ) -> Result<CalibratedResults> {
        // 1. Retrieve candidates with vector search
        let candidates = self.vector_search.search(query, k * 2).await?;

        // 2. Deduplicate with MinHash/SimHash
        let deduped = self.minhash_deduper.deduplicate(candidates)?;

        // 3. Rank by calibrated precision@k, recall@k
        let calibrated = self.calibration_tracker.rank_by_calibration(&deduped)?;

        // 4. Track attribution for each segment
        let attributed = self.add_attribution(calibrated).await?;

        Ok(CalibratedResults {
            segments: attributed,
            recall_estimate: self.calibration_tracker.estimate_recall(query),
            dedup_stats: deduped.stats,
        })
    }

    async fn add_attribution(&self, segments: Vec<ScoredSegment>) -> Result<Vec<AttributedSegment>> {
        // Add evidence IDs and source citations
        segments.into_iter().map(|seg| {
            Ok(AttributedSegment {
                content: seg.content,
                score: seg.score,
                evidence_id: generate_evidence_id(),
                source_citation: format!("source:{}", seg.source_id),
                timestamp: chrono::Utc::now(),
            })
        }).collect()
    }
}
```

**Key Features**:

- MinHash/SimHash deduplication for better density
- Per-source calibration (precision@k, recall@k)
- Document-level attribution tracking
- Evidence IDs for explainability

### 2.3 Hierarchical Context Manager with Budgeted Allocation

**Location**: `iterations/v3/self-prompting-agent/src/context/manager.rs`

Integrate budgeted allocation with hierarchical memories:

```rust
pub struct HierarchicalContextManager {
    allocator: ContextAllocator,
    retriever: CalibratedRetriever,
    compressor: ContextCompressor,
    working_memory: WorkingMemory,
    episodic_memory: EpisodicMemory,
    semantic_memory: SemanticMemory,
}

impl HierarchicalContextManager {
    pub async fn build_context(&self,
        task: &Task,
        budget: &ContextBudget
    ) -> Result<ContextBundle> {
        // 1. Estimate utility costs for each memory type
        let est_costs = self.estimate_context_costs(task).await?;

        // 2. Allocate tokens using knapsack optimization
        let allocation = self.allocator.allocate(budget, &est_costs);

        // 3. Retrieve from each memory with allocated budgets
        let working = self.working_memory.get_within_budget(allocation.working);
        let episodic = self.episodic_memory.retrieve_similar(task, allocation.episodic).await?;
        let semantic = self.retriever.retrieve_with_calibration(
            &task.description,
            allocation.episodic,
            budget
        ).await?;

        // 4. Compress if needed and track attributions
        let compressed = if self.exceeds_budget(&working, &episodic, &semantic, budget) {
            self.compressor.compress_with_attribution(&working, &episodic, &semantic, budget).await?
        } else {
            ContextBundle::new(working, episodic, semantic, allocation.citations)
        };

        Ok(compressed)
    }

    async fn estimate_context_costs(&self, task: &Task) -> Result<ContextCosts> {
        // Estimate utility and token costs for knapsack allocation
        let utility = ContextUtility {
            working: self.working_memory.estimate_utility(task),
            episodic: self.episodic_memory.estimate_utility(task).await?,
            semantic: self.retriever.estimate_utility(&task.description).await?,
        };

        let tokens = ContextTokens {
            working: self.working_memory.estimate_tokens(),
            episodic: self.episodic_memory.estimate_tokens(),
            semantic: self.retriever.estimate_tokens(),
        };

        Ok(ContextCosts { utility, tokens })
    }
}
```

**Integration Points**:

- Connect `VectorSearchEngine` from `research/src/vector_search.rs`
- Use `ContextBuilder` from `research/src/context_builder.rs`
- Integrate with existing `ContextMonitor` in `loop_controller.rs:88-118`

**Testing Requirements** (CAWS-compliant):

- Unit tests: Budget allocation, compression ratios, deduplication effectiveness
- Integration tests: Full context flow with calibrated retrieval
- Performance tests: Allocation speed (< 10ms), retrieval latency (< 50ms P95)
- Coverage: 80%+ line coverage, 90%+ branch coverage
- Memory tests: Verify no memory leaks, proper attribution tracking

---

## Phase 3: Latency/Cost-Aware MoE Routing (Calibrated Sparse Activation)

### 3.1 Implement Costed Objective Router

**Location**: `iterations/v3/self-prompting-agent/src/models/expert_router.rs`

Router with costed objective and calibrated confidence:

```rust
pub struct CostedRouter {
    router_network: RouterNetwork,
    calibration: PlattCalibrator,  // For confidence calibration
    performance_tracker: PerformanceTracker,
}

impl CostedRouter {
    pub async fn select_experts(&self,
        task: &Task,
        context: &ModelContext,
        budget: &RouterBudget
    ) -> Result<ExpertSelection> {
        // Objective: R = wq·quality − wl·latency_norm − wt·tokens_norm
        let candidates = self.score_all_experts(task, context).await?;

        // Apply calibrated confidence and sparse activation
        let calibrated_scores = candidates.into_iter()
            .map(|(expert, raw_score)| {
                let calibrated_conf = self.calibration.calibrate(raw_score);
                (expert, calibrated_conf)
            })
            .filter(|(_, conf)| *conf > budget.min_confidence)
            .collect::<Vec<_>>();

        // Select Top-1 by default, Top-k only if uplift justifies
        let selection = if calibrated_scores.len() <= 1 {
            calibrated_scores
        } else {
            self.select_sparse_activated(&calibrated_scores, budget)?
        };

        Ok(ExpertSelection {
            experts: selection.into_iter().map(|(e, _)| e).collect(),
            propensities: selection.into_iter().map(|(_, c)| c).collect(),
            expected_cost: self.estimate_cost(&selection),
            expected_latency: self.estimate_latency(&selection),
        })
    }

    fn select_sparse_activated(&self,
        candidates: &[(ExpertId, f64)],
        budget: &RouterBudget
    ) -> Result<Vec<(ExpertId, f64)>> {
        // Top-1 unless expected ensemble uplift > threshold
        let top1 = candidates[0];
        let ensemble = &candidates[0..budget.max_ensemble_size.min(candidates.len())];

        let uplift = self.estimate_ensemble_uplift(ensemble)?;
        if uplift > budget.ensemble_uplift_threshold {
            Ok(ensemble.to_vec())
        } else {
            Ok(vec![top1])
        }
    }
}
```

**Key Features**:

- Costed objective balancing quality vs latency/tokens
- Platt/Isotonic calibration for meaningful confidence
- Sparse activation (Top-1 default, Top-k with uplift justification)
- Propensity logging for offline IPS/DR evaluation

### 3.2 Shadow Router for Offline Evaluation

**Location**: `iterations/v3/self-prompting-agent/src/models/shadow_router.rs`

Shadow router for offline IPS/DR evaluation before canary deployment:

```rust
pub struct ShadowRouter {
    shadow_network: RouterNetwork,
    logged_decisions: Vec<ShadowDecision>,
}

impl ShadowRouter {
    pub async fn shadow_decision(&mut self,
        task: &Task,
        context: &ModelContext,
        live_selection: &ExpertSelection
    ) -> Result<()> {
        // Run shadow router in parallel
        let shadow_selection = self.shadow_network.select_experts(task, context).await?;

        // Log for offline IPS/DR evaluation
        self.logged_decisions.push(ShadowDecision {
            request_id: generate_request_id(),
            task_hash: hash_task(task),
            live_experts: live_selection.experts.clone(),
            live_propensity: live_selection.propensities[0],
            shadow_experts: shadow_selection.experts,
            shadow_propensity: shadow_selection.propensities[0],
            timestamp: chrono::Utc::now(),
        });

        Ok(())
    }

    pub fn export_logs(&self) -> Vec<ShadowDecision> {
        self.logged_decisions.clone()
    }
}

// Offline IPS/DR evaluation
pub struct OfflineEvaluator;
impl OfflineEvaluator {
    pub fn estimate_uplift(&self, logs: &[ShadowDecision], outcomes: &[TaskOutcome]) -> f64 {
        // IPS: E[outcome | shadow] / E[outcome | live] weighted by propensities
        // DR: Doubly robust estimation for unbiased uplift
        // Return estimated improvement from shadow policy
    }
}
```

**Key Features**:

- Shadow routing for offline policy evaluation
- IPS/DR estimation for causal uplift measurement
- Request ID and task hash tracking for joining logs

### 3.3 Consensus with Abstention

**Location**: `iterations/v3/self-prompting-agent/src/models/consensus.rs`

Consensus engine that handles abstentions and ties:

```rust
pub struct ConsensusEngine {
    voting_strategy: VotingStrategy,
    quality_weights: HashMap<String, f64>,
    abstain_threshold: f64,  // Minimum confidence to vote
}

pub enum VotingStrategy {
    Majority,
    QualityWeighted,
    Unanimous { fallback: FallbackStrategy },
}

pub enum FallbackStrategy {
    CheapestHighConfidence,
    WeightedByCost,
    RandomTopK { k: usize },
}

impl ConsensusEngine {
    pub fn build_consensus(&self,
        expert_responses: Vec<(ExpertId, ModelResponse, f64 /*confidence*/)>
    ) -> Result<ModelResponse> {
        // Filter out abstentions (low confidence)
        let valid_votes = expert_responses.into_iter()
            .filter(|(_, _, conf)| *conf >= self.abstain_threshold)
            .collect::<Vec<_>>();

        match &self.voting_strategy {
            VotingStrategy::QualityWeighted => {
                self.weighted_vote(&valid_votes)
            }
            VotingStrategy::Unanimous { fallback } => {
                if self.all_agree(&valid_votes) && !valid_votes.is_empty() {
                    Ok(valid_votes[0].1.clone())
                } else {
                    self.apply_fallback(fallback, &valid_votes)
                }
            }
            _ => self.majority_vote(&valid_votes)
        }
    }

    fn apply_fallback(&self,
        fallback: &FallbackStrategy,
        votes: &[(ExpertId, ModelResponse, f64)]
    ) -> Result<ModelResponse> {
        match fallback {
            FallbackStrategy::CheapestHighConfidence => {
                // Select cheapest expert above confidence threshold
                votes.iter()
                    .filter(|(_, _, conf)| *conf >= self.abstain_threshold)
                    .min_by_key(|(id, _, _)| self.get_cost(id))
                    .map(|(_, resp, _)| resp.clone())
                    .ok_or_else(|| anyhow::anyhow!("No eligible experts"))
            }
            _ => self.weighted_vote(votes) // Default fallback
        }
    }
}
```

**Key Features**:

- Abstention handling for uncertain experts
- Fallback to cheapest high-confidence expert
- Quality-weighted voting for consensus

**Testing Requirements** (CAWS-compliant):

- Unit tests: Router calibration, sparse activation, consensus with abstentions
- Integration tests: Shadow router logging, offline IPS/DR evaluation
- Performance tests: Router overhead (< 10ms), ensemble coordination
- Coverage: 80%+ line coverage, 90%+ branch coverage
- Offline evaluation: IPS/DR uplift estimation accuracy

---

## Cross-Cutting: Data Contracts, Safety, Observability

### Schema Registry + Adapter Layer

**Location**: `iterations/v3/tool-ecosystem/src/schema_registry.rs`

Single source of truth for tool/model I/O with autoconversion:

```rust
pub trait SchemaRegistry: Send + Sync {
    fn get(&self, key: &str) -> Option<serde_json::Value>;       // JSON Schema
    fn validate(&self, key: &str, value: &serde_json::Value) -> Result<()>;
    fn convert(&self, from: &str, to: &str, v: serde_json::Value) -> Result<serde_json::Value>;
}

pub struct JsonSchemaRegistry {
    schemas: HashMap<String, serde_json::Value>,
    converters: HashMap<String, Box<dyn Converter>>,
}
```

### Safety & CAWS Integration

**Location**: `iterations/v3/tool-ecosystem/src/safety.rs`

Pre-action constraint checks and CAWS compliance:

```rust
pub struct SafetyChecker {
    caws_validator: CawsValidator,
    compliance_rules: Vec<ComplianceRule>,
}

impl SafetyChecker {
    pub async fn validate_plan(&self, plan: &ToolChain) -> Result<()> {
        // Pre-action constraint check on every plan
        self.validate_latency_budget(plan)?;
        self.validate_cost_budget(plan)?;
        self.validate_compliance(plan)?;
        self.caws_validator.validate(plan).await?;
        Ok(())
    }

    fn validate_compliance(&self, plan: &ToolChain) -> Result<()> {
        // Check CAWS requirements, domain allowlists, etc.
        for rule in &self.compliance_rules {
            if !rule.check(plan) {
                anyhow::bail!("Compliance violation: {}", rule.description);
            }
        }
        Ok(())
    }
}
```

### Observability & Rollout Discipline

**Location**: `iterations/v3/tool-ecosystem/src/observability.rs`

OpenTelemetry spans, request tracking, and circuit breakers:

```rust
pub struct ObservabilityLayer {
    tracer: opentelemetry::Tracer,
    request_id: String,
    plan_hash: u64,
    propensity: f64,
}

impl ObservabilityLayer {
    pub fn span(&self, name: &str) -> opentelemetry::Span {
        self.tracer.span_builder(name)
            .with_attribute("request_id", self.request_id.clone())
            .with_attribute("plan_hash", self.plan_hash.to_string())
            .with_attribute("propensity", self.propensity.to_string())
            .start(&self.tracer)
    }
}

pub struct CircuitBreaker {
    state: CircuitState,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

pub enum CircuitState {
    Closed,
    Open { until: Instant },
    HalfOpen,
}
```

---

## Concrete Code Drop-Ins

### 1) LinUCB Bandit Policy Interface

**Location**: `iterations/v3/tool-ecosystem/src/tool_bandits.rs`

```rust
pub trait ToolPolicy: Send + Sync {
    fn select_tool(&self, ctx: &ToolContextFeatures, tools: &[ToolId], constraints: &ToolConstraints)
        -> (ToolId, f64 /*propensity*/, f64 /*confidence*/);
    fn update(&mut self, ctx: &ToolContextFeatures, tool: &ToolId, reward: f64);
    fn version(&self) -> &'static str;
}

// LinUCB implementation
pub struct LinUCBPolicy {
    alpha: f64,
    theta: HashMap<ToolId, Vec<f64>>,
    cov: HashMap<ToolId, Matrix>,
}

impl ToolPolicy for LinUCBPolicy {
    fn select_tool(&self, ctx: &ToolContextFeatures, tools: &[ToolId], constraints: &ToolConstraints)
        -> (ToolId, f64, f64) {
        // Feature vector from context
        let x = self.extract_features(ctx);

        // Score each feasible tool
        let mut best_tool = None;
        let mut best_score = f64::NEG_INFINITY;
        let mut best_propensity = 0.0;

        for tool_id in tools {
            if !self.satisfies_constraints(tool_id, constraints) {
                continue; // Reject infeasible tools
            }

            let theta_t = &self.theta[tool_id];
            let cov_t = &self.cov[tool_id];

            // LinUCB score: θᵀx + α·sqrt(xᵀ A⁻¹ x)
            let mean = theta_t.iter().zip(&x).map(|(t, x)| t * x).sum::<f64>();
            let variance = self.compute_variance(&x, cov_t);
            let score = mean + self.alpha * variance.sqrt();

            if score > best_score {
                best_score = score;
                best_tool = Some(tool_id.clone());
                best_propensity = self.compute_propensity(score);
            }
        }

        (best_tool.unwrap(), best_propensity, best_score)
    }

    fn update(&mut self, ctx: &ToolContextFeatures, tool: &ToolId, reward: f64) {
        // Update θ and A^-1 using Sherman-Morrison
        let x = self.extract_features(ctx);
        // ... matrix updates for LinUCB
    }

    fn version(&self) -> &'static str { "linucb-v1" }
}
```

### 2) Context Budget Allocator

**Location**: `iterations/v3/self-prompting-agent/src/context/budget.rs`

```rust
pub struct ContextAllocator;
impl ContextAllocator {
    pub fn allocate(budget: &ContextBudget, est_costs: &ContextCosts) -> Allocation {
        // Simple heuristic: ensure headroom, then proportional to utility weights
        let cap = (budget.max_tokens as f32 * (1.0 - budget.headroom)).round() as usize;
        let u = est_costs.utility;
        let total = u.working + u.episodic + u.semantic + 1e-6;
        Allocation {
            working: ((u.working / total) * cap as f32) as usize,
            episodic: ((u.episodic / total) * cap as f32) as usize,
            semantic: ((u.semantic / total) * cap as f32) as usize,
            citations: true,
        }
    }
}
```

---

## Phase 4: Integration and Optimization

### 4.1 Wire Up Components

**Location**: `iterations/v3/self-prompting-agent/src/lib.rs` and `agent.rs`

Connect all new modules to existing `SelfPromptingAgent`:

```rust
pub struct SelfPromptingAgent {
    // Existing fields...
    
    // New components
    tool_chain_planner: Arc<ToolChainPlanner>,
    context_manager: Arc<HierarchicalContextManager>,
    expert_router: Arc<ExpertSelectionRouter>,
    tool_learning_system: Arc<Mutex<ToolLearningSystem>>,
}

impl SelfPromptingAgent {
    pub async fn execute_task_enhanced(&self, task: Task) -> Result<TaskResult> {
        // 1. Plan tool chain for task
        let tool_chain = self.tool_chain_planner.plan_chain(&task).await?;
        
        // 2. Select expert models via MoE routing
        let experts = self.expert_router.select_experts(&task, &context).await?;
        
        // 3. Build hierarchical context from RAG
        let enhanced_context = self.context_manager.build_context(&task).await?;
        
        // 4. Execute with loop controller
        self.loop_controller.execute_with_enhanced_context(
            task, 
            tool_chain, 
            experts, 
            enhanced_context
        ).await
    }
}
```

### 4.2 Update Loop Controller

**Location**: `iterations/v3/self-prompting-agent/src/loop_controller.rs`

Enhance existing `SelfPromptingLoop` to use new capabilities:

- Replace context overload checks (lines 455-706) with intelligent compression
- Integrate tool chain execution into iteration cycle
- Add expert model switching based on task phase
- Update context metrics to track semantic relevance, not just token count

### 4.3 Configuration and Feature Flags

**Location**: `iterations/v3/self-prompting-agent/Cargo.toml` and `config.rs`

Add feature flags for gradual rollout:

```toml
[features]
default = []
enhanced-tools = ["tool-chain-planning", "rl-tool-selection"]
extended-context = ["hierarchical-memory", "context-compression"]
moe-routing = ["expert-selection", "sparse-computation"]
full-enhancements = ["enhanced-tools", "extended-context", "moe-routing"]
```

---

## Testing Upgrades (CAWS-Grade)

- **Plan validation**: Type-checking of every edge against Schema Registry; reject cycles; detect unbound inputs.
- **Executor**: Fault-injection tests (timeouts, partial failures, retries, cancellation).
- **Bandit**: Offline IPS/DR tests with logged propensities; assert uplift lower-bound > 0 before canary.
- **Context**: Retrieval precision/recall fixtures; compression quality against human gold summaries.
- **Router**: Calibration curves; budget rules (never exceed token/latency caps).

---

## Phase 5: Testing and Benchmarking (CAWS Production Standards)

### 5.1 Comprehensive Test Suite

**Location**: `iterations/v3/self-prompting-agent/tests/kimi_k2_integration_tests.rs`

Production-grade test coverage:

```rust
#[cfg(test)]
mod kimi_k2_tests {
    // Tool Chain Planning Tests
    #[tokio::test]
    async fn test_tool_chain_dependency_resolution() { }
    
    #[tokio::test]
    async fn test_tool_chain_cost_optimization() { }
    
    // Context Management Tests
    #[tokio::test]
    async fn test_hierarchical_context_retrieval() { }
    
    #[tokio::test]
    async fn test_context_compression_quality() { }
    
    // MoE Routing Tests
    #[tokio::test]
    async fn test_expert_selection_accuracy() { }
    
    #[tokio::test]
    async fn test_consensus_building() { }
    
    // Integration Tests
    #[tokio::test]
    async fn test_end_to_end_enhanced_execution() { }
}
```

### 5.2 Performance Benchmarks

**Location**: `iterations/v3/self-prompting-agent/benches/kimi_k2_benchmarks.rs`

Establish performance baselines:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_tool_chain_planning(c: &mut Criterion) {
    c.bench_function("tool_chain_plan_10_tools", |b| {
        b.iter(|| {
            // Benchmark tool chain planning overhead
        })
    });
}

fn benchmark_context_retrieval(c: &mut Criterion) {
    c.bench_function("context_retrieval_latency", |b| {
        b.iter(|| {
            // Benchmark semantic search + compression
        })
    });
}

fn benchmark_expert_selection(c: &mut Criterion) {
    c.bench_function("expert_router_overhead", |b| {
        b.iter(|| {
            // Benchmark MoE routing latency
        })
    });
}

criterion_group!(benches, 
    benchmark_tool_chain_planning,
    benchmark_context_retrieval,
    benchmark_expert_selection
);
criterion_main!(benches);
```

### 5.3 SWE-Bench Style Evaluations

**Location**: `iterations/v3/self-prompting-agent/evaluations/`

Create agentic performance benchmarks:

- Task completion rate across different complexity levels
- Code quality metrics (linting, type safety, test coverage)
- Tool usage efficiency (optimal vs actual tool chains)
- Context utilization effectiveness (relevant vs total context)

**Target Metrics** (inspired by Kimi K2):

- Task completion rate: > 60% on complex multi-step tasks
- Tool chain optimality: > 80% match to optimal chains
- Context relevance: > 70% of retrieved context used in solution
- Multi-model consensus accuracy: > 85% agreement on correct solutions

---

## Phase 6: Documentation and Migration Guide

### 6.1 Update Documentation

**Files to Update**:

- `iterations/v3/README.md` - Add Kimi K2 enhancements section
- `docs/MULTI_MODEL_AI_SYSTEM.md` - Document MoE routing patterns
- `docs/MCP/README.md` - Update with tool chain planning capabilities
- Create new: `docs/KIMI_K2_ARCHITECTURE.md` - Detailed architecture guide

### 6.2 Migration Guide

**Location**: `docs/migrations/KIMI_K2_MIGRATION.md`

Guide for enabling new features:

```markdown
# Migrating to Kimi K2-Inspired Enhancements

## Step 1: Enable Feature Flags
\`\`\`toml
[features]
default = ["enhanced-tools", "extended-context"]
\`\`\`

## Step 2: Configure Context Manager
\`\`\`rust
let context_config = ContextConfig {
    vector_db_url: "http://localhost:6333",
    max_working_memory_tokens: 8192,
    episodic_memory_limit: 100,
};
\`\`\`

## Step 3: Initialize Tool Chain Planner
...
```

---

## Implementation Order

1. **Phase 1** (Enhanced Tool Reasoning) - 1-2 weeks

   - Most impactful for immediate agent capabilities
   - Builds on existing MCP infrastructure

2. **Phase 2** (Extended Context) - 2-3 weeks

   - Leverages existing RAG components
   - Critical for handling complex tasks

3. **Phase 3** (MoE Routing) - 1-2 weeks

   - Enhances existing ModelRegistry
   - Enables intelligent model selection

4. **Phase 4** (Integration) - 1 week

   - Wire everything together
   - End-to-end testing

5. **Phase 5** (Testing) - 1-2 weeks

   - Production-grade test suite
   - Performance benchmarking

6. **Phase 6** (Documentation) - 3-5 days

   - Comprehensive documentation
   - Migration guides

**Total Estimated Time**: 6-10 weeks for full implementation

**Deliverables**:

- 6 new Rust modules with full test coverage
- 3 enhanced existing modules
- Comprehensive benchmark suite
- Complete documentation package
- Migration guide for gradual adoption

---

## Rollout & Operations

- **Shadow mode** by default; store `plan_hash`, `policy_version`, `propensity`; compute IPS uplift nightly.
- **Canary** at ≤5% with SLO guardrails (p95/p99 latency, error rate); **auto-rollback** on breach.
- **Drift monitor** over task distribution (JS divergence); freeze planner if drift > threshold.
- **Dashboards**: Plan success rate by plan_hash; tool health (breaker states); context budget usage vs utility; router calibration plots.

#### Phase 1: Tool Reasoning (DAG + Constrained Bandits)

- [ ] Implement Typed DAG Tool Chain Planner with petgraph (fan-out/fan-in, joins, retries)
- [ ] Create PortSchemaRef and ToolNode/Edge structures with schema validation
- [ ] Implement A* planning algorithm with cost/latency heuristics
- [ ] Build Chain Executor with tokio concurrency, semaphores, cancellation
- [ ] Create Tool Schema Registry trait with JSON Schema validation and autoconversion
- [ ] Implement Constrained Contextual Bandits (LinUCB/Thompson) with hard constraints
- [ ] Add propensity logging for offline IPS/DR evaluation
- [ ] Write tests: DAG validation, executor fault-injection, bandit IPS/DR evaluation

#### Phase 2: Context Management (Budgeted Knapsack + Calibration)

- [ ] Build ContextAllocator with knapsack optimization for token allocation
- [ ] Implement CalibratedRetriever with MinHash deduplication and attribution tracking
- [ ] Create HierarchicalContextManager with budgeted allocation across memories
- [ ] Add document-level deduplication and evidence ID tracking
- [ ] Integrate per-source calibration (precision@k, recall@k)
- [ ] Write tests: Budget allocation, deduplication effectiveness, attribution tracking

#### Phase 3: MoE Routing (Costed Objective + Sparse Activation)

- [ ] Implement CostedRouter with latency/cost-aware objective function
- [ ] Add Platt/Isotonic calibration for meaningful confidence scores
- [ ] Build ShadowRouter for offline IPS/DR evaluation
- [ ] Create ConsensusEngine with abstention handling and fallback strategies
- [ ] Implement sparse activation (Top-1 default, Top-k with uplift justification)
- [ ] Write tests: Router calibration curves, offline evaluation, consensus accuracy

#### Cross-Cutting: Safety, Observability, Data Contracts

- [ ] Implement Schema Registry with autoconversion (HTML→Markdown, CSV→Table)
- [ ] Build SafetyChecker with pre-action constraint validation and CAWS compliance
- [ ] Create ObservabilityLayer with OpenTelemetry spans and circuit breakers
- [ ] Add Request ID, plan hash, and propensity tracking everywhere
- [ ] Implement circuit breaker pattern for tool/model calls

#### Phase 4: Integration & Testing

- [ ] Wire up all components in SelfPromptingAgent (DAG planner, bandits, context allocator, MoE router)
- [ ] Add Cargo feature flags for gradual rollout (enhanced-tools, extended-context, moe-routing)
- [ ] Write CAWS-grade tests: Plan validation, fault injection, IPS/DR evaluation
- [ ] Create performance benchmarks (planning latency, retrieval speed, router overhead)

#### Phase 5: Documentation & Rollout

- [ ] Update all documentation with new architecture (DAG chains, budgeted context, costed routing)
- [ ] Create migration guide with step-by-step rollout (Shadow→Canary→Guarded)
- [ ] Implement auto-rollback and drift monitoring (JS divergence on task distribution)
- [ ] Build dashboards for plan success rates, tool health, budget utilization, calibration plots

### To-dos

- [ ] Create ToolChainPlanner module with dependency resolution, cost optimization, and fallback chain generation
- [ ] Implement RL-based tool selection with Q-learning updates and epsilon-greedy exploration
- [ ] Upgrade MCP server to support tool chain execution with inter-tool data passing and automatic fallbacks
- [ ] Write comprehensive tests for tool chain planning (80%+ line coverage, 90%+ branch coverage)
- [ ] Create HierarchicalContextManager integrating VectorSearchEngine and ContextBuilder from research module
- [ ] Implement ContextCompressor with importance scoring and intelligent summarization
- [ ] Replace ModelRegistry TODO with intelligent context normalization using HierarchicalContextManager
- [ ] Write tests for context management (compression ratios, retrieval accuracy, memory leak detection)
- [ ] Build ExpertSelectionRouter with MoE-style activation policies (TopK, ThresholdBased, Dynamic)
- [ ] Implement SparseRouter with lightweight classifier for expert prediction and cache optimization
- [ ] Create ConsensusBuilder with multiple voting strategies (Majority, QualityWeighted, Unanimous)
- [ ] Write tests for MoE routing (expert scoring, activation policies, consensus accuracy)
- [ ] Wire up all new components in SelfPromptingAgent with execute_task_enhanced method
- [ ] Enhance SelfPromptingLoop to use tool chains, expert routing, and hierarchical context
- [ ] Add Cargo feature flags for gradual rollout (enhanced-tools, extended-context, moe-routing)
- [ ] Write end-to-end integration tests for enhanced agent execution flow
- [ ] Create performance benchmarks with criterion (tool planning, context retrieval, expert routing)
- [ ] Implement SWE-bench style evaluations with target metrics (60%+ completion, 80%+ tool optimality)
- [ ] Update all documentation (README, architecture guides, MCP docs) with Kimi K2 enhancements
- [ ] Create migration guide with step-by-step instructions for enabling new features