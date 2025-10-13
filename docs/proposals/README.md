# Documentation Proposals

**Purpose**: Future-state architectural designs and specifications

> **⚠️ NOTICE**: All documents in this directory describe PROPOSED architectures, not current implementations.  
> **For Current Status**: See [COMPONENT_STATUS_INDEX.md](../../iterations/v2/COMPONENT_STATUS_INDEX.md)

---

## Directory Contents

### Technical Architecture Proposals

**What**: Detailed architectural designs for planned components

**Files** (8 total):

- `MCP-architecture.md` - Model Context Protocol integration design
- `agent-memory-architecture.md` - Agent memory system architecture
- `agent-orchestrator-architecture.md` - Orchestration layer design
- `ai-model-architecture.md` - AI model management architecture
- `data-layer-architecture.md` - Database and persistence design
- `mcp-integration-architecture.md` - MCP integration patterns
- `memory-system-architecture.md` - Memory subsystem design
- `quality-assurance-architecture.md` - QA framework design

**Original Location**: `docs/{component}/technical-architecture.md`

---

## How to Use Proposal Documents

### For Architectural Planning

These documents represent:

- ✅ Thoughtful design work
- ✅ Valid architectural patterns
- ✅ Reference implementations
- ⚠️ **NOT** current implementation state

**Before Implementing**:

1. Review the proposal for architectural guidance
2. Create CAWS-compliant working spec
3. Update with implementation realities
4. Move completed docs out of proposals

### For Contributing

**To Propose New Architecture**:

1. Create `{component}-architecture.md` in this directory
2. Add proposal disclaimer header
3. Link to related working specs (if exist)
4. Submit PR with `proposal` label

**To Update Existing Proposal**:

1. Review current implementation status
2. Update proposal to reflect learnings
3. Note changes in file header
4. Consider splitting if proposal diverged significantly

### For Implementation

**Before claiming "implemented"**:

- [ ] Create working spec in `iterations/v2/components/{component}/.caws/working-spec.yaml`
- [ ] Implement with ≥80% test coverage
- [ ] Update COMPONENT_STATUS_INDEX.md
- [ ] Move proposal to `docs/{component}/` as reference (or archive if superseded)

---

## Proposal Document Standards

### Required Header

```markdown
> **STATUS**: Architectural Proposal (Not Implemented)  
> **Last Updated**: YYYY-MM-DD  
> **Implementation Status**: See [COMPONENT_STATUS_INDEX.md](../../iterations/v2/COMPONENT_STATUS_INDEX.md)  
> **Related Specs**: [Link to working spec if exists]

---
```

### Document Structure

1. **Overview**: High-level architectural vision
2. **Components**: Detailed component breakdown
3. **Integration Points**: How it fits in the system
4. **Data Flows**: Information flow diagrams
5. **Implementation Considerations**: Challenges, trade-offs
6. **Future Work**: Potential enhancements

### Language Guidelines

**Use Future Tense**:

- ✅ "The system will provide..."
- ✅ "Components should integrate..."
- ✅ "This design proposes..."

**Avoid Past/Present as Fact**:

- ❌ "The system provides..." (implies implemented)
- ❌ "Components integrate..." (claims current state)
- ❌ "This design implements..." (suggests complete)

---

## Promotion Path: Proposal → Implementation

### Stage 1: Proposal

- **Location**: `docs/proposals/{component}-architecture.md`
- **Status**: Design document
- **Evidence**: None required

### Stage 2: Specification

- **Location**: `iterations/v2/components/{component}/.caws/working-spec.yaml`
- **Status**: CAWS-validated spec
- **Evidence**: `caws validate` passes

### Stage 3: Implementation

- **Location**: `src/{component}/`
- **Status**: Code exists with tests
- **Evidence**: ≥80% coverage, passing tests

### Stage 4: Production-Ready

- **Location**: Full integration
- **Status**: Deployed and monitored
- **Evidence**: Production metrics, user feedback

**Proposal Lifecycle**:

```
proposals/{component}-architecture.md
  → .caws/working-spec.yaml (Stage 2)
  → src/{component}/ (Stage 3)
  → docs/{component}/architecture.md (Stage 4, reference)
  OR
  → archive/ (if superseded)
```

---

## Current Proposals Status

| Proposal                        | Working Spec | Implementation   | Status        |
| ------------------------------- | ------------ | ---------------- | ------------- |
| MCP-architecture                | ❌ Missing   | 🟡 Partial (POC) | Needs Spec    |
| agent-memory-architecture       | ❌ Missing   | ❌ None          | Proposal Only |
| agent-orchestrator-architecture | ✅ Exists    | 🟡 Partial       | In Progress   |
| ai-model-architecture           | ❌ Missing   | ❌ None          | Proposal Only |
| data-layer-architecture         | ❌ Missing   | 🟡 Partial       | Needs Spec    |
| mcp-integration-architecture    | ❌ Missing   | 🟡 Partial       | Needs Spec    |
| memory-system-architecture      | ❌ Missing   | ❌ None          | Proposal Only |
| quality-assurance-architecture  | ✅ Exists    | 🟡 Partial       | In Progress   |

**For Detailed Status**: See [COMPONENT_STATUS_INDEX.md](../../iterations/v2/COMPONENT_STATUS_INDEX.md)

---

## Review Process

### Proposal Reviews

**Purpose**: Ensure architectural soundness before implementation investment

**Criteria**:

- Clear problem statement
- Well-defined components
- Integration points documented
- Trade-offs analyzed
- Implementation considerations noted

**Reviewers**: 2+ team members (architecture focus)

### Promotion Reviews

**Purpose**: Validate proposal → implementation alignment

**Criteria**:

- Implementation matches proposal intent
- Deviations documented and justified
- Tests validate architectural assumptions
- Performance meets expectations

**Reviewers**: Original proposal author + 1 maintainer

---

## Related Documents

- [DOCUMENTATION_QUALITY_STANDARDS.md](../../DOCUMENTATION_QUALITY_STANDARDS.md) - Documentation standards
- [COMPONENT_STATUS_INDEX.md](../../iterations/v2/COMPONENT_STATUS_INDEX.md) - Current implementation status
- [DOCUMENTATION_ACCURACY_AUDIT.md](../DOCUMENTATION_ACCURACY_AUDIT.md) - Audit that prompted reorganization

---

## Questions?

- **"Why is my architecture doc here?"**: To clearly indicate it's a proposal, not current implementation
- **"When can I move it out?"**: When component is fully implemented with evidence (tests, metrics)
- **"Should I update old proposals?"**: Yes! Proposals are living documents that evolve with understanding

---

**Maintained By**: @darianrosebrook  
**Next Review**: Quarterly with component status reviews  
**Last Reorganization**: 2025-10-13
