/**
 * CAWS Policy Adapter
 *
 * Handles policy loading, caching, budget derivation, and waiver management.
 * Provides efficient access to CAWS governance rules with minimal overhead.
 *
 * @author @darianrosebrook
 */

import * as fs from "fs/promises";
import * as yaml from "js-yaml";
import * as path from "path";
import type { CAWSPolicy } from "../../types/caws-types.js";
import type {
  AdapterOperationResult,
  BudgetDerivationRequest,
  BudgetDerivationResult,
  CAWSAdapterConfig,
  PolicyCacheEntry,
} from "../types/arbiter-caws-types.js";

/**
 * Waiver document structure (from CAWS)
 */
interface WaiverDocument {
  id: string;
  status: "active" | "expired" | "revoked";
  gates: string[];
  expires_at: string;
  delta?: {
    max_files?: number;
    max_loc?: number;
  };
}

/**
 * Adapter for CAWS policy operations
 *
 * Manages policy.yaml loading, caching, and budget derivation with waivers.
 */
export class CAWSPolicyAdapter {
  private readonly projectRoot: string;
  private readonly enableCaching: boolean;
  private readonly cacheTTL: number;
  private policyCache: PolicyCacheEntry | null = null;

  constructor(config: CAWSAdapterConfig) {
    this.projectRoot = config.projectRoot;
    this.enableCaching = config.enableCaching ?? true;
    this.cacheTTL = config.cacheTTL ?? 300000; // 5 minutes default
  }

  /**
   * Load CAWS policy from policy.yaml
   *
   * Uses cache if enabled and valid.
   *
   * @returns CAWS policy object
   */
  public async loadPolicy(): Promise<AdapterOperationResult<CAWSPolicy>> {
    const startTime = Date.now();

    try {
      // Check cache first
      if (this.enableCaching && this.policyCache) {
        const cacheAge = Date.now() - this.policyCache.cachedAt;
        if (cacheAge < this.cacheTTL) {
          return {
            success: true,
            data: this.policyCache.policy,
            durationMs: Date.now() - startTime,
          };
        }
      }

      // Load from file
      const policyPath = path.join(this.projectRoot, ".caws", "policy.yaml");

      try {
        const content = await fs.readFile(policyPath, "utf-8");
        const policy = yaml.load(content) as CAWSPolicy;

        // Validate policy structure
        this.validatePolicy(policy);

        // Update cache
        if (this.enableCaching) {
          this.policyCache = {
            policy,
            cachedAt: Date.now(),
            ttl: this.cacheTTL,
          };
        }

        return {
          success: true,
          data: policy,
          durationMs: Date.now() - startTime,
        };
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code === "ENOENT") {
          // Policy file doesn't exist - use default
          const defaultPolicy = this.getDefaultPolicy();

          if (this.enableCaching) {
            this.policyCache = {
              policy: defaultPolicy,
              cachedAt: Date.now(),
              ttl: this.cacheTTL,
            };
          }

          return {
            success: true,
            data: defaultPolicy,
            durationMs: Date.now() - startTime,
          };
        }
        throw error;
      }
    } catch (error) {
      return {
        success: false,
        error: {
          message: error instanceof Error ? error.message : "Unknown error",
          code: "POLICY_LOAD_ERROR",
          details: error,
        },
        durationMs: Date.now() - startTime,
      };
    }
  }

  /**
   * Derive budget from policy for a working spec
   *
   * Applies risk tier budgets and any active waivers.
   *
   * @param request Budget derivation request
   * @returns Derived budget with baseline and effective limits
   */
  public async deriveBudget(
    request: BudgetDerivationRequest
  ): Promise<AdapterOperationResult<BudgetDerivationResult>> {
    const startTime = Date.now();

    try {
      // Load policy
      const policyResult = await this.loadPolicy();
      if (!policyResult.success || !policyResult.data) {
        return {
          success: false,
          error: {
            message: "Failed to load policy",
            code: "POLICY_ERROR",
          },
          durationMs: Date.now() - startTime,
        };
      }

      const policy = policyResult.data;

      // Get baseline budget for risk tier
      const tierBudget = policy.risk_tiers[request.spec.risk_tier];
      if (!tierBudget) {
        return {
          success: false,
          error: {
            message: `Risk tier ${request.spec.risk_tier} not defined in policy`,
            code: "INVALID_RISK_TIER",
          },
          durationMs: Date.now() - startTime,
        };
      }

      const baseline = {
        max_files: tierBudget.max_files,
        max_loc: tierBudget.max_loc,
      };

      // Start with baseline
      const effective = { ...baseline };
      const waiversApplied: string[] = [];

      // Apply waivers if requested
      if (request.applyWaivers && request.spec.waiver_ids?.length) {
        for (const waiverId of request.spec.waiver_ids) {
          const waiver = await this.loadWaiver(waiverId);
          if (waiver && this.isWaiverValid(waiver)) {
            // Apply waiver delta
            if (waiver.delta) {
              effective.max_files += waiver.delta.max_files ?? 0;
              effective.max_loc += waiver.delta.max_loc ?? 0;
            }
            waiversApplied.push(waiverId);
          }
        }
      }

      const result: BudgetDerivationResult = {
        baseline,
        effective,
        waiversApplied,
        derivedAt: new Date().toISOString(),
        policyVersion: policy.version,
      };

      return {
        success: true,
        data: result,
        durationMs: Date.now() - startTime,
      };
    } catch (error) {
      return {
        success: false,
        error: {
          message: error instanceof Error ? error.message : "Unknown error",
          code: "BUDGET_DERIVATION_ERROR",
          details: error,
        },
        durationMs: Date.now() - startTime,
      };
    }
  }

  /**
   * Load a waiver document by ID
   *
   * @param waiverId Waiver ID (e.g., WV-0001)
   * @returns Waiver document or null if not found
   */
  private async loadWaiver(waiverId: string): Promise<WaiverDocument | null> {
    try {
      const waiverPath = path.join(
        this.projectRoot,
        ".caws",
        "waivers",
        `${waiverId}.yaml`
      );

      const content = await fs.readFile(waiverPath, "utf-8");
      return yaml.load(content) as WaiverDocument;
    } catch {
      return null;
    }
  }

  /**
   * Check if a waiver is valid (active and not expired)
   *
   * @param waiver Waiver document
   * @returns True if waiver is valid
   */
  private isWaiverValid(waiver: WaiverDocument): boolean {
    if (waiver.status !== "active") {
      return false;
    }

    const expiryDate = new Date(waiver.expires_at);
    return expiryDate > new Date();
  }

  /**
   * Validate policy structure
   *
   * @param policy Policy to validate
   * @throws Error if policy is invalid
   */
  private validatePolicy(policy: CAWSPolicy): void {
    if (!policy.version) {
      throw new Error("Policy missing version field");
    }

    if (!policy.risk_tiers) {
      throw new Error("Policy missing risk_tiers configuration");
    }

    // Validate all tiers have required fields
    for (const tier of [1, 2, 3]) {
      const budget = policy.risk_tiers[tier];
      if (!budget) {
        throw new Error(`Policy missing risk tier ${tier} configuration`);
      }

      if (
        typeof budget.max_files !== "number" ||
        typeof budget.max_loc !== "number"
      ) {
        throw new Error(`Risk tier ${tier} missing or invalid budget limits`);
      }
    }
  }

  /**
   * Get default CAWS policy
   *
   * Returns sensible defaults when policy.yaml doesn't exist.
   *
   * @returns Default policy
   */
  private getDefaultPolicy(): CAWSPolicy {
    return {
      version: "3.1.0",
      risk_tiers: {
        1: {
          max_files: 10,
          max_loc: 250,
          coverage_threshold: 0.9,
          mutation_threshold: 0.7,
          contracts_required: true,
          manual_review_required: true,
        },
        2: {
          max_files: 100,
          max_loc: 10000,
          coverage_threshold: 0.8,
          mutation_threshold: 0.5,
          contracts_required: true,
          manual_review_required: false,
        },
        3: {
          max_files: 500,
          max_loc: 40000,
          coverage_threshold: 0.7,
          mutation_threshold: 0.3,
          contracts_required: false,
          manual_review_required: false,
        },
      },
      edit_rules: {
        policy_and_code_same_pr: false,
        min_approvers_for_budget_raise: 2,
        require_signed_commits: true,
      },
    };
  }

  /**
   * Clear policy cache
   */
  public clearCache(): void {
    this.policyCache = null;
  }

  /**
   * Get cache status
   *
   * @returns Cache information
   */
  public getCacheStatus(): {
    cached: boolean;
    age?: number;
    ttl: number;
  } {
    if (!this.policyCache) {
      return {
        cached: false,
        ttl: this.cacheTTL,
      };
    }

    return {
      cached: true,
      age: Date.now() - this.policyCache.cachedAt,
      ttl: this.cacheTTL,
    };
  }

  /**
   * Reload policy from disk (bypassing cache)
   *
   * @returns Fresh policy
   */
  public async reloadPolicy(): Promise<AdapterOperationResult<CAWSPolicy>> {
    this.clearCache();
    return this.loadPolicy();
  }
}

/**
 * Create a CAWSPolicyAdapter instance
 *
 * @param projectRoot Project root directory
 * @param options Additional configuration options
 * @returns CAWSPolicyAdapter instance
 */
export function createCAWSPolicyAdapter(
  projectRoot: string,
  options?: Partial<CAWSAdapterConfig>
): CAWSPolicyAdapter {
  return new CAWSPolicyAdapter({
    projectRoot,
    ...options,
  });
}
