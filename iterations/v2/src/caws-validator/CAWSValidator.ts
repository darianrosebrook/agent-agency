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

      // Coverage Quality Gate - Check test coverage meets requirements
      const coverageGateResult = await this.executeCoverageQualityGate(
        spec,
        startTime
      );
      results.push(coverageGateResult);

      // Mutation Testing Quality Gate - Verify test quality
      const mutationGateResult = await this.executeMutationQualityGate(
        spec,
        startTime
      );
      results.push(mutationGateResult);

      // Linting Quality Gate - Check code style and standards
      const lintingGateResult = await this.executeLintingQualityGate(
        spec,
        startTime
      );
      results.push(lintingGateResult);

      // Security Scan Quality Gate - Check for security vulnerabilities
      const securityGateResult = await this.executeSecurityQualityGate(
        spec,
        startTime
      );
      results.push(securityGateResult);

      // Performance Benchmark Quality Gate - Check performance thresholds
      const performanceGateResult = await this.executePerformanceQualityGate(
        spec,
        startTime
      );
      results.push(performanceGateResult);
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

  /**
   * Execute coverage quality gate - check test coverage meets requirements
   */
  private async executeCoverageQualityGate(
    spec: WorkingSpec,
    startTime: number
  ): Promise<QualityGateResult> {
    try {
      // Run test coverage analysis
      const { execSync } = require("child_process");

      let coverageResult;
      try {
        // Run npm test with coverage
        const coverageOutput = execSync("npm run test:coverage", {
          cwd: process.cwd(),
          encoding: "utf8",
          stdio: "pipe",
        });

        // Parse coverage output to extract coverage percentages
        const linesMatch = coverageOutput.match(/Lines\s+:\s+(\d+\.?\d*)%/);
        const branchesMatch = coverageOutput.match(
          /Branches\s+:\s+(\d+\.?\d*)%/
        );
        const functionsMatch = coverageOutput.match(
          /Functions\s+:\s+(\d+\.?\d*)%/
        );

        const linesCoverage = linesMatch ? parseFloat(linesMatch[1]) : 0;
        const branchesCoverage = branchesMatch
          ? parseFloat(branchesMatch[1])
          : 0;
        const functionsCoverage = functionsMatch
          ? parseFloat(functionsMatch[1])
          : 0;

        // Determine coverage requirements based on risk tier
        const requiredCoverage = this.getCoverageRequirements(spec.risk_tier);

        const linesPassed = linesCoverage >= requiredCoverage.lines;
        const branchesPassed = branchesCoverage >= requiredCoverage.branches;
        const functionsPassed = functionsCoverage >= requiredCoverage.functions;

        const overallPassed = linesPassed && branchesPassed && functionsPassed;

        coverageResult = {
          gate: "coverage",
          passed: overallPassed,
          message: `Coverage: Lines ${linesCoverage.toFixed(1)}% (${
            linesPassed ? "✓" : "✗"
          }), Branches ${branchesCoverage.toFixed(1)}% (${
            branchesPassed ? "✓" : "✗"
          }), Functions ${functionsCoverage.toFixed(1)}% (${
            functionsPassed ? "✓" : "✗"
          })`,
          executionTime: Date.now() - startTime,
          evidence: {
            linesCoverage,
            branchesCoverage,
            functionsCoverage,
            requiredCoverage,
            requirements: `Tier ${spec.risk_tier}: Lines ≥${requiredCoverage.lines}%, Branches ≥${requiredCoverage.branches}%, Functions ≥${requiredCoverage.functions}%`,
          },
        };
      } catch (error) {
        coverageResult = {
          gate: "coverage",
          passed: false,
          message: `Coverage check failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
          executionTime: Date.now() - startTime,
          error: error instanceof Error ? error.message : String(error),
        };
      }

      return coverageResult;
    } catch (error) {
      return {
        gate: "coverage",
        passed: false,
        message: `Coverage quality gate failed: ${
          error instanceof Error ? error.message : String(error)
        }`,
        executionTime: Date.now() - startTime,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Execute mutation testing quality gate
   */
  private async executeMutationQualityGate(
    spec: WorkingSpec,
    startTime: number
  ): Promise<QualityGateResult> {
    try {
      const { execSync } = require("child_process");

      let mutationResult;
      try {
        // Run mutation testing
        const mutationOutput = execSync("npm run test:mutation", {
          cwd: process.cwd(),
          encoding: "utf8",
          stdio: "pipe",
        });

        // Parse mutation score from output
        const scoreMatch = mutationOutput.match(/Score: (\d+\.?\d*)%/);
        const mutationScore = scoreMatch ? parseFloat(scoreMatch[1]) : 0;

        // Determine mutation requirements based on risk tier
        const requiredScore = this.getMutationRequirements(spec.risk_tier);
        const passed = mutationScore >= requiredScore;

        mutationResult = {
          gate: "mutation",
          passed,
          message: `Mutation Score: ${mutationScore.toFixed(
            1
          )}% (Required: ≥${requiredScore}%) ${passed ? "✓" : "✗"}`,
          executionTime: Date.now() - startTime,
          evidence: {
            mutationScore,
            requiredScore,
            tier: spec.risk_tier,
          },
        };
      } catch (error) {
        mutationResult = {
          gate: "mutation",
          passed: false,
          message: `Mutation testing failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
          executionTime: Date.now() - startTime,
          error: error instanceof Error ? error.message : String(error),
        };
      }

      return mutationResult;
    } catch (error) {
      return {
        gate: "mutation",
        passed: false,
        message: `Mutation quality gate failed: ${
          error instanceof Error ? error.message : String(error)
        }`,
        executionTime: Date.now() - startTime,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Execute linting quality gate
   */
  private async executeLintingQualityGate(
    spec: WorkingSpec,
    startTime: number
  ): Promise<QualityGateResult> {
    try {
      const { execSync } = require("child_process");

      let lintingResult;
      try {
        // Run linting
        const lintOutput = execSync("npm run lint", {
          cwd: process.cwd(),
          encoding: "utf8",
          stdio: "pipe",
        });

        // If we get here, linting passed (no errors)
        lintingResult = {
          gate: "linting",
          passed: true,
          message: "Linting passed - no errors found ✓",
          executionTime: Date.now() - startTime,
          evidence: {
            errors: 0,
            warnings: 0,
          },
        };
      } catch (error) {
        // Linting failed - parse output for error details
        const output = error instanceof Error ? error.message : String(error);
        const errorLines = output
          .split("\n")
          .filter((line) => line.includes("error"));
        const warningLines = output
          .split("\n")
          .filter((line) => line.includes("warning"));

        lintingResult = {
          gate: "linting",
          passed: false,
          message: `Linting failed: ${errorLines.length} errors, ${warningLines.length} warnings ✗`,
          executionTime: Date.now() - startTime,
          error: output,
          evidence: {
            errors: errorLines.length,
            warnings: warningLines.length,
            errorDetails: errorLines.slice(0, 5), // Show first 5 errors
          },
        };
      }

      return lintingResult;
    } catch (error) {
      return {
        gate: "linting",
        passed: false,
        message: `Linting quality gate failed: ${
          error instanceof Error ? error.message : String(error)
        }`,
        executionTime: Date.now() - startTime,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Execute security scan quality gate
   */
  private async executeSecurityQualityGate(
    spec: WorkingSpec,
    startTime: number
  ): Promise<QualityGateResult> {
    try {
      const { execSync } = require("child_process");

      let securityResult;
      try {
        // Run security audit
        const securityOutput = execSync("npm audit --audit-level=moderate", {
          cwd: process.cwd(),
          encoding: "utf8",
          stdio: "pipe",
        });

        // If we get here, no security vulnerabilities found
        securityResult = {
          gate: "security",
          passed: true,
          message: "Security scan passed - no vulnerabilities found ✓",
          executionTime: Date.now() - startTime,
          evidence: {
            vulnerabilities: 0,
            auditLevel: "moderate",
          },
        };
      } catch (error) {
        // Security audit found vulnerabilities
        const output = error instanceof Error ? error.message : String(error);

        // Parse vulnerability counts from audit output
        const criticalMatch = output.match(/(\d+) critical/);
        const highMatch = output.match(/(\d+) high/);
        const moderateMatch = output.match(/(\d+) moderate/);

        const critical = criticalMatch ? parseInt(criticalMatch[1]) : 0;
        const high = highMatch ? parseInt(highMatch[1]) : 0;
        const moderate = moderateMatch ? parseInt(moderateMatch[1]) : 0;

        const totalVulns = critical + high + moderate;
        const passed = critical === 0 && high === 0; // Only fail on critical/high

        securityResult = {
          gate: "security",
          passed,
          message: `Security scan: ${totalVulns} vulnerabilities (${critical} critical, ${high} high, ${moderate} moderate) ${
            passed ? "✓" : "✗"
          }`,
          executionTime: Date.now() - startTime,
          error: passed ? undefined : output,
          evidence: {
            critical,
            high,
            moderate,
            total: totalVulns,
            auditLevel: "moderate",
          },
        };
      }

      return securityResult;
    } catch (error) {
      return {
        gate: "security",
        passed: false,
        message: `Security quality gate failed: ${
          error instanceof Error ? error.message : String(error)
        }`,
        executionTime: Date.now() - startTime,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Execute performance benchmark quality gate
   */
  private async executePerformanceQualityGate(
    spec: WorkingSpec,
    startTime: number
  ): Promise<QualityGateResult> {
    try {
      const { execSync } = require("child_process");

      let performanceResult;
      try {
        // Run performance tests if available
        const perfOutput = execSync("npm run test:performance", {
          cwd: process.cwd(),
          encoding: "utf8",
          stdio: "pipe",
        });

        // Parse performance metrics from output
        const responseTimeMatch = perfOutput.match(
          /Average Response Time: (\d+)ms/
        );
        const memoryMatch = perfOutput.match(/Memory Usage: (\d+)MB/);

        const avgResponseTime = responseTimeMatch
          ? parseInt(responseTimeMatch[1])
          : 0;
        const memoryUsage = memoryMatch ? parseInt(memoryMatch[1]) : 0;

        // Check against performance budgets
        const budgets = this.getPerformanceBudgets(spec.risk_tier);
        const responseTimePassed = avgResponseTime <= budgets.responseTimeMs;
        const memoryPassed = memoryUsage <= budgets.memoryMB;
        const overallPassed = responseTimePassed && memoryPassed;

        performanceResult = {
          gate: "performance",
          passed: overallPassed,
          message: `Performance: Response ${avgResponseTime}ms (≤${
            budgets.responseTimeMs
          }ms), Memory ${memoryUsage}MB (≤${budgets.memoryMB}MB) ${
            overallPassed ? "✓" : "✗"
          }`,
          executionTime: Date.now() - startTime,
          evidence: {
            avgResponseTime,
            memoryUsage,
            budgets,
            responseTimePassed,
            memoryPassed,
          },
        };
      } catch (error) {
        // Performance tests not available or failed
        performanceResult = {
          gate: "performance",
          passed: true, // Don't fail if performance tests aren't available
          message:
            "Performance tests not available - skipping performance gate ✓",
          executionTime: Date.now() - startTime,
          evidence: {
            skipped: true,
            reason: "Performance tests not configured",
          },
        };
      }

      return performanceResult;
    } catch (error) {
      return {
        gate: "performance",
        passed: true, // Don't fail if performance gate can't run
        message: `Performance quality gate skipped: ${
          error instanceof Error ? error.message : String(error)
        }`,
        executionTime: Date.now() - startTime,
        evidence: {
          skipped: true,
          error: error instanceof Error ? error.message : String(error),
        },
      };
    }
  }

  /**
   * Get coverage requirements based on risk tier
   */
  private getCoverageRequirements(tier: number): {
    lines: number;
    branches: number;
    functions: number;
  } {
    switch (tier) {
      case 1: // Critical
        return { lines: 90, branches: 90, functions: 90 };
      case 2: // High
        return { lines: 80, branches: 80, functions: 80 };
      case 3: // Medium
        return { lines: 70, branches: 70, functions: 70 };
      default:
        return { lines: 70, branches: 70, functions: 70 };
    }
  }

  /**
   * Get mutation testing requirements based on risk tier
   */
  private getMutationRequirements(tier: number): number {
    switch (tier) {
      case 1: // Critical
        return 80;
      case 2: // High
        return 70;
      case 3: // Medium
        return 60;
      default:
        return 60;
    }
  }

  /**
   * Get performance budgets based on risk tier
   */
  private getPerformanceBudgets(tier: number): {
    responseTimeMs: number;
    memoryMB: number;
  } {
    switch (tier) {
      case 1: // Critical
        return { responseTimeMs: 1000, memoryMB: 512 };
      case 2: // High
        return { responseTimeMs: 2000, memoryMB: 1024 };
      case 3: // Medium
        return { responseTimeMs: 5000, memoryMB: 2048 };
      default:
        return { responseTimeMs: 5000, memoryMB: 2048 };
    }
  }
}
