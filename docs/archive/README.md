# Documentation Archive

**Purpose**: Historical and superseded documentation moved during reorganization (2025-10-13)

---

## Directory Structure

### `archive/aspirational/`

**Contains**: Aspirational planning documents that described features as if implemented

**Why Archived**: These documents used past tense and completion language for unimplemented features, creating misleading impressions about project status.

**Files Archived** (16 total):

- **8 Implementation Roadmaps**: Past-tense roadmaps that implied completion
  - Format: `{component}-roadmap.md`
  - Original location: `docs/{component}/implementation-roadmap.md`
- **8 Summary Documents**: Executive summaries claiming "COMPLETE" status
  - Format: `{component}-summary.md`
  - Original location: `docs/{component}/SUMMARY.md`

**Components Archived**:

- MCP
- agent-memory
- agent-orchestrator
- ai-model
- data-layer
- mcp-integration
- memory-system
- quality-assurance

### `archive/api-proposals/`

**Contains**: OpenAPI specifications for APIs that may not be implemented

**Why Archived**: These API specs describe endpoints without verified backend implementations. They represent proposed APIs, not documented reality.

**Files Archived** (7 total):

- `agent-orchestrator.yaml`
- `ai-model.yaml`
- `data-layer.yaml`
- `mcp-protocol.yaml`
- `mcp-tools.yaml`
- `memory-system.yaml`
- `quality-gates.yaml`

**Original Location**: `docs/api/`

### `archive/misleading-claims/`

**Contains**: Documents with inaccurate completion claims

**Existing Files**:

- `SPEC_VALIDATION_SUMMARY.md` - Claimed "100% COMPLIANT" for unvalidated specs

---

## Why This Archive Exists

During the 2025-10-13 documentation audit (see `DOCUMENTATION_ACCURACY_AUDIT.md`), we discovered:

1. **80% of component READMEs** described unimplemented features as if they existed
2. **Implementation roadmaps** used past tense, implying tasks were complete
3. **API documentation** existed for non-existent endpoints
4. **Summary documents** claimed "COMPLETE" status for specification-only components

**Impact**: Stakeholders were misled about project maturity and implementation status.

**Solution**: Reorganize documentation to separate:

- **Current documentation** (matches implementation reality)
- **Proposals** (clearly marked future designs)
- **Archive** (superseded or misleading historical docs)

---

## How to Use Archived Documents

### If You Need Historical Context

These documents are preserved for reference but should not be cited as current state. Use them to understand:

- Original architectural vision
- Planning history
- Evolution of requirements

### If You Want to Revive Features

1. Review the archived document for context
2. Create new CAWS-compliant working spec in `iterations/v2/components/{component}/.caws/working-spec.yaml`
3. Update `COMPONENT_STATUS_INDEX.md` with current status
4. Implement with TDD approach and evidence-based documentation

### If You Find Inaccuracies

Report issues via:

- GitHub issues with label `documentation-accuracy`
- Direct feedback to maintainers

---

## Reorganization Details

**Date**: 2025-10-13  
**Audit Document**: `docs/DOCUMENTATION_ACCURACY_AUDIT.md`  
**Standards**: `DOCUMENTATION_QUALITY_STANDARDS.md`  
**Authority**: @darianrosebrook

**Changes Made**:

- Moved 16 aspirational docs from component dirs to `archive/aspirational/`
- Moved 7 API specs from `docs/api/` to `archive/api-proposals/`
- Added disclaimer headers to remaining component READMEs
- Established documentation quality standards

---

## Related Documents

- [DOCUMENTATION_ACCURACY_AUDIT.md](../DOCUMENTATION_ACCURACY_AUDIT.md) - Full audit findings
- [DOCUMENTATION_QUALITY_STANDARDS.md](../../DOCUMENTATION_QUALITY_STANDARDS.md) - Quality standards
- [COMPONENT_STATUS_INDEX.md](../../iterations/v2/COMPONENT_STATUS_INDEX.md) - Current component status

---

**Note**: This archive represents a commitment to documentation honesty. By clearly separating aspirational documents from implementation reality, we maintain stakeholder trust and prevent confusion.
