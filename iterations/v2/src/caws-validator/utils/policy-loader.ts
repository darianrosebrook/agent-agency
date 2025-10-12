/**
 * @fileoverview CAWS Policy Loader
 * Loads and validates policy.yaml configuration
 * @module caws-validator/utils
 */

import * as fs from "fs/promises";
import * as yaml from "js-yaml";
import * as path from "path";
import { CAWSPolicy } from "../../types/caws-types";

/**
 * Loads CAWS policy configuration from policy.yaml
 */
export class PolicyLoader {
  /**
   * Load policy from project root
   */
  public async loadPolicy(projectRoot: string): Promise<CAWSPolicy> {
    const policyPath = path.join(projectRoot, ".caws", "policy.yaml");

    try {
      const content = await fs.readFile(policyPath, "utf-8");
      const policy = yaml.load(content) as CAWSPolicy;

      // Validate policy structure
      this.validatePolicy(policy);

      return policy;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") {
        throw new Error(
          `Policy file not found: ${policyPath}\nRun 'caws init' to create default policy`
        );
      }
      throw new Error(`Failed to load policy: ${(error as Error).message}`);
    }
  }

  /**
   * Validate policy structure
   */
  private validatePolicy(policy: CAWSPolicy): void {
    if (!policy.version) {
      throw new Error("Policy missing version field");
    }

    if (!policy.risk_tiers) {
      throw new Error("Policy missing risk_tiers configuration");
    }

    // Validate each tier
    for (const tier of [1, 2, 3]) {
      if (!policy.risk_tiers[tier]) {
        throw new Error(`Policy missing configuration for risk tier ${tier}`);
      }

      const tierConfig = policy.risk_tiers[tier];
      if (!tierConfig.max_files || tierConfig.max_files <= 0) {
        throw new Error(`Invalid max_files for tier ${tier}`);
      }

      if (!tierConfig.max_loc || tierConfig.max_loc <= 0) {
        throw new Error(`Invalid max_loc for tier ${tier}`);
      }
    }
  }

  /**
   * Get default policy (fallback)
   */
  public getDefaultPolicy(): CAWSPolicy {
    return {
      version: "3.1.0",
      risk_tiers: {
        1: {
          max_files: 25,
          max_loc: 1000,
          coverage_threshold: 90,
          mutation_threshold: 70,
          contracts_required: true,
          manual_review_required: true,
        },
        2: {
          max_files: 50,
          max_loc: 2000,
          coverage_threshold: 80,
          mutation_threshold: 50,
          contracts_required: true,
          manual_review_required: false,
        },
        3: {
          max_files: 100,
          max_loc: 5000,
          coverage_threshold: 70,
          mutation_threshold: 30,
          contracts_required: false,
          manual_review_required: false,
        },
      },
      waiver_approval: {
        required_approvers: 1,
        max_duration_days: 90,
      },
    };
  }
}
