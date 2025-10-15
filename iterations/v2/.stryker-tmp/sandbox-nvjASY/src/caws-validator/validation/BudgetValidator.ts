/**
 * @fileoverview Budget Validator
 * Derives budgets from policy and validates compliance
 * Adapted from CAWS CLI budget-derivation.js
 * @module caws-validator/validation
 */
// @ts-nocheck


import { WorkingSpec } from "../../types/caws-types";
import type {
  BudgetCompliance,
  BudgetLimits,
  BudgetViolation,
  ChangeStats,
  DerivedBudget,
} from "../types/validation-types";
import { PolicyLoader } from "../utils/policy-loader";
import { WaiverManager } from "../waivers/WaiverManager";

/**
 * Validates budget compliance for CAWS working specs
 * Derives budgets from policy.yaml and applies waivers
 */
export class BudgetValidator {
  constructor(
    private policyLoader: PolicyLoader,
    private waiverManager: WaiverManager
  ) {}

  /**
   * Derive budget from policy and waivers
   */
  public async deriveBudget(
    spec: WorkingSpec,
    projectRoot: string
  ): Promise<DerivedBudget> {
    try {
      // Load policy
      const policy = await this.policyLoader.loadPolicy(projectRoot);

      // Validate risk tier exists in policy
      if (!policy.risk_tiers[spec.risk_tier]) {
        throw new Error(
          `Risk tier ${spec.risk_tier} not defined in policy.yaml`
        );
      }

      // Get baseline budget from policy
      const tierConfig = policy.risk_tiers[spec.risk_tier];
      const baseline: BudgetLimits = {
        max_files: tierConfig.max_files,
        max_loc: tierConfig.max_loc,
      };

      // Start with baseline
      let effectiveBudget: BudgetLimits = { ...baseline };

      // Apply waivers if any
      const waiversApplied: string[] = [];
      if (spec.waiver_ids && spec.waiver_ids.length > 0) {
        effectiveBudget = await this.applyWaivers(
          effectiveBudget,
          spec.waiver_ids,
          projectRoot,
          waiversApplied
        );
      }

      return {
        baseline,
        effective: effectiveBudget,
        waiversApplied,
        derivedAt: new Date().toISOString(),
      };
    } catch (error) {
      throw new Error(`Budget derivation failed: ${(error as Error).message}`);
    }
  }

  /**
   * Check if current changes comply with budget
   */
  public async checkBudgetCompliance(
    derivedBudget: DerivedBudget,
    currentStats: ChangeStats
  ): Promise<BudgetCompliance> {
    const violations: BudgetViolation[] = [];

    // Check file count
    if (currentStats.filesChanged > derivedBudget.effective.max_files) {
      violations.push({
        gate: "budget_limit",
        type: "max_files",
        current: currentStats.filesChanged,
        limit: derivedBudget.effective.max_files,
        baseline: derivedBudget.baseline.max_files,
        message: `File count (${currentStats.filesChanged}) exceeds budget (${derivedBudget.effective.max_files})`,
      });
    }

    // Check lines of code
    if (currentStats.linesChanged > derivedBudget.effective.max_loc) {
      violations.push({
        gate: "budget_limit",
        type: "max_loc",
        current: currentStats.linesChanged,
        limit: derivedBudget.effective.max_loc,
        baseline: derivedBudget.baseline.max_loc,
        message: `Lines of code (${currentStats.linesChanged}) exceeds budget (${derivedBudget.effective.max_loc})`,
      });
    }

    return {
      compliant: violations.length === 0,
      baseline: derivedBudget.baseline,
      effective: derivedBudget.effective,
      current: currentStats,
      violations,
      waiversApplied: derivedBudget.waiversApplied,
    };
  }

  /**
   * Apply waivers to budget
   */
  private async applyWaivers(
    baseline: BudgetLimits,
    waiverIds: string[],
    projectRoot: string,
    waiversApplied: string[]
  ): Promise<BudgetLimits> {
    const effectiveBudget = { ...baseline };

    for (const waiverId of waiverIds) {
      const waiver = await this.waiverManager.loadWaiver(waiverId, projectRoot);

      if (waiver && this.waiverManager.isWaiverValid(waiver)) {
        // Apply additive deltas
        if (waiver.delta) {
          if (waiver.delta.max_files) {
            effectiveBudget.max_files += waiver.delta.max_files;
          }
          if (waiver.delta.max_loc) {
            effectiveBudget.max_loc += waiver.delta.max_loc;
          }
        }

        waiversApplied.push(waiverId);
      } else {
        console.warn(`Waiver ${waiverId} is invalid or expired, skipping`);
      }
    }

    return effectiveBudget;
  }

  /**
   * Generate burn-up report for scope visibility
   */
  public generateBurnupReport(
    budgetCompliance: BudgetCompliance,
    riskTier: number
  ): string {
    const lines: string[] = [];

    lines.push("📊 CAWS Budget Burn-up Report");
    lines.push("===============================");
    lines.push("");
    lines.push(`Risk Tier: ${riskTier}`);
    lines.push(
      `Baseline: ${budgetCompliance.baseline.max_files} files, ${budgetCompliance.baseline.max_loc} LOC`
    );
    lines.push(
      `Current: ${budgetCompliance.current.filesChanged} files, ${budgetCompliance.current.linesChanged} LOC`
    );

    if (budgetCompliance.waiversApplied.length > 0) {
      lines.push("");
      lines.push(
        `Waivers Applied: ${budgetCompliance.waiversApplied.join(", ")}`
      );
      lines.push(
        `Effective Budget: ${budgetCompliance.effective.max_files} files, ${budgetCompliance.effective.max_loc} LOC`
      );
    }

    const filePercent = Math.round(
      (budgetCompliance.current.filesChanged /
        budgetCompliance.effective.max_files) *
        100
    );
    const locPercent = Math.round(
      (budgetCompliance.current.linesChanged /
        budgetCompliance.effective.max_loc) *
        100
    );

    lines.push("");
    lines.push(
      `File Usage: ${filePercent}% (${budgetCompliance.current.filesChanged}/${budgetCompliance.effective.max_files})`
    );
    lines.push(
      `LOC Usage: ${locPercent}% (${budgetCompliance.current.linesChanged}/${budgetCompliance.effective.max_loc})`
    );

    if (filePercent > 90 || locPercent > 90) {
      lines.push("");
      lines.push("⚠️  WARNING: Approaching budget limits");
    }

    if (!budgetCompliance.compliant) {
      lines.push("");
      lines.push("❌ BUDGET EXCEEDED:");
      for (const violation of budgetCompliance.violations) {
        lines.push(`   • ${violation.message}`);
      }
    }

    return lines.join("\n");
  }

  /**
   * Calculate budget utilization percentage
   */
  public calculateUtilization(budgetCompliance: BudgetCompliance): {
    files: number;
    loc: number;
    overall: number;
  } {
    const filesPercent =
      (budgetCompliance.current.filesChanged /
        budgetCompliance.effective.max_files) *
      100;
    const locPercent =
      (budgetCompliance.current.linesChanged /
        budgetCompliance.effective.max_loc) *
      100;
    const overallPercent = Math.max(filesPercent, locPercent);

    return {
      files: Math.round(filesPercent),
      loc: Math.round(locPercent),
      overall: Math.round(overallPercent),
    };
  }

  /**
   * Check if change is approaching budget limit
   */
  public isApproachingLimit(
    budgetCompliance: BudgetCompliance,
    threshold = 90
  ): boolean {
    const utilization = this.calculateUtilization(budgetCompliance);
    return utilization.overall >= threshold;
  }
}
