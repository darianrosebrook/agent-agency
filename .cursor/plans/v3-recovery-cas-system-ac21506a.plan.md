<!-- ac21506a-1ea5-4960-810e-e375994e70e0 34540653-b361-440f-9c0a-a3cd7f2262fa -->
# V3 Recovery System: Content-Addressable Storage with Merkle Provenance

## Architecture Overview

Create a dedicated `iterations/v3/recovery/` crate implementing Git-like CAS with BLAKE3 content addressing, journaled writes, and CAWS governance integration.

```
.v3rec/
├── objects/           # BLAKE3-addressed blobs (zstd compressed)
├── refs/
│   ├── sessions/      # Session → commit mappings
│   └── checkpoints/   # Protected labels
├── index/
│   ├── metadata.db    # SQLite: paths ↔ commits
│   └── pack/          # Packed objects for cold storage
└── journal/
    └── 000001.log     # Write-ahead log (fsync ordered)
```

## Phase 1: CAS Foundations (Week 1)

### Milestone 1: Core Storage Layer

**Goal**: Establish BLAKE3-based CAS with atomic journaled writes

**Create `iterations/v3/recovery/` crate structure**:

```
recovery/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── api.rs          # Public traits
│   ├── cas/
│   │   ├── mod.rs
│   │   ├── blob.rs     # Blob storage
│   │   ├── compression.rs  # Zstd/gzip adapters
│   │   └── chunking.rs # CDC implementation
│   ├── merkle/
│   │   ├── mod.rs
│   │   ├── tree.rs     # Merkle tree nodes
│   │   └── commit.rs   # Commit objects
│   ├── journal/
│   │   ├── mod.rs
│   │   └── wal.rs      # Write-ahead log
│   └── refs/
│       ├── mod.rs
│       └── manager.rs  # Ref management
└── tests/
    └── integration/
```

**Extend `iterations/v3/source-integrity/`**:

- Add `Digest` newtype wrapping BLAKE3 hash with serde support
- Add `MerkleTree` implementation over blob lists
- Add streaming `Hasher::update(&[u8])` for large files
- Export types for use by recovery crate

**Key Types** (`recovery/src/api.rs`):

```rust
pub struct Digest(pub [u8; 32]); // BLAKE3 hash

pub enum ChangePayload {
    Full(Vec<u8>),              // < 2 KiB files
    UnifiedDiff(Vec<u8>),       // Text diffs
    ChunkMap(ChunkList),        // CDC chunks
}

pub struct FileChange {
    pub path: PathBuf,
    pub precondition: Option<Digest>,
    pub payload: ChangePayload,
    pub source: ChangeSource,
}

#[async_trait]
pub trait RecoveryStore {
    async fn begin_session(&self, meta: SessionMeta) -> Result<SessionRef>;
    async fn record_change(&self, session: &SessionRef, change: FileChange) -> Result<ChangeId>;
    async fn checkpoint(&self, session: &SessionRef, label: Option<String>) -> Result<CommitId>;
}
```

**Acceptance**:

- [ ] BLAKE3 digests correctly address blobs in `objects/`
- [ ] Journaled write path: journal → blob → index → ref (fsync ordered)
- [ ] Crash simulation: no dangling refs after replay
- [ ] Zstd compression with configurable level (default: 4)

---

## Phase 2: Diffs, Chunking & Restore (Week 2)

### Milestone 2: Content Strategies & Recovery

**Goal**: Smart content handling with unified diffs and CDC chunking

**Implement Content Strategy** (`cas/blob.rs`):

```rust
fn determine_strategy(
    old_content: &[u8],
    new_content: &[u8]
) -> ChangePayload {
    if new_content.len() <= 2048 {
        return ChangePayload::Full(new_content.to_vec());
    }
    
    let diff = unified_diff(old_content, new_content);
    let diff_ratio = diff.len() as f64 / new_content.len() as f64;
    
    if diff_ratio <= 0.45 {
        ChangePayload::UnifiedDiff(diff)
    } else {
        ChangePayload::ChunkMap(cdc_chunk(new_content))
    }
}
```

**CDC Chunking** (`cas/chunking.rs`):

- Use gear-based rolling hash with 16 KiB target chunks
- Min chunk: 4 KiB, Max chunk: 64 KiB
- Store chunks as individual blobs with digest addressing
- Build `ChunkList` referencing chunk digests

**Unified Diff Format**:

```rust
struct UnifiedDiff {
    base_digest: Digest,
    hunks: Vec<Hunk>,
    metadata: DiffMetadata,
}
```

**Optimistic Concurrency** (`cas/blob.rs`):

```rust
pub fn write_with_precondition(
    &self,
    path: &Path,
    content: ChangePayload,
    precondition: Option<Digest>
) -> Result<ChangeId, ConflictError> {
    if let Some(expected) = precondition {
        let current = self.get_current_digest(path)?;
        if current != expected {
            return Err(ConflictError {
                base: expected,
                theirs: current,
                yours: content,
            });
        }
    }
    // ... write blob
}
```

**Restore API** (`api.rs`):

```rust
pub struct RestorePlan {
    pub files: Vec<FileRestoreAction>,
    pub total_size_bytes: u64,
    pub estimated_duration_ms: u64,
}

pub struct RestoreFilters {
    pub globs: Vec<String>,
    pub since: Option<CommitId>,
    pub path_prefix: Option<PathBuf>,
}

impl RecoveryStore {
    async fn plan_restore(
        &self,
        target: RefOrCommit,
        filters: RestoreFilters
    ) -> Result<RestorePlan>;
    
    async fn apply_restore(
        &self,
        plan: RestorePlan,
        target_dir: PathBuf,
        dry_run: bool
    ) -> Result<RestoreResult>;
}
```

**Acceptance**:

- [ ] Unified diffs correctly reconstruct text files
- [ ] CDC chunks deduplicate across file versions
- [ ] Conflict detection on precondition mismatch
- [ ] Restore plan preview shows files/sizes before execution
- [ ] Dry-run restore validates without writing files
- [ ] Full restore verifies all file digests match commit tree

---

## Phase 3: CAWS Integration & Governance (Week 3)

### Milestone 3: Policy Enforcement & GC

**Goal**: CAWS-compliant budgets, retention, redaction, and garbage collection

**CAWS Policy Schema** (`.caws/working-spec.yaml`):

```yaml
recovery_policy:
  auto_checkpoint: true
  checkpoint_frequency: ["every-iteration", "pre-merge"]
  storage_budget_mb: 512
  retention:
    min_days: 30
    max_sessions: 200
    protected_labels: ["release/*", "postmortem/*"]
  compression:
    codec: "zstd"
    level: 4
  chunking:
    mode: "cdc"
    target_kib: 16
  redaction:
    rules:
      - type: secret
        patterns: ["BEGIN RSA PRIVATE KEY", "AWS_*", "GH_TOKEN"]
      - type: pii
        patterns: ["email", "phone"]
  encryption:
    at_rest: "aes-gcm-256"
    key_scope: "tenant"
```

**Policy Enforcement** (`recovery/src/policy.rs`):

```rust
pub struct PolicyEnforcer {
    config: RecoveryPolicy,
    redactor: SecretRedactor,
}

impl PolicyEnforcer {
    pub async fn check_budget(&self, current_usage: u64) -> Result<BudgetStatus>;
    
    pub async fn redact_content(&self, content: &[u8]) -> Result<Vec<u8>>;
    
    pub async fn enforce_retention(&self, sessions: Vec<SessionRef>) -> Result<Vec<SessionRef>>;
}
```

**Garbage Collection** (`recovery/src/gc/mod.rs`):

```rust
pub struct GarbageCollector {
    store: Arc<dyn RecoveryStore>,
    policy: RecoveryPolicy,
}

impl GarbageCollector {
    // Reachability-based GC from protected refs
    pub async fn collect(&self) -> Result<GcReport> {
        // 1. Mark: traverse from all refs in protected_labels
        let reachable = self.mark_reachable().await?;
        
        // 2. Sweep: delete unreferenced objects
        let freed = self.sweep_unreachable(reachable).await?;
        
        // 3. Pack: consolidate cold objects
        self.pack_cold_objects().await?;
        
        Ok(GcReport { freed_bytes: freed, packed_objects: ... })
    }
}
```

**Secret Redaction** (`recovery/src/policy/redaction.rs`):

```rust
pub struct SecretRedactor {
    patterns: Vec<RedactionPattern>,
}

impl SecretRedactor {
    pub fn scan_and_redact(&self, content: &[u8]) -> Result<RedactionResult> {
        let mut redacted = content.to_vec();
        let mut findings = Vec::new();
        
        for pattern in &self.patterns {
            if let Some(matches) = pattern.find_all(&redacted) {
                findings.extend(matches);
                redacted = pattern.redact_all(redacted);
            }
        }
        
        Ok(RedactionResult { content: redacted, findings })
    }
}
```

**Acceptance**:

- [ ] Storage budget enforced; soft limit triggers compaction, hard limit blocks
- [ ] Protected refs never garbage collected
- [ ] Secret patterns blocked at admission (test with seeded secrets)
- [ ] Retention policy removes sessions older than min_days
- [ ] GC removes only unreachable objects, preserves reachable ones

---

## Phase 4: Integration & Tooling (Week 4)

### Milestone 4: Agent Integration & CLI

**Goal**: Wire recovery system into V3 agents and provide operational tooling

**Self-Prompting Loop Integration** (`iterations/v3/self-prompting-agent/src/loop_controller.rs`):

```rust
impl SelfPromptingLoop {
    async fn execute_iteration_with_recovery(
        &self,
        task: &Task
    ) -> Result<IterationResult> {
        // Create recovery session
        let session = self.recovery.begin_session(SessionMeta {
            task_id: task.id.clone(),
            iteration: self.current_iteration,
            agent_id: self.agent_id.clone(),
        }).await?;
        
        // Execute iteration
        let result = self.run_iteration(task).await?;
        
        // Track all file changes with optimistic concurrency
        for file_change in result.file_changes {
            self.recovery.record_change(&session, FileChange {
                path: file_change.path,
                precondition: Some(file_change.old_digest),
                payload: self.determine_payload(&file_change),
                source: ChangeSource::AgentIteration {
                    iteration: self.current_iteration,
                    agent_id: self.agent_id.clone(),
                },
            }).await.or_else(|e| match e {
                ConflictError { .. } => {
                    // Handle conflict via orchestrator
                    self.handle_conflict(e).await
                }
                _ => Err(e),
            })?;
        }
        
        // Checkpoint after iteration
        self.recovery.checkpoint(&session, None).await?;
        
        Ok(result)
    }
}
```

**Worker Integration** (`iterations/v3/workers/src/autonomous_executor.rs`):

```rust
impl AutonomousExecutor {
    pub async fn restore_from_checkpoint(
        &self,
        session_id: String,
        target_path: PathBuf
    ) -> Result<RestoreResult> {
        let filters = RestoreFilters {
            globs: vec!["**/*".to_string()],
            since: None,
            path_prefix: None,
        };
        
        let plan = self.recovery.plan_restore(
            RefOrCommit::Session(session_id),
            filters
        ).await?;
        
        // Preview for logging
        info!("Restore plan: {} files, {} MB", 
              plan.files.len(), 
              plan.total_size_bytes / 1_000_000);
        
        // Apply with verification
        self.recovery.apply_restore(plan, target_path, false).await
    }
}
```

**CLI Tool** (`recovery/recov-cli/src/main.rs`):

```rust
// recov-cli: Operational tooling for recovery system

Commands:
  init              Initialize recovery store in current directory
  track <file>      Track a file change
  checkpoint [msg]  Create a checkpoint with optional label
  plan <ref>        Preview restore from ref/commit
  restore <ref>     Execute restore from ref/commit
  pack              Pack cold objects
  gc                Run garbage collection
  fsck              Verify store integrity
  stat              Show storage statistics
```

**Database Metadata Integration** (`iterations/v3/database/src/artifact_store.rs`):

```rust
// Add recovery metadata to artifact store
impl ArtifactStore {
    pub async fn link_recovery_session(
        &self,
        task_id: &str,
        session_id: &str,
        commit_id: &str
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO recovery_sessions 
             (task_id, session_id, commit_id, created_at)
             VALUES ($1, $2, $3, NOW())",
            task_id, session_id, commit_id
        ).execute(&self.pool).await?;
        Ok(())
    }
}
```

**Acceptance**:

- [ ] Self-prompting loop auto-checkpoints each iteration
- [ ] Conflict detection surfaces to orchestrator for resolution
- [ ] Workers can restore from any session/commit
- [ ] CLI commands work end-to-end (init → track → checkpoint → restore)
- [ ] Database tracks recovery session metadata

---

## Cross-Cutting Concerns

### Observability

**Metrics** (`recovery/src/metrics.rs`):

```rust
pub struct RecoveryMetrics {
    pub dedupe_ratio: f64,           // Unique bytes / total bytes
    pub restore_latency_p50_ms: u64,
    pub restore_latency_p95_ms: u64,
    pub conflict_rate: f64,          // Conflicts / total writes
    pub redaction_hits: u64,
    pub gc_freed_mb: u64,
    pub pack_efficiency: f64,        // Packed size / original size
}
```

### Security

**At-Rest Encryption** (`recovery/src/encryption.rs`):

```rust
pub struct EncryptionManager {
    key_scope: KeyScope, // Per-tenant or per-repo
}

impl EncryptionManager {
    pub fn encrypt_blob(&self, plaintext: &[u8]) -> Result<Vec<u8>>;
    pub fn decrypt_blob(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
}
```

### Testing

**Integration Tests** (`recovery/tests/integration/`):

- Crash simulation: kill process mid-write, verify journal replay
- Restore determinism: random commits → verify checksums
- GC correctness: mark protected refs, verify unreachable removed
- Budget enforcement: exceed limits, verify blocking/compaction
- Redaction: seed secrets, verify blocked at admission

---

## Dependencies

Add to `recovery/Cargo.toml`:

```toml
[dependencies]
blake3 = "1.5"
zstd = "0.13"
flate2 = "1.0"
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-native-tls"] }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
similar = "2.0"  # For unified diffs
fastcdc = "3.0"  # For content-defined chunking
```

---

## Success Criteria

**System-Level**:

- [ ] CAS stores deduplicated content with BLAKE3 addressing
- [ ] Journal + fsync guarantees atomicity across crashes
- [ ] Restore produces files matching commit tree digests
- [ ] GC removes unreachable objects while preserving protected refs
- [ ] CAWS budgets enforced with waiver path
- [ ] Multi-agent optimistic concurrency with conflict detection

**Performance**:

- [ ] Restore latency P95 < 5 seconds for typical sessions
- [ ] Dedupe ratio > 60% for iterative changes
- [ ] Storage growth < 2MB per iteration with compression

**Integration**:

- [ ] Self-prompting loop auto-checkpoints
- [ ] Workers restore from any session
- [ ] Database tracks recovery metadata
- [ ] CLI provides full operational control

---

## Rollout Strategy

1. **Week 1**: Build CAS foundations, journal, refs - validate with unit tests
2. **Week 2**: Add diffs, chunking, restore - integration test full cycle
3. **Week 3**: CAWS policy enforcement, GC - test budget/retention/redaction
4. **Week 4**: Agent integration, CLI tooling - end-to-end validation

**Pilot**: Enable on single V3 project with `.v3rec/` directory

**Canary**: Roll out to self-prompting loop with auto-checkpoint

**Production**: Enable CAWS budgets and scheduled GC

---

## Open Questions for Implementation

1. Should CDC chunking use gear hash (simpler) or Rabin fingerprints (proven)?
2. Pack format: custom binary or leverage existing (Git packfiles, protobuf)?
3. SQLite vs embedded Postgres for metadata index?
4. Conflict resolution strategies: auto-rebase for small hunks or always surface?

### To-dos

- [ ] Extend source-integrity crate with Digest newtype, MerkleTree implementation, and streaming hasher
- [ ] Create iterations/v3/recovery/ crate with module structure (cas, merkle, journal, refs, gc)
- [ ] Implement CAS blob storage with BLAKE3 addressing and zstd compression
- [ ] Implement write-ahead journal with fsync ordering and crash replay
- [ ] Implement refs manager for sessions, checkpoints, and labels
- [ ] Implement unified diff generation and application for text files
- [ ] Implement content-defined chunking with gear-based rolling hash
- [ ] Implement smart content strategy (full/diff/chunk) based on file size and diff ratio
- [ ] Implement optimistic concurrency with precondition digests and conflict detection
- [ ] Implement restore planning and execution with filters and dry-run support
- [ ] Implement CAWS policy enforcement (budgets, retention, compression config)
- [ ] Implement secret redaction with pattern matching and pre-admission blocking
- [ ] Implement reachability-based GC with protected refs and packing
- [ ] Integrate recovery tracking into self-prompting loop with auto-checkpoint
- [ ] Add restore capabilities to autonomous executor for recovery operations
- [ ] Create recov-cli tool with init, track, checkpoint, plan, restore, gc, fsck commands
- [ ] Add recovery session tracking to database artifact store
- [ ] Write comprehensive integration tests (crash simulation, restore verification, GC correctness)