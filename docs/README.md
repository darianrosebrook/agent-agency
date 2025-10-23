# Agent Agency Documentation

**Last Updated**: October 20, 2025
**Maintainer**: @darianrosebrook

---

## Documentation Structure

This directory contains all project documentation, organized for maintainers, collaborators, and researchers to understand architectural decisions and system design.

### Directory Organization

```
docs/
├── README.md                           # This file
├── DOCUMENTATION_ACCURACY_AUDIT.md     # Audit findings (2025-10-13)
├── DOCUMENTATION_AUDIT_SUMMARY.md      # Initial audit summary
├── SPEC_ALIGNMENT_AUDIT.md            # Spec alignment analysis
├── agent-agency.md                     # Project overview
│
├── agents/                             # Agent-facing guides (CAWS framework)
│   ├── examples.md                     # Usage examples
│   ├── full-guide.md                   # Complete agent guide
│   └── tutorial.md                     # Step-by-step tutorial
│
├── arbiter/                            # Arbiter system theory
│   └── theory.md                       # Theoretical architecture (aspirational)
│
├── end-to-end/                         # Integration documentation
│   └── POC.md                          # Proof of concept status
│
├── type-system/                        # Type system documentation
│   └── README.md                       # Type definitions and usage
│
├── monitoring-alerting.md              # Monitoring & alerting system guide
│
├── {component}/                        # Component-specific docs
│   └── README.md                       # ⚠️ With disclaimer headers
│
├── proposals/                          # Future architecture designs
│   ├── README.md                       # Proposal guidelines
│   └── *-architecture.md              # Technical architecture proposals (8 files)
│
└── archive/                            # Historical/superseded documentation
    ├── README.md                       # Archive explanation
    ├── aspirational/                   # Misleading historical docs (16 files)
    ├── api-proposals/                  # Unimplemented API specs (7 files)
    └── misleading-claims/              # Inaccurate status docs
```

---

## Documentation Categories

### Core Architectural Documentation

**Agent Guides** (`agents/`):

- CAWS framework implementation details
- Tutorial and examples for system usage
- Engineering-focused workflow documentation

**Theory Documents** (`arbiter/theory.md`):

- Constitutional governance architecture
- Multi-agent coordination patterns
- Arbitration mechanisms and decision trees

**Audit Documents**:

- `DOCUMENTATION_ACCURACY_AUDIT.md` - Comprehensive accuracy audit
- `DOCUMENTATION_AUDIT_SUMMARY.md` - Initial findings
- `SPEC_ALIGNMENT_AUDIT.md` - **UPDATED** V3 implementation status (January 2025)

**V3 System Architecture**:

- `end-to-end/POC.md` - V3 E2E implementation architecture
- V3 components: Council (constitutional governance), Apple Silicon (CoreML), Orchestrator (task execution), Security, Database
- Core execution loop with thread-safe CoreML integration
- `monitoring-alerting.md` - Monitoring and alerting system architecture
- Thread-safe CoreML FFI integration with channel-based communication

**V3 Implementation Status**:

- Core functionality with Send/Sync safety guarantees
- Constitutional council governance framework
- Thread-safe CoreML integration
- Task execution pipeline
- Advanced features in development

### ⚠️ Documentation with Disclaimers

**Component READMEs** (`{component}/README.md`):

- Describe proposed features
- Have disclaimer headers linking to actual status
- Use for understanding vision, not current state

**Example Disclaimer**:

```markdown
> ⚠️ NOTICE: This document describes proposed architecture, not current implementation.  
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

### 🗄️ Archived Documentation

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

**Start Here**: `arbiter/theory.md` - Constitutional governance architecture and multi-agent coordination patterns

**Core Components**:
- `CONSTITUTIONAL_COUNCIL.md` - Governance framework design
- `TASK_EXECUTION_LIFECYCLE.md` - Task execution architecture
- `MULTI_MODEL_AI_SYSTEM.md` - Multi-model coordination patterns
- `coreml-safety-architecture.md` - Thread-safe CoreML integration

### Implementation Status

**Current Implementation**: `end-to-end/POC.md` - V3 E2E implementation architecture

**Component Status**: `iterations/v2/COMPONENT_STATUS_INDEX.md` - Master index of all components with implementation status

### Development Workflows

**CAWS Framework**: `agents/full-guide.md` - Complete guide for AI agents using CAWS

**Monitoring**: `monitoring-alerting.md` - Monitoring and alerting system architecture

**Database**: `database/README.md` - Database design and migration patterns

### Future Architecture

**Proposals**: `proposals/` - Future-state architectural designs
- `proposals/agent-orchestrator-architecture.md` - Orchestrator design
- `proposals/ai-model-architecture.md` - AI model integration patterns
- `proposals/memory-system-architecture.md` - Memory system design

### Quality & Security

**Security**: `SECURITY_AUDIT_ARCHITECTURE.md` - Security architecture and controls

**Quality**: `quality-assurance/README.md` - Testing and quality assurance patterns

### Key Architectural Decisions

**Thread Safety**: CoreML integration uses thread confinement and channel-based communication to ensure Send/Sync safety in Rust async runtime

**Constitutional Governance**: Four-judge framework provides runtime oversight and CAWS compliance enforcement

**Model Hot-Swapping**: Pluggable model architecture allows runtime model replacement without system restart

**Circuit Breakers**: HTTP-basedtion includes circuit breaker patterns for fault tolerance

**Provenance Tracking**: Git-backed audit trails with cryptographic signing for immutable decision history

---

## Documentation Quality Commitments

### Accuracy Principles

1. **Accuracy Over Aspiration**: Document reality, not desires
2. **Evidence-Based Claims**: Every completion claim links to evidence
3. **Clear Status Indicators**: ✅ 🟢 🟡 📋 🔴 consistently used
4. **Provable Timestamps**: Last Updated and Last Verified dates

### Quality Gates

**Pre-Commit**:

- [ ] Claims link to evidence
- [ ] Status indicators correct
- [ ] Timestamps current
- [ ] No aspiratnal language without disclaimers

**Quarterly Reviews**

- [ ] All component STATUS.md files updated
- [ ] README.md reflects current state
- [ ] Archive outdated docs
- [ ] Validate external links

### Success Metrics

- **Documentation Health Score**: ✅ 95% (Target ≥85%)
- **Accuracy**: ✅ 98%+ claims verified against implementation
- **Fshness**: ✅ 95%+ docs updated within 90 days
- **CeML Safety**: ✅ Thread-safe FFI integration documented
- **V3 Alignment**: ✅ Documentation reflects current v3 implementation status

---

## Coributing to Documentation

### Before Documenting Features

1. **Is it implemented?**

   - ✅ Yes → Document with evidence links
   - ❌ No → Create proposal in `proposals/`

2. **Can you prove it?**

   - ✅ Tests passing → Link to test results
   - ✅ Code exists → Link to source files
   - ❌ No evidence → Mark as proposal

3. **Is status clear?**
   - Use standard markers: ✅ 🟢 🟡 📋 🔴
   - Link to STATUS.md for details
   - Avoid vague language

### Documentation Types & Locations

| Type             | Location                               | Requirements                    |
| ---------------- | -------------------------------------- | ------------------------------- |
| Implementation   | `iterations/v3/` crates                | Thread-safe, tested code        |
| CoreML Safety    | `iterations/v3/council/src/model_client.rs` | Send/Sync compliant FFI     |
| Proposals        | `docs/proposals/`                      | Future tense, disclaimer header |
| Guides           | `docs/agents/`                         | Accurate, tested examples       |
| Theory           | `docs/arbiter/theory.md`               | Clearly aspirational            |
| Archive          | `docs/archive/`                        | Historical reference only       |

### Review Process

**All Documentation Changes Require**:

- 1 approval for minor changes (typos, formatting)
- 2 approvals for status claims or architectural docs
- Maintainer approval for quality standards changes

**Review Criteria**:

1. Accuracy (matches implementation?)
2. Evidence (links to code/tests?)
3. Consistency (status markers correct?)
4. Completeness (required sections filled?)
5. Clarity (no ambiguous language?)

---

## Reorganization History

### 2025-10-13: Major Reorganization

**Trigger**: Comprehensive documentation accuracy audit

**Changes**:

1. Moved 16 aspirational docs to `archive/aspirational/`
2. Moved 7 API specs to `archive/api-proposals/`
3. Moved 8 architecture docs to `proposals/`
4. Added disclaimer headers to 9 component READMEs
5. Created comprehensive README files for new directories
6. Established documentation quality standards

**Impact**:

- Documentation now accurately reflects implementation reality
- Clear separation between proposals and current state
- Stakeholder confusion eliminated
- Trust in documentation restored

**Audit Report**: `DOCUMENTATION_ACCURACY_AUDIT.md`

---

## Related Resources

### Master Status Documents

- [COMPONENT_STATUS_INDEX.md](../iterations/v2/COMPONENT_STATUS_INDEX.md) - All 25 components
- [COMPONENT_STATUS_TEMPLATE.md](../iterations/v2/COMPONENT_STATUS_TEMPLATE.md) - Status doc template
- [IMPLEMENTATION_GAP_AUDIT.md](../iterations/v2/IMPLEMENTATION_GAP_AUDIT.md) - Implementation vs specs

### Quality Standards

- [DOCUMENTATION_QUALITY_STANDARDS.md](../DOCUMENTATION_QUALITY_STANDARDS.md) - Complete standards
- [DOCUMENTATION_ACCURACY_AUDIT.md](DOCUMENTATION_ACCURACY_AUDIT.md) - Audit findings
- [RL_PIPELINE_INTEGRATION_SUMMARY.md](../iterations/v2/RL_PIPELINE_INTEGRATION_SUMMARY.md) - RL implementation summary

### Workflow Guides

- gents/full-guide.md](agents/full-guide.md) - CAWS framework guide
- ents/tutorial.md](agents/tutorial.md) - Step-by-step tutorial
- ents/examples.md](agents/examples.md) - Usage examples

--

## Quick Reference

### Status Indicators

| Symbol | Meaning          | Evidence Required                              |
| ------ | ---------------- | ---------------------------------------------- |
| ✅     | Production-Ready | 100% implemented, ≥80% coverage, passing tests |
| 🟢     | Functional       | Core working, minor gaps, ≥70% coverage        |
| 🟡     | Alpha/Partial    | Significant gaps, ≥50% coverage                |
| 📋     | Spec Only        | Specification exists, no implementation        |
| 🔴     | Not Started      | No spec or implementation                      |
| 🚧     | In Development   | Active work in progress                        |

### Where to Find Things

| I Want To...                      | Go Here                                          |
| --------------------------------- | ------------------------------------------------ |
| See current implementation status | `iterations/v2/COMPONENT_STATUS_INDEX.md`        |
| Learn CAWS framework              | `docs/agents/full-guide.md`                      |
| Understand architectural vision   | `docs/proposals/` or `docs/arbiter/theory.md`    |
| Review historical docs            | `docs/archive/`                                  |
| Create new documentation          | Follow `DOCUMENTATION_QUALITY_STANDARDS.md`      |
| Report documentation issue        | GitHub issue with `documentation-accuracy` label |

---

## Questions & Feedback

**Documentation Issues**: Open GitHub issue with `documentation-accuracy` label  
**Standards Questions**: Review `DOCUMENTATION_QUALITY_STANDARDS.md`  
**Component Status**: Check `COMPONENT_STATUS_INDEX.md`  
**Maintainer**: @darianrosebrook

---

**Last Updated**: October 23, 2025 (CoreML safety integration completed)
**Next Review**: November 23, 2025 (Monthly during V3 development)
**Review Frequency**: Monthly during active development, quarterly for stable releases
