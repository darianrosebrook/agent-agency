# TODO[runtime-validator-integration]: Implement CAWS runtime validator crate

## Requirements
1. **Policy ingestion**
   - Load `.caws` policy bundles (working spec, scope, risk tier) with schema validation.
   - Support deterministic overrides for integration tests via fixtures.
   - Emit descriptive errors when policy inputs are incomplete or malformed.
2. **Validation pipeline**
   - Run lint, diff, test, and contract checks as discrete stages with structured outputs.
   - Produce CAWS rule references and remediation guidance for each violation.
   - Allow dependency injection of check adapters (e.g., `cargo`, `npm`, custom scripts) for mocks.
3. **Integration surfaces**
   - Expose async API consumed by `orchestration::orchestrate_task` and integration tests.
   - Stream validation events to provenance service with deterministic ordering.
   - Provide snapshot-friendly summaries so tests can assert outcomes without a git repository.
4. **Observability & rollback**
   - Record timing/metrics per validation stage for Tier-1 SLA verification.
   - Support rollback hooks that detail which files/commands must be undone on failure.

## Acceptance Criteria
- Integration tests using `TestFixtures::consensus_infrastructure_bundle` can invoke the validator
  API and assert stage-by-stage results that match fixture expectations.
- Runtime validation failures surface typed errors with CAWS rule codes, enabling orchestration to
  short-circuit tasks deterministically.
- Provenance entries capture validation spans, pass/fail status, and remediation notes, and tests
  confirm their presence via the provenance emitter.
- Snapshot-based environments (no git) can still diff validation output using the upcoming
  `snapshot_diff_plan` fixture without losing audit fidelity.
