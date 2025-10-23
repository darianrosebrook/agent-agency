# Documentation Quality Guide

## Overview

This guide establishes comprehensive quality standards for all project documentation to prevent problematic content, ensure engineering-grade accuracy, and maintain professional standards.

## Core Principles

### 1. Engineering-Grade Focus
- **Architectural decisions** and rationale
- **Implementation details** and patterns  
- **System design** and component relationships
- **Technical specifications** and constraints
- **Integration points** and APIs
- **Quality gates** and verification methods

### 2. Reality Alignment
- **Documentation must match implementation reality**
- **No claims of features that don't work**
- **No APIs that aren't implemented**
- **No capabilities that are just placeholders**

### 3. Professional Standards
- **No marketing language or superiority claims**
- **No unfounded achievement claims**
- **No temporal documentation in main directories**
- **Accurate status reporting with evidence**

## Prohibited Content Patterns

### ❌ Superiority Claims (BANNED)

**Never use these terms:**
- `revolutionary`, `breakthrough`, `innovative`, `groundbreaking`
- `cutting-edge`, `state-of-the-art`, `next-generation`
- `advanced`, `premium`, `superior`, `best`, `leading`
- `industry-leading`, `award-winning`, `game-changing`

**Why banned:** These terms are marketing language that don't provide technical value and can mislead users about actual capabilities.

### ❌ Unfounded Achievement Claims (BANNED)

**Never claim these without verification:**
- `production-ready`, `enterprise-grade`, `battle-tested`
- `complete`, `finished`, `done`, `achieved`, `delivered`
- `implemented`, `operational`, `ready`, `deployed`
- `launched`, `released`, `100%`, `fully`
- `comprehensive`, `entire`, `total`, `all`, `every`
- `perfect`, `ideal`, `optimal`, `maximum`, `minimum`
- `unlimited`, `infinite`, `endless`

**Why banned:** These claims require evidence and verification. Without proof, they mislead users and create unrealistic expectations.

### ❌ Temporal Documentation (Archive Only)

**Never create these in main directories:**
- `SESSION_*_SUMMARY.md`
- `IMPLEMENTATION_STATUS.md`
- `TODO_*_COMPLETE.md`
- `*_SUMMARY.md`
- `*_REPORT.md`
- `*_AUDIT.md`
- `*_CHECKLIST.md`
- `PHASE*_*.md`
- `NEXT_ACTIONS.md`

**Why banned:** These are temporary tracking documents that clutter main documentation and become stale quickly.

## Required Content Standards

### ✅ Engineering-Grade Language

**Use these patterns instead:**
- `implemented` (not "production-ready")
- `operational` (not "enterprise-grade")
- `in development` (not "complete")
- `working` (not "finished")
- `functional` (not "achieved")
- `available` (not "delivered")

### ✅ Accurate Status Claims

**Before claiming any status, verify:**
- [ ] Feature fully implemented and tested
- [ ] All documented endpoints exist and work
- [ ] Code examples run without errors
- [ ] Installation instructions work on clean environment
- [ ] Performance claims backed by benchmarks
- [ ] Security claims validated by testing

### ✅ Professional Tone

**Maintain:**
- **Neutral, technical language**
- **Evidence-based claims**
- **Clear, concise explanations**
- **Consistent terminology**
- **Appropriate level of detail**

## Documentation Organization

### 📁 Directory Structure

**Root Directory** - Essential project files only:
- `README.md` - Main project overview and setup
- `CHANGELOG.md` - Version history and release notes
- `agents.md` - Core agent system documentation
- Configuration files (package.json, Cargo.toml, etc.)

**Documentation Directory** (`docs/`) - Active project documentation:
- Architecture documents (`README.md`, component docs)
- API documentation
- User guides and tutorials
- Active development documentation

**Archive Directory** (`docs/archive/`) - Historical and reference materials:
- Session summaries and progress reports
- Implementation tracking documents
- Completed project phases
- Historical assessments and audits

### 📋 Approved Documentation Locations

| Document Type | Location | Examples |
|---------------|----------|----------|
| **Session Reports** | `docs/archive/session-reports/` | `EXTENDED_SESSION_SUMMARY.md`, `SESSION_COMPLETION_SUMMARY.md` |
| **Implementation Tracking** | `docs/archive/implementation-tracking/` | `IMPLEMENTATION_STATUS.md`, `DEPLOYMENT_READINESS.md` |
| **Project Assessments** | `docs/archive/implementation-tracking/` | `EXECUTIVE_BRIEFING.md`, `SECURITY_HARDENING_AUDIT.md` |
| **Multi-Project Docs** | `docs/archive/multimodal-rag/` | `MULTIMODAL_RAG_*.md` |
| **Core Documentation** | `docs/` | `agent-agency.md`, component READMEs |
| **Version History** | Root | `CHANGELOG.md` |

## Quality Checklist

### Pre-Publication Verification

- [ ] No superiority claims or marketing language
- [ ] No unfounded achievement claims
- [ ] All status claims verified and accurate
- [ ] Code examples tested and working
- [ ] Installation instructions verified
- [ ] API documentation matches implementation
- [ ] No temporal documentation in main directories
- [ ] Professional, neutral tone throughout
- [ ] Focus on engineering-grade content

### Content Quality Gates

- [ ] **Accuracy**: All claims backed by evidence
- [ ] **Completeness**: All documented features implemented
- [ ] **Clarity**: Technical concepts clearly explained
- [ ] **Consistency**: Terminology used consistently
- [ ] **Currency**: Information reflects current state
- [ ] **Relevance**: Content serves maintainers/collaborators/researchers

## Automated Quality Checks

### Documentation Quality Linter

**Usage:**
```bash
# Run quality checks
python3 scripts/doc-quality-linter.py

# Run with specific path
python3 scripts/doc-quality-linter.py --path docs/

# Generate JSON report
python3 scripts/doc-quality-linter.py --format json

# Exit with error code if issues found
python3 scripts/doc-quality-linter.py --exit-code
```

**What it checks:**
- Superiority claims and marketing language
- Unfounded achievement claims
- Temporal documentation in wrong locations
- Emoji usage (except approved ones)
- Code example syntax issues

### Pre-Commit Hooks

**Setup:**
```bash
# Install documentation quality hooks
bash scripts/setup-doc-quality-hooks.sh
```

**What it prevents:**
- Commits with superiority claims
- Commits with unfounded achievements
- Commits with temporal docs in wrong locations
- Commits with marketing language

### CAWS Integration

**Quality Gates:**
- **Commit Gates**: No superiority claims, no temporal docs in main, max 5 warnings
- **Push Gates**: No superiority claims, no unfounded achievements, no temporal docs in main, max 2 warnings
- **Release Gates**: No superiority claims, no unfounded achievements, no temporal docs in main, no warnings, all code examples verified

## Correct Documentation Patterns

### ✅ Accurate Feature Documentation

```markdown
## Features

### Implemented
- User authentication with JWT tokens
- Real-time notifications via WebSocket
- Basic analytics dashboard

### In Development
- Advanced ML recommendations (Q2 2025)
- Multi-tenant isolation (Q1 2025)

### Planned
- AI-powered chat support (Q3 2025)
```

### ✅ Working Code Examples

```javascript
// POST /api/users
const response = await fetch("/api/users", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    Authorization: `Bearer ${token}`,
  },
  body: JSON.stringify({
    name: "John Doe",
    email: "john@example.com", // Required field included
    password: "securePassword123",
  }),
});

if (!response.ok) {
  throw new Error(`Failed to create user: ${response.statusText}`);
}

const user = await response.json();
console.log("Created user:", user);
```

### ✅ Realistic Status Claims

```markdown
# Project Status

**Current Status**: In Development (Proof of Concept)

**Implementation Progress**: 25% - Core authentication and basic API working, but missing:

- Database persistence (in-memory only)
- Security hardening
- Comprehensive testing
- Production deployment pipeline

**Next Milestone**: Alpha release (Q1 2025) with full persistence layer
```

## Quality Metrics

### Tracking Metrics

- **Accuracy Score**: % of claims verified by evidence
- **Reality Alignment**: % of documented features that work
- **Engineering Focus**: % of content focused on technical details
- **Marketing Language**: % of content free of marketing terms
- **Temporal Content**: % of content in appropriate directories

### Improvement Targets

- **Accuracy Score**: >95%
- **Reality Alignment**: >90%
- **Engineering Focus**: >80%
- **Marketing Language**: 100% free
- **Temporal Content**: 100% organized

## Continuous Improvement

### Regular Audits

**Monthly reviews:**
- Check for new problematic patterns
- Update prohibited terms list
- Review quality metrics
- Improve detection accuracy

### Feedback Integration

**User feedback:**
- Report problematic content
- Suggest quality improvements
- Identify new patterns to detect
- Validate quality metrics

### Rule Evolution

**Update rules based on:**
- New problematic patterns discovered
- Industry best practices
- User feedback and complaints
- Quality metric trends
- Production incident analysis

## Enforcement

### Automatic Prevention

- **IDE warnings** when creating files in inappropriate locations
- **Pre-commit hooks** block commits of problematic content
- **Documentation lints** flag issues automatically
- **CI/CD integration** prevents deployment of problematic docs

### Manual Correction

- **Move misplaced files** to appropriate archive locations
- **Update file references** in related documentation
- **Clean up duplicate** or redundant documentation
- **Review and approve** all documentation changes

## Integration with CAWS

### CAWS Quality Standards

These documentation quality standards integrate with CAWS quality tiers:

| Tier | Documentation Quality | Use Case |
|------|----------------------|----------|
| **T1** | No marketing language, 100% accuracy | Auth, billing, migrations |
| **T2** | No superiority claims, 95% accuracy | Features, APIs, data writes |
| **T3** | Professional tone, 90% accuracy | UI, internal tools |

### CAWS Workflow Integration

- **Validation**: `caws validate` checks documentation quality
- **Testing**: Documentation examples tested in CI
- **Quality Gates**: Automated enforcement of standards
- **Provenance**: Documentation changes tracked and attributable

## Summary

This guide ensures documentation maintains engineering-grade quality and prevents the accumulation of problematic content. By following these standards, we create documentation that serves maintainers, collaborators, and researchers with accurate, professional, and technically valuable content.

**Key Takeaways:**
1. **Focus on engineering-grade content** with architectural decisions and implementation details
2. **Avoid marketing language** and superiority claims
3. **Verify all status claims** with evidence
4. **Organize temporal content** in appropriate archive directories
5. **Use automated tools** to prevent problematic content
6. **Maintain professional standards** throughout all documentation
