# Benchmark Data Quality Gates

**Author**: @darianrosebrook

---

## Executive Summary

Quality gates ensure that only clean, valid, privacy-compliant data enters the benchmark pool and eventually the RL training pipeline. Poor quality data leads to poor quality training—these gates are essential for reliable agent improvement.

**Principle**: Better to have less high-quality data than more low-quality data.

---

## Quality Gate Categories

### 1. Completeness Gates

**Requirement**: All essential fields must be present

```typescript
const completenessGates: QualityGate[] = [
  {
    name: "required-fields-present",
    validate: (dataPoint) => {
      const required = [
        "id",
        "timestamp",
        "task.id",
        "task.type",
        "routing.selectedAgent",
        "execution.success",
        "evaluation.overallScore",
      ];

      const missing = required.filter((field) => {
        const value = this.getNestedValue(dataPoint, field);
        return value === undefined || value === null;
      });

      return {
        passed: missing.length === 0,
        errors: missing.map((f) => `Missing required field: ${f}`),
      };
    },
    severity: "error",
    blockRL: true,
  },

  {
    name: "rubric-scores-complete",
    validate: (dataPoint) => {
      const rubric = dataPoint.evaluation.rubricScores;
      const required = ["format", "tool", "task", "minimal", "cost", "safety"];

      const missing = required.filter((field) => rubric[field] === undefined);

      return {
        passed: missing.length === 0,
        errors: missing.map((f) => `Missing rubric score: ${f}`),
      };
    },
    severity: "error",
    blockRL: true,
  },
];
```

### 2. Type Safety Gates

**Requirement**: All values must match schema types

```typescript
const typeSafetyGates: QualityGate[] = [
  {
    name: "numeric-bounds",
    validate: (dataPoint) => {
      const errors: string[] = [];

      // Scores must be 0-1
      const scores = [
        dataPoint.evaluation.overallScore,
        ...Object.values(dataPoint.evaluation.rubricScores),
      ];

      for (const score of scores) {
        if (score < 0 || score > 1) {
          errors.push(`Score out of bounds: ${score}`);
        }
      }

      // Latency must be positive
      if (dataPoint.execution.latencyMs <= 0) {
        errors.push(`Invalid latency: ${dataPoint.execution.latencyMs}`);
      }

      // Tokens must be positive
      if (dataPoint.execution.tokensUsed < 0) {
        errors.push(`Invalid token count: ${dataPoint.execution.tokensUsed}`);
      }

      return {
        passed: errors.length === 0,
        errors,
      };
    },
    severity: "error",
    blockRL: true,
  },

  {
    name: "enum-validation",
    validate: (dataPoint) => {
      const errors: string[] = [];

      const validTaskTypes = [
        "code-editing",
        "research",
        "data-analysis",
        "design",
        "planning",
      ];
      if (!validTaskTypes.includes(dataPoint.task.type)) {
        errors.push(`Invalid task type: ${dataPoint.task.type}`);
      }

      const validComplexities = ["trivial", "standard", "complex"];
      if (!validComplexities.includes(dataPoint.task.complexity)) {
        errors.push(`Invalid complexity: ${dataPoint.task.complexity}`);
      }

      return {
        passed: errors.length === 0,
        errors,
      };
    },
    severity: "error",
    blockRL: true,
  },
];
```

### 3. Privacy & Security Gates

**Requirement**: No PII or sensitive data in training dataset

```typescript
const privacyGates: QualityGate[] = [
  {
    name: "no-pii",
    validate: async (dataPoint) => {
      const piiDetector = new PIIDetector();

      const fields = [
        dataPoint.task.description,
        dataPoint.routing.rationale,
        JSON.stringify(dataPoint.execution.toolsUsed),
      ];

      const violations: string[] = [];

      for (const field of fields) {
        const detected = await piiDetector.scan(field);
        if (detected.length > 0) {
          violations.push(
            ...detected.map((d) => `PII detected: ${d.type} in ${d.field}`)
          );
        }
      }

      return {
        passed: violations.length === 0,
        errors: violations,
      };
    },
    severity: "critical",
    blockRL: true,
  },

  {
    name: "tenant-anonymization",
    validate: (dataPoint) => {
      // Tenant ID must be hashed (64-char hex)
      const isHashed = /^[a-f0-9]{64}$/.test(dataPoint.tenantId);

      return {
        passed: isHashed,
        errors: isHashed ? [] : ["Tenant ID not properly anonymized"],
      };
    },
    severity: "critical",
    blockRL: true,
  },

  {
    name: "no-secrets",
    validate: async (dataPoint) => {
      const secretScanner = new SecretScanner();
      const text = JSON.stringify(dataPoint);

      const secrets = await secretScanner.scan(text);

      return {
        passed: secrets.length === 0,
        errors: secrets.map((s) => `Secret detected: ${s.type}`),
      };
    },
    severity: "critical",
    blockRL: true,
  },
];
```

### 4. Consistency Gates

**Requirement**: Data must be internally consistent

```typescript
const consistencyGates: QualityGate[] = [
  {
    name: "success-quality-consistency",
    validate: (dataPoint) => {
      const errors: string[] = [];

      // If success, quality should be reasonable
      if (
        dataPoint.execution.success &&
        dataPoint.evaluation.overallScore < 0.5
      ) {
        errors.push("Inconsistent: task succeeded but quality score <0.5");
      }

      // If failure, quality should be low
      if (
        !dataPoint.execution.success &&
        dataPoint.evaluation.overallScore > 0.8
      ) {
        errors.push("Inconsistent: task failed but quality score >0.8");
      }

      return {
        passed: errors.length === 0,
        errors,
      };
    },
    severity: "warning",
    blockRL: false,
  },

  {
    name: "timing-consistency",
    validate: (dataPoint) => {
      // Time to first action shouldn't exceed total latency
      const consistent =
        dataPoint.execution.timeToFirstAction <= dataPoint.execution.latencyMs;

      return {
        passed: consistent,
        errors: consistent
          ? []
          : ["Time to first action exceeds total latency"],
      };
    },
    severity: "error",
    blockRL: true,
  },

  {
    name: "token-tool-consistency",
    validate: (dataPoint) => {
      // If tokens used, should have tool calls OR thinking tokens
      const hasActivity =
        dataPoint.execution.toolCallCount > 0 ||
        dataPoint.execution.thinkingTokens > 0;

      if (dataPoint.execution.tokensUsed > 100 && !hasActivity) {
        return {
          passed: false,
          errors: ["High token usage but no tool calls or thinking recorded"],
        };
      }

      return { passed: true, errors: [] };
    },
    severity: "warning",
    blockRL: false,
  },
];
```

### 5. Statistical Quality Gates

**Requirement**: Data should not be statistical outliers

```typescript
const statisticalGates: QualityGate[] = [
  {
    name: "latency-outlier-check",
    validate: (dataPoint, statistics) => {
      const p99 = statistics.latencyPercentiles.p99;
      const isOutlier = dataPoint.execution.latencyMs > p99 * 3;

      return {
        passed: !isOutlier,
        errors: isOutlier
          ? [
              `Latency outlier: ${dataPoint.execution.latencyMs}ms (P99: ${p99}ms)`,
            ]
          : [],
        warnings: isOutlier ? ["Consider investigating task complexity"] : [],
      };
    },
    severity: "warning",
    blockRL: false,
  },

  {
    name: "token-outlier-check",
    validate: (dataPoint, statistics) => {
      const p99 = statistics.tokenPercentiles.p99;
      const isOutlier = dataPoint.execution.tokensUsed > p99 * 3;

      return {
        passed: !isOutlier,
        errors: [],
        warnings: isOutlier
          ? [
              `High token usage: ${dataPoint.execution.tokensUsed} (P99: ${p99})`,
            ]
          : [],
      };
    },
    severity: "info",
    blockRL: false,
  },
];
```

---

## Quality Gate Execution

### Validation Pipeline

```typescript
class QualityGatePipeline {
  private gates: QualityGate[] = [
    ...completenessGates,
    ...typeSafetyGates,
    ...privacyGates,
    ...consistencyGates,
    ...statisticalGates,
  ];

  async validate(dataPoint: BenchmarkDataPoint): Promise<ValidationResult> {
    const results: GateResult[] = [];
    let blockRL = false;

    for (const gate of this.gates) {
      const result = await gate.validate(dataPoint, this.statistics);
      results.push({
        gateName: gate.name,
        passed: result.passed,
        severity: gate.severity,
        errors: result.errors,
        warnings: result.warnings,
      });

      if (!result.passed && gate.blockRL) {
        blockRL = true;
      }
    }

    const allPassed = results.every(
      (r) => r.passed || r.severity === "warning"
    );
    const criticalFailed = results.some(
      (r) => !r.passed && r.severity === "critical"
    );

    return {
      valid: allPassed && !criticalFailed,
      rlReady: allPassed && !blockRL,
      gateResults: results,
      summary: this.generateSummary(results),
    };
  }
}
```

### Post-Validation Actions

```typescript
class PostValidationHandler {
  async handleValidationResult(
    dataPoint: BenchmarkDataPoint,
    validation: ValidationResult
  ): Promise<void> {
    if (validation.valid && validation.rlReady) {
      // Mark as ready for RL training
      await this.store.update(dataPoint.id, {
        rlReady: true,
        validatedAt: new Date(),
      });
    } else if (validation.valid && !validation.rlReady) {
      // Store but don't use for RL
      await this.store.update(dataPoint.id, {
        rlReady: false,
        validationWarnings: validation.gateResults
          .filter((r) => r.severity === "warning")
          .map((r) => r.gateName),
      });
    } else {
      // Invalid data - quarantine for review
      await this.quarantine.add(dataPoint, validation);

      this.logger.error("Data validation failed", {
        dataPointId: dataPoint.id,
        errors: validation.gateResults
          .filter((r) => !r.passed)
          .map((r) => ({ gate: r.gateName, errors: r.errors })),
      });
    }
  }
}
```

---

## Quality Monitoring Dashboard

### Key Indicators

```typescript
interface QualityDashboard {
  // Overall health
  totalDataPoints: number;
  validDataPoints: number;
  rlReadyDataPoints: number;
  quarantinedDataPoints: number;

  // Gate performance
  gatePassRates: Map<string, number>; // Per gate
  mostFailedGates: Array<{ gate: string; failureRate: number }>;

  // Trends
  validationTrend: "improving" | "stable" | "degrading";
  qualityScoreTrend: "improving" | "stable" | "degrading";

  // Privacy
  privacyViolationsToday: number; // Must be 0
  piiDetectionRate: number;

  // RL readiness
  rlReadyPercentage: number; // Target: ≥90%
  estimatedTimeToRLBatch: string; // "2 days" at current collection rate
}
```

---

## Success Criteria

**Gate Performance**:

- ≥95% of data points pass all gates
- 0 critical violations
- ≥90% RL-ready rate

**Privacy Compliance**:

- 0 PII violations
- 100% tenant anonymization
- 0 secret leaks

**RL Training Impact**:

- Quality-validated data improves training stability
- Clean data reduces RL training failures
- Privacy compliance enables cross-tenant learning

---

**Quality gates are the guardians of data integrity—ensuring that only high-quality, privacy-compliant data fuels the RL training pipeline.**
