<!-- 317f1eba-aa5c-4439-ac2d-4882892ba1a0 643adee2-04cb-47b5-a563-1b8d21f936fb -->
# V3 TODO Implementation Roadmap (Dependency‑Driven, Milestones, Evidence‑Gated)

## Executive Summary

26 TODOs block acceptance criteria **A1–A9**. Root‑cause analysis identifies **four dependency chains** that throttle progress:

1. **Tokenization Infrastructure** → blocks 15 Apple‑Silicon/Core ML tasks (A4).
2. **Observability Stack** → blocks metrics and all acceptance evidence (A1–A9).
3. **Database Artifact Persistence** → blocks provenance & artifacts (A7).
4. **ML Model Loading & Caching** → blocks inference & vector search (A4, A8).

We replace “phases/weeks” with **milestones**. Each milestone has: *objective, scope, changes, interfaces, tests, evidence gates, rollbacks*. A minimal feature‑flag strategy ensures safe iterative delivery.

---

## Dependency DAG (condensed)

```
M0 (Bootstrap) ───────┐
                      ├─▶ M1 Tokenization ──┐
                      │                      ├─▶ M5 Apple‑Silicon (A4 full)
                      ├─▶ M2 Observability ─┼─▶ M7 Executor Loop (A1)
                      │                      └─▶ Evidence for A6/A9 et al.
                      ├─▶ M3 DB Artifacts ─────▶ A7 Provenance
                      └─▶ M4 Model Loading ────▶ A8 Vector Search, M5
```

---

## Global Guardrails

- **Feature flags**: `coreml`, `observability.prometheus`, `observability.statsd`, `observability.redis`, `storage.db_artifacts`.
- **Hard timeouts** for all external I/O (ASR, Core ML, Redis, DB).
- **Circuit breakers** per backend (trip → degrade gracefully).
- **Telemetry naming**: `v3.<component>.<event>`; labels: `host`, `commit`, `feature`, `risk_tier`.
- **Provenance**: every artifact stored has `{task_id, input_hash, output_hash, parent_id}`.

---

## Milestone M0 — Bootstrap (Unblocks All)

### Objective

Establish common interfaces and configs so later milestones only plug in backends.

### Scope & Changes

- Create `observability/src/metrics.rs` with a **MetricsBackend** trait and no‑op default.
- Create `orchestration/src/artifacts/storage.rs` with **ArtifactStorage** trait + in‑memory impl.
- Add `config/default.yaml` keys for `redis`, `prometheus`, `statsd`, `storage.database`.
- Ensure Docker compose services exist (Postgres/Redis/Prometheus already present).

### Interfaces

```rust
pub trait MetricsBackend: Send + Sync {
    fn counter(&self, name: &str, labels: &[(&str,&str)], v: u64);
    fn gauge(&self, name: &str, labels: &[(&str,&str)], v: f64);
    fn histogram(&self, name: &str, labels: &[(&str,&str)], v: f64);
}

pub trait ArtifactStorage: Send + Sync {
    fn put(&self, a: ExecutionArtifact) -> Result<ArtifactId>;
    fn get(&self, id: &ArtifactId) -> Result<Option<ExecutionArtifact>>;
    fn latest_for_task(&self, task: Uuid) -> Result<Vec<ExecutionArtifact>>;
}
```

### Tests

- Unit tests for no‑op metrics and in‑memory artifacts.
- Config load test; environment overrides honored.

### Evidence Gate M0

- Build succeeds with `--features ''` (no backends).
- All downstream crates compile against the traits.

---

## Milestone M1 — Tokenization Infrastructure (A4 prereq)

### Objective

Supply a pluggable tokenizer and wire it through Core ML / Apple‑Silicon pipeline.

### Scope & Changes

- Add `tokenizers = "0.19"` to `apple-silicon/Cargo.toml`.
- Create `apple-silicon/src/tokenization.rs` exposing:
```rust
pub trait Tokenizer: Send + Sync {
    fn encode(&self, text: &str) -> anyhow::Result<Vec<u32>>;
    fn decode(&self, tokens: &[u32]) -> anyhow::Result<String>;
    fn vocab_size(&self) -> usize;
}
pub struct HfTokenizer { /* model + pretokenizer cfg */ }
```

- Replace placeholders in `core_ml.rs` (`create_text_input_array`, `extract_text_from_ml_multiarray`).
- Extend `CoreMLConfig` with tokenizer path + special tokens.

### Integration Points

- `apple-silicon/src/core_ml_backend.rs` input preparation → `Tokenizer::encode`.
- Optional shims for `tiktoken-rs` or `sentencepiece` behind enum `TokenizerImpl`.

### Tests

- Round‑trip tests (encode→decode), unicode & punctuation sets.
- Snapshot vectors for fixtures.

### Evidence Gate M1

- All 3 tokenization APIs used in Apple‑Silicon codepaths compile & pass tests.
- Tokenization latency histogram emitted (via M2 once ready).

### Research Notes

- If a model requires SentencePiece, add adapter with `sentencepiece = "0.11"` and map to common `Tokenizer` trait.

---

## Milestone M2 — Observability Stack (Enables A1–A9 evidence)

### Objective

Provide production‑grade metrics and caching with health‑checks and fallbacks.

### Scope & Changes

- Add deps (workspace `Cargo.toml`):
```toml
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
prometheus = "0.13"
cadence = "1.4"   # StatsD client
```

- Implement backends:

  - `observability/src/metrics/prometheus.rs` (registry, exporters).
  - `observability/src/metrics/statsd.rs`.
  - `observability/src/cache/redis_cache.rs` (TTL, connection manager).
- Wire emits in: `council/src/consensus.rs` (A1 timings), `model-benchmarking/src/collector.rs` (A6), Apple‑Silicon timings, DB latencies.
- `/healthz` and `/metrics` HTTP endpoints in observer or gateway.

### Tests

- Docker compose integration: Redis reachable, Prometheus scrape returns series.
- Failure injection: Redis timeout triggers circuit breaker to in‑memory cache.

### Evidence Gate M2

- Live metrics observed for counters/gauges/histograms.
- Health checks fail‑safe with degraded mode logged.

### Research Notes

- Metrics cardinality budget (< 1k active series).
- Decide single “source of truth” (Prometheus recommended; StatsD optional for legacy).

---

## Milestone M3 — Database Artifact Persistence (A7)

### Objective

Back ArtifactStorage with PostgreSQL and tie artifacts to provenance.

### Scope & Changes

- Create `database/src/artifact_store.rs` implementing `ArtifactStorage` via SQLx.
- Ensure migrations present:

  - `007_execution_artifacts.sql`, `008_artifact_versions.sql` (idempotent).
- Wire in `orchestration/src/artifacts/manager.rs` and `provenance/src/service.rs` linking `{task_id, parent_id}`.

### Interfaces

```rust
pub struct DatabaseArtifactStorage { pub pool: Arc<sqlx::PgPool> }
impl ArtifactStorage for DatabaseArtifactStorage { /* CRUD, versions */ }
```

### Tests

- CRUD + versioning, pagination, and large‑blob storage.
- Provenance join queries.

### Evidence Gate M3

- E2E: submit task → artifacts persisted → retrievable with provenance chain.
- Grafana/Prometheus shows DB latencies + error rates.

### Rollback

- Feature flag `storage.db_artifacts=false` falls back to in‑memory store.

---

## Milestone M4 — Model Loading & Caching (A8 + prereq for M5)

### Objective

Standardize model loading for embeddings/inference and introduce an LRU cache.

### Scope & Changes

- `embedding-service/src/provider.rs`: ONNX model loading (ORT) with session options (intra/inter‑op threads, graph optimizations).
- `embedding-service/src/cache.rs`: model cache (path→session) with size and idle TTL limits.
- `apple-silicon/src/quantization.rs`: SafeTensors loader for weights used by CPU fallback.

### Tests

- Load failure surfaces proper errors, no panics across FFI.
- Cache eviction unit tests (LRU + TTL).
- Throughput baseline for embedding inference.

### Evidence Gate M4

- Vector search (A8) can call the provider and receive embeddings under SLA.
- Cache hit ratio and latency exported via M2.

### Research Notes (important)

- **Do not plan** a "Core ML → SafeTensors" conversion path; that’s not a supported round‑trip. Maintain **two authoring artifacts**: `.mlpackage` for Core ML, and `.onnx`/`.safetensors` for CPU/GPU backends.

---

## Milestone M5 — Apple‑Silicon (Core ML) Integration (A4)

### Objective

Deliver compile+load+predict via Swift bridge with hard safety bounds and fallbacks.

### Scope & Changes

- Swift C‑ABI bridge (`coreml-bridge/`): `coreml_compile_model`, `coreml_load_model`, `coreml_predict`, `coreml_free_model`, `coreml_free_cstr`; compute units policy.
- Rust safe wrapper: `apple-silicon/src/core_ml_bridge.rs`, `core_ml_backend.rs`.
- Timeouts and cancellation in Swift (Task/Dispatch), per‑call autorelease pools.
- Telemetry: compile time p50/p99, predict p50/p99, success rate; **requested vs actual** dispatch labels.

### Tests

- 10k compile+load leak test (Instruments: steady memory).
- Parity harness vs CPU for one small model; numeric thresholds documented.
- Circuit breaker auto‑disables on <95% success or p99 > SLA.

### Evidence Gate M5

- Gate A: leak‑free compile/load; Gate B: single‑IO predict correctness under SLA.
- Feature flag `coreml` toggles path; graceful fallback to Candle/CPU.

### Research Notes

- Ensure ML Program backend when targeting transformer ops; no custom layers on hot paths.

---

## Milestone M6 — Autonomous Executor Loop (A1)

### Objective

Replace the stub loop with a robust execution engine emitting progress and artifacts.

### Scope & Changes

- `workers/src/autonomous_executor.rs`: implement queueing, timeouts, cancellation, heartbeats.
- `orchestration/src/tracking/` event bus streams events (WebSocket/CLI).
- Integrate CAWS gates as checkpoints; on failure → emit `RefinementRequest`.

### Tests

- Mock workers produce artifacts; timeouts and cancellations exercised.
- Progress event stream ordering and back‑pressure tests.

### Evidence Gate M6

- Consensus window metric (`council.consensus.ms`) meets A1 SLA with real tasks.
- Artifacts persisted (M3) and metrics exported (M2).

---

## Acceptance Criteria Mapping (A1–A9)

| Acceptance           | Blocked By                   | Satisfied When                                                   |

| -------------------- | ---------------------------- | ---------------------------------------------------------------- |

| **A1** Consensus     | executor stub, no metrics    | M2 metrics wired + M6 executor loop; p95 consensus < target      |

| **A2** Claims        | —                            | Already implemented (sanity e2e passes continue)                 |

| **A3** Learning      | —                            | Already implemented; record episodes via M2 metrics hooks        |

| **A4** Apple‑Silicon | tokenization, Core ML bridge | M1 + M5; success ≥95%, p99 compile<5s, predict<target            |

| **A5** MCP           | —                            | Existing; expose new task endpoints as tools (out of scope here) |

| **A6** Benchmarking  | no metrics                   | M2 provides Prom+StatsD; collector emits distributions           |

| **A7** Provenance    | no DB artifact store         | M3 DB storage + provenance joins validated                       |

| **A8** Vector Search | model provider/cache         | M4 provider + cache, e2e vector search passes                    |

| **A9** CAWS          | —                            | Existing; checkpoints enforced in M6                             |

---

## Risk Register (targeted mitigations)

1. **Tokenizer mismatch** → model incompatibility.

*Mitigation*: `Tokenizer` trait + adapters (`HfTokenizer`, `SentencePiece`, `Tiktoken`). Version pin models + test fixtures.

2. **Observability outages** (Redis/Prometheus).

*Mitigation*: circuit breakers, health probes, in‑memory fallback, bounded cardinality.

3. **Core ML instability / leaks**.

*Mitigation*: Swift autorelease pools, Instruments gate, feature flag + immediate fallback to CPU.

4. **Schema drift for artifacts**.

*Mitigation*: idempotent migrations; canary DB; E2E provenance test suite.

5. **ORT/ONNX ops missing**.

*Mitigation*: provider capability check at load; skip model or drop to CPU impl; log.

---

## Concrete Tasks (Backlog Items)

- **M0**: Define `MetricsBackend` + `ArtifactStorage` traits; no‑op + in‑mem impls; load config; `/healthz` basic.
- **M1**: Add `tokenizers`; implement `Tokenizer` trait + `HfTokenizer`; wire to `core_ml.rs` encode/decode; unit tests.
- **M2**: `PrometheusMetrics`, `StatsDMetrics`, `RedisCache` + circuit breaker; wire emits in council, benchmarking, Apple‑Silicon; `/metrics` endpoint.
- **M3**: `DatabaseArtifactStorage` via SQLx; CRUD/versioning; provenance linking; tests.
- **M4**: ONNX model provider + LRU cache; SafeTensors loader for CPU fallback; vector search e2e.
- **M5**: Swift bridge functions; Rust wrapper; timeouts/cancellation; telemetry + circuit breaker; leak & parity gates.
- **M6**: Executor loop; event bus; progress streaming; CAWS checkpoints; timeouts/cancellation.

---

## Research Checkpoints (explicit decisions before/while implementing)

- **Tokenizer**: default to HuggingFace `tokenizers`; add `TokenizerImpl` enum to allow `sentencepiece` for T5/BERT; capture per‑model configs in repo.
- **Metrics**: prefer Prometheus as primary; StatsD optional. Decide histograms buckets for latency (e.g., `[1, 5, 10, 20, 50, 100, 250, 500, 1000] ms`).
- **Model Artifacts**: keep dual authoring (Core ML `.mlpackage` and ONNX/SafeTensors) rather than attempting cross‑conversion.
- **Cache Policy**: model cache `max_items`, `max_bytes`, `idle_ttl`; eviction telemetry required.
- **Core ML Compute Units**: request `cpuAndNe` by default; record **actual** dispatch; auto‑demote on repeated CPU fallbacks.

---

## Evidence & Observability (what reviewers should see per gate)

- **Dashboards**: Consensus latency, Compile/Predict latencies, Cache hit ratio, DB latencies, Error rates.
- **Logs**: Circuit breaker state changes, Fallback reasons, Health probe results.
- **Artifacts**: Stored diffs, reports, and provenance chains per task in DB.

---

## Immediate Next Steps (sequence)

1. Land **M0** traits + config + no‑ops.
2. Start **M1** (tokenization) and **M2** (observability) in parallel.
3. Proceed with **M3** (artifacts) and **M4** (model provider/cache) in parallel.
4. Complete **M5** (Core ML) gated by M1/M4.
5. Finish **M6** (executor) using M2/M3 evidence.

---

## Appendix — File/Path Checklist

- `observability/src/metrics.rs` (+ `prometheus.rs`, `statsd.rs`), `observability/src/cache/redis_cache.rs`
- `orchestration/src/artifacts/storage.rs`, `orchestration/src/artifacts/manager.rs`
- `database/src/artifact_store.rs`; migrations `007_execution_artifacts.sql`, `008_artifact_versions.sql`
- `apple-silicon/src/tokenization.rs`, `apple-silicon/src/core_ml.rs`, `apple-silicon/src/core_ml_backend.rs`
- `embedding-service/src/provider.rs`, `embedding-service/src/cache.rs`
- `workers/src/autonomous_executor.rs`, `orchestration/src/tracking/*`
- Swift bridge: `coreml-bridge/Package.swift`, `Sources/CoreMLBridge.swift`

### To-dos

- [ ] Add tokenizer, Redis, Prometheus, StatsD dependencies to Cargo.toml files
- [ ] Implement DatabaseArtifactStorage in database/src/artifact_store.rs
- [ ] Create MetricsBackend trait with Redis/Prometheus/StatsD implementations
- [ ] Implement Tokenizer trait and replace tokenization placeholders in apple-silicon
- [ ] Implement Redis cache, Prometheus metrics, StatsD metrics with health checks
- [ ] Wire DatabaseArtifactStorage to ArtifactManager and provenance service
- [ ] Implement SafeTensors and ONNX model loading with LRU cache
- [ ] Implement Metal GPU pipeline, ANE monitoring, thermal monitoring, actual inference
- [ ] Implement autonomous executor loop with progress tracking