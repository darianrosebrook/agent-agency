/**
 * @fileoverview Verdict Schema Validator for CAWS Constitutional Framework
 *
 * Validates verdict structure, signatures, clause citations, and spec hash integrity.
 * Ensures all verdicts are constitutionally compliant before ingestion.
 *
 * @author @darianrosebrook
 */

import * as crypto from "crypto";
import * as fs from "fs";

export interface CAWSVerdict {
  id: string;
  taskId: string;
  specId: string;
  timestamp: string;
  cawsVersion: string;

  compliance: {
    budgetAdherence: {
      passed: boolean;
      filesUsed: number;
      filesLimit: number;
      locUsed: number;
      locLimit: number;
      violations: string[];
    };
    qualityGates: Array<{
      name: string;
      passed: boolean;
      score?: number;
      threshold?: number;
      details?: string;
    }>;
  };

  waivers: Array<{
    id: string;
    reason: string;
    approved: boolean;
    justification?: string;
    expiresAt?: string;
  }>;

  verdict: "pass" | "fail" | "waiver-required";
  remediation?: string[];

  // CAWS Constitutional Metadata
  constitutionalContext: {
    specHash: string; // SHA-256 of .caws/working-spec.yaml
    clauseCitations: Array<{
      clauseId: string; // e.g., "CAWS:5.2"
      decision: "pass" | "fail" | "waiver";
      rationale: string;
      evidence: string[];
    }>;
    governanceMetrics: {
      waiverRate: number;
      gateIntegrity: number;
      budgetCompliance: number;
      evidenceCompleteness: number;
    };
  };

  // Cryptographic proof
  signature: {
    algorithm: string; // "ed25519"
    publicKey: string; // base64
    signature: string; // base64
    signedAt: string;
  };

  // Provenance chain
  provenance: {
    previousVerdictHash?: string;
    chainHash: string;
    ledgerEntry: string;
  };
}

export interface ValidationResult {
  valid: boolean;
  verdict: CAWSVerdict;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  metadata: {
    validationTime: number;
    schemaVersion: string;
    validatorVersion: string;
  };
}

export interface ValidationError {
  code: string;
  message: string;
  field?: string;
  details?: any;
  severity: "error" | "warning";
}

export interface ValidationWarning {
  code: string;
  message: string;
  field?: string;
  suggestion?: string;
}

export class VerdictValidator {
  private readonly CAWS_CLAUSES = new Set([
    // Budget Section
    "CAWS:4.1",
    "CAWS:4.2",
    "CAWS:4.3",
    "CAWS:4.4",
    "CAWS:4.5",
    // Waiver Section
    "CAWS:5.1",
    "CAWS:5.2",
    "CAWS:5.3",
    "CAWS:5.4",
    "CAWS:5.5",
    // Quality Gates Section
    "CAWS:6.1",
    "CAWS:6.2",
    "CAWS:6.3",
    "CAWS:6.4",
    "CAWS:6.5",
    "CAWS:6.6",
    // Provenance Section
    "CAWS:7.1",
    "CAWS:7.2",
    "CAWS:7.3",
    // Constitutional Authority
    "CAWS:8.1",
    "CAWS:8.2",
    "CAWS:8.3",
  ]);

  private readonly REQUIRED_FIELDS = [
    "id",
    "taskId",
    "specId",
    "timestamp",
    "cawsVersion",
    "compliance",
    "verdict",
    "constitutionalContext",
    "signature",
    "provenance",
  ];

  /**
   * Validate a complete verdict
   */
  async validateVerdict(
    verdict: any,
    specPath?: string
  ): Promise<ValidationResult> {
    const startTime = Date.now();
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Type check
    if (!this.isValidVerdictShape(verdict)) {
      errors.push({
        code: "INVALID_STRUCTURE",
        message: "Verdict does not match expected CAWS structure",
        severity: "error",
      });
      return this.createResult(
        false,
        verdict as CAWSVerdict,
        errors,
        warnings,
        startTime
      );
    }

    const typedVerdict = verdict as CAWSVerdict;

    // Required fields validation
    const missingFields = this.validateRequiredFields(typedVerdict);
    errors.push(...missingFields);

    // Schema validation
    const schemaErrors = this.validateSchema(typedVerdict);
    errors.push(...schemaErrors);

    // Signature validation
    const signatureErrors = await this.validateSignature(typedVerdict);
    errors.push(...signatureErrors);

    // Spec hash validation
    if (specPath) {
      const specErrors = await this.validateSpecHash(typedVerdict, specPath);
      errors.push(...specErrors);
    } else {
      warnings.push({
        code: "SPEC_PATH_MISSING",
        message: "Spec path not provided for hash validation",
        suggestion:
          "Provide .caws/working-spec.yaml path for complete validation",
      });
    }

    // Clause citation validation
    const clauseErrors = this.validateClauseCitations(typedVerdict);
    errors.push(...clauseErrors);

    // Governance metrics validation
    const metricsErrors = this.validateGovernanceMetrics(typedVerdict);
    errors.push(...metricsErrors);

    // Waiver validation
    const waiverErrors = this.validateWaivers(typedVerdict);
    errors.push(...waiverErrors);

    // Provenance chain validation
    const provenanceErrors = await this.validateProvenanceChain(typedVerdict);
    errors.push(...provenanceErrors);

    // Business logic validation
    const logicErrors = this.validateBusinessLogic(typedVerdict);
    errors.push(...logicErrors);

    const isValid = errors.filter((e) => e.severity === "error").length === 0;

    return this.createResult(
      isValid,
      typedVerdict,
      errors,
      warnings,
      startTime
    );
  }

  /**
   * Validate verdict structure
   */
  private isValidVerdictShape(obj: any): obj is CAWSVerdict {
    return (
      obj &&
      typeof obj === "object" &&
      typeof obj.id === "string" &&
      typeof obj.verdict === "string"
    );
  }

  /**
   * Validate required fields are present
   */
  private validateRequiredFields(verdict: CAWSVerdict): ValidationError[] {
    const errors: ValidationError[] = [];

    for (const field of this.REQUIRED_FIELDS) {
      if (!(field in verdict)) {
        errors.push({
          code: "MISSING_REQUIRED_FIELD",
          message: `Required field '${field}' is missing`,
          field,
          severity: "error",
        });
      }
    }

    return errors;
  }

  /**
   * Validate verdict schema compliance
   */
  private validateSchema(verdict: CAWSVerdict): ValidationError[] {
    const errors: ValidationError[] = [];

    // Validate verdict enum
    if (!["pass", "fail", "waiver-required"].includes(verdict.verdict)) {
      errors.push({
        code: "INVALID_VERDICT_VALUE",
        message: `Invalid verdict value: ${verdict.verdict}`,
        field: "verdict",
        severity: "error",
      });
    }

    // Validate compliance structure
    if (
      !verdict.compliance?.budgetAdherence ||
      !verdict.compliance?.qualityGates
    ) {
      errors.push({
        code: "INVALID_COMPLIANCE_STRUCTURE",
        message:
          "Compliance section missing required budgetAdherence or qualityGates",
        field: "compliance",
        severity: "error",
      });
    }

    // Validate constitutional context
    if (
      !verdict.constitutionalContext?.specHash ||
      !verdict.constitutionalContext?.clauseCitations
    ) {
      errors.push({
        code: "INVALID_CONSTITUTIONAL_CONTEXT",
        message: "Constitutional context missing specHash or clauseCitations",
        field: "constitutionalContext",
        severity: "error",
      });
    }

    // Validate signature structure
    if (
      !verdict.signature?.algorithm ||
      !verdict.signature?.publicKey ||
      !verdict.signature?.signature
    ) {
      errors.push({
        code: "INVALID_SIGNATURE_STRUCTURE",
        message:
          "Signature missing required algorithm, publicKey, or signature",
        field: "signature",
        severity: "error",
      });
    }

    return errors;
  }

  /**
   * Validate cryptographic signature
   */
  private async validateSignature(
    verdict: CAWSVerdict
  ): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    try {
      if (verdict.signature.algorithm !== "ed25519") {
        errors.push({
          code: "UNSUPPORTED_SIGNATURE_ALGORITHM",
          message: `Unsupported signature algorithm: ${verdict.signature.algorithm}`,
          field: "signature.algorithm",
          severity: "error",
        });
        return errors;
      }

      // Create signature payload (exclude signature field itself)
      const payload = {
        ...verdict,
        signature: {
          ...verdict.signature,
          signature: undefined,
          signedAt: verdict.signature.signedAt,
        },
      };

      const payloadJson = JSON.stringify(payload, Object.keys(payload).sort());
      const payloadHash = crypto
        .createHash("sha256")
        .update(payloadJson)
        .digest();

      // Decode signature and public key
      const signature = Buffer.from(verdict.signature.signature, "base64");
      const publicKey = Buffer.from(verdict.signature.publicKey, "base64");

      // Verify signature (simplified - would use actual ed25519 verification)
      // For now, we'll do a basic hash verification
      const expectedSignature = crypto
        .createHmac("sha256", publicKey)
        .update(payloadHash)
        .digest("base64");

      if (signature.toString("base64") !== expectedSignature) {
        errors.push({
          code: "INVALID_SIGNATURE",
          message: "Verdict signature verification failed",
          field: "signature",
          severity: "error",
        });
      }
    } catch (error) {
      errors.push({
        code: "SIGNATURE_VALIDATION_ERROR",
        message: `Signature validation error: ${
          error instanceof Error ? error.message : String(error)
        }`,
        field: "signature",
        details: error,
        severity: "error",
      });
    }

    return errors;
  }

  /**
   * Validate spec hash against actual spec file
   */
  private async validateSpecHash(
    verdict: CAWSVerdict,
    specPath: string
  ): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    try {
      if (!fs.existsSync(specPath)) {
        errors.push({
          code: "SPEC_FILE_NOT_FOUND",
          message: `Spec file not found: ${specPath}`,
          field: "constitutionalContext.specHash",
          severity: "error",
        });
        return errors;
      }

      const specContent = fs.readFileSync(specPath, "utf8");
      const actualHash = crypto
        .createHash("sha256")
        .update(specContent)
        .digest("hex");

      if (actualHash !== verdict.constitutionalContext.specHash) {
        errors.push({
          code: "SPEC_HASH_MISMATCH",
          message: `Spec hash mismatch. Expected: ${actualHash}, Got: ${verdict.constitutionalContext.specHash}`,
          field: "constitutionalContext.specHash",
          severity: "error",
        });
      }
    } catch (error) {
      errors.push({
        code: "SPEC_HASH_VALIDATION_ERROR",
        message: `Spec hash validation error: ${
          error instanceof Error ? error.message : String(error)
        }`,
        field: "constitutionalContext.specHash",
        details: error,
        severity: "error",
      });
    }

    return errors;
  }

  /**
   * Validate clause citations
   */
  private validateClauseCitations(verdict: CAWSVerdict): ValidationError[] {
    const errors: ValidationError[] = [];

    const citations = verdict.constitutionalContext.clauseCitations;

    if (!citations || citations.length === 0) {
      errors.push({
        code: "MISSING_CLAUSE_CITATIONS",
        message: "No clause citations found in constitutional context",
        field: "constitutionalContext.clauseCitations",
        severity: "error",
      });
      return errors;
    }

    for (const citation of citations) {
      if (!citation.clauseId || !citation.decision || !citation.rationale) {
        errors.push({
          code: "INCOMPLETE_CLAUSE_CITATION",
          message:
            "Clause citation missing required clauseId, decision, or rationale",
          field: "constitutionalContext.clauseCitations",
          severity: "error",
        });
      }

      if (!this.CAWS_CLAUSES.has(citation.clauseId)) {
        errors.push({
          code: "INVALID_CLAUSE_ID",
          message: `Invalid CAWS clause ID: ${citation.clauseId}`,
          field: "constitutionalContext.clauseCitations",
          severity: "error",
        });
      }

      if (!["pass", "fail", "waiver"].includes(citation.decision)) {
        errors.push({
          code: "INVALID_CLAUSE_DECISION",
          message: `Invalid clause decision: ${citation.decision}`,
          field: "constitutionalContext.clauseCitations",
          severity: "error",
        });
      }
    }

    return errors;
  }

  /**
   * Validate governance metrics
   */
  private validateGovernanceMetrics(verdict: CAWSVerdict): ValidationError[] {
    const errors: ValidationError[] = [];
    const metrics = verdict.constitutionalContext.governanceMetrics;

    if (!metrics) {
      errors.push({
        code: "MISSING_GOVERNANCE_METRICS",
        message: "Governance metrics missing from constitutional context",
        field: "constitutionalContext.governanceMetrics",
        severity: "error",
      });
      return errors;
    }

    const requiredMetrics = [
      "waiverRate",
      "gateIntegrity",
      "budgetCompliance",
      "evidenceCompleteness",
    ];

    for (const metric of requiredMetrics) {
      const value = (metrics as any)[metric];
      if (typeof value !== "number" || value < 0 || value > 1) {
        errors.push({
          code: "INVALID_GOVERNANCE_METRIC",
          message: `Invalid governance metric ${metric}: ${value}`,
          field: `constitutionalContext.governanceMetrics.${metric}`,
          severity: "error",
        });
      }
    }

    return errors;
  }

  /**
   * Validate waivers
   */
  private validateWaivers(verdict: CAWSVerdict): ValidationError[] {
    const errors: ValidationError[] = [];

    for (const waiver of verdict.waivers || []) {
      if (!waiver.id || !waiver.reason) {
        errors.push({
          code: "INCOMPLETE_WAIVER",
          message: "Waiver missing required id or reason",
          field: "waivers",
          severity: "error",
        });
      }

      // Check expiry if present
      if (waiver.expiresAt) {
        const expiryDate = new Date(waiver.expiresAt);
        if (isNaN(expiryDate.getTime())) {
          errors.push({
            code: "INVALID_WAIVER_EXPIRY",
            message: `Invalid waiver expiry date: ${waiver.expiresAt}`,
            field: "waivers",
            severity: "error",
          });
        } else if (expiryDate < new Date()) {
          errors.push({
            code: "EXPIRED_WAIVER",
            message: `Waiver has expired: ${waiver.expiresAt}`,
            field: "waivers",
            severity: "error",
          });
        }
      }
    }

    return errors;
  }

  /**
   * Validate provenance chain
   */
  private async validateProvenanceChain(
    verdict: CAWSVerdict
  ): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    // Validate chain hash
    const verdictJson = JSON.stringify(verdict, Object.keys(verdict).sort());
    const calculatedHash = crypto
      .createHash("sha256")
      .update(verdictJson)
      .digest("hex");

    if (calculatedHash !== verdict.provenance.chainHash) {
      errors.push({
        code: "CHAIN_HASH_MISMATCH",
        message: "Provenance chain hash does not match verdict content",
        field: "provenance.chainHash",
        severity: "error",
      });
    }

    return errors;
  }

  /**
   * Validate business logic rules
   */
  private validateBusinessLogic(verdict: CAWSVerdict): ValidationError[] {
    const errors: ValidationError[] = [];

    // If verdict is "pass", there should be no waivers
    if (verdict.verdict === "pass" && verdict.waivers.length > 0) {
      errors.push({
        code: "INVALID_PASS_WITH_WAIVERS",
        message: "Verdict marked as 'pass' but contains waivers",
        field: "verdict",
        severity: "error",
      });
    }

    // If verdict is "waiver-required", there should be waivers
    if (verdict.verdict === "waiver-required" && verdict.waivers.length === 0) {
      errors.push({
        code: "WAIVER_REQUIRED_WITHOUT_WAIVERS",
        message: "Verdict marked as 'waiver-required' but contains no waivers",
        field: "verdict",
        severity: "error",
      });
    }

    // Budget violations should be reflected in compliance
    const budgetViolations =
      verdict.compliance.budgetAdherence.violations.length;
    if (budgetViolations > 0 && verdict.compliance.budgetAdherence.passed) {
      errors.push({
        code: "BUDGET_VIOLATION_BUT_PASSED",
        message: "Budget has violations but marked as passed",
        field: "compliance.budgetAdherence",
        severity: "error",
      });
    }

    return errors;
  }

  /**
   * Create validation result
   */
  private createResult(
    valid: boolean,
    verdict: CAWSVerdict,
    errors: ValidationError[],
    warnings: ValidationWarning[],
    startTime: number
  ): ValidationResult {
    return {
      valid,
      verdict,
      errors,
      warnings,
      metadata: {
        validationTime: Date.now() - startTime,
        schemaVersion: "1.0.0",
        validatorVersion: "1.0.0",
      },
    };
  }

  /**
   * Validate verdict from file
   */
  async validateVerdictFile(
    filePath: string,
    specPath?: string
  ): Promise<ValidationResult> {
    try {
      const content = fs.readFileSync(filePath, "utf8");
      const verdict = JSON.parse(content);
      return this.validateVerdict(verdict, specPath);
    } catch (error) {
      return {
        valid: false,
        verdict: {} as CAWSVerdict,
        errors: [
          {
            code: "FILE_READ_ERROR",
            message: `Failed to read verdict file: ${
              error instanceof Error ? error.message : String(error)
            }`,
            severity: "error",
            details: error,
          },
        ],
        warnings: [],
        metadata: {
          validationTime: 0,
          schemaVersion: "1.0.0",
          validatorVersion: "1.0.0",
        },
      };
    }
  }

  /**
   * Get validation summary
   */
  getValidationSummary(results: ValidationResult[]): {
    totalVerdicts: number;
    validVerdicts: number;
    invalidVerdicts: number;
    totalErrors: number;
    totalWarnings: number;
    errorBreakdown: Record<string, number>;
    warningBreakdown: Record<string, number>;
  } {
    const summary = {
      totalVerdicts: results.length,
      validVerdicts: results.filter((r) => r.valid).length,
      invalidVerdicts: results.filter((r) => !r.valid).length,
      totalErrors: 0,
      totalWarnings: 0,
      errorBreakdown: {} as Record<string, number>,
      warningBreakdown: {} as Record<string, number>,
    };

    for (const result of results) {
      summary.totalErrors += result.errors.length;
      summary.totalWarnings += result.warnings.length;

      for (const error of result.errors) {
        summary.errorBreakdown[error.code] =
          (summary.errorBreakdown[error.code] || 0) + 1;
      }

      for (const warning of result.warnings) {
        summary.warningBreakdown[warning.code] =
          (summary.warningBreakdown[warning.code] || 0) + 1;
      }
    }

    return summary;
  }
}
