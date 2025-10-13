# Documentation Quality Standards

**Version**: 1.0.0  
**Last Updated**: 2025-10-13  
**Maintainer**: @darianrosebrook  
**Review Frequency**: Quarterly

---

## Purpose

Establish enforceable standards for documentation accuracy, consistency, and maintenance across Agent Agency. These standards prevent aspirational documentation from being mistaken for implementation reality.

---

## Core Principles

### 1. Accuracy Over Aspiration

**Rule**: Documentation must describe actual implementation, not desired features.

**Forbidden**:

- ❌ "Production-ready multi-tenant system" (when only spec exists)
- ❌ "Fully implemented agent orchestration" (when 20% complete)
- ❌ "Complete integration testing" (when tests are partial)

**Required**:

- ✅ "Multi-tenant system (specification complete, implementation 0%)"
- ✅ "Agent orchestration (5 of 25 components production-ready)"
- ✅ "Integration testing (22 of 28 tests passing)"

### 2. Evidence-Based Claims

**Rule**: Every claim of completion must link to verifiable evidence.

**Required Evidence**:

- Test results (e.g., "95.8% branch coverage")
- File references (e.g., "see src/registry/AgentRegistry.ts")
- Metrics (e.g., "47/47 tests passing")
- Status documents (e.g., "see STATUS.md")

### 3. Clear Status Indicators

**Rule**: Use standardized status markers consistently.

**Status Markers**:

- ✅ **Production-Ready**: 100% implemented, tested (≥80% coverage), documented
- 🟢 **Functional**: Core features work, minor gaps acceptable (≥70% coverage)
- 🟡 **Alpha/Partial**: Significant implementation, major gaps (≥50% coverage)
- 📋 **Spec Only**: Specification exists, no implementation
- 🔴 **Not Started**: No specification or implementation
- 🚧 **In Development**: Active work in progress

### 4. Provable Timestamps

**Rule**: All status documents must include "Last Updated" and "Last Verified" dates.

**Required Headers**:

```markdown
**Last Updated**: YYYY-MM-DD  
**Last Verified**: YYYY-MM-DD (or "Never")  
**Verification Method**: [Manual review | Automated tests | CI pipeline]
```

---

## Document Type Standards

### Component STATUS.md Files

**Required Sections**:

1. Executive Summary (honest 1-2 sentence assessment)
2. Implementation Status (✅ completed, 🟡 partial, ❌ missing)
3. Working Specification Status (spec file, validation, acceptance criteria)
4. Quality Metrics (test coverage, linting, performance)
5. Dependencies & Integration
6. Critical Path Items (must-haves vs nice-to-haves)
7. Risk Assessment
8. Timeline & Effort (realistic estimates)
9. Files & Directories (actual paths, not planned)
10. Status Assessment (honest conclusion with rationale)

**Template**: See [COMPONENT_STATUS_TEMPLATE.md](iterations/v2/COMPONENT_STATUS_TEMPLATE.md)

### README Files

**Required Elements**:

```markdown
> **Implementation Status**: [Status] ([X]% Complete) - [Evidence]  
> **Last Updated**: YYYY-MM-DD  
> **For Accurate Status**: [Link to detailed status]
```

**Content Requirements**:

- Distinguish "Implemented" vs "Proposed" features
- Link to component STATUS.md files
- Include test coverage metrics
- Provide realistic timelines

### Roadmap Documents

**Prohibited Format**:

```markdown
## Phase 1

- Implemented core feature # Past tense implies done
- Created database schema # Sounds complete when it's not
```

**Required Format**:

```markdown
## Phase 1 (TARGET: Q2 2025)

**Status**: 🟡 In Progress (40% complete)

- [x] Core feature foundation (STATUS.md)
- [ ] Database schema (planned, not started)
- [ ] Integration tests (blocked by COMPONENT-X)
```

**Alternative**: Move aspirational roadmaps to `docs/proposals/` directory

### API Documentation

**Requirements**:

- Mark proposed APIs: `<!-- PROPOSED - NOT IMPLEMENTED -->`
- Link specs to implementation: `<!-- Implemented in src/api/endpoint.ts -->`
- Validate contracts: `<!-- Contract tests: tests/contract/endpoint.test.ts -->`

**Directory Structure**:

```
docs/
├── api-proposals/      # Future API designs
└── api/                # Only documented APIs with implementations
```

---

## Quality Gates

### Pre-Commit Checks

**Automated Linters** (future enhancement):

```bash
# Check for aspirational language
forbidden_phrases=(
  "production-ready"
  "fully implemented"
  "complete"
  "deployed"
  "battle-tested"
)
```

**Manual Checklist**:

- [ ] Claims link to evidence
- [ ] Status indicators consistent
- [ ] Timestamps current
- [ ] Test coverage accurate

### Pre-Release Checks

Before any release:

- [ ] All STATUS.md files updated
- [ ] README.md reflects current state
- [ ] No aspirational language without disclaimers
- [ ] Documentation audit complete

### Quarterly Reviews

Every 3 months:

- [ ] Review all component STATUS.md files
- [ ] Update README.md with latest metrics
- [ ] Archive outdated docs
- [ ] Validate external links

---

## Review Process

### Documentation Changes

**Peer Review Required For**:

- All component STATUS.md changes
- README.md updates
- API documentation
- Architecture diagrams

**Self-Review Acceptable For**:

- Typo fixes
- Formatting improvements
- Link updates

### Review Criteria

Reviewers must verify:

1. **Accuracy**: Claims match implementation reality
2. **Evidence**: Links to code, tests, or metrics
3. **Consistency**: Status markers used correctly
4. **Completeness**: All required sections filled
5. **Clarity**: No ambiguous or misleading language

### Approval Process

**Documentation Changes**:

- 1 approval required for minor changes
- 2 approvals for major status claims
- Maintainer approval for architectural docs

---

## Maintenance Procedures

### Monthly Tasks

- [ ] Update component STATUS.md for changed components
- [ ] Run test suites and update coverage metrics
- [ ] Review and close stale documentation issues

### Quarterly Tasks

- [ ] Full documentation audit (see DOCUMENTATION_ACCURACY_AUDIT.md process)
- [ ] Update README.md with latest overall status
- [ ] Archive outdated or superseded documents
- [ ] Review and update this standards document

### Annual Tasks

- [ ] Comprehensive doc restructure if needed
- [ ] Update documentation tooling and automation
- [ ] Establish new quality standards based on lessons learned

---

## Enforcement Mechanisms

### CI/CD Integration (Future)

**Automated Checks**:

```yaml
# .github/workflows/docs-quality.yml
name: Documentation Quality

on: [pull_request]

jobs:
  check-accuracy:
    runs-on: ubuntu-latest
    steps:
      - name: Check for aspirational language
        run: |
          # Scan for forbidden phrases without evidence
          grep -r "production-ready" docs/ README.md
          if [ $? -eq 0 ]; then
            echo "Found 'production-ready' without evidence link"
            exit 1
          fi

      - name: Validate STATUS.md files
        run: |
          # Check all STATUS.md have required sections
          for file in $(find . -name "STATUS.md"); do
            if ! grep -q "## Executive Summary" "$file"; then
              echo "Missing required section in $file"
              exit 1
            fi
          done

      - name: Check timestamps
        run: |
          # Verify "Last Updated" is within 90 days
          python scripts/check-doc-freshness.py
```

### Pull Request Template

```markdown
## Documentation Changes

- [ ] All claims link to evidence (code, tests, metrics)
- [ ] Status indicators used correctly
- [ ] Timestamps updated
- [ ] No aspirational language without disclaimers
- [ ] Passed peer review

## Evidence

- Test coverage: [Link or N/A]
- Implementation files: [List or N/A]
- Status document: [Link or N/A]
```

---

## Common Violations & Fixes

### Violation 1: Vague Status

**Problem**:

```markdown
Status: Production Ready (with minor gaps)
```

**Fix**:

```markdown
Status: 🟡 Beta - Feature complete but needs hardening

**Gaps**:

- Test coverage: 75% (target: 90%)
- Performance: P95 150ms (target: 100ms)
- Security: Audit pending
```

### Violation 2: Unsubstantiated Claims

**Problem**:

```markdown
- ✅ Fully implemented multi-agent coordination
```

**Fix**:

```markdown
- 🟡 Multi-agent coordination partially implemented
  - **Complete**: Agent registry (ARBITER-001), task routing (ARBITER-002)
  - **Missing**: Conflict resolution (ARBITER-016), consensus mechanisms
  - **Evidence**: See [STATUS.md](components/task-routing-manager/STATUS.md)
```

### Violation 3: Past Tense in Roadmaps

**Problem**:

```markdown
## Phase 1

- Implemented core architecture
- Created database schemas
```

**Fix**:

```markdown
## Phase 1 (TARGET: Q2 2025)

**Status**: 🟡 40% Complete

- [x] Core architecture foundation (ARBITER-001, ARBITER-002 complete)
- [ ] Database schemas (in progress, 3 of 8 tables complete)
- [ ] Full integration (blocked by ARBITER-015)

**Evidence**: See [COMPONENT_STATUS_INDEX.md](iterations/v2/COMPONENT_STATUS_INDEX.md)
```

### Violation 4: Aspirational API Docs

**Problem**:

```markdown
# GET /api/agents/{id}/predictions

Returns AI-powered predictions for agent performance.
```

**Fix**:

```markdown
<!-- PROPOSED API - NOT IMPLEMENTED -->

# GET /api/agents/{id}/predictions

**Status**: 📋 Specification only  
**Implementation**: None  
**Depends On**: ARBITER-004 (Performance Tracker)

Returns AI-powered predictions for agent performance.
```

---

## Success Metrics

### Documentation Health Score

Calculate quarterly:

```
Documentation Health = (
  Accuracy Score (40%) +
  Evidence Score (30%) +
  Freshness Score (20%) +
  Consistency Score (10%)
) * 100

Where:
- Accuracy = % of claims verified against implementation
- Evidence = % of claims with linked evidence
- Freshness = % of docs updated within 90 days
- Consistency = % of docs using correct status markers
```

**Target**: ≥85% health score

### Review Metrics

Track monthly:

- Documentation changes reviewed: Target 100%
- Average review time: Target <2 hours
- Issues found per review: Track trend
- User-reported inaccuracies: Target <2 per quarter

---

## Training & Onboarding

### New Contributors

**Required Reading**:

1. This standards document
2. [COMPONENT_STATUS_TEMPLATE.md](iterations/v2/COMPONENT_STATUS_TEMPLATE.md)
3. [DOCUMENTATION_ACCURACY_AUDIT.md](docs/DOCUMENTATION_ACCURACY_AUDIT.md)

**Onboarding Checklist**:

- [ ] Read documentation standards
- [ ] Review 3 existing STATUS.md files
- [ ] Write STATUS.md for one component (with review)
- [ ] Understand evidence requirements

### Maintainers

**Additional Responsibilities**:

- Enforce standards in PR reviews
- Conduct quarterly documentation audits
- Update standards document based on learnings
- Mentor contributors on documentation quality

---

## Exceptions & Waivers

### Allowed Exceptions

**Theoretical Documents**:

- `docs/arbiter/theory.md` - Clearly aspirational architecture
- `docs/proposals/` - Proposed features (must be in proposals directory)
- Design documents marked as "PROPOSED" or "DESIGN ONLY"

**Requirement**: All theoretical/proposed docs must have clear disclaimer headers.

### Waiver Process

To document aspirational features outside `docs/proposals/`:

1. Add prominent disclaimer header:

   ```markdown
   > **NOTICE**: This document describes proposed architecture, not current implementation.  
   > **Implementation Status**: See [COMPONENT_STATUS_INDEX.md](iterations/v2/COMPONENT_STATUS_INDEX.md)  
   > **Last Verified**: Never (theoretical)
   ```

2. Get approval from 2 maintainers
3. Add to waiver registry: `docs/DOCUMENTATION_WAIVERS.md`

---

## Conclusion

These standards ensure Agent Agency documentation accurately reflects implementation reality, preventing confusion and misrepresentation. By enforcing evidence-based claims and clear status indicators, we maintain trust with users, contributors, and stakeholders.

**Key Takeaway**: If it's not implemented and tested, don't claim it's done.

---

## Version History

- **1.0.0** (2025-10-13): Initial standards document
  - Core principles established
  - Component STATUS.md requirements
  - Review processes defined
  - Enforcement mechanisms outlined

---

## Related Documents

- [COMPONENT_STATUS_TEMPLATE.md](iterations/v2/COMPONENT_STATUS_TEMPLATE.md) - Status doc template
- [COMPONENT_STATUS_INDEX.md](iterations/v2/COMPONENT_STATUS_INDEX.md) - Master component index
- [DOCUMENTATION_ACCURACY_AUDIT.md](docs/DOCUMENTATION_ACCURACY_AUDIT.md) - Audit findings and recommendations
- [IMPLEMENTATION_GAP_AUDIT.md](iterations/v2/IMPLEMENTATION_GAP_AUDIT.md) - Implementation status vs claims

---

**Maintained By**: @darianrosebrook  
**Next Review**: 2026-01-13 (Quarterly)  
**Contact**: Open an issue for clarifications or proposed changes
