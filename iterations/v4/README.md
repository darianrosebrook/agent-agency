# Agent Agency V4

A principled AI agent system built on Sterling's operator taxonomy and constitutional governance.

## Current State

**575 tests** across layers (as of 2026-01-25).

| Layer | Crates | Tests | Status |
|-------|--------|-------|--------|
| Core | v4-types, v4-invariants, v4-governance | 99 | ✅ |
| Reasoning | v4-symbolic, v4-council, v4-arbiter | 112 | ✅ |
| Infrastructure | v4-storage, v4-postgres, v4-inference, v4-memory, v4-observability | 113 | ✅ |
| Execution | v4-tools, v4-workers, v4-sandbox | 73 | ✅ |
| Interface | v4-api | 24 | ✅ |
| Integration | tests/ | 20 | ✅ |

## Quick Start

```bash
# Build
cargo build --workspace

# Test
cargo test

# Run API server
cargo run -p v4-api --bin v4-server
# Server starts on http://127.0.0.1:8080

# Test the API
curl http://localhost:8080/health
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Test", "description": "Read a file"}'

# Test LLM inference
curl -X POST http://localhost:8080/api/v1/probe \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Explain recursion", "max_tokens": 100}'
```

## Architecture

```
HTTP POST /api/v1/tasks
        │
        ▼
    v4-api (timing metrics)
        │
        ▼
TaskRequest → v4-symbolic → v4-council → v4-arbiter → v4-workers → v4-sandbox → v4-tools
                  │              │             │
                  │              │             └── Generate VerificationCertificate
                  │              └── 3 Judges: Constitutional, Technical, Quality
                  └── Deterministic operator selection with provenance
```

### Operator Taxonomy (S/M/P/K/C)

| Operator | Purpose | Side Effects |
|----------|---------|--------------|
| **S**eek | Information retrieval | No |
| **M**emorize | Store information | Yes |
| **P**erceive | Interpret input | No |
| **K**nowledge | Apply domain knowledge | No |
| **C**ontrol | Flow control | Yes |

### Security Levels

| Level | Filesystem | Network | Use Case |
|-------|------------|---------|----------|
| Permissive | Full | Yes | Development |
| Standard | /tmp | No | Default |
| Restricted | /tmp (read) | No | Sensitive |
| Strict | None | No | High-risk |

## Crate Structure

```
crates/
├── core/
│   ├── v4-types/         # Shared types, OperatorType, CouncilVerdict
│   ├── v4-invariants/    # Sterling invariants, CAWS constraints
│   └── v4-governance/    # CAWS gates, policy enforcement
│
├── reasoning/
│   ├── v4-symbolic/      # Operator graphs, rule engine, provenance
│   ├── v4-council/       # 3-judge evaluation, veto logic
│   └── v4-arbiter/       # Final decisions, certificates, routing
│
├── infrastructure/
│   ├── v4-storage/       # Content-addressable storage, event sourcing
│   ├── v4-postgres/      # PostgreSQL + pgvector for embeddings
│   ├── v4-inference/     # Local LLM inference (Mock, CoreML)
│   ├── v4-memory/        # Knowledge graph, Sterling-style decay
│   └── v4-observability/ # Metrics, tracing, health checks
│
├── execution/
│   ├── v4-tools/         # Tool trait, registry, built-in tools
│   ├── v4-workers/       # Worker pool, task queue, execution
│   └── v4-sandbox/       # Security policies, isolated execution
│
└── interfaces/
    └── v4-api/           # HTTP API server with timing metrics
```

## Key Invariants

- **INV-CORE-04**: Deterministic operator selection
- **INV-CORE-05**: Provenance required for decisions
- **INV-CORE-07**: Termination guarantee (bounded iterations)
- **INV-CORE-09**: Fail-closed on uncertainty
- **INV-CORE-10**: Cryptographic audit trail

## Documentation

- [Implementation Status](docs/IMPLEMENTATION_STATUS.md) - Current state and next steps
- [Unified Architecture](docs/UNIFIED_ARCHITECTURE.md) - Design rationale and patterns
- [V4 Source Mapping](docs/V4_SOURCE_MAPPING.md) - How V3 maps to V4

## Next Steps

1. **CoreML Backend**: Add CoreML inference for Apple Silicon (currently using MLX or mock provider)
2. **MCP Integration**: External tool protocol support
3. **Dashboard**: Connect to Next.js management UI

## License

See workspace root for license information.
