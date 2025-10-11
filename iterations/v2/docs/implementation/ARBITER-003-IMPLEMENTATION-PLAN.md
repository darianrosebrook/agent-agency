# ARBITER-003: CAWS Validator Implementation Plan

**Component**: CAWS Validator  
**Spec ID**: ARBITER-003  
**Risk Tier**: T1 (Critical - Constitutional Authority)  
**Estimated Effort**: 2-3 weeks (with CAWS CLI reference)  
**Status**: 🔄 Ready to Implement

---

## 1. Overview

### Purpose

Implement the CAWS constitutional authority layer that validates working specs, enforces quality gates, and generates verdicts for all arbiter operations.

### Reference Implementation

**CAWS CLI** (`@paths.design/caws-cli` v3.4.0) provides production-ready patterns:

- `src/commands/validate.js` - Working spec validation
- `src/commands/evaluate.js` - Quality gate execution
- `src/validation/spec-validation.js` - Validation logic
- `src/budget-derivation.js` - Budget compliance checking

### Key Adaptations

1. **TypeScript**: Adapt JavaScript to TypeScript with strict types
2. **Integration**: Connect to V2's database, events, and security layers
3. **Async/Await**: Use async patterns throughout
4. **Modular**: Separate concerns (validation, gates, verdicts)

---

## 2. Architecture

### Component Structure

```
src/caws-validator/
├── CAWSValidator.ts               # Main validator orchestrator
├── types/
│   ├── validation-types.ts        # Validation result types
│   ├── verdict-types.ts           # Verdict types
│   └── gate-types.ts              # Quality gate types
├── validation/
│   ├── SpecValidator.ts           # Working spec validation
│   ├── BudgetValidator.ts         # Budget compliance checking
│   └── ContractValidator.ts       # Contract validation
├── quality-gates/
│   ├── QualityGateExecutor.ts     # Gate execution orchestrator
│   ├── CoverageGate.ts            # Coverage threshold gate
│   ├── MutationGate.ts            # Mutation score gate
│   ├── LintGate.ts                # Linting gate
│   ├── SecurityGate.ts            # Security scan gate
│   └── ContractGate.ts            # Contract test gate
├── verdict/
│   ├── VerdictGenerator.ts        # Generate pass/fail/waiver verdicts
│   └── VerdictPublisher.ts        # Publish to git/provenance
├── waivers/
│   ├── WaiverManager.ts           # Waiver loading and validation
│   └── WaiverValidator.ts         # Waiver expiry and approval checks
└── utils/
    ├── budget-derivation.ts       # Derive budgets from policy
    ├── git-integration.ts         # Git operations for provenance
    └── policy-loader.ts           # Load policy.yaml

tests/unit/caws-validator/
├── spec-validator.test.ts         # 20+ tests
├── budget-validator.test.ts       # 15+ tests
├── quality-gate-executor.test.ts  # 25+ tests
└── verdict-generator.test.ts      # 15+ tests
```

### Data Flow

```
Working Spec Input
       ↓
1. Spec Validation (structure, required fields)
       ↓
2. Budget Derivation (policy.yaml + waivers)
       ↓
3. Budget Compliance (check current changes)
       ↓
4. Quality Gates Execution (coverage, mutation, lints, security)
       ↓
5. Verdict Generation (pass/fail/waiver-required)
       ↓
6. Verdict Publication (git commit, provenance chain)
       ↓
Result: CAWSValidationResult
```

---

## 3. Implementation Phases

### Phase 1: Core Validation (Week 1, Days 1-3)

**Goal**: Implement working spec validation and budget checking

#### Tasks

1. **Create Type Definitions** (4 hours)

```typescript
// src/types/caws-validation.ts
export interface CAWSValidationResult {
  passed: boolean;
  cawsVersion: string;
  timestamp: string;
  budgetCompliance: BudgetCompliance;
  qualityGates: QualityGateResult[];
  waivers: WaiverApplication[];
  verdict: "pass" | "fail" | "waiver-required";
  remediation?: string[];
  signature?: string;
}

export interface BudgetCompliance {
  compliant: boolean;
  baseline: { max_files: number; max_loc: number };
  effective: { max_files: number; max_loc: number };
  current: { files_changed: number; lines_changed: number };
  violations: BudgetViolation[];
}

export interface QualityGateResult {
  gate: string;
  passed: boolean;
  score?: number;
  threshold?: number;
  message: string;
  evidence?: unknown;
}
```

2. **Implement SpecValidator** (6 hours)

Adapt from CAWS CLI `spec-validation.js`:

```typescript
// src/caws-validator/validation/SpecValidator.ts
export class SpecValidator {
  validateWorkingSpec(spec: WorkingSpec): ValidationResult {
    // Adapt from CAWS CLI validateWorkingSpec()
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Check required fields
    this.validateRequiredFields(spec, errors);

    // Check ID format (PREFIX-NUMBER)
    this.validateIdFormat(spec.id, errors);

    // Check risk tier (1, 2, 3)
    this.validateRiskTier(spec.risk_tier, errors);

    // Check scope.in not empty
    this.validateScope(spec.scope, errors);

    // Check tier-specific requirements
    this.validateTierRequirements(spec, errors);

    return {
      valid: errors.length === 0,
      errors,
      warnings,
    };
  }

  private validateRequiredFields(
    spec: WorkingSpec,
    errors: ValidationError[]
  ): void {
    const required = [
      "id",
      "title",
      "risk_tier",
      "mode",
      "blast_radius",
      "scope",
      "acceptance",
    ];

    for (const field of required) {
      if (!spec[field]) {
        errors.push({
          field,
          message: `Missing required field: ${field}`,
          suggestion: this.getFieldSuggestion(field),
        });
      }
    }
  }
}
```

3. **Implement BudgetValidator** (6 hours)

Adapt from CAWS CLI `budget-derivation.js`:

```typescript
// src/caws-validator/validation/BudgetValidator.ts
export class BudgetValidator {
  constructor(
    private policyLoader: PolicyLoader,
    private waiverManager: WaiverManager
  ) {}

  async deriveBudget(
    spec: WorkingSpec,
    projectRoot: string
  ): Promise<DerivedBudget> {
    // Load policy.yaml
    const policy = await this.policyLoader.loadPolicy(projectRoot);

    // Get tier baseline
    const baseline = policy.risk_tiers[spec.risk_tier];

    // Apply waivers
    const effectiveBudget = await this.applyWaivers(
      baseline,
      spec.waiver_ids || [],
      projectRoot
    );

    return {
      baseline,
      effective: effectiveBudget,
      waiversApplied: spec.waiver_ids || [],
      derivedAt: new Date().toISOString(),
    };
  }

  async checkBudgetCompliance(
    derivedBudget: DerivedBudget,
    currentStats: ChangeStats
  ): Promise<BudgetCompliance> {
    const violations: BudgetViolation[] = [];

    if (currentStats.filesChanged > derivedBudget.effective.max_files) {
      violations.push({
        gate: "budget_limit",
        type: "max_files",
        current: currentStats.filesChanged,
        limit: derivedBudget.effective.max_files,
        message: `File count (${currentStats.filesChanged}) exceeds budget (${derivedBudget.effective.max_files})`,
      });
    }

    if (currentStats.linesChanged > derivedBudget.effective.max_loc) {
      violations.push({
        gate: "budget_limit",
        type: "max_loc",
        current: currentStats.linesChanged,
        limit: derivedBudget.effective.max_loc,
        message: `Lines changed (${currentStats.linesChanged}) exceeds budget (${derivedBudget.effective.max_loc})`,
      });
    }

    return {
      compliant: violations.length === 0,
      violations,
      baseline: derivedBudget.baseline,
      effective: derivedBudget.effective,
      current: currentStats,
    };
  }
}
```

4. **Write Tests** (4 hours)

```typescript
// tests/unit/caws-validator/spec-validator.test.ts
describe("SpecValidator", () => {
  let validator: SpecValidator;

  beforeEach(() => {
    validator = new SpecValidator();
  });

  describe("validateWorkingSpec", () => {
    it("should pass with valid spec", () => {
      const spec = createValidSpec();
      const result = validator.validateWorkingSpec(spec);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("should fail with missing required fields", () => {
      const spec = { title: "Test" } as WorkingSpec;
      const result = validator.validateWorkingSpec(spec);
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it("should fail with invalid ID format", () => {
      const spec = createValidSpec();
      spec.id = "invalid";
      const result = validator.validateWorkingSpec(spec);
      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "id")).toBe(true);
    });

    it("should fail with invalid risk tier", () => {
      const spec = createValidSpec();
      spec.risk_tier = 5;
      const result = validator.validateWorkingSpec(spec);
      expect(result.valid).toBe(false);
    });

    it("should require contracts for Tier 1 and 2", () => {
      const spec = createValidSpec();
      spec.risk_tier = 1;
      spec.contracts = [];
      const result = validator.validateWorkingSpec(spec);
      expect(result.valid).toBe(false);
    });
  });
});
```

---

### Phase 2: Quality Gates (Week 1, Days 4-7)

**Goal**: Implement quality gate execution framework

#### Tasks

1. **Create QualityGateExecutor** (8 hours)

Adapt from CAWS CLI `evaluate.js`:

```typescript
// src/caws-validator/quality-gates/QualityGateExecutor.ts
export class QualityGateExecutor {
  private gates: Map<string, QualityGate>;

  constructor() {
    this.gates = new Map();
    this.registerDefaultGates();
  }

  private registerDefaultGates(): void {
    this.gates.set("coverage", new CoverageGate());
    this.gates.set("mutation", new MutationGate());
    this.gates.set("lint", new LintGate());
    this.gates.set("security", new SecurityGate());
    this.gates.set("contract", new ContractGate());
  }

  async executeGates(
    spec: WorkingSpec,
    projectRoot: string
  ): Promise<QualityGateResult[]> {
    const results: QualityGateResult[] = [];
    const tierRequirements = this.getTierRequirements(spec.risk_tier);

    for (const [gateName, gate] of this.gates.entries()) {
      const requirement = tierRequirements[gateName];
      if (!requirement) continue;

      const result = await gate.execute(projectRoot, requirement);
      results.push(result);
    }

    return results;
  }

  private getTierRequirements(tier: number): Record<string, GateRequirement> {
    const requirements = {
      1: {
        coverage: { threshold: 90, branch: true },
        mutation: { threshold: 70 },
        lint: { errors: 0, warnings: 0 },
        security: { critical: 0, high: 0 },
        contract: { required: true },
      },
      2: {
        coverage: { threshold: 80, branch: true },
        mutation: { threshold: 50 },
        lint: { errors: 0 },
        security: { critical: 0 },
        contract: { required: true },
      },
      3: {
        coverage: { threshold: 70, branch: false },
        mutation: { threshold: 30 },
        lint: { errors: 0 },
        security: { critical: 0 },
        contract: { required: false },
      },
    };

    return requirements[tier] || requirements[2];
  }
}
```

2. **Implement Individual Gates** (12 hours)

```typescript
// src/caws-validator/quality-gates/CoverageGate.ts
export class CoverageGate implements QualityGate {
  async execute(
    projectRoot: string,
    requirement: GateRequirement
  ): Promise<QualityGateResult> {
    try {
      // Read coverage report (coverage/coverage-summary.json)
      const coverageReport = await this.readCoverageReport(projectRoot);

      const score = requirement.branch
        ? coverageReport.total.branches.pct
        : coverageReport.total.lines.pct;

      const passed = score >= requirement.threshold;

      return {
        gate: "coverage",
        passed,
        score,
        threshold: requirement.threshold,
        message: passed
          ? `Coverage ${score}% meets threshold ${requirement.threshold}%`
          : `Coverage ${score}% below threshold ${requirement.threshold}%`,
        evidence: coverageReport.total,
      };
    } catch (error) {
      return {
        gate: "coverage",
        passed: false,
        message: `Coverage check failed: ${error.message}`,
      };
    }
  }

  private async readCoverageReport(
    projectRoot: string
  ): Promise<CoverageReport> {
    const reportPath = path.join(
      projectRoot,
      "coverage",
      "coverage-summary.json"
    );
    const content = await fs.readFile(reportPath, "utf-8");
    return JSON.parse(content);
  }
}

// src/caws-validator/quality-gates/MutationGate.ts
export class MutationGate implements QualityGate {
  async execute(
    projectRoot: string,
    requirement: GateRequirement
  ): Promise<QualityGateResult> {
    try {
      // Read mutation report (mutation/stryker-report.json)
      const mutationReport = await this.readMutationReport(projectRoot);

      const score = mutationReport.mutationScore;
      const passed = score >= requirement.threshold;

      return {
        gate: "mutation",
        passed,
        score,
        threshold: requirement.threshold,
        message: passed
          ? `Mutation score ${score}% meets threshold ${requirement.threshold}%`
          : `Mutation score ${score}% below threshold ${requirement.threshold}%`,
        evidence: {
          killed: mutationReport.killed,
          survived: mutationReport.survived,
          timeout: mutationReport.timeout,
        },
      };
    } catch (error) {
      return {
        gate: "mutation",
        passed: false,
        message: `Mutation check failed: ${error.message}`,
      };
    }
  }
}
```

3. **Write Quality Gate Tests** (6 hours)

```typescript
// tests/unit/caws-validator/quality-gate-executor.test.ts
describe("QualityGateExecutor", () => {
  let executor: QualityGateExecutor;

  beforeEach(() => {
    executor = new QualityGateExecutor();
  });

  describe("executeGates", () => {
    it("should execute all gates for Tier 1", async () => {
      const spec = createValidSpec();
      spec.risk_tier = 1;

      const results = await executor.executeGates(spec, "/fake/root");

      expect(results.length).toBeGreaterThan(0);
      expect(results.some((r) => r.gate === "coverage")).toBe(true);
      expect(results.some((r) => r.gate === "mutation")).toBe(true);
    });

    it("should apply correct thresholds for each tier", async () => {
      const tier1 = await executor.executeGates(
        { ...createValidSpec(), risk_tier: 1 },
        "/fake"
      );
      const tier2 = await executor.executeGates(
        { ...createValidSpec(), risk_tier: 2 },
        "/fake"
      );

      const t1Coverage = tier1.find((r) => r.gate === "coverage");
      const t2Coverage = tier2.find((r) => r.gate === "coverage");

      expect(t1Coverage?.threshold).toBe(90);
      expect(t2Coverage?.threshold).toBe(80);
    });
  });
});
```

---

### Phase 3: Verdict Generation (Week 2, Days 1-3)

**Goal**: Generate pass/fail/waiver verdicts

#### Tasks

1. **Implement VerdictGenerator** (6 hours)

```typescript
// src/caws-validator/verdict/VerdictGenerator.ts
export class VerdictGenerator {
  generateVerdict(
    specValidation: ValidationResult,
    budgetCompliance: BudgetCompliance,
    qualityGateResults: QualityGateResult[],
    waivers: WaiverApplication[]
  ): CAWSValidationResult {
    // Determine overall verdict
    const hasSpecErrors = !specValidation.valid;
    const hasBudgetViolations = !budgetCompliance.compliant;
    const hasGateFailures = qualityGateResults.some((r) => !r.passed);

    let verdict: "pass" | "fail" | "waiver-required";
    const remediation: string[] = [];

    if (!hasSpecErrors && !hasBudgetViolations && !hasGateFailures) {
      verdict = "pass";
    } else if (waivers.length > 0 && this.waiversCoverViolations(waivers)) {
      verdict = "pass";
    } else if (this.canWaive(budgetCompliance, qualityGateResults)) {
      verdict = "waiver-required";
      remediation.push("Create waiver for budget/gate violations");
    } else {
      verdict = "fail";
      remediation.push(...this.generateRemediationSteps(specValidation));
    }

    return {
      passed: verdict === "pass",
      cawsVersion: "3.1.0",
      timestamp: new Date().toISOString(),
      budgetCompliance,
      qualityGates: qualityGateResults,
      waivers,
      verdict,
      remediation: remediation.length > 0 ? remediation : undefined,
    };
  }

  private generateRemediationSteps(specValidation: ValidationResult): string[] {
    const steps: string[] = [];

    for (const error of specValidation.errors) {
      if (error.suggestion) {
        steps.push(`Fix ${error.field}: ${error.suggestion}`);
      }
    }

    return steps;
  }
}
```

2. **Implement VerdictPublisher** (8 hours)

```typescript
// src/caws-validator/verdict/VerdictPublisher.ts
export class VerdictPublisher {
  constructor(private gitIntegration: GitIntegration) {}

  async publishVerdict(
    verdict: CAWSValidationResult,
    spec: WorkingSpec
  ): Promise<PublicationResult> {
    try {
      // 1. Generate verdict file
      const verdictPath = await this.writeVerdictFile(verdict, spec);

      // 2. Create provenance entry
      const provenanceEntry = this.createProvenanceEntry(verdict, spec);

      // 3. Commit to git
      const commitHash = await this.gitIntegration.commit({
        message: `CAWS Verdict: ${verdict.verdict} for ${spec.id}`,
        files: [verdictPath, ".caws/provenance.json"],
      });

      // 4. Sign verdict
      const signature = await this.signVerdict(verdict, commitHash);

      return {
        published: true,
        verdictPath,
        commitHash,
        signature,
        provenanceEntry,
      };
    } catch (error) {
      return {
        published: false,
        error: error.message,
      };
    }
  }

  private async writeVerdictFile(
    verdict: CAWSValidationResult,
    spec: WorkingSpec
  ): Promise<string> {
    const verdictPath = path.join(
      ".caws",
      "verdicts",
      `${spec.id}-${Date.now()}.json`
    );

    await fs.mkdir(path.dirname(verdictPath), { recursive: true });
    await fs.writeFile(verdictPath, JSON.stringify(verdict, null, 2));

    return verdictPath;
  }
}
```

3. **Write Verdict Tests** (4 hours)

---

### Phase 4: Integration & Testing (Week 2, Days 4-7)

**Goal**: Integrate all components and complete testing

#### Tasks

1. **Create CAWSValidator Orchestrator** (6 hours)

```typescript
// src/caws-validator/CAWSValidator.ts
export class CAWSValidator {
  constructor(
    private specValidator: SpecValidator,
    private budgetValidator: BudgetValidator,
    private qualityGateExecutor: QualityGateExecutor,
    private verdictGenerator: VerdictGenerator,
    private verdictPublisher: VerdictPublisher
  ) {}

  async validate(
    spec: WorkingSpec,
    projectRoot: string,
    options: ValidationOptions = {}
  ): Promise<CAWSValidationResult> {
    try {
      // 1. Validate spec structure
      const specValidation = this.specValidator.validateWorkingSpec(spec);
      if (!specValidation.valid && !options.skipSpecValidation) {
        throw new ValidationError("Spec validation failed", specValidation);
      }

      // 2. Derive and check budget
      const derivedBudget = await this.budgetValidator.deriveBudget(
        spec,
        projectRoot
      );
      const currentStats = await this.getChangeStats(projectRoot);
      const budgetCompliance = await this.budgetValidator.checkBudgetCompliance(
        derivedBudget,
        currentStats
      );

      // 3. Execute quality gates
      const qualityGateResults = await this.qualityGateExecutor.executeGates(
        spec,
        projectRoot
      );

      // 4. Load and apply waivers
      const waivers = await this.loadWaivers(spec.waiver_ids || []);

      // 5. Generate verdict
      const verdict = this.verdictGenerator.generateVerdict(
        specValidation,
        budgetCompliance,
        qualityGateResults,
        waivers
      );

      // 6. Publish verdict (if not dry-run)
      if (!options.dryRun && verdict.passed) {
        await this.verdictPublisher.publishVerdict(verdict, spec);
      }

      return verdict;
    } catch (error) {
      throw new CAWSValidationError("Validation failed", error);
    }
  }

  private async getChangeStats(projectRoot: string): Promise<ChangeStats> {
    // Use git to get current changes
    const diff = await this.gitIntegration.diff();
    return {
      filesChanged: diff.files.length,
      linesChanged: diff.insertions + diff.deletions,
    };
  }
}
```

2. **Integration Tests** (8 hours)

```typescript
// tests/integration/caws-validator.test.ts
describe("CAWSValidator Integration", () => {
  let validator: CAWSValidator;
  let testProject: string;

  beforeEach(async () => {
    validator = createCAWSValidator();
    testProject = await createTestProject();
  });

  it("should pass valid Tier 2 spec with all gates passing", async () => {
    const spec = createValidSpec();
    spec.risk_tier = 2;

    // Setup test project with passing gates
    await setupPassingProject(testProject, { coverage: 85, mutation: 55 });

    const result = await validator.validate(spec, testProject);

    expect(result.passed).toBe(true);
    expect(result.verdict).toBe("pass");
    expect(result.qualityGates.every((g) => g.passed)).toBe(true);
  });

  it("should require waiver for budget violation", async () => {
    const spec = createValidSpec();
    spec.risk_tier = 2;

    // Exceed budget
    await createManyFiles(testProject, 100); // Max is 50 for Tier 2

    const result = await validator.validate(spec, testProject);

    expect(result.passed).toBe(false);
    expect(result.verdict).toBe("waiver-required");
    expect(result.budgetCompliance.violations.length).toBeGreaterThan(0);
  });

  it("should fail with low coverage", async () => {
    const spec = createValidSpec();
    spec.risk_tier = 1;

    // Low coverage
    await setupPassingProject(testProject, { coverage: 70, mutation: 75 });

    const result = await validator.validate(spec, testProject);

    expect(result.passed).toBe(false);
    expect(result.qualityGates.find((g) => g.gate === "coverage")?.passed).toBe(
      false
    );
  });
});
```

3. **End-to-End Tests** (6 hours)

```typescript
// tests/e2e/caws-validator-e2e.test.ts
describe("CAWSValidator E2E", () => {
  it("should complete full validation workflow", async () => {
    // 1. Create working spec
    const spec = await createWorkingSpec({
      id: "FEAT-001",
      title: "Test Feature",
      risk_tier: 2,
    });

    // 2. Initialize project
    const project = await initializeProject(spec);

    // 3. Write code and tests
    await writeImplementation(project);
    await writeTests(project);

    // 4. Run tests with coverage
    await runTests(project);

    // 5. Validate
    const validator = createCAWSValidator();
    const result = await validator.validate(spec, project);

    // 6. Verify verdict published
    expect(result.passed).toBe(true);
    expect(fs.existsSync(path.join(project, ".caws/verdicts"))).toBe(true);
  });
});
```

4. **Documentation** (4 hours)

- API documentation
- Usage examples
- Integration guide
- Troubleshooting

---

## 4. Integration Points

### Database Integration

```typescript
// Store validation results in database
await this.database.storeValidationResult(verdict);

// Query historical results
const history = await this.database.getValidationHistory(spec.id);
```

### Event Integration

```typescript
// Emit validation events
this.eventEmitter.emit("validation:started", { spec });
this.eventEmitter.emit("validation:completed", { verdict });
this.eventEmitter.emit("validation:failed", { error });
```

### Security Integration

```typescript
// Validate security context
await this.securityManager.validateContext(securityContext);

// Audit log validation operations
await this.securityManager.auditLog("validation", { spec, verdict });
```

---

## 5. Testing Strategy

### Unit Tests (75+ tests)

- **SpecValidator**: 20 tests
- **BudgetValidator**: 15 tests
- **QualityGateExecutor**: 25 tests
- **VerdictGenerator**: 15 tests

### Integration Tests (20+ tests)

- Full validation workflow
- Gate execution with real reports
- Verdict publication to git

### End-to-End Tests (5+ tests)

- Complete CAWS workflow
- Multi-component interaction
- Failure recovery

### Target Coverage

- **Branch Coverage**: 90%+ (Tier 1)
- **Mutation Score**: 70%+ (Tier 1)
- **All gates pass**: Required

---

## 6. Success Criteria

### Functional

- [ ] Validates working specs according to CAWS schema
- [ ] Derives budgets from policy.yaml + waivers
- [ ] Checks budget compliance against current changes
- [ ] Executes all quality gates (coverage, mutation, lint, security, contract)
- [ ] Generates pass/fail/waiver-required verdicts
- [ ] Publishes verdicts to git with provenance

### Non-Functional

- [ ] Validation completes in <5 seconds for typical project
- [ ] Handles projects with 1000+ files
- [ ] Provides clear error messages and remediation steps
- [ ] Integrates seamlessly with V2 orchestrator
- [ ] Produces comprehensive audit trail

### Quality

- [ ] 90%+ branch coverage
- [ ] 70%+ mutation score
- [ ] Zero TypeScript errors
- [ ] Zero linter errors
- [ ] All tests passing

---

## 7. Migration Notes

### From CAWS CLI

**Key Changes**:

1. **Language**: JavaScript → TypeScript
2. **Async**: Callbacks → async/await
3. **Types**: Add strict type definitions
4. **Integration**: Connect to V2 database and events
5. **Modular**: Separate concerns into focused classes

**Reusable Patterns**:

- Validation logic and field suggestions
- Budget derivation algorithm
- Quality gate thresholds by tier
- Waiver application logic
- Verdict generation logic

**What to Keep**:

- Required field definitions
- ID format validation (`PREFIX-NUMBER`)
- Risk tier ranges (1-3)
- Tier-specific gate thresholds
- Budget compliance checking

**What to Enhance**:

- Add database persistence
- Add event emission
- Add security context validation
- Add comprehensive audit logging
- Add performance tracking

---

## 8. Timeline

### Week 1

- **Days 1-3**: Core validation (spec + budget) + tests
- **Days 4-7**: Quality gates implementation + tests

### Week 2

- **Days 1-3**: Verdict generation + publication
- **Days 4-7**: Integration + E2E tests + documentation

### Week 3 (Buffer)

- Performance optimization
- Additional edge case tests
- Documentation polish
- Code review and refinement

---

## 9. Next Steps

1. **Review this plan** with team
2. **Set up test project** for validation
3. **Begin Phase 1** (Core Validation)
4. **Daily standups** to track progress
5. **Weekly demos** to stakeholders

---

**Status**: Ready to implement  
**Estimated Completion**: 2-3 weeks  
**Risk Level**: Medium (well-defined scope, clear reference implementation)  
**Dependencies**: None (all infrastructure in place)

_This plan leverages CAWS CLI as a battle-tested reference while adapting to V2's TypeScript architecture and enhanced capabilities._
