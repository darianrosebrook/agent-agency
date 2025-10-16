# CAWS-Verifiable Provenance Schema

## Overview

This document defines the schema extensions for CAWS-verifiable provenance in benchmark data pools. Provenance must be immutable and cryptographically signed to ensure that training data is _constitutionally verifiable_.

## Core Schema Extension

```yaml
caws_metadata:
  spec_id: string # Reference to working-spec.yaml
  verdict_id: string # Unique verdict identifier
  waivers_used: [string] # List of waiver IDs applied
  gates_passed: [string] # Quality gates that passed
  scorecard:
    evidence_completeness: float # 0-1: Proof obligations satisfied
    budget_adherence: float # 0-1: Budget compliance score
    gate_integrity: float # 0-1: Gate execution integrity
    provenance_clarity: float # 0-1: Audit trail completeness
  signature: string # ed25519 or secure enclave signature
  timestamp: string # ISO 8601 with millisecond precision
  constitutional_context: # CAWS clause references
    budget_clauses: [string] # ["CAWS:Section4.2"]
    waiver_clauses: [string] # ["CAWS:Section5.1"]
    gate_clauses: [string] # ["CAWS:Section6.4"]
```

## Implementation Strategy

### Provenance Ledger

- **SQLite-based**: Lightweight, deterministic ordering with `verdict_id` as primary key
- **Hash Chain**: Each verdict includes SHA-256 hash of previous verdict for immutability
- **Append-Only**: `verdict_chain.log` functions as lightweight blockchain substitute

### Cryptographic Integrity

- **Signature Algorithm**: ed25519 for performance and security
- **Key Management**: Hardware-backed secure enclave when available
- **Chain Validation**: Automated verification of hash chain integrity

### Constitutional Binding

Every Arbiter verdict includes hash of `.caws/working-spec.yaml` used at decision time, binding verdicts to their constitutional context.

## Usage in RL Training

The CAWS metadata enables governance-weighted reward functions:

```python
def calculate_reward(verdict_data):
    Q = verdict_data.success_rate
    waiver_rate = len(verdict_data.waivers_used) / total_gates
    G = verdict_data.scorecard.gate_integrity
    M = verdict_data.scorecard.provenance_clarity

    return (0.4 * Q) + (0.3 * (1 - waiver_rate)) + (0.2 * G) + (0.1 * M)
```

This ensures RL systems are rewarded for constitutional compliance, not just task success.
