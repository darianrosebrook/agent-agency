/**
 * @fileoverview CAWS Rule Engine
 * Evaluates policy rules against working specifications
 * @module caws-validator/validation
 */

import { WorkingSpec } from "../../types/caws-types";
import type {
  CAWSPolicy,
  RuleEvaluation,
  RuleResult,
  RuleViolation,
} from "../types/validation-types";

/**
 * Rule engine for evaluating CAWS policy rules against working specs
 */
export class RuleEngine {
  constructor(private policy: CAWSPolicy) {}

  /**
   * Evaluate all rules against a working spec
   */
  public evaluateRules(spec: WorkingSpec): RuleResult {
    const violations: RuleViolation[] = [];
    const evaluations: RuleEvaluation[] = [];

    // Evaluate all rule categories
    this.evaluateScopeRules(spec, violations, evaluations);
    this.evaluateBlastRadiusRules(spec, violations, evaluations);
    this.evaluateRollbackRules(spec, violations, evaluations);
    this.evaluateAcceptanceRules(spec, violations, evaluations);
    this.evaluateContractRules(spec, violations, evaluations);
    this.evaluateObservabilityRules(spec, violations, evaluations);
    this.evaluateNonFunctionalRules(spec, violations, evaluations);

    return {
      passed: violations.length === 0,
      violations,
      evaluations,
      evaluatedAt: new Date().toISOString(),
    };
  }

  /**
   * Evaluate scope definition rules
   */
  private evaluateScopeRules(
    spec: WorkingSpec,
    violations: RuleViolation[],
    evaluations: RuleEvaluation[]
  ): void {
    const ruleId = "SCOPE-001";

    evaluations.push({
      ruleId,
      description: "Scope definition must be clear and actionable",
      passed: true,
    });

    // Rule: Scope.in must not be empty
    if (!spec.scope?.in || spec.scope.in.length === 0) {
      violations.push({
        ruleId,
        severity: "high",
        message:
          "Scope.in cannot be empty - must specify files/directories included",
        field: "scope.in",
        suggestion: "Specify directories like: ['src/features/', 'tests/']",
      });
    }

    // Rule: Scope.out should not include critical directories
    const criticalDirs = ["node_modules", ".git", "dist", "build"];
    if (spec.scope?.out) {
      for (const excluded of spec.scope.out) {
        if (criticalDirs.some((critical) => excluded.includes(critical))) {
          violations.push({
            ruleId,
            severity: "medium",
            message: `Scope.out should not exclude critical directory: ${excluded}`,
            field: "scope.out",
            suggestion: "Remove critical infrastructure from exclusions",
          });
        }
      }
    }

    // Rule: Scope.in paths should be reasonable
    if (spec.scope?.in) {
      for (const included of spec.scope.in) {
        if (included === "/" || included === "/*" || included === "./") {
          violations.push({
            ruleId,
            severity: "high",
            message: "Scope.in too broad - avoid root or wildcard paths",
            field: "scope.in",
            suggestion: "Use specific directories like 'src/features/' instead",
          });
        }
      }
    }
  }

  /**
   * Evaluate blast radius rules
   */
  private evaluateBlastRadiusRules(
    spec: WorkingSpec,
    violations: RuleViolation[],
    evaluations: RuleEvaluation[]
  ): void {
    const ruleId = "BLAST-001";

    evaluations.push({
      ruleId,
      description: "Blast radius must be assessed and documented",
      passed: true,
    });

    // Rule: Modules must be specified
    if (!spec.blast_radius?.modules || spec.blast_radius.modules.length === 0) {
      violations.push({
        ruleId,
        severity: "high",
        message: "Blast radius modules must be specified",
        field: "blast_radius.modules",
        suggestion: "List affected modules like: ['auth', 'api', 'database']",
      });
    }

    // Rule: Data migration flag must be explicit
    if (spec.blast_radius?.data_migration === undefined) {
      violations.push({
        ruleId,
        severity: "medium",
        message: "Data migration impact must be explicitly stated",
        field: "blast_radius.data_migration",
        suggestion: "Set to true if database schema changes are required",
      });
    }

    // Rule: High-risk modules require justification
    const highRiskModules = ["auth", "billing", "security", "database"];
    if (spec.blast_radius?.modules) {
      for (const module of spec.blast_radius.modules) {
        if (highRiskModules.includes(module.toLowerCase())) {
          // Check if there's a comment or justification in invariants
          const hasJustification = spec.invariants?.some(
            (inv) =>
              inv.toLowerCase().includes(module.toLowerCase()) ||
              inv.toLowerCase().includes("justification") ||
              inv.toLowerCase().includes("approved")
          );

          if (!hasJustification) {
            violations.push({
              ruleId,
              severity: "medium",
              message: `High-risk module '${module}' requires explicit justification`,
              field: "invariants",
              suggestion:
                "Add invariant explaining why this high-risk module change is acceptable",
            });
          }
        }
      }
    }
  }

  /**
   * Evaluate rollback rules
   */
  private evaluateRollbackRules(
    spec: WorkingSpec,
    violations: RuleViolation[],
    evaluations: RuleEvaluation[]
  ): void {
    const ruleId = "ROLLBACK-001";

    evaluations.push({
      ruleId,
      description: "Rollback procedures must be defined for production safety",
      passed: true,
    });

    // Rule: Rollback SLO must be reasonable
    const validSLOs = ["1m", "5m", "15m", "1h", "4h", "24h"];
    if (!validSLOs.includes(spec.operational_rollback_slo)) {
      violations.push({
        ruleId,
        severity: "medium",
        message: `Invalid rollback SLO: ${spec.operational_rollback_slo}`,
        field: "operational_rollback_slo",
        suggestion: `Use one of: ${validSLOs.join(", ")}`,
      });
    }

    // Rule: Tier 1 changes require rollback procedures
    if (spec.risk_tier === 1) {
      if (!spec.rollback || spec.rollback.length === 0) {
        violations.push({
          ruleId,
          severity: "high",
          message: "Tier 1 changes require explicit rollback procedures",
          field: "rollback",
          suggestion:
            "Document step-by-step rollback process and data restoration",
        });
      }
    }

    // Rule: Rollback procedures should be testable
    if (spec.rollback && spec.rollback.length > 0) {
      const hasTestableSteps = spec.rollback.some(
        (step) =>
          step.toLowerCase().includes("test") ||
          step.toLowerCase().includes("verify") ||
          step.toLowerCase().includes("validate")
      );

      if (!hasTestableSteps) {
        violations.push({
          ruleId,
          severity: "low",
          message: "Rollback procedures should include verification steps",
          field: "rollback",
          suggestion: "Add testing/validation steps to rollback procedures",
        });
      }
    }
  }

  /**
   * Evaluate acceptance criteria rules
   */
  private evaluateAcceptanceRules(
    spec: WorkingSpec,
    violations: RuleViolation[],
    evaluations: RuleEvaluation[]
  ): void {
    const ruleId = "ACCEPTANCE-001";

    evaluations.push({
      ruleId,
      description: "Acceptance criteria must be specific and testable",
      passed: true,
    });

    // Rule: Must have acceptance criteria
    if (!spec.acceptance || spec.acceptance.length === 0) {
      violations.push({
        ruleId,
        severity: "high",
        message: "Acceptance criteria are required",
        field: "acceptance",
        suggestion: "Add Given-When-Then scenarios for feature validation",
      });
      return;
    }

    // Rule: Each criterion must have required fields
    for (let i = 0; i < spec.acceptance.length; i++) {
      const criterion = spec.acceptance[i];
      const requiredFields = ["id", "given", "when", "then"];

      for (const field of requiredFields) {
        if (!criterion[field as keyof typeof criterion]) {
          violations.push({
            ruleId,
            severity: "high",
            message: `Acceptance criterion ${
              i + 1
            } missing required field: ${field}`,
            field: `acceptance[${i}].${field}`,
            suggestion: `Add ${field} field to acceptance criterion`,
          });
        }
      }
    }

    // Rule: IDs must be unique
    const ids = spec.acceptance.map((a) => a.id).filter(Boolean);
    const uniqueIds = new Set(ids);
    if (ids.length !== uniqueIds.size) {
      violations.push({
        ruleId,
        severity: "medium",
        message: "Acceptance criterion IDs must be unique",
        field: "acceptance",
        suggestion: "Use unique IDs like A1, A2, A3 for each criterion",
      });
    }

    // Rule: Criteria should be specific and testable
    for (let i = 0; i < spec.acceptance.length; i++) {
      const criterion = spec.acceptance[i];

      // Check for vague terms
      const vagueTerms = ["works", "functions", "is good", "is correct"];
      const criterionText =
        `${criterion.given} ${criterion.when} ${criterion.then}`.toLowerCase();

      for (const vagueTerm of vagueTerms) {
        if (criterionText.includes(vagueTerm)) {
          violations.push({
            ruleId,
            severity: "low",
            message: `Acceptance criterion ${
              criterion.id || i + 1
            } contains vague term: "${vagueTerm}"`,
            field: `acceptance[${i}]`,
            suggestion:
              "Use specific, measurable outcomes instead of vague terms",
          });
        }
      }
    }
  }

  /**
   * Evaluate contract rules
   */
  private evaluateContractRules(
    spec: WorkingSpec,
    violations: RuleViolation[],
    evaluations: RuleEvaluation[]
  ): void {
    const ruleId = "CONTRACT-001";

    evaluations.push({
      ruleId,
      description: "Contracts must be specified for external interfaces",
      passed: true,
    });

    // Rule: Contracts required for Tiers 1 and 2
    const tierRequiresContracts = spec.risk_tier === 1 || spec.risk_tier === 2;
    if (
      tierRequiresContracts &&
      (!spec.contracts || spec.contracts.length === 0)
    ) {
      violations.push({
        ruleId,
        severity: "high",
        message: `Tier ${spec.risk_tier} requires contract specifications`,
        field: "contracts",
        suggestion: "Specify OpenAPI, GraphQL, or other interface contracts",
      });
    }

    // Rule: Contract paths should be valid
    if (spec.contracts) {
      for (let i = 0; i < spec.contracts.length; i++) {
        const contract = spec.contracts[i];

        // Check for required fields
        if (!contract.type) {
          violations.push({
            ruleId,
            severity: "high",
            message: `Contract ${i + 1} missing type field`,
            field: `contracts[${i}].type`,
            suggestion: 'Specify type like "openapi", "graphql", or "protobuf"',
          });
        }

        if (!contract.path) {
          violations.push({
            ruleId,
            severity: "high",
            message: `Contract ${i + 1} missing path field`,
            field: `contracts[${i}].path`,
            suggestion:
              "Specify path to contract file relative to project root",
          });
        }

        // Check for valid contract types
        const validTypes = [
          "openapi",
          "swagger",
          "graphql",
          "protobuf",
          "avro",
          "thrift",
        ];
        if (
          contract.type &&
          !validTypes.includes(contract.type.toLowerCase())
        ) {
          violations.push({
            ruleId,
            severity: "medium",
            message: `Unknown contract type: ${contract.type}`,
            field: `contracts[${i}].type`,
            suggestion: `Use one of: ${validTypes.join(", ")}`,
          });
        }
      }
    }
  }

  /**
   * Evaluate observability rules
   */
  private evaluateObservabilityRules(
    spec: WorkingSpec,
    violations: RuleViolation[],
    evaluations: RuleEvaluation[]
  ): void {
    const ruleId = "OBSERVABILITY-001";

    evaluations.push({
      ruleId,
      description:
        "Observability requirements must be specified for monitoring",
      passed: true,
    });

    // Rule: Tier 1 requires observability
    if (spec.risk_tier === 1 && !spec.observability) {
      violations.push({
        ruleId,
        severity: "high",
        message: "Tier 1 changes require observability configuration",
        field: "observability",
        suggestion: "Define logging, metrics, tracing, and alerting strategy",
      });
    }

    // Rule: Observability should include required fields
    if (spec.observability) {
      const requiredObservabilityFields = ["logs", "metrics", "alerts"];

      for (const field of requiredObservabilityFields) {
        if (!(field in spec.observability)) {
          violations.push({
            ruleId,
            severity: "medium",
            message: `Observability missing ${field} configuration`,
            field: `observability.${field}`,
            suggestion: `Define ${field} strategy for production monitoring`,
          });
        }
      }
    }
  }

  /**
   * Evaluate non-functional requirements rules
   */
  private evaluateNonFunctionalRules(
    spec: WorkingSpec,
    violations: RuleViolation[],
    evaluations: RuleEvaluation[]
  ): void {
    const ruleId = "NONFUNC-001";

    evaluations.push({
      ruleId,
      description:
        "Non-functional requirements must be realistic and measurable",
      passed: true,
    });

    // Rule: Performance requirements should be realistic
    if (spec.non_functional?.perf) {
      const perf = spec.non_functional.perf;

      // Check API P95 latency
      if (perf.api_p95_ms && perf.api_p95_ms < 10) {
        violations.push({
          ruleId,
          severity: "medium",
          message: "API P95 latency requirement too aggressive (< 10ms)",
          field: "non_functional.perf.api_p95_ms",
          suggestion: "Realistic P95 latency is typically 50-500ms for APIs",
        });
      }

      // Check LCP
      if (perf.lcp_ms && perf.lcp_ms < 100) {
        violations.push({
          ruleId,
          severity: "medium",
          message: "LCP requirement too aggressive (< 100ms)",
          field: "non_functional.perf.lcp_ms",
          suggestion: "Realistic LCP is typically 500-2500ms for web apps",
        });
      }
    }

    // Rule: Security requirements should be comprehensive
    if (spec.risk_tier === 1 && spec.non_functional?.security) {
      const security = spec.non_functional.security;
      const requiredSecurity = [
        "input-validation",
        "authentication",
        "authorization",
      ];

      for (const required of requiredSecurity) {
        if (!security.includes(required)) {
          violations.push({
            ruleId,
            severity: "high",
            message: `Tier 1 security requirements missing: ${required}`,
            field: "non_functional.security",
            suggestion: `Add "${required}" to security requirements`,
          });
        }
      }
    }

    // Rule: Accessibility requirements should be specific
    if (spec.non_functional?.a11y) {
      const a11y = spec.non_functional.a11y;
      const specificRequirements = [
        "keyboard-navigation",
        "screen-reader",
        "color-contrast",
        "focus-management",
      ];

      // Check if requirements are specific enough
      const hasSpecific = a11y.some((req) =>
        specificRequirements.some((specific) =>
          req.toLowerCase().includes(specific.split("-")[0])
        )
      );

      if (a11y.length > 0 && !hasSpecific) {
        violations.push({
          ruleId,
          severity: "low",
          message: "Accessibility requirements should be specific",
          field: "non_functional.a11y",
          suggestion: `Use specific requirements like: ${specificRequirements.join(
            ", "
          )}`,
        });
      }
    }
  }
}
