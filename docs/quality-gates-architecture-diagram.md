# Quality Gates System Architecture Diagram

```mermaid
graph TB
    subgraph "Development Environment"
        DEV[Developer]
        CURSOR[Cursor IDE]
    end

    subgraph "Real-time Enforcement Layer"
        RULES[".cursor/rules/<br/>• Production readiness<br/>• Testing standards<br/>• Infrastructure standards<br/>• Documentation integrity<br/>• Scope management<br/>• Implementation completeness"]
        HOOKS[".cursor/hooks/<br/>• audit.sh<br/>• naming-check.sh<br/>• scope-guard.sh<br/>• scan-secrets.sh<br/>• format.sh"]
    end

    subgraph "Commit-time Enforcement"
        PRE_COMMIT[".git/hooks/pre-commit<br/>• Quality gates runner<br/>• Hidden TODO analysis<br/>• Crisis response"]
        QUALITY_GATES["scripts/quality-gates/<br/>• Naming conventions<br/>• Duplication prevention<br/>• God object prevention<br/>• Code freeze controls"]
    end

    subgraph "CAWS Framework"
        CAWS_CORE["apps/tools/caws/<br/>• gate-checker.ts<br/>• waivers-manager.ts<br/>• validator.ts<br/>• base-tool.ts"]
        CAWS_TOOLS["CAWS Tools<br/>• gates.js<br/>• validate.js<br/>• provenance.js<br/>• flake-detector.ts"]
    end

    subgraph "Risk Tiers"
        T1["Tier 1 (Critical)<br/>• 90%+ coverage<br/>• 70%+ mutation<br/>• Manual review<br/>• Auth, billing, migrations"]
        T2["Tier 2 (Standard)<br/>• 80%+ coverage<br/>• 50%+ mutation<br/>• Optional review<br/>• Features, APIs"]
        T3["Tier 3 (Low Risk)<br/>• 70%+ coverage<br/>• 30%+ mutation<br/>• Optional review<br/>• UI, internal tools"]
    end

    subgraph "Quality Dimensions"
        COVERAGE["Coverage (30%)<br/>• Line coverage<br/>• Branch coverage<br/>• Function coverage"]
        MUTATION["Mutation Testing (30%)<br/>• Test quality<br/>• Mutation score<br/>• Surviving mutants"]
        CONTRACTS["Contract Testing (20%)<br/>• API contracts<br/>• Consumer/provider tests<br/>• Integration validation"]
        A11Y["Accessibility (10%)<br/>• Screen reader support<br/>• Keyboard navigation<br/>• Color contrast"]
        PERF["Performance (10%)<br/>• Response times<br/>• Bundle size<br/>• Core Web Vitals"]
    end

    subgraph "Emergency Systems"
        WAIVERS["Waiver System<br/>• Emergency waivers<br/>• Human override<br/>• Experiment mode<br/>• Documentation required"]
        CRISIS["Crisis Response<br/>• Code freeze<br/>• Duplication prevention<br/>• God object blocking<br/>• Naming enforcement"]
    end

    subgraph "Monitoring & Tracking"
        PROVENANCE["Provenance Tracking<br/>• AI event logging<br/>• Quality metrics<br/>• Audit trail<br/>• Change attribution"]
        AUDIT["Audit Logs<br/>• .cursor/logs/<br/>• .agent/provenance.json<br/>• Quality reports<br/>• Historical data"]
    end

    %% Development Flow
    DEV --> CURSOR
    CURSOR --> RULES
    CURSOR --> HOOKS

    %% Enforcement Flow
    RULES --> PRE_COMMIT
    HOOKS --> PRE_COMMIT
    PRE_COMMIT --> QUALITY_GATES
    QUALITY_GATES --> CAWS_CORE

    %% CAWS Integration
    CAWS_CORE --> CAWS_TOOLS
    CAWS_CORE --> T1
    CAWS_CORE --> T2
    CAWS_CORE --> T3

    %% Quality Assessment
    T1 --> COVERAGE
    T2 --> COVERAGE
    T3 --> COVERAGE
    T1 --> MUTATION
    T2 --> MUTATION
    T3 --> MUTATION
    T1 --> CONTRACTS
    T2 --> CONTRACTS
    T3 --> CONTRACTS
    T1 --> A11Y
    T2 --> A11Y
    T3 --> A11Y
    T1 --> PERF
    T2 --> PERF
    T3 --> PERF

    %% Emergency Systems
    CAWS_CORE --> WAIVERS
    QUALITY_GATES --> CRISIS
    WAIVERS --> CAWS_CORE
    CRISIS --> PRE_COMMIT

    %% Monitoring
    HOOKS --> PROVENANCE
    CAWS_TOOLS --> PROVENANCE
    PROVENANCE --> AUDIT

    %% Styling
    classDef devLayer fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef enforcement fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef caws fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef risk fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef quality fill:#fce4ec,stroke:#880e4f,stroke-width:2px
    classDef emergency fill:#ffebee,stroke:#c62828,stroke-width:2px
    classDef monitoring fill:#f1f8e9,stroke:#33691e,stroke-width:2px

    class DEV,CURSOR devLayer
    class RULES,HOOKS,PRE_COMMIT,QUALITY_GATES enforcement
    class CAWS_CORE,CAWS_TOOLS caws
    class T1,T2,T3 risk
    class COVERAGE,MUTATION,CONTRACTS,A11Y,PERF quality
    class WAIVERS,CRISIS emergency
    class PROVENANCE,AUDIT monitoring
```

## Key Integration Points

### 1. Real-time Development Flow
- **Developer** writes code in **Cursor IDE**
- **Cursor Rules** provide immediate guidance and warnings
- **Cursor Hooks** enforce naming conventions, scope, and security

### 2. Commit-time Validation
- **Pre-commit Hook** runs comprehensive quality gates
- **Quality Gates Collection** enforces crisis response measures
- **CAWS Framework** provides tier-based assessment

### 3. Risk-based Quality Requirements
- **Tier 1**: Critical systems with highest requirements
- **Tier 2**: Standard features with moderate requirements
- **Tier 3**: Low-risk components with minimal requirements

### 4. Emergency Procedures
- **Waiver System** allows emergency bypasses with documentation
- **Crisis Response** prevents further codebase degradation
- **Human Override** enables manual approval for exceptional cases

### 5. Comprehensive Monitoring
- **Provenance Tracking** logs all AI interactions and quality metrics
- **Audit Logs** provide complete change history and attribution
- **Quality Reports** track trends and identify issues

## Enforcement Hierarchy

1. **Real-time** (Cursor Rules + Hooks) - Immediate feedback
2. **Commit-time** (Pre-commit + Quality Gates) - Blocking enforcement
3. **Assessment** (CAWS Framework) - Comprehensive evaluation
4. **Emergency** (Waivers + Crisis Response) - Controlled bypasses
5. **Monitoring** (Provenance + Audit) - Historical tracking

This multi-layered architecture ensures quality is enforced at every stage of development while providing emergency procedures for critical situations.
