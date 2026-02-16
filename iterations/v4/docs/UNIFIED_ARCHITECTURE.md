# V4 Unified Architecture

**Version**: 2.1.0
**Last Updated**: 2026-01-25
**Status**: Architecture Specification

## Executive Summary

V4 integrates architectural patterns from five complementary projects:

| Project | Contribution to V4 |
|---------|-------------------|
| **V3** | Orchestration patterns, constitutional council, 5D evaluation |
| **Sterling** | Symbolic reasoning, semantic parsing, operator taxonomy (S/M/P/K/C) |
| **Distill** | CAWS governance gates, SHA-256 fingerprinting, CoreML deployment |
| **ARBITER_THEORY** | Claim extraction, factual verification, LLM debate arbitration |
| **Surgery-Ward** | Pre-computed training data, dataset mixing, practical model sizes, command capability catalog, agentic evaluation harness |

This document defines how these integrate into a coherent system that avoids V3's failures while achieving Sterling's neurosymbolic reasoning goals.

---

## Current State Assessment (2026-01-25)

### What Works
- 575 tests passing across all layers
- API server operational with timing metrics
- MCP server exposing tools via JSON-RPC
- Three-judge council with weighted scoring
- Provenance chain with SHA-256 hashing

### Critical Gaps Identified

| Gap | Impact | Reference |
|-----|--------|-----------|
| **Semantic parsing missing** | "Delete all files" scores same as "Read config" | STERLING_INTEGRATION.md |
| **Evidence not populated** | Judges score defaults, not actual content | Quality evaluation |
| **No claim extraction** | Can't verify factual accuracy of outputs | ARBITER_THEORY.md |
| **No model performance tracking** | Can't learn which models work best | ARBITER_THEORY.md |
| **Single-pass evaluation** | No graph search or alternative exploration | Sterling principles |

---

## V3 Failures We Must Avoid

| V3 Anti-Pattern | Impact | V4 Solution |
|-----------------|--------|-------------|
| 60+ crates with complex interdependencies | Compilation brittleness | 16 focused crates |
| 6000+ line files | Unmaintainable code | Hard limit: 500 lines/file |
| 32/32 E2E tests were placeholders | False confidence | Fixture replay + invariant enforcement |
| Documentation claimed "operational" with 65 errors | Wasted time debugging lies | Evidence-based status only |
| Tight coupling across services | Changes cascade everywhere | Message bus + adapter pattern |
| Keyword matching for task analysis | Risky tasks not detected | Semantic parsing |

---

## Core Architectural Principles

### 1. Neural Advisory, Symbolic Authoritative (from Sterling)

LLMs are powerful but unpredictable. V4 treats them as **advisors**, not decision-makers:

```
┌─────────────────────────────────────────────────────────────┐
│                    DECISION FLOW                             │
├─────────────────────────────────────────────────────────────┤
│  1. Semantic parser extracts intent + risk signals          │
│  2. LLM proposes operators (advisory)                       │
│  3. Symbolic system validates against invariants            │
│  4. Council judges evaluate with rich evidence              │
│  5. Claim extractor verifies output factuality              │
│  6. If all gates pass → action executed                     │
│  7. Result logged with cryptographic proof                  │
└─────────────────────────────────────────────────────────────┘
```

**Key insight**: Sterling's INV-CORE-01 ("No Free-Form CoT in decision loops") prevents the LLM from reasoning itself into bad decisions.

### 2. Semantic Parsing, Not Keyword Matching (from Sterling)

Task descriptions must be parsed into structured intents:

```rust
// WRONG (current): Keyword matching
if description.contains("read") { /* propose read operators */ }

// RIGHT (target): Semantic parsing
let intent = parser.parse(&description).await?;
match intent {
    TaskIntent::Read { target } => /* safe operation */,
    TaskIntent::Delete { target, scope: DeleteScope::Bulk } => /* elevated risk */,
    TaskIntent::Execute { command, requires_shell: true } => /* high risk */,
}
```

### 3. Claim Extraction for Output Verification (from ARBITER_THEORY)

All LLM outputs must be decomposed into verifiable claims:

```
┌─────────────────────────────────────────────────────────────┐
│              4-STAGE CLAIM PIPELINE                          │
├─────────────────────────────────────────────────────────────┤
│  Stage 1: Contextual Disambiguation                         │
│    • Resolve ambiguous pronouns and references              │
│    • If unresolvable → skip, don't guess                    │
│                                                              │
│  Stage 2: Verifiable Content Qualification                  │
│    • Detect factual indicators (dates, quantities, APIs)    │
│    • Strip subjective/speculative language                  │
│                                                              │
│  Stage 3: Atomic Claim Decomposition                        │
│    • Break into single, verifiable statements               │
│    • Add contextual brackets for standalone meaning         │
│                                                              │
│  Stage 4: CAWS-Compliant Verification                       │
│    • Validate against evidence within declared budgets      │
│    • Emit verification result with audit trail              │
└─────────────────────────────────────────────────────────────┘
```

### 4. Hard Threshold Gates (from Distill)

V3's council made subjective decisions. V4 uses **numeric thresholds**:

| Gate | Threshold | Failure Mode |
|------|-----------|--------------|
| Integration F1 | >= 0.90 | Block deployment |
| Privacy OK Rate | = 1.0 | Block deployment |
| Constitutional Score | >= 0.85 | Veto |
| Risk Tier Elevation | Auto-detect | Escalate review |
| Claim Verification Rate | >= 0.90 | Flag for review |
| Invariant Violations | = 0 | Hard fail |

### 5. Evidence-Rich Evaluation (Gap Fix)

Judges must receive actual task content, not defaults:

```rust
// WRONG (current): Evidence built from defaults
let evidence = JudgeEvidence::default();
let score = judge.evaluate(&evidence); // Always ~0.85

// RIGHT (target): Evidence built from parsed task
let parsed = parser.parse(&task.description).await?;
let evidence = EvidenceBuilder::build(&task, &parsed, &operators);
evidence.risk_tier = compute_risk_tier(&parsed.risk_signals); // 1-5
evidence.proposed_operators = operators.iter().map(|o| format!("{:?}", o)).collect();
let score = judge.evaluate(&evidence); // Varies based on actual content
```

---

## Layered Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         V4 ARCHITECTURE v2                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   INTERFACE LAYER                        │    │
│  │  v4-api (REST) │ v4-mcp (JSON-RPC) │ v4-cli (planned)   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │               SEMANTIC PARSING LAYER (NEW)              │    │
│  │  ┌───────────────┐  ┌───────────────┐  ┌─────────────┐  │    │
│  │  │ Intent Parser │  │ Risk Detector │  │ Evidence    │  │    │
│  │  │ (task→S/M/P/  │  │ (bulk, shell, │  │ Builder     │  │    │
│  │  │  K/C intent)  │  │  credentials) │  │ (→judges)   │  │    │
│  │  └───────────────┘  └───────────────┘  └─────────────┘  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                 REASONING LAYER                          │    │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────────────┐    │    │
│  │  │ StateGraph│  │  Council  │  │  Arbiter          │    │    │
│  │  │ (search)  │  │ (3 judge) │  │  (final decision) │    │    │
│  │  └───────────┘  └───────────┘  └───────────────────┘    │    │
│  │  ┌─────────────────┐  ┌────────────────────────────┐    │    │
│  │  │ Symbolic Engine │  │ Invariant Enforcer         │    │    │
│  │  │ (Sterling-style)│  │ (11 testable invariants)   │    │    │
│  │  └─────────────────┘  └────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              CLAIM VERIFICATION LAYER (NEW)             │    │
│  │  ┌───────────────┐  ┌───────────────┐  ┌─────────────┐  │    │
│  │  │ Disambiguator │  │ Claim         │  │ Verifier    │  │    │
│  │  │ (resolve refs)│  │ Extractor     │  │ (fact-check)│  │    │
│  │  └───────────────┘  └───────────────┘  └─────────────┘  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  EXECUTION LAYER                         │    │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────────────┐    │    │
│  │  │ MCP Tools │  │ Workers   │  │ Sandbox Runtime   │    │    │
│  │  │ (sealed)  │  │ (pooled)  │  │ (isolated)        │    │    │
│  │  └───────────┘  └───────────┘  └───────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  INFRASTRUCTURE LAYER                    │    │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────────────┐    │    │
│  │  │ Memory    │  │ Storage   │  │ Model Inference   │    │    │
│  │  │ (graph+   │  │ (Postgres │  │ (MLX primary,     │    │    │
│  │  │  vector)  │  │  +event)  │  │  Mock fallback)   │    │    │
│  │  └───────────┘  └───────────┘  └───────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                    GOVERNANCE LAYER (cross-cutting)              │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────┐    │
│  │ CAWS Gates    │  │ Fingerprint   │  │ Audit Trail       │    │
│  │ (hard thresh) │  │ Verification  │  │ (TD-12 style)     │    │
│  └───────────────┘  └───────────────┘  └───────────────────┘    │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────┐    │
│  │ Model Perf    │  │ Reflexive     │  │ Benchmark         │    │
│  │ Tracker (NEW) │  │ Learning (NEW)│  │ System (NEW)      │    │
│  └───────────────┘  └───────────────┘  └───────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## New Components (v2.0)

### 1. Semantic Parsing Layer

Replaces keyword matching with structured intent extraction.

```rust
// crates/reasoning/v4-symbolic/src/parser.rs

/// Semantic parser for task descriptions
pub struct SemanticParser {
    intent_classifier: IntentClassifier,
    entity_extractor: EntityExtractor,
    risk_detector: RiskDetector,
}

/// Parsed semantic intent
#[derive(Debug, Clone)]
pub struct ParsedIntent {
    /// Primary intent mapped to S/M/P/K/C
    pub intent: TaskIntent,
    /// Extracted entities (paths, patterns, values)
    pub entities: Vec<ExtractedEntity>,
    /// Risk signals detected
    pub risk_signals: Vec<RiskSignal>,
    /// Confidence score
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub enum TaskIntent {
    /// Seek: Information retrieval (read, search, query)
    Read { target: ReadTarget },
    Search { pattern: SearchPattern },
    Query { source: QuerySource },

    /// Memorize: Store information (save, log, record)
    Write { target: WriteTarget, destructive: bool },
    Log { level: LogLevel, message: String },

    /// Control: Flow control with side effects
    Delete { target: DeleteTarget, scope: DeleteScope },
    Execute { command: String, requires_shell: bool },

    /// Perceive: Interpret input
    Analyze { target: AnalyzeTarget },
    Parse { format: ParseFormat },

    /// Knowledge: Apply domain knowledge
    Refactor { target: RefactorTarget },
    Explain { topic: String },

    /// Unknown: Requires clarification
    Unknown { raw: String },
}

#[derive(Debug, Clone)]
pub enum RiskSignal {
    /// Bulk operation (delete all, modify all)
    BulkOperation,
    /// System directory access (/etc, /var, ~/.ssh)
    SystemDirectory(String),
    /// Credential/secret access
    CredentialAccess,
    /// Network operation to external host
    NetworkOperation { host: String },
    /// Shell/command execution
    ShellExecution,
    /// Destructive operation (delete, truncate)
    DestructiveOperation,
}

impl SemanticParser {
    pub async fn parse(&self, description: &str) -> Result<ParsedIntent, ParseError> {
        // 1. Classify primary intent
        let intent = self.intent_classifier.classify(description).await?;

        // 2. Extract entities
        let entities = self.entity_extractor.extract(description).await?;

        // 3. Detect risk signals
        let risk_signals = self.risk_detector.detect(description, &intent).await?;

        Ok(ParsedIntent {
            intent,
            entities,
            risk_signals,
            confidence: self.compute_confidence(&intent, &entities),
        })
    }
}
```

### 2. Evidence Builder

Wires parsed intent to JudgeEvidence so council actually evaluates content.

```rust
// crates/reasoning/v4-symbolic/src/evidence.rs

pub struct EvidenceBuilder;

impl EvidenceBuilder {
    pub fn build(
        task: &TaskRequest,
        parsed: &ParsedIntent,
        operators: &[OperatorType],
    ) -> JudgeEvidence {
        let mut evidence = JudgeEvidence::new(TaskSpec::from_request(task));

        // Derive risk tier from signals (1-5)
        evidence.task_spec.risk_tier = Self::compute_risk_tier(&parsed.risk_signals);

        // Populate proposed operators
        evidence.proposed_operators = operators
            .iter()
            .map(|op| format!("{:?}", op))
            .collect();

        // Add intent classification
        evidence.metadata.insert(
            "intent_class".to_string(),
            format!("{:?}", parsed.intent),
        );

        // Add risk signals for audit
        evidence.metadata.insert(
            "risk_signals".to_string(),
            serde_json::to_string(&parsed.risk_signals).unwrap(),
        );

        evidence
    }

    fn compute_risk_tier(signals: &[RiskSignal]) -> u8 {
        let mut tier = 1u8;
        for signal in signals {
            tier = tier.max(match signal {
                RiskSignal::BulkOperation => 3,
                RiskSignal::SystemDirectory(_) => 4,
                RiskSignal::CredentialAccess => 5,
                RiskSignal::NetworkOperation { .. } => 2,
                RiskSignal::ShellExecution => 4,
                RiskSignal::DestructiveOperation => 3,
            });
        }
        tier
    }
}
```

### 3. StateGraph Search (Sterling-style)

Replace single-pass evaluation with multi-path exploration.

```rust
// crates/reasoning/v4-symbolic/src/state_graph.rs

/// State in the reasoning graph
#[derive(Debug, Clone)]
pub struct ReasoningState {
    pub id: String,
    pub interpretation: ParsedIntent,
    pub operators: Vec<OperatorType>,
    pub evidence: JudgeEvidence,
    pub parent: Option<String>,
    pub depth: u32,
}

/// Sterling-style graph search
pub struct ReasoningGraph {
    states: HashMap<String, ReasoningState>,
    transitions: Vec<StateTransition>,
    frontier: VecDeque<String>,
    max_depth: u32,      // INV-CORE-07: Termination guarantee
    max_iterations: u32, // INV-CORE-07: Bounded iterations
}

impl ReasoningGraph {
    /// Search for best path to goal state
    pub async fn search(
        &mut self,
        initial: ReasoningState,
        goal: impl Fn(&ReasoningState) -> bool,
        council: &Council,
    ) -> Result<SearchResult, GraphError> {
        self.frontier.push_back(initial.id.clone());
        let mut iterations = 0;

        while let Some(state_id) = self.frontier.pop_front() {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(GraphError::MaxIterationsExceeded);
            }

            let state = self.get_state(&state_id)?;

            if goal(&state) {
                return Ok(SearchResult::success(state, self.reconstruct_path(&state_id)));
            }

            if state.depth >= self.max_depth {
                continue;
            }

            // Expand candidates
            let candidates = self.expand(&state).await?;

            // Score each with council (parallel)
            let scored = self.score_candidates(&candidates, council).await;

            // Add non-vetoed to frontier (sorted by score)
            for (candidate, verdict) in scored.into_iter().filter(|(_, v)| !v.vetoed) {
                self.add_state(candidate.clone());
                self.frontier.push_back(candidate.id);
            }
        }

        Ok(SearchResult::no_path())
    }
}
```

### 4. Claim Extraction Pipeline (from ARBITER_THEORY)

Verify factual accuracy of LLM outputs.

```rust
// crates/reasoning/v4-claims/src/lib.rs (NEW CRATE)

/// 4-stage claim extraction pipeline
pub struct ClaimPipeline {
    disambiguator: Disambiguator,
    qualifier: ContentQualifier,
    decomposer: ClaimDecomposer,
    verifier: ClaimVerifier,
}

/// Stage 1: Contextual Disambiguation
pub struct Disambiguator;

impl Disambiguator {
    pub async fn disambiguate(
        &self,
        text: &str,
        context: &ConversationContext,
    ) -> Result<DisambiguationResult, DisambiguationError> {
        // Identify ambiguous references
        let ambiguities = self.detect_ambiguities(text, context).await?;

        // Attempt to resolve each
        let mut resolved_text = text.to_string();
        let mut unresolved = Vec::new();

        for ambiguity in ambiguities {
            match self.resolve(&ambiguity, context).await {
                Ok(resolution) => {
                    resolved_text = resolved_text.replace(&ambiguity.phrase, &resolution);
                }
                Err(_) => {
                    // Cannot resolve - mark for exclusion, don't guess
                    unresolved.push(ambiguity);
                }
            }
        }

        Ok(DisambiguationResult {
            text: resolved_text,
            unresolved_ambiguities: unresolved,
        })
    }
}

/// Stage 2: Verifiable Content Qualification
pub struct ContentQualifier;

impl ContentQualifier {
    pub async fn qualify(&self, text: &str) -> QualificationResult {
        // Detect factual indicators
        let indicators = self.detect_indicators(text);

        // If no verifiable content, return early
        if indicators.is_empty() {
            return QualificationResult {
                has_verifiable_content: false,
                qualified_text: None,
                indicators: vec![],
            };
        }

        // Strip subjective language
        let qualified = self.strip_subjective(text);

        QualificationResult {
            has_verifiable_content: true,
            qualified_text: Some(qualified),
            indicators,
        }
    }
}

/// Stage 3: Atomic Claim Decomposition
pub struct ClaimDecomposer;

impl ClaimDecomposer {
    pub async fn decompose(&self, text: &str) -> Vec<AtomicClaim> {
        // Split conjunctions and conditionals
        let sentences = self.split_into_clauses(text);

        sentences
            .into_iter()
            .map(|s| AtomicClaim {
                id: uuid::Uuid::new_v4().to_string(),
                statement: s.clone(),
                source_context: text.to_string(),
                verification_status: VerificationStatus::Pending,
            })
            .collect()
    }
}

/// Stage 4: CAWS-Compliant Verification
pub struct ClaimVerifier;

impl ClaimVerifier {
    pub async fn verify(
        &self,
        claim: &AtomicClaim,
        evidence: &EvidenceManifest,
    ) -> VerificationResult {
        // Check if claim can be verified within budget
        if !self.within_budget(claim, evidence) {
            return VerificationResult::InsufficientBudget;
        }

        // Attempt verification
        match self.check_evidence(claim, evidence).await {
            Ok(true) => VerificationResult::Verified {
                evidence_quality: self.compute_quality(evidence),
                audit_trail: self.build_trail(claim, evidence),
            },
            Ok(false) => VerificationResult::Refuted {
                reason: "Evidence contradicts claim".to_string(),
            },
            Err(_) => VerificationResult::Unverifiable {
                reason: "Insufficient evidence".to_string(),
            },
        }
    }
}
```

### 5. Model Performance Tracker (from ARBITER_THEORY)

Learn which models work best for different task types.

```rust
// crates/infrastructure/v4-observability/src/performance.rs

/// Tracks model performance for adaptive routing
pub struct ModelPerformanceTracker {
    /// Performance records by model and task surface
    records: HashMap<(ModelId, TaskSurface), PerformanceRecord>,
    /// Learning rate for score updates
    learning_rate: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceRecord {
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub caws_compliance_rate: f64,
    pub sample_count: u64,
    pub last_updated: DateTime<Utc>,
}

impl ModelPerformanceTracker {
    /// Record outcome of a task
    pub fn record_outcome(
        &mut self,
        model: ModelId,
        surface: TaskSurface,
        outcome: TaskOutcome,
    ) {
        let record = self.records
            .entry((model, surface))
            .or_insert_with(PerformanceRecord::default);

        // Exponential moving average
        let lr = self.learning_rate;
        record.success_rate = record.success_rate * (1.0 - lr)
            + (outcome.success as u8 as f64) * lr;
        record.avg_latency_ms = record.avg_latency_ms * (1.0 - lr)
            + outcome.latency_ms as f64 * lr;
        record.caws_compliance_rate = record.caws_compliance_rate * (1.0 - lr)
            + (outcome.caws_compliant as u8 as f64) * lr;
        record.sample_count += 1;
        record.last_updated = Utc::now();
    }

    /// Select best model for a task surface
    pub fn select_model(
        &self,
        surface: TaskSurface,
        available: &[ModelId],
    ) -> ModelId {
        available
            .iter()
            .max_by(|a, b| {
                let score_a = self.compute_score(*a, surface);
                let score_b = self.compute_score(*b, surface);
                score_a.partial_cmp(&score_b).unwrap()
            })
            .cloned()
            .unwrap_or_else(|| available[0].clone())
    }

    fn compute_score(&self, model: ModelId, surface: TaskSurface) -> f64 {
        self.records
            .get(&(model, surface))
            .map(|r| {
                // Weighted combination
                0.4 * r.success_rate
                    + 0.3 * r.caws_compliance_rate
                    + 0.2 * (1.0 - r.avg_latency_ms / 10000.0).max(0.0)
                    + 0.1 * (r.sample_count as f64 / 100.0).min(1.0)
            })
            .unwrap_or(0.5) // Unknown model gets neutral score
    }
}
```

---

## Surgery-Ward Integration (v2.1)

Surgery-Ward provides production-ready training infrastructure for distilling agentic LLMs on Apple Silicon. V4 integrates three key components:

### 1. Command Capability Catalog

The `terminal-use.csv` catalog (407 commands) provides structured risk assessment data:

```rust
// crates/reasoning/v4-symbolic/src/command_catalog.rs

/// Command capability flags from Surgery-Ward catalog
#[derive(Debug, Clone, Copy)]
pub struct CommandCapabilities {
    pub fs_read: bool,
    pub fs_write: bool,
    pub fs_delete: bool,
    pub exec_arbitrary: bool,
    pub net_outbound: bool,
    pub net_inbound: bool,
    pub container_mutate: bool,
    pub k8s_mutate: bool,
    pub cloud_mutate: bool,
    pub iac_apply: bool,
    pub secrets_read: bool,
}

/// Command risk classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandRisk {
    Safe,       // Read-only operations
    Moderate,   // Reversible writes
    Dangerous,  // May cause data loss
    Destructive,// Irreversible deletions
}

/// Catalog entry from terminal-use.csv
pub struct CommandEntry {
    pub category: String,        // VCS, Container, K8S, IaC, Cloud, Network, Control
    pub command: String,         // git, docker, kubectl, terraform, aws, curl, etc.
    pub verb_class: String,      // read, write, delete, mutate, network_client, exec
    pub pattern: Regex,          // Pattern to match command arguments
    pub capabilities: CommandCapabilities,
    pub risk: CommandRisk,
    pub reversible: bool,
    pub requires_sudo: bool,
}

impl CommandCatalog {
    /// Load from Surgery-Ward CSV
    pub fn from_csv(path: &Path) -> Result<Self, CatalogError> {
        let mut entries = Vec::new();
        let mut reader = csv::Reader::from_path(path)?;

        for record in reader.deserialize() {
            let entry: CommandEntry = record?;
            entries.push(entry);
        }

        Ok(Self { entries })
    }

    /// Classify a shell command
    pub fn classify(&self, command: &str) -> CommandClassification {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return CommandClassification::unknown();
        }

        let base_cmd = parts[0];
        let args = parts[1..].join(" ");

        // Find matching entries
        for entry in &self.entries {
            if entry.command == base_cmd && entry.pattern.is_match(&args) {
                return CommandClassification {
                    category: entry.category.clone(),
                    verb_class: entry.verb_class.clone(),
                    capabilities: entry.capabilities,
                    risk: entry.risk,
                    reversible: entry.reversible,
                    matched_pattern: entry.pattern.as_str().to_string(),
                };
            }
        }

        CommandClassification::unknown()
    }
}
```

**Categories from Surgery-Ward:**

| Category | Commands | Risk Range |
|----------|----------|------------|
| **VCS** | git, gh, delta, tig, lazygit | safe → destructive |
| **Container** | docker, podman, helm, k9s, kind | safe → destructive |
| **K8S** | kubectl (get, apply, delete) | safe → destructive |
| **IaC** | terraform, pulumi, ansible | safe → destructive |
| **Cloud** | aws, az, gcloud | safe → dangerous |
| **Network** | curl, wget, ssh, nc, socat | safe → moderate |
| **Control** | eval, xargs, parallel, watch | dangerous (exec arbitrary) |

### 2. Agentic Evaluation Harness

Surgery-Ward's `eval_agentic.py` provides 50+ tests for agentic capabilities:

```rust
// crates/infrastructure/v4-observability/src/evaluation.rs

/// Agentic evaluation categories
pub enum EvalCategory {
    TerminalCommands,  // ls, cat, mkdir, rm, du, find
    GitCommands,       // status, log, diff, add, commit
    SearchQueries,     // ripgrep syntax
    Planning,          // Task decomposition
    ToolCreation,      // Script generation
    SemanticReasoning, // <thinking> block quality
}

/// Evaluation result per category
pub struct CategoryResult {
    pub category: EvalCategory,
    pub total: u32,
    pub passed: u32,
    pub accuracy: f64,
}

/// Performance targets from Surgery-Ward
pub const EVAL_TARGETS: &[(EvalCategory, f64)] = &[
    (EvalCategory::TerminalCommands, 0.60),
    (EvalCategory::GitCommands, 0.80),
    (EvalCategory::SearchQueries, 0.60),
    (EvalCategory::Planning, 0.60),
    (EvalCategory::ToolCreation, 0.80),
    (EvalCategory::SemanticReasoning, 0.65),
];

impl AgenticEvaluator {
    /// Run evaluation suite
    pub async fn evaluate(&self, model: &dyn InferenceProvider) -> EvaluationReport {
        let mut results = Vec::new();

        for (category, target) in EVAL_TARGETS {
            let tests = self.get_tests(*category);
            let mut passed = 0;

            for test in tests {
                let response = model.generate(&test.prompt).await?;
                if self.check_response(&response, &test.expected) {
                    passed += 1;
                }
            }

            results.push(CategoryResult {
                category: *category,
                total: tests.len() as u32,
                passed,
                accuracy: passed as f64 / tests.len() as f64,
            });
        }

        EvaluationReport { results }
    }
}
```

### 3. Model Architecture Blueprints

Surgery-Ward's GQA configurations for distilled models:

```rust
// crates/infrastructure/v4-inference/src/config.rs

/// Student model architectures from Surgery-Ward config.py
pub const MODEL_ARCHITECTURES: &[ModelArchitecture] = &[
    ModelArchitecture {
        name: "olmo-1b",
        layers: 12,
        hidden_size: 2048,
        attention_heads: 16,
        kv_heads: 4,  // GQA 4:1 ratio
        intermediate_size: 5632,
        estimated_params: 1_000_000_000,
        size_fp16_gb: 2.0,
        speed_vs_7b: 3.5,
    },
    ModelArchitecture {
        name: "olmo-2b",
        layers: 16,
        hidden_size: 2560,
        attention_heads: 20,
        kv_heads: 5,
        intermediate_size: 7168,
        estimated_params: 2_000_000_000,
        size_fp16_gb: 4.0,
        speed_vs_7b: 2.5,
    },
    ModelArchitecture {
        name: "olmo-3b",
        layers: 20,
        hidden_size: 3072,
        attention_heads: 24,
        kv_heads: 6,
        intermediate_size: 8192,
        estimated_params: 3_000_000_000,
        size_fp16_gb: 6.0,
        speed_vs_7b: 2.3,
    },
];

/// GQA benefits (from Surgery-Ward docs)
/// - ~50% memory reduction in K/V attention projections
/// - Faster inference (critical for RL/post-training)
/// - Better GPU utilization
/// - Enables longer context with same memory
```

### 4. Training Data Templates

Surgery-Ward provides 2,763 lines of production agentic templates:

| Template File | Lines | Purpose |
|--------------|-------|---------|
| `agentic_templates.json` | 1,742 | Investigative debugging, file ops, git workflows |
| `semantic_reasoning_templates.json` | 602 | `<thinking>` block patterns |
| `planning_templates.json` | 99 | Task decomposition |
| `execution_aware_templates.json` | 123 | Side-effect awareness |

**Integration**: These templates can seed V4's memory system for few-shot prompting:

```rust
// crates/infrastructure/v4-memory/src/seeding.rs

/// Seed memory with Surgery-Ward agentic templates
pub async fn seed_from_templates(
    memory: &MemoryService,
    templates_dir: &Path,
) -> Result<usize, SeedError> {
    let mut count = 0;

    for template_file in &[
        "agentic_templates.json",
        "semantic_reasoning_templates.json",
        "planning_templates.json",
    ] {
        let path = templates_dir.join(template_file);
        let templates: HashMap<String, Vec<Template>> =
            serde_json::from_reader(File::open(path)?)?;

        for (category, examples) in templates {
            for example in examples {
                memory.store(MemoryEntry {
                    content: format!("User: {}\nAssistant: {}",
                        example.user, example.assistant),
                    category: MemoryCategory::AgenticTemplate,
                    tags: vec![category.clone()],
                    embedding: None, // Computed on storage
                }).await?;
                count += 1;
            }
        }
    }

    Ok(count)
}
```

### Surgery-Ward Integration Roadmap

| Phase | Component | Source File | V4 Integration |
|-------|-----------|-------------|----------------|
| 1 | Command Catalog | `terminal-use.csv` | v4-symbolic risk detection |
| 2 | Evaluation Harness | `eval_agentic.py` | v4-observability metrics |
| 3 | Model Configs | `config.py` | v4-inference MLX provider |
| 4 | Training Templates | `*.json` templates | v4-memory seeding |

---

## Updated Crate Structure (16 Crates)

```
iterations/v4/
├── Cargo.toml                    # Workspace manifest
│
├── crates/
│   │
│   ├── core/                     # 4 core crates
│   │   ├── v4-types/             # Shared types, events, contracts
│   │   ├── v4-config/            # Configuration, environment
│   │   ├── v4-invariants/        # Testable invariants (Sterling-style)
│   │   └── v4-governance/        # CAWS gates, fingerprinting, audit
│   │
│   ├── reasoning/                # 4 reasoning crates
│   │   ├── v4-symbolic/          # Semantic parser, operator graph, state machine
│   │   ├── v4-council/           # 3-judge constitutional council
│   │   ├── v4-arbiter/           # Task routing, workflow management
│   │   └── v4-claims/            # Claim extraction & verification (NEW)
│   │
│   ├── execution/                # 3 execution crates
│   │   ├── v4-tools/             # MCP tools, sealed interface
│   │   ├── v4-workers/           # Worker pool, task execution
│   │   └── v4-sandbox/           # Isolated execution runtime
│   │
│   ├── infrastructure/           # 5 infrastructure crates
│   │   ├── v4-memory/            # Graph + vector memory
│   │   ├── v4-storage/           # Content-addressable storage
│   │   ├── v4-postgres/          # PostgreSQL + pgvector
│   │   ├── v4-inference/         # MLX/Mock inference providers
│   │   └── v4-observability/     # Metrics, tracing, performance tracking
│   │
│   └── interfaces/               # 2 interface crates
│       ├── v4-api/               # HTTP API server
│       └── v4-mcp/               # MCP protocol server
│
├── scripts/
│   └── smoke-test.sh             # Automated verification
│
└── docs/
    ├── UNIFIED_ARCHITECTURE.md   # This document
    ├── STERLING_INTEGRATION.md   # Semantic parsing design
    └── IMPLEMENTATION_STATUS.md  # Current state
```

---

## Operator Taxonomy (from Sterling)

All operations are classified into 5 types:

| Operator | Symbol | Side Effects | Risk Level | Example |
|----------|--------|--------------|------------|---------|
| **Seek** | S | No | Low | Read file, search code, query memory |
| **Memorize** | M | Yes (store) | Low | Save to memory, log decision |
| **Perceive** | P | No | Low | Parse user intent, extract entities |
| **Knowledge** | K | No | Low | Code patterns, API conventions |
| **Control** | C | Yes (execute) | Variable | Branch, loop, delete, execute |

```rust
pub enum OperatorType {
    Seek(SeekOp),
    Memorize(MemorizeOp),
    Perceive(PerceiveOp),
    Knowledge(KnowledgeOp),
    Control(ControlOp),
}

impl OperatorType {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Seek(_) => "S",
            Self::Memorize(_) => "M",
            Self::Perceive(_) => "P",
            Self::Knowledge(_) => "K",
            Self::Control(_) => "C",
        }
    }

    pub fn has_side_effects(&self) -> bool {
        matches!(self, Self::Memorize(_) | Self::Control(_))
    }

    pub fn base_risk_tier(&self) -> u8 {
        match self {
            Self::Seek(_) | Self::Perceive(_) | Self::Knowledge(_) => 1,
            Self::Memorize(_) => 2,
            Self::Control(op) => op.risk_tier(),
        }
    }
}
```

---

## Constitutional Council (Enhanced)

Three judges with weighted scoring and veto logic:

```rust
pub struct CouncilVerdict {
    pub scores: JudgeScores,
    pub aggregate: f64,
    pub approved: bool,
    pub vetoed: bool,
    pub veto_reason: Option<String>,
    pub reasoning: String,
}

pub struct JudgeScores {
    pub constitutional: f64,  // Weight: 0.45
    pub technical: f64,       // Weight: 0.30
    pub quality: f64,         // Weight: 0.25
}

impl Council {
    pub async fn full_review(&self, evidence: &JudgeEvidence) -> CouncilVerdict {
        // 1. Each judge evaluates independently
        let constitutional = self.constitutional_judge.evaluate(evidence).await;
        let technical = self.technical_judge.evaluate(evidence).await;
        let quality = self.quality_judge.evaluate(evidence).await;

        // 2. Check for veto (any score < 0.5)
        let vetoed = constitutional.score < 0.5
            || technical.score < 0.5
            || quality.score < 0.5;

        // 3. Weighted aggregate
        let aggregate = constitutional.score * 0.45
            + technical.score * 0.30
            + quality.score * 0.25;

        CouncilVerdict {
            scores: JudgeScores {
                constitutional: constitutional.score,
                technical: technical.score,
                quality: quality.score,
            },
            aggregate,
            approved: !vetoed && aggregate >= 0.7,
            vetoed,
            veto_reason: if vetoed {
                Some(self.determine_veto_reason(&constitutional, &technical, &quality))
            } else {
                None
            },
            reasoning: self.compile_reasoning(&constitutional, &technical, &quality),
        }
    }
}
```

---

## Data Flow (Updated)

```
HTTP Request (POST /api/v1/tasks)
        │
        ▼
┌───────────────────────────────────────────────────────────────┐
│ v4-api: Parse request, generate task ID, start timing         │
└───────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────┐
│ SemanticParser: Parse description → TaskIntent + RiskSignals  │
│   • "Delete all temp files" → Delete{scope:Bulk} + [Bulk,Sys] │
│   • "Read Cargo.toml" → Read{target:File} + []                │
└───────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────┐
│ EvidenceBuilder: Build JudgeEvidence from parsed intent       │
│   • risk_tier = 4 (detected BulkOperation + SystemDirectory)  │
│   • proposed_operators = ["Control::Delete{...}"]             │
└───────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────┐
│ ReasoningGraph: Explore alternative operator sequences        │
│   • Candidate 1: [ListDirectory, Delete] - score 0.65         │
│   • Candidate 2: [ListDirectory, Confirm, Delete] - score 0.78│
│   • Select best non-vetoed path                               │
└───────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────┐
│ Council: 3 judges evaluate with rich evidence                 │
│   • Constitutional: 0.65 (ethics concern: bulk delete)        │
│   • Technical: 0.80 (valid operators)                         │
│   • Quality: 0.75 (meets requirements)                        │
│   • Aggregate: 0.72, Approved: true (no veto)                 │
└───────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────┐
│ Arbiter: Run CAWS gates, make final decision                  │
│   • All gates pass → ExecutionAuthorization                   │
│   • Generate VerificationCertificate with provenance          │
└───────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────┐
│ Workers + Sandbox: Execute with security policy               │
│   • Acquire worker, apply sandbox level (Restricted for Tier 4)│
│   • Execute operators, collect results                        │
└───────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────┐
│ ClaimPipeline: Verify output claims (if output has text)      │
│   • Disambiguate → Qualify → Decompose → Verify               │
│   • Flag unverified claims for review                         │
└───────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────┐
│ PerformanceTracker: Record outcome for model learning         │
│   • Update success_rate, latency, caws_compliance             │
│   • Inform future model selection                             │
└───────────────────────────────────────────────────────────────┘
        │
        ▼
HTTP Response (JSON with timing metrics, verdict, claims)
```

---

## Implementation Roadmap

### Phase 1: Semantic Parsing (Weeks 1-2)
- [ ] Define `TaskIntent` enum with all intent types
- [ ] Implement rule-based `IntentClassifier`
- [ ] Implement `EntityExtractor` for paths/patterns
- [ ] Implement `RiskDetector` with signal patterns
- [ ] Add `EvidenceBuilder` to wire parsed intent to judges
- [ ] Update API service to use new pipeline
- [ ] Tests: "Delete all files" → risk tier 4

### Phase 2: StateGraph Search (Weeks 2-3)
- [ ] Implement `ReasoningState` and `ReasoningGraph`
- [ ] Add candidate expansion logic
- [ ] Wire council scoring into search loop
- [ ] Implement path reconstruction with provenance
- [ ] Add termination guarantees (INV-CORE-07)
- [ ] Tests: Multiple paths explored, best selected

### Phase 3: Claim Extraction (Weeks 3-4)
- [ ] Create `v4-claims` crate
- [ ] Implement `Disambiguator` (Stage 1)
- [ ] Implement `ContentQualifier` (Stage 2)
- [ ] Implement `ClaimDecomposer` (Stage 3)
- [ ] Implement `ClaimVerifier` (Stage 4)
- [ ] Wire into response pipeline
- [ ] Tests: Claims extracted and verified

### Phase 4: Performance Tracking (Weeks 4-5)
- [ ] Implement `ModelPerformanceTracker`
- [ ] Add outcome recording to task completion
- [ ] Implement model selection based on history
- [ ] Add dashboard metrics for model performance
- [ ] Tests: Learning from outcomes

### Phase 5: Surgery-Ward Integration (Weeks 5-6)
- [ ] Import `terminal-use.csv` into v4-symbolic command catalog
- [ ] Implement `CommandCatalog::classify()` for shell command risk assessment
- [ ] Port `eval_agentic.py` tests to Rust evaluation harness
- [ ] Seed v4-memory with agentic templates (2,763 lines)
- [ ] Add model architecture configs for MLX inference
- [ ] Tests: Git commands → 80%+ accuracy, terminal → 60%+

### Phase 6: Integration & Testing (Weeks 6-7)
- [ ] Update smoke test with new validations
- [ ] Add integration tests for full pipeline
- [ ] Performance benchmarking against Surgery-Ward baselines
- [ ] Update dashboard to show new observability
- [ ] Documentation updates

---

## Success Criteria

### Semantic Parsing Quality

| Metric | Current | Target |
|--------|---------|--------|
| "Delete all files" risk tier | 1 (default) | 4+ |
| "Read config" vs "Delete files" score diff | 0.05 | 0.15+ |
| Parse accuracy on test corpus | N/A | 90%+ |
| Intent classification accuracy | N/A | 85%+ |

### Council Evaluation Quality

| Metric | Current | Target |
|--------|---------|--------|
| Judges see actual operators | No | Yes |
| Risk tier derived from content | No | Yes |
| Ethics patterns trigger on content | No | Yes |
| Reasoning explains actual concerns | No | Yes |

### Claim Verification

| Metric | Target |
|--------|--------|
| Claims extracted from outputs | 100% of verifiable content |
| Disambiguation success rate | 90%+ |
| Verification rate (when evidence exists) | 95%+ |
| Unresolved ambiguities flagged | 100% |

### Performance Tracking

| Metric | Target |
|--------|--------|
| Model selection improves over time | Yes |
| Performance records per model/surface | Complete |
| Routing decisions logged | 100% |

### Surgery-Ward Integration (Agentic Evaluation)

| Metric | Baseline (Surgery-Ward) | V4 Target |
|--------|------------------------|-----------|
| Git command accuracy | 60-80% | 80%+ |
| Terminal command accuracy | 20-30% | 60%+ |
| Tool creation accuracy | 50-100% | 80%+ |
| Planning task accuracy | 0-40% | 60%+ |
| Search query accuracy | 30-50% | 60%+ |
| Semantic reasoning quality | 65% | 70%+ |
| Command risk classification | N/A | 95%+ (from catalog) |

---

## Appendix: Invariant Specifications

### INV-CORE-01: No Free-Form Chain-of-Thought

**Rule**: LLM outputs in decision loops must be structured (JSON, enum), not free-form text.

**Rationale**: Free-form CoT allows the LLM to reason itself into arbitrary conclusions.

**Check**: Parse LLM output as structured type; reject if parsing fails.

### INV-CORE-04: Deterministic Operator Selection

**Rule**: Given the same task state, the same operators must be proposed.

**Rationale**: Non-determinism makes debugging and auditing impossible.

**Check**: Hash task state + operators; verify same hash produces same result.

### INV-CORE-05: Provenance Required

**Rule**: Every decision must have a provenance chain explaining why.

**Rationale**: "Trust but verify" requires knowing what to verify.

**Check**: All decisions include non-empty provenance with hashes.

### INV-CORE-07: Termination Guarantee

**Rule**: All loops must have bounded iterations.

**Rationale**: Unbounded loops can cause resource exhaustion.

**Check**: StateGraph has max_depth and max_iterations enforced.

### INV-CORE-09: Fail-Closed on Uncertainty

**Rule**: When in doubt, reject rather than approve.

**Rationale**: False negatives are recoverable; false positives may not be.

**Check**: Default verdicts are rejection; approval requires explicit evidence.

---

## Appendix: Source Attribution

| Component | Source Project | Key Files |
|-----------|----------------|-----------|
| Semantic parsing | Sterling | `sterling/README.md` (S/M/P/K/C operators) |
| Invariant enforcement | Sterling | `sterling/README.md` (INV-CORE-* section) |
| Claim extraction | ARBITER_THEORY | `distill/docs/ARBITER_THEORY.md` (4-stage pipeline) |
| Model performance tracking | ARBITER_THEORY | `distill/docs/ARBITER_THEORY.md` (Benchmarking) |
| CAWS gates | Distill | `distill/README.md` (Evaluation Harness) |
| SHA-256 fingerprinting | Distill | `distill/README.md` (Reproducibility) |
| Constitutional council | V3 | `iterations/v3/agent-constitutional-council/` |
| Memory system | V3 + Sterling | `sterling/README.md` (Decay categories) |
| Command capability catalog | Surgery-Ward | `surgery_ward_training/terminal-use.csv` (407 commands) |
| Agentic evaluation harness | Surgery-Ward | `surgery_ward_training/distillation/eval_agentic.py` |
| Model architecture configs | Surgery-Ward | `surgery_ward_training/distillation/config.py` |
| Agentic training templates | Surgery-Ward | `surgery_ward_training/distillation/*.json` (2,763 lines) |
| Pre-computed logits | Surgery-Ward | `surgery_ward_training/distillation/*_logits/` (~10GB) |
| Distilled model checkpoint | Surgery-Ward | `distilled_model/` (GQA 1B, 3.8GB) |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 2.1.0 | 2026-01-25 | Claude | Added Surgery-Ward integration: command catalog, agentic evaluation, model configs, templates |
| 2.0.0 | 2026-01-25 | Claude | Added semantic parsing, claim extraction, performance tracking; identified quality gaps |
| 1.0.0 | 2026-01-25 | Claude | Initial unified architecture |
