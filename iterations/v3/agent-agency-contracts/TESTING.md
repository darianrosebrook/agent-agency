# Contract Testing Infrastructure

This document describes the testing infrastructure for `agent-agency-contracts`.

## Test Suites

### Round-Trip Serialization Tests (`tests/round_trip_serde.rs`)

Validates that all contract types can be serialized to JSON and deserialized back to the same values, ensuring:

- **Forward Compatibility**: Types remain stable across schema evolution
- **Serde Correctness**: Serialization/deserialization works correctly
- **Type Preservation**: Values are preserved through the round-trip

**Run**: `cargo test --test round_trip_serde`

**Coverage**:
- All enum types (TaskPriority, ExecutionMode, RiskTier, CouncilVerdict)
- All struct types (BlastRadius, TaskScope, ExecutionContext, Milestone, etc.)
- Optional fields handling
- UUID serialization compatibility

### Schema Snapshot Tests (`tests/schema_snapshot.rs`)

Ensures JSON Schema generation remains stable and matches expected snapshots. Detects accidental schema changes that could break API compatibility.

**Run**: `cargo test --test schema_snapshot`

**Coverage**:
- Schema generation for all exported types
- Schema structure validation
- Field presence verification
- Enum schema validation

**Schema Snapshots**: Generated in `target/schemas/` directory

### Contract Validation Tests (`tests/examples.rs`)

Validates example JSON files against their schemas, ensuring contract examples remain valid.

**Run**: `cargo test --test examples`

## Public API Checks

Public API stability is enforced via CI scripts:

- **`scripts/check-contracts-api.sh`**: Detects removed/changed public items
- **`scripts/check-duplicate-types.sh`**: Detects duplicate type definitions

**Requirements**:
- `cargo-public-api`
- `cargo-semver-checks`

**Setup**:
```bash
cargo install cargo-public-api cargo-semver-checks
```

## Running All Tests

```bash
cd iterations/v3/agent-agency-contracts
cargo test
```

## Test Maintenance

When adding new types to contracts:

1. Add round-trip test in `tests/round_trip_serde.rs`
2. Add schema snapshot test in `tests/schema_snapshot.rs`
3. Update public API snapshot: `scripts/check-contracts-api.sh`
4. Verify no duplicate types: `scripts/check-duplicate-types.sh`

