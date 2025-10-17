# V3 TODO Audit

Purpose: Track critical TODOs blocking or sequencing integration. Keep entries concise and actionable. Update as implementations land.

- [ ] Orchestration: pass real patches/language hints to validator
  - File: `v3/orchestration/src/orchestrate.rs:~36`
  - Note: Replace empty slices with data from worker artifacts (patch diffs + file extensions)

- [ ] Orchestration: map full TaskSpec fields
  - File: `v3/orchestration/src/orchestrate.rs:~10`
  - Note: Populate acceptance_criteria and description from WorkingSpec/TaskDescriptor once available

- [ ] Validator: replace NoopMde with real MDE
  - File: `v3/orchestration/src/caws_runtime.rs:~95`
  - Note: Implement MinimalDiffEvaluator and calibrate thresholds per tier/language

- [ ] Validator: expand MDE violation mapping
  - File: `v3/orchestration/src/caws_runtime.rs:~132`
  - Note: Map risky_patterns to specific violation codes (e.g., LargeRefactor, GeneratedCodeDump)

- [ ] Persistence: persist waivers alongside verdicts
  - File: `v3/orchestration/src/persistence_postgres.rs:~37`
  - Note: Call persist_waivers() after final decision; consider FK to verdict_id

- [ ] Persistence: add signing + hash chain per ADR-003
  - File: `v3/orchestration/src/persistence_postgres.rs:~20`
  - Note: Introduce signer trait, store signature/hash_chain into verdicts table

- [ ] Coordinator: accept validator + research context
  - File: `v3/council/src/coordinator.rs:~60`
  - Note: Extend evaluation context to include ValidationResult and research evidence bundle

- [ ] Coordinator: populate remediation and constitutional_refs from inputs
  - File: `v3/council/src/coordinator.rs:~130`
  - Note: Merge validator remediation and refs into FinalVerdict consistently

- [ ] Coordinator: compute verification_summary when claims present
  - File: `v3/council/src/coordinator.rs:~150`
  - Note: Use WorkerOutput.claims/evidence_refs to build coverage stats

- [ ] DB: tune pool and retries per environment
  - File: `v3/orchestration/src/db.rs:~8`
  - Note: Configurable timeouts, retries, and backoff

- [ ] Docs: link MDE implementation doc when created
  - File: `v3/docs/caws-runtime-validator.md:~28`
  - Note: Add link once MDE module exists

- [ ] Docs: signer and git trailer integration guide
  - File: `v3/docs/database/provenance.md:~20`
  - Note: Document signer trait usage and git trailer workflow

Legend: line numbers are approximate (~). Update them when code moves.
