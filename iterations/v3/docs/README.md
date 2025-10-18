# Agent Agency V3 Documentation

This directory contains the core architectural documentation for V3 - the maintainable, long-term documentation that defines the system's design and contracts.

## Core Architecture

- **[architecture.md](./architecture.md)** - High-level system architecture, design principles, and current implementation status
- **[interaction-contracts.md](./interaction-contracts.md)** - API contracts and interaction patterns between components
- **[agents.md](./agents.md)** - CAWS agent workflow guide and collaboration patterns
- **[coordinating-concurrency.md](./coordinating-concurrency.md)** - Framework for concurrent agent coordination within council-based systems

## Component Documentation

- **[components/](./components/)** - Detailed documentation for each system component:
  - [council.md](./components/council.md) - Council of judges system
  - [worker-pool.md](./components/worker-pool.md) - Worker pool management
  - [research-agent.md](./components/research-agent.md) - Research agent system
  - [orchestration-core.md](./components/orchestration-core.md) - Core orchestration
  - [apple-silicon-optimization.md](./components/apple-silicon-optimization.md) - Apple Silicon optimization

## API Contracts & Schemas

- **[contracts/](./contracts/)** - JSON schemas and contract definitions:
  - [final-verdict.schema.json](./contracts/final-verdict.schema.json) - Council verdict format
  - [judge-verdict.schema.json](./contracts/judge-verdict.schema.json) - Individual judge verdicts
  - [worker-output.schema.json](./contracts/worker-output.schema.json) - Worker output format
  - [router-decision.schema.json](./contracts/router-decision.schema.json) - Task routing decisions

## Architectural Decision Records (ADRs)

- **[adr/](./adr/)** - Key architectural decisions:
  - [ADR-001-consensus-weighting.md](./adr/ADR-001-consensus-weighting.md) - Consensus algorithm design
  - [ADR-002-quantization-placement.md](./adr/ADR-002-quantization-placement.md) - Model quantization strategy
  - [ADR-003-verdict-provenance.md](./adr/ADR-003-verdict-provenance.md) - Verdict audit trail design
  - [ADR-004-research-policy.md](./adr/ADR-004-research-policy.md) - Research agent policies

## Specialized Documentation

- **[caws-runtime-validator.md](./caws-runtime-validator.md)** - CAWS runtime validation system
- **[database/provenance.md](./database/provenance.md)** - Database provenance tracking design

## Integration Patterns

- **[INTEGRATION_PATTERNS.md](./INTEGRATION_PATTERNS.md)** - How components integrate and communicate

---

## Documentation Organization

This directory contains only **maintainable, architectural documentation** that defines the system's design and contracts. Temporal documentation (status reports, progress summaries, gap analyses) has been moved to `docs-status/` (git-ignored) to keep this directory focused on the core architecture.

For implementation status and progress tracking, see the main project README.md.
