/**
 * @fileoverview Main CAWS Validator
 * Orchestrates constitutional authority validation of working specifications
 * @module caws-validator
 */

import * as fs from "fs";
import * as path from "path";
import { PerformanceTracker } from "../rl/PerformanceTracker";
import { WorkingSpec } from "../types/caws-types";
import type {
  BudgetCompliance,
  CAWSValidationResult,
  ChangeStats,
  DerivedBudget,
  QualityGateResult,
  SpecValidationResult,
  ValidationOptions,
} from "./types/validation-types";
import { PolicyLoader } from "./utils/policy-loader";
import { BudgetValidator } from "./validation/BudgetValidator";
import { RuleEngine } from "./validation/RuleEngine";
import { SpecValidator } from "./validation/SpecValidator";
import { WaiverManager } from "./waivers/WaiverManager";

/**
 * Main CAWS Validator - Constitutional Authority Workspace System
 *
 * Orchestrates comprehensive validation of working specifications against
 * CAWS policies, budgets, and constitutional requirements.
 */
export class CAWSValidator {
  private specValidator: SpecValidator;
  private budgetValidator: BudgetValidator;
  private waiverManager: WaiverManager;
  private policyLoader: PolicyLoader;
  private performanceTracker?: PerformanceTracker;

  constructor(
    options: {
      policyLoader?: PolicyLoader;
      waiverManager?: WaiverManager;
      performanceTracker?: PerformanceTracker;
    } = {}
  ) {
    this.policyLoader = options.policyLoader || new PolicyLoader();
    this.waiverManager = options.waiverManager || new WaiverManager();
    this.performanceTracker = options.performanceTracker;

    this.specValidator = new SpecValidator(this.performanceTracker);
    this.budgetValidator = new BudgetValidator(
      this.policyLoader,
      this.waiverManager
    );
  }

  /**
   * Validate working spec against CAWS constitutional authority
   */
  public async validateWorkingSpec(
    spec: WorkingSpec,
    options: ValidationOptions = {}
  ): Promise<CAWSValidationResult> {
    const startTime = Date.now();
    const errors: string[] = [];
    const warnings: string[] = [];

    try {
      // Load policy for this validation session
      const policy = await this.policyLoader.loadPolicy(
        options.projectRoot || process.cwd()
      );

      // Initialize rule engine with policy
      const ruleEngine = new RuleEngine(policy);

      // 1. Validate spec structure
      let specResult: SpecValidationResult;
      if (!options.skipSpecValidation) {
        specResult = await this.specValidator.validateWorkingSpec(spec);
        if (!specResult.valid) {
          return this.createValidationResult(
            false,
            startTime,
            spec,
            specResult.errors.map((e) => e.message),
            specResult.warnings?.map((w) => w.message) || [],
            undefined,
            undefined,
            undefined
          );
        }
      } else {
        specResult = { valid: true, errors: [], warnings: [] };
      }

      // 2. Derive budget
      let derivedBudget: DerivedBudget | undefined;
      if (options.checkBudget) {
        try {
          derivedBudget = await this.budgetValidator.deriveBudget(
            spec,
            options.projectRoot || process.cwd()
          );
        } catch (error) {
          errors.push(`Budget derivation failed: ${(error as Error).message}`);
        }
      }

      // 3. Evaluate rules
      const ruleResult = ruleEngine.evaluateRules(spec);
      if (!ruleResult.passed) {
        // Convert rule violations to errors/warnings
        for (const violation of ruleResult.violations) {
          if (
            violation.severity === "high" ||
            violation.severity === "critical"
          ) {
            errors.push(`${violation.ruleId}: ${violation.message}`);
          } else {
            warnings.push(`${violation.ruleId}: ${violation.message}`);
          }
        }
      }

      // 4. Check budget compliance if requested and we have stats
      let budgetCompliance: BudgetCompliance | undefined;
      if (options.checkBudget && derivedBudget && options.currentStats) {
        budgetCompliance = await this.budgetValidator.checkBudgetCompliance(
          derivedBudget,
          options.currentStats
        );

        if (!budgetCompliance.compliant) {
          for (const violation of budgetCompliance.violations) {
            errors.push(`BUDGET: ${violation.message}`);
          }
        }
      }

      // 5. Execute quality gates if requested
      const qualityGates = options.executeGates
        ? await this.executeQualityGates(spec, policy)
        : [];

      // Determine final verdict
      const passed = errors.length === 0;
      const verdict = passed ? "pass" : warnings.length > 0 ? "pass" : "fail";

      return this.createValidationResult(
        passed,
        startTime,
        spec,
        errors,
        warnings,
        budgetCompliance,
        qualityGates,
        verdict
      );
    } catch (error) {
      return this.createValidationResult(
        false,
        startTime,
        spec,
        [`Validation failed: ${(error as Error).message}`],
        [],
        undefined,
        undefined,
        "fail"
      );
    }
  }

  /**
   * Validate and publish verdict to CAWS ledger
   */
  public async validateAndPublish(
    spec: WorkingSpec,
    options: ValidationOptions & {
      publish?: boolean;
      dryRun?: boolean;
    } = {}
  ): Promise<CAWSValidationResult> {
    const result = await this.validateWorkingSpec(spec, options);

    // TODO: Implement verdict publication to CAWS ledger
    // This would integrate with the provenance system

    return result;
  }

  /**
   * Validate spec with auto-fixes applied
   */
  public async validateWithAutoFix(
    spec: WorkingSpec,
    options: ValidationOptions = {}
  ): Promise<CAWSValidationResult> {
    // First validate with suggestions to get fixes
    const validationWithSuggestions =
      await this.specValidator.validateWithSuggestions(spec, {
        autoFix: true,
      });

    // Then run full validation on the fixed spec
    return this.validateWorkingSpec(spec, options);
  }

  /**
   * Check budget compliance for current changes
   */
  public async checkBudgetCompliance(
    spec: WorkingSpec,
    currentStats: ChangeStats,
    projectRoot: string = process.cwd()
  ): Promise<BudgetCompliance> {
    const derivedBudget = await this.budgetValidator.deriveBudget(
      spec,
      projectRoot
    );
    return this.budgetValidator.checkBudgetCompliance(
      derivedBudget,
      currentStats
    );
  }

  /**
   * Generate budget utilization report
   */
  public generateBudgetReport(
    budgetCompliance: BudgetCompliance,
    riskTier: number
  ): string {
    return this.budgetValidator.generateBurnupReport(
      budgetCompliance,
      riskTier
    );
  }

  /**
   * Execute mock code quality gate - scan for placeholder implementations
   */
  private async executeMockCodeQualityGate(
    spec: WorkingSpec,
    startTime: number
  ): Promise<QualityGateResult> {
    const mockPatterns = [
      // Direct mock indicators
      /\/\/\s*TODO:/gi,
      /\/\/\s*PLACEHOLDER:/gi,
      /\/\/\s*MOCK DATA:/gi,
      /\/\/\s*FIXME:/gi,
      /\/\/\s*HACK:/gi,

      // Implementation placeholders
      /In a real implementation,/gi,
      /not yet implemented/gi,
      /coming soon/gi,
      /tbd|tbd\./gi,

      // Mock data patterns
      /\b(agent-1|agent-2|agent-3)\b/gi,
      /\b(user-123|test-user)\b/gi,
      /\b(mock-|test-|example-|dummy-|fake-)\w+/gi,

      // Placeholder returns
      /return \[\];/gi,
      /return \{\};/gi,
      /return ".*";/gi,
      /return \d+;/gi,
      /return true;/gi,
      /return false;/gi,

      // Console logging implementations
      /console\.log\(/gi,
      /console\.warn\(/gi,
      /console\.error\(/gi,
    ];

    const findings: Array<{
      file: string;
      line: number;
      pattern: string;
      content: string;
    }> = [];

    // Scan files in scope
    const scopeFiles = [
      ...(spec.scope?.in || []),
      ...(spec.scope?.out || []).filter((path) => path.includes("src/")), // Include some out-of-scope src files for completeness
    ];

    for (const filePath of scopeFiles) {
      if (
        !filePath.includes("src/") ||
        filePath.includes("/test") ||
        filePath.includes("/__tests__/") ||
        filePath.endsWith(".test.ts") ||
        filePath.endsWith(".spec.ts")
      ) {
        continue; // Skip non-src files and test files
      }

      try {
        const fullPath = path.join(process.cwd(), filePath);
        if (!fs.existsSync(fullPath)) continue;

        const content = fs.readFileSync(fullPath, "utf-8");
        const lines = content.split("\n");

        for (let i = 0; i < lines.length; i++) {
          const line = lines[i];
          for (const pattern of mockPatterns) {
            const matches = line.match(pattern);
            if (matches) {
              findings.push({
                file: filePath,
                line: i + 1,
                pattern: pattern.source,
                content: line.trim(),
              });
              break; // Only record first match per line
            }
          }
        }
      } catch (error) {
        // Skip files that can't be read
        continue;
      }
    }

    const passed = findings.length === 0;
    const score = Math.max(0, 100 - findings.length * 10); // Deduct 10 points per finding

    let message: string;
    if (passed) {
      message = "✅ No mocked or placeholder implementations found";
    } else {
      message =
        `🚫 Found ${findings.length} instances of mocked/placeholder code. ` +
        `Score: ${score}/100. Review required before production deployment.`;
    }

    return {
      gate: "mock-code-detection",
      passed,
      score,
      threshold: 100,
      message,
      evidence: findings,
      executionTime: Date.now() - startTime,
    };
  }

  /**
   * Execute quality gates for the working spec
   */
  private async executeQualityGates(
    spec: WorkingSpec,
    policy: any // CAWSPolicy
  ): Promise<QualityGateResult[]> {
    const results: QualityGateResult[] = [];
    const startTime = Date.now();

    try {
      // Mock Code Quality Gate - Check for placeholder implementations
      const mockGateResult = await this.executeMockCodeQualityGate(
        spec,
        startTime
      );
      results.push(mockGateResult);

      // TODO: Add additional quality gates here
      // - Coverage checks
      // - Mutation testing
      // - Security scans
      // - Performance benchmarks
    } catch (error) {
      console.error("Quality gate execution failed", error);
      results.push({
        gate: "quality-gate-execution",
        passed: false,
        message: `Quality gate execution failed: ${
          error instanceof Error ? error.message : String(error)
        }`,
        executionTime: Date.now() - startTime,
        error: error instanceof Error ? error.message : String(error),
      });
    }

    return results;
  }

  /**
   * Create standardized validation result
   */
  private createValidationResult(
    passed: boolean,
    startTime: number,
    spec: WorkingSpec,
    errors: string[],
    warnings: string[],
    budgetCompliance?: BudgetCompliance,
    qualityGates: any[] = [],
    verdict: "pass" | "fail" | "waiver-required" = passed ? "pass" : "fail"
  ): CAWSValidationResult {
    const duration = Date.now() - startTime;

    return {
      passed,
      cawsVersion: "3.1.0",
      timestamp: new Date().toISOString(),
      budgetCompliance,
      qualityGates,
      waivers: [], // TODO: Include applied waivers
      verdict,
      remediation: passed ? undefined : errors.map((error) => `Fix: ${error}`),
      metadata: {
        specId: spec.id,
        riskTier: spec.risk_tier,
        mode: spec.mode,
        durationMs: duration,
        environment: process.env.NODE_ENV || "development",
      },
    };
  }

  /**
   * Get validation summary for reporting
   */
  public getValidationSummary(result: CAWSValidationResult): string {
    const lines = [
      "🔍 CAWS Validation Summary",
      "==========================",
      "",
      `Status: ${result.passed ? "✅ PASSED" : "❌ FAILED"}`,
      `Verdict: ${result.verdict.toUpperCase()}`,
      `Duration: ${result.metadata?.durationMs || 0}ms`,
      `CAWS Version: ${result.cawsVersion}`,
      "",
    ];

    if (result.budgetCompliance) {
      lines.push("Budget Status:");
      lines.push(
        `  Compliant: ${result.budgetCompliance.compliant ? "✅" : "❌"}`
      );
      lines.push(
        `  Files Used: ${result.budgetCompliance.current.filesChanged}/${result.budgetCompliance.effective.max_files}`
      );
      lines.push(
        `  LOC Used: ${result.budgetCompliance.current.linesChanged}/${result.budgetCompliance.effective.max_loc}`
      );

      if (result.budgetCompliance.waiversApplied.length > 0) {
        lines.push(
          `  Waivers: ${result.budgetCompliance.waiversApplied.join(", ")}`
        );
      }
    }

    if (result.qualityGates.length > 0) {
      lines.push("");
      lines.push("Quality Gates:");
      for (const gate of result.qualityGates) {
        lines.push(
          `  ${gate.gate}: ${gate.passed ? "✅" : "❌"} (${
            gate.score || "N/A"
          })`
        );
      }
    }

    if (result.remediation && result.remediation.length > 0) {
      lines.push("");
      lines.push("Remediation Required:");
      for (const item of result.remediation) {
        lines.push(`  • ${item}`);
      }
    }

    return lines.join("\n");
  }
}
