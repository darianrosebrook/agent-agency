/**
 * @fileoverview Working Spec Validator
 * Validates CAWS working specifications for structural correctness
 * Adapted from CAWS CLI spec-validation.js
 * @module caws-validator/validation
 */

import { PerformanceTracker } from "../../rl/PerformanceTracker";
import { WorkingSpec } from "../../types/caws-types";
import type {
  AutoFix,
  SpecValidationResult,
  ValidationError,
  ValidationWarning,
} from "../types/validation-types";

/**
 * Validates CAWS working specifications
 * Checks structural integrity, required fields, and tier-specific requirements
 */
export class SpecValidator {
  private performanceTracker?: PerformanceTracker;

  constructor(performanceTracker?: PerformanceTracker) {
    this.performanceTracker = performanceTracker;
  }

  /**
   * Set the performance tracker for compliance metrics recording.
   *
   * @param tracker - Performance tracker instance
   */
  setPerformanceTracker(tracker: PerformanceTracker): void {
    this.performanceTracker = tracker;
  }

  /**
   * Validate working spec structure and content
   */
  public async validateWorkingSpec(
    spec: WorkingSpec
  ): Promise<SpecValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const fixes: AutoFix[] = [];

    // Check required fields
    this.validateRequiredFields(spec, errors);

    // Check ID format (PREFIX-NUMBER)
    if (spec.id) {
      this.validateIdFormat(spec.id, errors);
    }

    // Check risk tier (1, 2, 3)
    if (spec.risk_tier !== undefined) {
      this.validateRiskTier(spec.risk_tier, errors, fixes);
    }

    // Check development mode
    if (spec.mode) {
      this.validateMode(spec.mode, errors);
    }

    // Check scope definition
    this.validateScope(spec, errors);

    // Check tier-specific requirements
    if (spec.risk_tier) {
      this.validateTierRequirements(spec, errors);
    }

    // Check experimental mode if present
    if (spec.experimental_mode) {
      this.validateExperimentalMode(spec, errors);
    }

    // Check acceptance criteria (required - should be error not warning)
    if (!spec.acceptance || spec.acceptance.length === 0) {
      errors.push({
        field: "acceptance",
        message: "Acceptance criteria are required - at least one acceptance criterion in Given-When-Then format must be provided",
      });
    }

    // Check invariants
    if (!spec.invariants || spec.invariants.length === 0) {
      warnings.push({
        field: "invariants",
        message: "No system invariants defined",
        suggestion: "Add 1-3 statements about what must always remain true",
      });
    }

    // Record constitutional validation performance metrics
    if (this.performanceTracker) {
      try {
        const startTime = Date.now();
        // Simulate processing time for now (would be measured in real implementation)
        const processingTimeMs = Date.now() - startTime;

        await this.performanceTracker.recordConstitutionalValidation({
          taskId: spec.id || "unknown-spec",
          agentId: "caws-validator", // System agent for spec validation
          validationResult: {
            valid: errors.length === 0,
            violations: [
              ...errors.map((error) => ({
                severity: "high" as const,
                message: error.message,
                rule: error.field,
              })),
              ...warnings.map((warning) => ({
                severity: "medium" as const,
                message: warning.message,
                rule: warning.field,
              })),
            ],
            complianceScore: Math.max(
              0,
              1 - (errors.length * 0.2 + warnings.length * 0.1)
            ),
            processingTimeMs,
            ruleCount: 10, // Approximate number of validation rules
          },
        });
      } catch (error) {
        // Log but don't fail validation due to performance tracking issues
        console.warn(
          "Failed to record spec validation performance metrics:",
          error
        );
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      fixes: fixes.length > 0 ? fixes : undefined,
    };
  }

  /**
   * Validate working spec with auto-fix suggestions
   */
  public async validateWithSuggestions(
    spec: WorkingSpec,
    options: { autoFix?: boolean } = {}
  ): Promise<SpecValidationResult> {
    const result = await this.validateWorkingSpec(spec);

    // Apply auto-fixes if requested
    if (options.autoFix && result.fixes && result.fixes.length > 0) {
      this.applyAutoFixes(spec, result.fixes);
    }

    return result;
  }

  /**
   * Validate required fields
   */
  private validateRequiredFields(
    spec: WorkingSpec,
    errors: ValidationError[]
  ): void {
    const requiredFields: Array<keyof WorkingSpec> = [
      "id",
      "title",
      "risk_tier",
      "mode",
      "blast_radius",
      "operational_rollback_slo",
      "scope",
      "invariants",
      "acceptance",
      "non_functional",
      "contracts",
    ];

    for (const field of requiredFields) {
      if (!spec[field]) {
        errors.push({
          field: field as string,
          message: `Missing required field: ${field}`,
          suggestion: this.getFieldSuggestion(field),
          canAutoFix: this.canAutoFixField(field),
        });
      }
    }
  }

  /**
   * Validate ID format (PREFIX-NUMBER)
   */
  private validateIdFormat(id: string, errors: ValidationError[]): void {
    const idPattern = /^[A-Z]+-\d+$/;
    if (!idPattern.test(id)) {
      errors.push({
        field: "id",
        message:
          "Project ID should be in format: PREFIX-NUMBER (e.g., FEAT-1234)",
        suggestion: "Use format like: PROJ-001, FEAT-002, FIX-003",
        canAutoFix: false,
      });
    }
  }

  /**
   * Validate risk tier
   */
  private validateRiskTier(
    riskTier: number,
    errors: ValidationError[],
    fixes: AutoFix[]
  ): void {
    if (riskTier < 1 || riskTier > 3) {
      errors.push({
        field: "risk_tier",
        message: "Risk tier must be 1, 2, or 3",
        suggestion:
          "Tier 1: Critical (auth, billing), Tier 2: Standard (features), Tier 3: Low risk (UI)",
        canAutoFix: true,
      });

      // Suggest auto-fix
      const fixed = Math.max(1, Math.min(3, riskTier));
      fixes.push({
        field: "risk_tier",
        value: fixed,
        description: `Clamp risk tier to valid range: ${fixed}`,
      });
    }
  }

  /**
   * Validate development mode
   */
  private validateMode(mode: string, errors: ValidationError[]): void {
    const validModes = ["feature", "refactor", "fix", "doc", "chore"];
    if (!validModes.includes(mode)) {
      errors.push({
        field: "mode",
        message: `Invalid mode: ${mode}`,
        suggestion: `Must be one of: ${validModes.join(", ")}`,
        canAutoFix: false,
      });
    }
  }

  /**
   * Validate scope definition
   */
  private validateScope(spec: WorkingSpec, errors: ValidationError[]): void {
    if (!spec.scope) {
      errors.push({
        field: "scope",
        message: "Scope definition is required",
        suggestion:
          "Define what's included (in) and excluded (out) from changes",
        canAutoFix: false,
      });
      return;
    }

    if (!spec.scope.in || spec.scope.in.length === 0) {
      errors.push({
        field: "scope.in",
        message: "Scope IN must not be empty",
        suggestion: "Specify directories/files that are included in changes",
        canAutoFix: false,
      });
    }
  }

  /**
   * Validate tier-specific requirements
   */
  private validateTierRequirements(
    spec: WorkingSpec,
    errors: ValidationError[]
  ): void {
    // Tier 1 and 2 require contracts
    if (spec.risk_tier === 1 || spec.risk_tier === 2) {
      if (!spec.contracts || spec.contracts.length === 0) {
        errors.push({
          field: "contracts",
          message: "Contracts required for Tier 1 and 2 changes",
          suggestion: "Specify API contracts (OpenAPI, GraphQL, etc.)",
          canAutoFix: false,
        });
      }
    }

    // Tier 1 requires stricter observability
    if (spec.risk_tier === 1) {
      if (!spec.observability) {
        errors.push({
          field: "observability",
          message: "Observability required for Tier 1 changes",
          suggestion: "Define logging, metrics, and tracing strategy",
          canAutoFix: false,
        });
      }

      if (!spec.rollback || spec.rollback.length === 0) {
        errors.push({
          field: "rollback",
          message: "Rollback procedures required for Tier 1 changes",
          suggestion: "Document rollback steps and data migration reversal",
          canAutoFix: false,
        });
      }
    }

    // Validate non-functional requirements based on tier
    if (!spec.non_functional) {
      return;
    }

    if (spec.risk_tier === 1) {
      if (
        !spec.non_functional.security ||
        spec.non_functional.security.length === 0
      ) {
        errors.push({
          field: "non_functional.security",
          message: "Security requirements required for Tier 1 changes",
          suggestion:
            "Define authentication, authorization, and data protection requirements",
          canAutoFix: false,
        });
      }
    }
  }

  /**
   * Validate experimental mode
   */
  private validateExperimentalMode(
    spec: WorkingSpec,
    errors: ValidationError[]
  ): void {
    const expMode = spec.experimental_mode;
    if (!expMode) {
      return;
    }

    // Check required fields
    const requiredFields = ["enabled", "rationale", "expires_at"];
    for (const field of requiredFields) {
      if (!(field in expMode)) {
        errors.push({
          field: `experimental_mode.${field}`,
          message: `Missing required experimental mode field: ${field}`,
          suggestion: "Provide all required experimental mode fields",
          canAutoFix: false,
        });
      }
    }

    // Experimental mode only allowed for Tier 3
    if (expMode.enabled && spec.risk_tier < 3) {
      errors.push({
        field: "experimental_mode",
        message:
          "Experimental mode can only be used with Tier 3 (low risk) changes",
        suggestion: "Either disable experimental mode or lower risk tier to 3",
        canAutoFix: false,
      });
    }

    // Validate expiration date
    if (expMode.expires_at) {
      try {
        const expiryDate = new Date(expMode.expires_at);
        const now = new Date();
        if (expiryDate <= now) {
          errors.push({
            field: "experimental_mode.expires_at",
            message: "Experimental mode expiration date must be in the future",
            suggestion:
              "Set a future expiration date for experimental features",
            canAutoFix: false,
          });
        }
      } catch {
        errors.push({
          field: "experimental_mode.expires_at",
          message: "Invalid date format for experimental mode expiration",
          suggestion: "Use ISO 8601 date format (YYYY-MM-DD)",
          canAutoFix: false,
        });
      }
    }
  }

  /**
   * Get suggestion for missing field
   */
  private getFieldSuggestion(field: keyof WorkingSpec): string {
    const suggestions: Partial<Record<keyof WorkingSpec, string>> = {
      id: "Use format like: PROJ-001, FEAT-002, FIX-003",
      title: "Add a descriptive project title",
      risk_tier: "Choose: 1 (critical), 2 (standard), or 3 (low risk)",
      mode: "Choose: feature, refactor, fix, doc, or chore",
      waiver_ids:
        'Reference active waivers by ID (e.g., ["WV-0001"]) if budget exceptions needed',
      blast_radius: "List affected modules and data migration needs",
      operational_rollback_slo: "Choose: 1m, 5m, 15m, or 1h",
      scope: "Define what's included (in) and excluded (out) from changes",
      invariants: "Add 1-3 statements about what must always remain true",
      acceptance: "Add acceptance criteria in Given-When-Then format",
      non_functional:
        "Define accessibility, performance, and security requirements",
      contracts: "Specify API contracts (OpenAPI, GraphQL, etc.)",
    };

    return suggestions[field] || `Add the ${field} field`;
  }

  /**
   * Check if field can be auto-fixed
   */
  private canAutoFixField(field: keyof WorkingSpec): boolean {
    const autoFixable: Array<keyof WorkingSpec> = ["risk_tier"];
    return autoFixable.includes(field);
  }

  /**
   * Apply auto-fixes to spec
   */
  private applyAutoFixes(spec: WorkingSpec, fixes: AutoFix[]): void {
    for (const fix of fixes) {
      const pathParts = fix.field.split(".");
      let current: any = spec;

      // Navigate to the parent object
      for (let i = 0; i < pathParts.length - 1; i++) {
        if (!current[pathParts[i]]) {
          current[pathParts[i]] = {};
        }
        current = current[pathParts[i]];
      }

      // Apply the fix
      const finalKey = pathParts[pathParts.length - 1];
      current[finalKey] = fix.value;
    }
  }
}
