/**
 * CAWS Gate Checker
 * Consolidated gate checking logic for coverage, mutation, contracts, and trust score
 *
 * @author @darianrosebrook
 */

import * as path from "path";
import { CawsBaseTool } from "./base-tool.js";
import {
  GateResult,
  GateCheckOptions,
  MutationData,
  ContractTestResults,
  TierPolicy,
  WaiverConfig,
  HumanOverride,
  AIAssessment,
} from "./types.js";
import { WaiversManager } from "./waivers-manager.js";

export class CawsGateChecker extends CawsBaseTool {
  private tierPolicies: Record<number, TierPolicy> = {
    1: {
      min_branch: 0.9,
      min_mutation: 0.7,
      min_coverage: 0.9,
      requires_contracts: true,
      requires_manual_review: true,
    },
    2: {
      min_branch: 0.8,
      min_mutation: 0.5,
      min_coverage: 0.8,
      requires_contracts: true,
    },
    3: {
      min_branch: 0.7,
      min_mutation: 0.3,
      min_coverage: 0.7,
      requires_contracts: false,
    },
  };

  private waiversManager: WaiversManager;

  constructor() {
    super();
    this.loadTierPolicies();
    this.waiversManager = new WaiversManager();
  }

  /**
   * Load tier policies from configuration
   */
  private loadTierPolicies(): void {
    const policy = this.loadTierPolicy();
    if (policy) {
      this.tierPolicies = { ...this.tierPolicies, ...policy };
    }
  }

  /**
   * Auto-detect the correct working directory for coverage/mutation reports in monorepos
   */
  private findReportDirectory(
    startPath: string = this.getWorkingDirectory()
  ): string {
    // Priority 1: Check if the current directory has the reports or test results
    if (
      this.hasCoverageReports(startPath) ||
      this.hasMutationReports(startPath) ||
      this.hasTestResults(startPath)
    ) {
      return startPath;
    }

    // Priority 2: Check for npm workspaces configuration
    const packageJsonPath = path.join(startPath, "package.json");
    if (this.pathExists(packageJsonPath)) {
      try {
        const packageJson = this.readJsonFile<any>(packageJsonPath);
        if (packageJson?.workspaces) {
          const workspaces = packageJson.workspaces;

          // Handle workspace patterns (e.g., ["packages/*", "iterations/*"])
          for (const wsPattern of workspaces) {
            if (wsPattern.includes("*")) {
              const baseDir = wsPattern.split("*")[0];
              const fullBaseDir = path.join(startPath, baseDir);

              if (this.pathExists(fullBaseDir)) {
                const entries = fs.readdirSync(fullBaseDir, {
                  withFileTypes: true,
                });
                for (const entry of entries) {
                  if (entry.isDirectory()) {
                    const wsPath = path.join(fullBaseDir, entry.name);
                    if (
                      this.hasCoverageReports(wsPath) ||
                      this.hasMutationReports(wsPath) ||
                      this.hasTestResults(wsPath)
                    ) {
                      return wsPath;
                    }
                  }
                }
              }
            } else {
              // Direct workspace path
              const wsPath = path.join(startPath, wsPattern);
              if (
                this.hasCoverageReports(wsPath) ||
                this.hasMutationReports(wsPath) ||
                this.hasTestResults(wsPath)
              ) {
                return wsPath;
              }
            }
          }
        }

        // Priority 3: If no reports found in workspaces, look for workspaces with test scripts
        if (packageJson?.workspaces) {
          for (const wsPattern of workspaces) {
            if (wsPattern.includes("*")) {
              const baseDir = wsPattern.split("*")[0];
              const fullBaseDir = path.join(startPath, baseDir);

              if (this.pathExists(fullBaseDir)) {
                const entries = fs.readdirSync(fullBaseDir, {
                  withFileTypes: true,
                });
                for (const entry of entries) {
                  if (entry.isDirectory()) {
                    const wsPath = path.join(fullBaseDir, entry.name);
                    if (this.hasTestScript(wsPath)) {
                      // Found a workspace with tests, prefer this even without reports
                      return wsPath;
                    }
                  }
                }
              }
            } else {
              const wsPath = path.join(startPath, wsPattern);
              if (this.hasTestScript(wsPath)) {
                return wsPath;
              }
            }
          }
        }
      } catch (error) {
        // Ignore workspace parsing errors
      }
    }

    // Fall back to original working directory
    return startPath;
  }

  /**
   * Check if a directory has coverage reports
   */
  private hasCoverageReports(dirPath: string): boolean {
    const coveragePath = path.join(dirPath, "coverage", "coverage-final.json");
    return this.pathExists(coveragePath);
  }

  /**
   * Check if a directory has mutation reports
   */
  private hasMutationReports(dirPath: string): boolean {
    const mutationPath = path.join(
      dirPath,
      "reports",
      "mutation",
      "mutation.json"
    );
    return this.pathExists(mutationPath);
  }

  /**
   * Check if a directory has test results
   */
  private hasTestResults(dirPath: string): boolean {
    const testResultsPath = path.join(dirPath, "test-results");
    if (this.pathExists(testResultsPath)) {
      try {
        const entries = fs.readdirSync(testResultsPath);
        return entries.some(
          (entry) => entry.endsWith(".json") || entry.endsWith(".xml")
        );
      } catch (error) {
        // Ignore read errors
      }
    }
    return false;
  }

  /**
   * Check if a directory has a package.json with test scripts
   */
  private hasTestScript(dirPath: string): boolean {
    const packageJsonPath = path.join(dirPath, "package.json");
    if (this.pathExists(packageJsonPath)) {
      try {
        const packageJson = this.readJsonFile<any>(packageJsonPath);
        return !!packageJson?.scripts?.test;
      } catch (error) {
        // Ignore parse errors
      }
    }
    return false;
  }

  /**
   * Check if a waiver applies to the given gate
   */
  private async checkWaiver(
    gate: string,
    workingDirectory?: string
  ): Promise<{
    waived: boolean;
    waiver?: WaiverConfig;
    reason?: string;
  }> {
    try {
      const waivers = await this.waiversManager.getWaiversByGate(gate);
      if (waivers.length === 0) {
        return { waived: false };
      }

      // Implement waiver validation with comprehensive checks
      // 1. Waiver database: Maintain database of active waivers and policies
      //    - Store and manage waiver configurations and policies
      //    - Implement waiver lifecycle management and expiration
      //    - Handle waiver validation and authorization verification
      // 2. Waiver matching: Match requests against applicable waivers
      //    - Implement waiver matching algorithms and criteria
      //    - Handle waiver scope and condition validation
      //    - Process waiver application and approval workflows
      // 3. Waiver enforcement: Enforce waiver policies and restrictions
      //    - Implement waiver enforcement and compliance checking
      //    - Handle waiver violations and remediation
      //    - Track waiver usage and compliance metrics
      // 4. Waiver auditing: Audit waiver usage and compliance
      //    - Generate waiver usage reports and analytics
      //    - Implement waiver compliance monitoring and alerting
      //    - Ensure waiver management meets security and compliance standards

      for (const waiver of waivers) {
        // Validate waiver is in active state and not expired
        const validationResult = await this.validateWaiverStatus(waiver);
        if (!validationResult.valid) {
          this.logWaiverAudit(gate, waiver, 'validation_failed', validationResult.reason);
          continue;
        }

        // Verify waiver scope and conditions match request context
        const scopeMatch = this.validateWaiverScope(waiver, workingDirectory);
        if (!scopeMatch.matched) {
          this.logWaiverAudit(gate, waiver, 'scope_mismatch', scopeMatch.reason);
          continue;
        }

        // Check waiver usage limits and frequency
        const usageCheck = await this.checkWaiverUsageCompliance(waiver);
        if (!usageCheck.compliant) {
          this.logWaiverAudit(gate, waiver, 'usage_limit_exceeded', usageCheck.reason);
          continue;
        }

        // Waiver is valid and applicable
        this.logWaiverAudit(gate, waiver, 'waiver_applied', 'Waiver criteria met');
        this.recordWaiverUsage(waiver);

        return {
          waived: true,
          waiver,
          reason: `Waiver approved by ${waiver.approved_by}: ${waiver.reason}`,
        };
      }

      return { waived: false };
    } catch (error) {
      this.logError(`Waiver check failed: ${error}`);
      return { waived: false, reason: `Waiver check failed: ${error}` };
    }
  }

  /**
   * Validate waiver status and expiration
   */
  private async validateWaiverStatus(waiver: WaiverConfig): Promise<{
    valid: boolean;
    reason?: string;
  }> {
    // Check if waiver status is explicitly revoked
    if (waiver.status === 'revoked') {
      return { valid: false, reason: 'Waiver has been revoked' };
    }

    // Parse expiry date
    const expiryDate = new Date(waiver.expiry);
    const now = new Date();

    if (expiryDate < now) {
      return { valid: false, reason: `Waiver expired on ${waiver.expiry}` };
    }

    // Verify waiver has required authorization fields
    if (!waiver.approved_by) {
      return { valid: false, reason: 'Waiver lacks approval authorization' };
    }

    return { valid: true };
  }

  /**
   * Validate waiver scope matches request context
   */
  private validateWaiverScope(
    waiver: WaiverConfig,
    workingDirectory?: string
  ): { matched: boolean; reason?: string } {
    // If waiver specifies a scope path, verify it matches
    // This allows waivers to be scoped to specific projects or modules
    const waiverScope = (waiver as any).scope;
    if (waiverScope && workingDirectory) {
      const isInScope = workingDirectory.includes(waiverScope) ||
        workingDirectory.startsWith(waiverScope);

      if (!isInScope) {
        return {
          matched: false,
          reason: `Waiver scope "${waiverScope}" does not match working directory "${workingDirectory}"`,
        };
      }
    }

    return { matched: true };
  }

  /**
   * Check waiver usage compliance (frequency and limits)
   */
  private async checkWaiverUsageCompliance(waiver: WaiverConfig): Promise<{
    compliant: boolean;
    reason?: string;
  }> {
    const usageLog = this.loadWaiverUsageLog();
    const waiverId = `${waiver.gate}:${waiver.owner}`;

    if (!usageLog[waiverId]) {
      // First usage of this waiver
      return { compliant: true };
    }

    const usage = usageLog[waiverId];
    const now = new Date();
    const daysSinceCreation = Math.floor(
      (now.getTime() - new Date(waiver.created_at).getTime()) / (1000 * 60 * 60 * 24)
    );

    // Check usage frequency: max 5 uses per week during waiver lifetime
    const weeklyUsages = usage.uses.filter((use: string) => {
      const useDate = new Date(use);
      const daysAgo = Math.floor(
        (now.getTime() - useDate.getTime()) / (1000 * 60 * 60 * 24)
      );
      return daysAgo <= 7;
    }).length;

    if (weeklyUsages > 5) {
      return {
        compliant: false,
        reason: `Waiver usage limit exceeded: ${weeklyUsages} uses in past week (max 5)`,
      };
    }

    // Check total usage limit: max 20 uses per waiver lifetime
    if (usage.uses.length >= 20) {
      return {
        compliant: false,
        reason: 'Waiver has reached maximum usage count (20)',
      };
    }

    return { compliant: true };
  }

  /**
   * Load waiver usage tracking log
   */
  private loadWaiverUsageLog(): Record<string, any> {
    const usageLogPath = path.join(this.getCawsDirectory(), 'waiver-usage.json');

    if (this.pathExists(usageLogPath)) {
      try {
        return this.readJsonFile(usageLogPath) || {};
      } catch {
        return {};
      }
    }

    return {};
  }

  /**
   * Record waiver usage for compliance tracking
   */
  private recordWaiverUsage(waiver: WaiverConfig): void {
    const usageLog = this.loadWaiverUsageLog();
    const waiverId = `${waiver.gate}:${waiver.owner}`;

    if (!usageLog[waiverId]) {
      usageLog[waiverId] = { uses: [] };
    }

    usageLog[waiverId].uses.push(new Date().toISOString());

    const usageLogPath = path.join(this.getCawsDirectory(), 'waiver-usage.json');
    try {
      const dir = path.dirname(usageLogPath);
      if (!this.pathExists(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      this.writeJsonFile(usageLogPath, usageLog, { createDir: true });
    } catch (error) {
      this.logError(`Failed to record waiver usage: ${error}`);
    }
  }

  /**
   * Log waiver validation events for audit trail
   */
  private logWaiverAudit(
    gate: string,
    waiver: WaiverConfig,
    event: string,
    details: string
  ): void {
    const auditLogPath = path.join(this.getCawsDirectory(), 'waiver-audit.log');

    const auditEntry = {
      timestamp: new Date().toISOString(),
      gate,
      waiver_id: `${waiver.gate}:${waiver.owner}`,
      event,
      details,
      approved_by: waiver.approved_by,
    };

    try {
      const dir = path.dirname(auditLogPath);
      if (!this.pathExists(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }

      const existingLog = this.pathExists(auditLogPath)
        ? fs.readFileSync(auditLogPath, 'utf-8')
        : '';
      fs.appendFileSync(
        auditLogPath,
        JSON.stringify(auditEntry) + '\n',
        'utf-8'
      );
    } catch (error) {
      this.logError(`Failed to write waiver audit log: ${error}`);
    }
  }

  /**
   * Load and validate working spec from project
   */
  private async loadWorkingSpec(workingDirectory?: string): Promise<{
    spec?: any;
    experiment_mode?: boolean;
    human_override?: HumanOverride;
    ai_assessment?: AIAssessment;
    errors?: string[];
  }> {
    try {
      const specPath = path.join(
        workingDirectory || this.getWorkingDirectory(),
        ".caws/working-spec.yml"
      );

      if (!this.pathExists(specPath)) {
        return { errors: ["Working spec not found at .caws/working-spec.yml"] };
      }

      const spec = await this.readYamlFile(specPath);
      if (!spec) {
        return { errors: ["Failed to parse working spec"] };
      }

      return {
        spec,
        experiment_mode: spec.experiment_mode,
        human_override: spec.human_override,
        ai_assessment: spec.ai_assessment,
      };
    } catch (error) {
      return { errors: [`Failed to load working spec: ${error}`] };
    }
  }

  /**
   * Check if human override applies to waive requirements
   */
  private checkHumanOverride(
    humanOverride: HumanOverride | undefined,
    requirement: string
  ): { waived: boolean; reason?: string } {
    if (!humanOverride) {
      return { waived: false };
    }

    if (humanOverride.waived_requirements?.includes(requirement)) {
      return {
        waived: true,
        reason: `Human override by ${humanOverride.approved_by}: ${humanOverride.reason}`,
      };
    }

    return { waived: false };
  }

  /**
   * Check if experiment mode applies reduced requirements
   */
  private checkExperimentMode(experimentMode: boolean | undefined): {
    reduced: boolean;
    adjustments?: Record<string, any>;
  } {
    if (!experimentMode) {
      return { reduced: false };
    }

    return {
      reduced: true,
      adjustments: {
        skip_mutation: true,
        skip_contracts: true,
        reduced_coverage: 0.5, // Minimum coverage for experiments
        skip_manual_review: true,
      },
    };
  }

  /**
   * Check branch coverage against tier requirements
   */
  async checkCoverage(options: GateCheckOptions): Promise<GateResult> {
    try {
      // Check waivers and overrides first
      const waiverCheck = await this.checkWaiver(
        "coverage",
        options.workingDirectory
      );
      if (waiverCheck.waived) {
        return {
          passed: true,
          score: 1.0, // Waived checks pass with perfect score
          details: {
            waived: true,
            waiver_reason: waiverCheck.waiver?.reason,
            waiver_owner: waiverCheck.waiver?.owner,
          },
          tier: options.tier,
        };
      }

      // Load working spec for overrides and experiment mode
      const specData = await this.loadWorkingSpec(options.workingDirectory);

      // Check human override
      const overrideCheck = this.checkHumanOverride(
        specData.human_override,
        "coverage"
      );
      if (overrideCheck.waived) {
        return {
          passed: true,
          score: 1.0,
          details: {
            overridden: true,
            override_reason: overrideCheck.reason,
          },
          tier: options.tier,
        };
      }

      // Check experiment mode
      const experimentCheck = this.checkExperimentMode(
        specData.experiment_mode
      );

      let effectiveTier = options.tier;
      if (
        experimentCheck.reduced &&
        experimentCheck.adjustments?.reduced_coverage
      ) {
        // For experiments, use reduced coverage requirement
        effectiveTier = 4; // Special experiment tier
        this.tierPolicies[4] = {
          min_branch: experimentCheck.adjustments.reduced_coverage,
          min_mutation: 0,
          min_coverage: experimentCheck.adjustments.reduced_coverage,
          requires_contracts: false,
          requires_manual_review: false,
        };
      }

      // Auto-detect the correct directory for coverage reports
      const reportDir = this.findReportDirectory(
        options.workingDirectory || this.getWorkingDirectory()
      );
      const coveragePath = path.join(
        reportDir,
        "coverage",
        "coverage-final.json"
      );

      if (!this.pathExists(coveragePath)) {
        return {
          passed: false,
          score: 0,
          details: {
            error: "Coverage report not found. Run tests with coverage first.",
            searched_paths: [
              path.join(reportDir, "coverage", "coverage-final.json"),
              path.join(
                this.getWorkingDirectory(),
                "coverage",
                "coverage-final.json"
              ),
            ],
            expected_format: "Istanbul coverage format (coverage-final.json)",
            expected_schema: {
              description: "JSON object with coverage data by file",
              example: {
                "/path/to/file.js": {
                  statementMap: {
                    /* ... */
                  },
                  fnMap: {
                    /* ... */
                  },
                  branchMap: {
                    /* ... */
                  },
                  s: {
                    /* hit counts */
                  },
                  f: {
                    /* function hits */
                  },
                  b: {
                    /* branch hits */
                  },
                },
              },
            },
            run_command: "npm test -- --coverage --coverageReporters=json",
            alternative_commands: [
              "npm run test:coverage",
              "jest --coverage --coverageReporters=json",
              "vitest run --coverage",
            ],
            workspace_hint:
              reportDir !== this.getWorkingDirectory()
                ? `Auto-detected workspace: ${path.relative(
                    this.getWorkingDirectory(),
                    reportDir
                  )}`
                : "Run from workspace directory if using monorepo",
            waiver_available: true,
            waiver_suggestion:
              "If this is an exceptional case, consider creating a coverage waiver",
            waiver_command:
              'caws waivers create --title="Coverage waiver" --reason=emergency_hotfix --gates=coverage',
          },
          errors: [
            `Coverage report not found at ${path.relative(
              this.getWorkingDirectory(),
              coveragePath
            )}`,
          ],
        };
      }

      const coverageData = this.readJsonFile<any>(coveragePath);
      if (!coverageData) {
        return {
          passed: false,
          score: 0,
          details: { error: "Failed to parse coverage data" },
          errors: ["Failed to parse coverage data"],
        };
      }

      // Calculate coverage from detailed data
      let totalStatements = 0;
      let coveredStatements = 0;
      let totalBranches = 0;
      let coveredBranches = 0;
      let totalFunctions = 0;
      let coveredFunctions = 0;

      for (const file of Object.values(coverageData)) {
        const fileData = file as any;
        if (fileData.s) {
          totalStatements += Object.keys(fileData.s).length;
          coveredStatements += Object.values(fileData.s).filter(
            (s: any) => s > 0
          ).length;
        }
        if (fileData.b) {
          for (const branches of Object.values(fileData.b) as number[][]) {
            totalBranches += branches.length;
            coveredBranches += branches.filter((b: number) => b > 0).length;
          }
        }
        if (fileData.f) {
          totalFunctions += Object.keys(fileData.f).length;
          coveredFunctions += Object.values(fileData.f).filter(
            (f: any) => f > 0
          ).length;
        }
      }

      // Calculate percentages
      const statementsPct =
        totalStatements > 0 ? (coveredStatements / totalStatements) * 100 : 0;
      const branchesPct =
        totalBranches > 0 ? (coveredBranches / totalBranches) * 100 : 0;
      const functionsPct =
        totalFunctions > 0 ? (coveredFunctions / totalFunctions) * 100 : 0;

      const branchCoverage = branchesPct / 100;
      const policy = this.tierPolicies[effectiveTier];
      const passed = branchCoverage >= policy.min_branch;

      return {
        passed,
        score: branchCoverage,
        details: {
          branch_coverage: branchCoverage,
          required_branch: policy.min_branch,
          functions_coverage: functionsPct / 100,
          lines_coverage: statementsPct / 100,
          statements_coverage: statementsPct / 100,
        },
      };
    } catch (error) {
      return {
        passed: false,
        score: 0,
        details: { error: `Coverage check failed: ${error}` },
        errors: [`Coverage check failed: ${error}`],
      };
    }
  }

  /**
   * Check mutation testing score
   */
  async checkMutation(options: GateCheckOptions): Promise<GateResult> {
    try {
      // Check waivers and overrides first
      const waiverCheck = await this.checkWaiver(
        "mutation",
        options.workingDirectory
      );
      if (waiverCheck.waived) {
        return {
          passed: true,
          score: 1.0,
          details: {
            waived: true,
            waiver_reason: waiverCheck.waiver?.reason,
            waiver_owner: waiverCheck.waiver?.owner,
          },
          tier: options.tier,
        };
      }

      // Load working spec for overrides and experiment mode
      const specData = await this.loadWorkingSpec(options.workingDirectory);

      // Check human override
      const overrideCheck = this.checkHumanOverride(
        specData.human_override,
        "mutation_testing"
      );
      if (overrideCheck.waived) {
        return {
          passed: true,
          score: 1.0,
          details: {
            overridden: true,
            override_reason: overrideCheck.reason,
          },
          tier: options.tier,
        };
      }

      // Check experiment mode
      const experimentCheck = this.checkExperimentMode(
        specData.experiment_mode
      );
      if (
        experimentCheck.reduced &&
        experimentCheck.adjustments?.skip_mutation
      ) {
        return {
          passed: true,
          score: 1.0,
          details: {
            experiment_mode: true,
            mutation_skipped: true,
          },
          tier: options.tier,
        };
      }

      // Auto-detect the correct directory for mutation reports
      const reportDir = this.findReportDirectory(
        options.workingDirectory || this.getWorkingDirectory()
      );
      const mutationPath = path.join(
        reportDir,
        "reports",
        "mutation",
        "mutation.json"
      );

      if (!this.pathExists(mutationPath)) {
        return {
          passed: false,
          score: 0,
          details: {
            error: "Mutation report not found. Run mutation tests first.",
            searched_paths: [
              path.join(reportDir, "reports", "mutation", "mutation.json"),
              path.join(
                this.getWorkingDirectory(),
                "reports",
                "mutation",
                "mutation.json"
              ),
            ],
            expected_format: "Stryker mutation testing JSON report",
            expected_schema: {
              description: "JSON object with mutation testing results",
              example: {
                files: {
                  /* file-specific results */
                },
                testFiles: {
                  /* test file results */
                },
                mutants: [
                  {
                    /* mutant details */
                  },
                ],
                metrics: {
                  killed: 85,
                  survived: 5,
                  timeout: 2,
                  totalDetected: 92,
                  totalUndetected: 0,
                  totalValid: 92,
                },
              },
            },
            run_command: "npx stryker run",
            alternative_commands: [
              "npm run test:mutation",
              "npx stryker run --configFile stryker.conf.json",
              "yarn mutation:test",
            ],
            workspace_hint:
              reportDir !== this.getWorkingDirectory()
                ? `Auto-detected workspace: ${path.relative(
                    this.getWorkingDirectory(),
                    reportDir
                  )}`
                : "Run from workspace directory if using monorepo",
          },
          errors: [
            `Mutation report not found at ${path.relative(
              this.getWorkingDirectory(),
              mutationPath
            )}`,
          ],
        };
      }

      const mutationData = this.readJsonFile<MutationData>(mutationPath);
      if (!mutationData) {
        return {
          passed: false,
          score: 0,
          details: { error: "Failed to parse mutation data" },
          errors: ["Failed to parse mutation data"],
        };
      }

      const killed = mutationData.metrics.killed || 0;
      const total = mutationData.metrics.totalDetected || 1;
      const mutationScore = killed / total;
      const policy = this.tierPolicies[options.tier];
      const passed = mutationScore >= policy.min_mutation;

      return {
        passed,
        score: mutationScore,
        details: {
          mutation_score: mutationScore,
          required_mutation: policy.min_mutation,
          killed,
          total,
          survived: mutationData.metrics.survived || 0,
        },
      };
    } catch (error) {
      return {
        passed: false,
        score: 0,
        details: { error: `Mutation check failed: ${error}` },
        errors: [`Mutation check failed: ${error}`],
      };
    }
  }

  /**
   * Check contract test compliance
   */
  async checkContracts(options: GateCheckOptions): Promise<GateResult> {
    try {
      // Check waivers and overrides first
      const waiverCheck = await this.checkWaiver(
        "contracts",
        options.workingDirectory
      );
      if (waiverCheck.waived) {
        return {
          passed: true,
          score: 1.0,
          details: {
            waived: true,
            waiver_reason: waiverCheck.waiver?.reason,
            waiver_owner: waiverCheck.waiver?.owner,
          },
          tier: options.tier,
        };
      }

      const policy = this.tierPolicies[options.tier];

      if (!policy.requires_contracts) {
        return {
          passed: true,
          score: 1.0,
          details: { contracts_required: false, tier: options.tier },
        };
      }

      // Auto-detect the correct directory for contract test results
      const reportDir = this.findReportDirectory(
        options.workingDirectory || this.getWorkingDirectory()
      );
      const contractResultsPath = path.join(
        reportDir,
        "test-results",
        "contract-results.json"
      );

      if (!this.pathExists(contractResultsPath)) {
        return {
          passed: false,
          score: 0,
          details: {
            error: "Contract test results not found",
            searched_paths: [
              path.join(reportDir, "test-results", "contract-results.json"),
              path.join(
                this.getWorkingDirectory(),
                "test-results",
                "contract-results.json"
              ),
              path.join(reportDir, ".caws", "contract-results.json"),
              path.join(
                this.getWorkingDirectory(),
                ".caws",
                "contract-results.json"
              ),
            ],
            expected_format:
              "JSON with { tests: [], passed: boolean, numPassed: number, numTotal: number }",
            example_command:
              "npm run test:contract -- --json --outputFile=test-results/contract-results.json",
          },
          errors: [
            `Contract test results not found. Searched in: ${[
              path.relative(
                this.getWorkingDirectory(),
                path.join(reportDir, "test-results", "contract-results.json")
              ),
              "test-results/contract-results.json",
              ".caws/contract-results.json",
            ].join(", ")}`,
          ],
        };
      }

      const results =
        this.readJsonFile<ContractTestResults>(contractResultsPath);
      if (!results) {
        return {
          passed: false,
          score: 0,
          details: { error: "Failed to parse contract test results" },
          errors: ["Failed to parse contract test results"],
        };
      }

      const passed =
        results.numPassed === results.numTotal && results.numTotal > 0;

      return {
        passed,
        score: passed ? 1.0 : 0,
        details: {
          tests_passed: results.numPassed,
          tests_total: results.numTotal,
          consumer_tests: results.consumer || false,
          provider_tests: results.provider || false,
        },
      };
    } catch (error) {
      return {
        passed: false,
        score: 0,
        details: { error: `Contract check failed: ${error}` },
        errors: [`Contract check failed: ${error}`],
      };
    }
  }

  /**
   * Calculate overall trust score
   */
  async calculateTrustScore(options: GateCheckOptions): Promise<GateResult> {
    try {
      // Run all gate checks
      const [coverageResult, mutationResult, contractResult] =
        await Promise.all([
          this.checkCoverage(options),
          this.checkMutation(options),
          this.checkContracts(options),
        ]);

      // Load provenance if available
      let provenance = null;
      try {
        const provenancePath = path.join(
          options.workingDirectory || this.getWorkingDirectory(),
          ".agent/provenance.json"
        );
        if (this.pathExists(provenancePath)) {
          provenance = this.readJsonFile(provenancePath);
        }
      } catch {
        // Provenance not available
      }

      // CAWS trust score weights
      const weights = {
        coverage: 0.3,
        mutation: 0.3,
        contracts: 0.2,
        a11y: 0.1,
        perf: 0.1,
      };

      // Calculate weighted score
      let totalScore = 0;
      let totalWeight = 0;

      // Coverage component
      totalScore += coverageResult.score * weights.coverage;
      totalWeight += weights.coverage;

      // Mutation component
      totalScore += mutationResult.score * weights.mutation;
      totalWeight += weights.mutation;

      // Contracts component
      totalScore += contractResult.score * weights.contracts;
      totalWeight += weights.contracts;

      // A11y component (placeholder - would check axe results)
      const a11yScore = provenance?.results?.a11y === "pass" ? 1.0 : 0.5;
      totalScore += a11yScore * weights.a11y;
      totalWeight += weights.a11y;

      // Performance component (placeholder - would check perf budgets)
      const perfScore = provenance?.results?.perf ? 0.8 : 0.5;
      totalScore += perfScore * weights.perf;
      totalWeight += weights.perf;

      const trustScore = totalScore / totalWeight;
      const tierPolicy = this.tierPolicies[options.tier];
      const passed = trustScore >= 0.8;

      // Apply tier-specific penalties
      let adjustedScore = trustScore;
      if (options.tier <= 2 && !contractResult.passed) {
        adjustedScore *= 0.8; // 20% penalty for missing contracts on high tiers
      }

      return {
        passed,
        score: adjustedScore,
        details: {
          tier: options.tier,
          tier_policy: tierPolicy,
          coverage: coverageResult,
          mutation: mutationResult,
          contracts: contractResult,
          a11y: { score: a11yScore, details: provenance?.results?.a11y },
          perf: { score: perfScore, details: provenance?.results?.perf },
          raw_score: trustScore,
          weights,
        },
      };
    } catch (error) {
      return {
        passed: false,
        score: 0,
        details: { error: `Trust score calculation failed: ${error}` },
        errors: [`Trust score calculation failed: ${error}`],
      };
    }
  }

  /**
   * Get tier policy for a specific tier
   */
  getTierPolicy(tier: number): TierPolicy | null {
    return this.tierPolicies[tier] || null;
  }

  /**
   * Get all available tiers
   */
  getAvailableTiers(): number[] {
    return Object.keys(this.tierPolicies).map(Number);
  }
}
