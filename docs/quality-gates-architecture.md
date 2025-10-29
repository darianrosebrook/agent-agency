# Quality Gates System Architecture

## Overview

The Agent Agency project implements a comprehensive, multi-layered quality gates system that operates across four primary components:

1. **Cursor IDE Rules** (`.cursor/rules/`) - Real-time development guidance
2. **CAWS Framework** (`apps/tools/caws/`) - Comprehensive quality assessment
3. **Git Hooks** (`.git/hooks/`) - Pre-commit enforcement
4. **Quality Gates Collection** (`scripts/quality-gates/`) - Crisis response enforcement

## Architectural Components

### 1. Cursor IDE Rules System

**Location**: `.cursor/rules/`

**Purpose**: Provides real-time guidance and enforcement during development in Cursor IDE

**Key Files**:
- `00-claims-verification.mdc` - Production readiness verification
- `02-quality-gates.mdc` - Testing standards and verification
- `03-infrastructure-standards.mdc` - Infrastructure and deployment standards
- `04-documentation-integrity.mdc` - Documentation reality alignment
- `05-production-readiness-checklist.mdc` - Quick reference checklist
- `17-scope-management-waivers.mdc` - Scope management and emergency waivers
- `18-implementation-completeness.mdc` - Anti-fake implementation guardrails
- `19-language-agnostic-standards.mdc` - Universal engineering standards

**Enforcement Mechanism**:
- **Always Applied**: Core governance rules that are always active
- **Glob-based**: Rules auto-attach when editing matching file patterns
- **Risk Tier Based**: Different quality requirements based on risk tiers (T1/T2/T3)

**Risk Tiers**:
- **Tier 1 (Critical)**: 90%+ coverage, 70%+ mutation, manual review required
- **Tier 2 (Standard)**: 80%+ coverage, 50%+ mutation, optional review  
- **Tier 3 (Low Risk)**: 70%+ coverage, 30%+ mutation, optional review

### 2. CAWS Framework

**Location**: `apps/tools/caws/`

**Purpose**: Comprehensive quality assessment and workflow management system

**Core Components**:

#### Shared Architecture (`shared/`)
- `base-tool.ts` - Base class with common functionality
- `types.ts` - Centralized type definitions
- `validator.ts` - Validation utilities
- `config-manager.ts` - Configuration management
- `gate-checker.ts` - Gate checking logic with tier policies
- `waivers-manager.ts` - Waivers management system

#### Gate Checker (`gate-checker.ts`)
**Key Features**:
- **Tier-based Policies**: Different requirements per risk tier
- **Waiver System**: Emergency bypass capabilities with proper documentation
- **Human Override**: Manual approval for exceptional circumstances
- **Experiment Mode**: Reduced requirements for experimental features
- **Trust Score Calculation**: Weighted scoring across multiple quality dimensions

**Quality Dimensions**:
- Coverage (30% weight)
- Mutation Testing (30% weight)
- Contract Testing (20% weight)
- Accessibility (10% weight)
- Performance (10% weight)

#### CAWS Tools
- `gates.js` - Basic gate enforcement
- `validate.js` - Working spec validation
- `provenance.js` - Provenance tracking
- `attest.js` - SBOM and SLSA attestations
- `flake-detector.ts` - Test variance monitoring
- `spec-test-mapper.ts` - Spec-to-test traceability

### 3. Git Hooks System

**Location**: `.git/hooks/` and `.cursor/hooks/`

**Purpose**: Automated enforcement at commit time and during IDE interactions

#### Pre-commit Hook (`pre-commit`)
**Triggers**: Before every commit
**Enforcement**:
- Runs quality gates (`scripts/quality-gates/run-quality-gates.js`)
- Blocks commits with critical violations
- Runs hidden TODO analysis (`scripts/v3/analysis/todo_analyzer.py`)
- Prevents further codebase degradation

#### Cursor IDE Hooks (`.cursor/hooks/`)
**Configuration**: `hooks.json` defines trigger points
**Hooks**:
- `audit.sh` - Logs all AI events for provenance tracking
- `block-dangerous.sh` - Prevents dangerous shell commands
- `scan-secrets.sh` - Prevents reading sensitive files
- `naming-check.sh` - Enforces naming conventions
- `scope-guard.sh` - Prevents out-of-scope file access
- `format.sh` - Auto-formats code after edits
- `validate-spec.sh` - Validates working specs

**Trigger Points**:
- `beforeShellExecution` - Before shell commands
- `beforeMCPExecution` - Before MCP tool calls
- `beforeReadFile` - Before file reads
- `afterFileEdit` - After code changes
- `beforeSubmitPrompt` - Before AI prompts
- `stop` - On session end

### 4. Quality Gates Collection

**Location**: `scripts/quality-gates/`

**Purpose**: Crisis response enforcement to prevent codebase degradation

**Core Scripts**:
- `run-quality-gates.js` - Main orchestrator
- `check-naming.js` - Naming convention enforcement
- `check-duplication.js` - Duplication prevention
- `check-god-objects.js` - Large file prevention
- `check-code-freeze.js` - Crisis response controls

**Enforcement Levels**:
- **Warning Mode**: Allows commits with warnings
- **Block Mode**: Blocks commits with violations
- **CI Mode**: Strict enforcement for automation

**Quality Gates**:
1. **Naming Conventions**: Prevents `enhanced-`, `new-`, `copy-` patterns
2. **Duplication Prevention**: Tracks functional duplication regression
3. **God Object Prevention**: Prevents files >2K LOC
4. **Code Freeze**: Crisis response controls
5. **Documentation Quality**: Ensures documentation integrity

## Integration Flow

### Development Workflow

1. **Real-time Guidance** (Cursor Rules)
   - Developer writes code in Cursor IDE
   - Rules provide immediate feedback
   - Hooks enforce naming conventions and scope

2. **Pre-commit Validation** (Git Hooks)
   - Developer attempts to commit
   - Pre-commit hook runs quality gates
   - Hidden TODO analysis runs
   - Commit blocked if critical violations found

3. **CAWS Assessment** (CAWS Framework)
   - Working spec validation
   - Tier-based quality requirements
   - Waiver system for emergencies
   - Trust score calculation

4. **Crisis Response** (Quality Gates Collection)
   - Prevents further degradation
   - Enforces naming conventions
   - Blocks duplication regression
   - Prevents god objects

### Data Flow

```
Developer Code → Cursor Rules → Cursor Hooks → Git Pre-commit → Quality Gates → CAWS Assessment → Commit/Block
```

### Enforcement Hierarchy

1. **Cursor Rules**: Real-time guidance and warnings
2. **Cursor Hooks**: IDE-level enforcement
3. **Git Pre-commit**: Commit-level blocking
4. **CAWS Framework**: Comprehensive assessment
5. **Quality Gates**: Crisis response enforcement

## Key Features

### Waiver System
- **Emergency Waivers**: For critical fixes
- **Human Override**: Manual approval process
- **Experiment Mode**: Reduced requirements for experiments
- **Documentation**: All waivers must be documented

### Provenance Tracking
- **AI Event Logging**: All Cursor AI interactions logged
- **Quality Metrics**: Coverage, mutation, trust scores tracked
- **Audit Trail**: Complete change history
- **Attribution**: Links changes to working specs

### Risk-Based Enforcement
- **Tier 1**: Critical systems (auth, billing) - highest requirements
- **Tier 2**: Standard features - moderate requirements  
- **Tier 3**: Low-risk components - minimal requirements

### Crisis Response
- **Code Freeze**: Prevents new features during crisis
- **Duplication Prevention**: Stops functional duplication
- **God Object Prevention**: Prevents large files
- **Naming Enforcement**: Prevents problematic patterns

## Configuration

### Working Spec (`.caws/working-spec.yaml`)
- Defines risk tier and requirements
- Specifies scope boundaries
- Enables waiver system
- Sets change budgets

### Enforcement Configuration
- **Naming Exceptions**: `.caws/naming-exceptions.json`
- **Hook Configuration**: `.cursor/hooks.json`
- **Quality Thresholds**: Tier-based policies in CAWS

## Monitoring and Metrics

### Quality Metrics
- **Coverage**: Line, branch, function coverage
- **Mutation Score**: Test quality assessment
- **Trust Score**: Overall quality composite
- **Duplication Rate**: Functional duplication tracking

### Audit Logs
- **Cursor Events**: `.cursor/logs/audit-*.log`
- **Provenance**: `.agent/provenance.json`
- **Quality Reports**: Coverage and mutation reports

## Benefits

1. **Multi-layered Defense**: Quality enforced at multiple levels
2. **Risk-based Approach**: Different requirements for different risk levels
3. **Emergency Procedures**: Waiver system for critical situations
4. **Comprehensive Tracking**: Full provenance and audit trail
5. **Crisis Response**: Prevents further degradation during issues
6. **Real-time Feedback**: Immediate guidance during development
7. **Automated Enforcement**: Reduces manual quality checking

## Future Enhancements

1. **Machine Learning**: Predictive quality assessment
2. **Dynamic Thresholds**: Adaptive quality requirements
3. **Cross-project Analysis**: Organization-wide quality metrics
4. **Integration APIs**: Third-party tool integration
5. **Advanced Analytics**: Quality trend analysis and predictions

---

*This architecture ensures comprehensive quality enforcement while maintaining development velocity and providing emergency procedures for critical situations.*
