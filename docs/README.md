# Agent Agency Documentation

**Last Updated**: November 6, 2025
**Maintainer**: @darianrosebrook

---

## Documentation Structure

This directory contains architectural documentation for the Agent Agency system, organized to support implementation and maintenance activities.

### Directory Organization

```
docs/
├── README.md                           # This file - documentation overview
├── agent-agency.md                     # Core system architecture
├── coreml-acceleration.md              # Core ML acceleration system architecture
│
├── api/                                # API documentation
│   └── coreml-acceleration.md          # Core ML acceleration API reference
│
├── agents/                             # Agent system guides
│   ├── examples.md                     # Usage examples
│   ├── full-guide.md                   # Complete CAWS implementation guide
│   └── tutorial.md                     # Getting started tutorial
│
├── arbiter/                            # Arbiter system documentation
│   └── theory.md                       # Arbiter theoretical architecture
│
├── end-to-end/                         # Integration documentation
│   └── POC.md                          # Proof of concept implementation
│
├── type-system/                        # Type system documentation
│   └── README.md                       # Type definitions and contracts
│
├── monitoring-alerting.md              # Monitoring and alerting system
│
├── {component}/                        # Component-specific documentation
│   └── README.md                       # Component architecture and usage
│
├── proposals/                          # Architecture design proposals
│   ├── README.md                       # Proposal development guidelines
│   └── *-architecture.md              # Technical architecture proposals
│
└── archive/                            # Historical documentation
    ├── README.md                       # Archive organization and purpose
    ├── aspirational/                   # Theoretical designs
    ├── api-proposals/                  # API specification drafts
    └── audits/                         # Historical audit reports
```

---

## Documentation Categories

### Core Architectural Documentation

**System Architecture** (`agent-agency.md`):
- Overall system design and component relationships
- Data flow patterns and integration points
- Quality assurance and deployment architecture

**Core ML Acceleration** (`coreml-acceleration.md`):
- Apple Silicon hardware acceleration system
- Performance characteristics and device compatibility
- Integration with agent execution pipeline

**API Documentation** (`api/`):
- Public interface specifications
- Usage examples and error handling
- Configuration and environment variables

**Agent System Guides** (`agents/`):
- CAWS framework implementation
- Tutorial and examples for system usage
- Workflow documentation and patterns

**Component Documentation** (`{component}/`):
- Individual component architecture and interfaces
- Implementation details and design decisions
- Integration patterns and dependencies

### Historical Documentation

**Archive** (`archive/`):
- Superseded design documents
- Historical implementation records
- Audit reports and analysis results

### Design Proposals

**Architecture Proposals** (`proposals/`):
- Future system design considerations
- Technical architecture explorations
- Component evolution planning  
> Implementation Status: See COMPONENT_STATUS_INDEX.md for actual status.
```

### Proposal Documentation

**Location**: `proposals/`

**Contains**: Future-state architectural designs

**Use For**:

- Architectural planning and reference
- Understanding intended designs
- **NOT** for claiming current implementation

**See**: `proposals/README.md` for full guidelines

###  Archived Documentation

**Location**: `archive/`

**Contains**: Superseded or misleading historical docs

**Categories**:

1. **Aspirational** (`archive/aspirational/`): Past-tense roadmaps and summaries that implied completion (16 files)
2. **API Proposals** (`archive/api-proposals/`): Unimplemented API specs (7 files)
3. **Misleading Claims** (`archive/misleading-claims/`): Inaccurate status documents

**Use For**: Historical context only, not current state

**See**: `archive/README.md` for full explanation

---

## Architectural Navigation Guide

### System Architecture Overview

**Start Here**: `agent-agency.md` - Overall system architecture and component relationships

**Core Components**:
- `coreml-acceleration.md` - Core ML acceleration system implementation
- `api/coreml-acceleration.md` - Core ML acceleration API reference
- `monitoring-alerting.md` - Monitoring and alerting system

---

## Key System Components

### Core Systems
- **Agent Agency**: Main orchestration system (`agent-agency.md`)
- **Core ML Acceleration**: Apple Silicon AI acceleration (`coreml-acceleration.md`)
- **Agent Memory**: Knowledge persistence system (`agent-memory/`)
- **Data Layer**: Database abstraction (`data-layer/`)
- **Quality Assurance**: Testing and validation (`quality-assurance/`)

### Integration Points
- **API Layer**: REST API specifications (`api/`)
- **Type System**: Shared type definitions (`type-system/`)
- **Monitoring**: Observability and alerting (`monitoring-alerting.md`)

---

## Documentation Standards

All documentation in this directory follows engineering-grade standards:

- **No marketing language** or superiority claims
- **Evidence-based claims** backed by working code
- **Accurate status reporting** without unfounded achievements
- **Architectural focus** on design and implementation details

Temporal documentation (progress reports, session summaries) is maintained in `docs-status/` and excluded from version control.

---

## Contributing to Documentation

When adding or updating documentation:

1. **Focus on architecture**: Document design decisions, interfaces, and implementation details
2. **Provide evidence**: Link to source code, tests, and working examples
3. **Use engineering language**: Avoid marketing terms and unfounded claims
4. **Keep current**: Update documentation when implementation changes

---

*This documentation describes the implemented Agent Agency system architecture as of the Core ML acceleration completion.*

