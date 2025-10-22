/**
 * Central Feature Flag Service
 *
 * Extracted from CAWS feature flag system to provide centralized
 * feature flag management across the agent agency system.
 *
 * @author @darianrosebrook
 */

import type {
  FeatureFlag,
  FeatureFlagEvaluation,
  FeatureContext,
  FeatureFlagUpdate,
} from "../types/index.js";

export class FeatureFlagService {
  private flags: Map<string, FeatureFlag> = new Map();

  /**
   * Register a new feature flag
   */
  registerFlag(flag: FeatureFlag): void {
    this.flags.set(flag.name, { ...flag });
  }

  /**
   * Update an existing feature flag
   */
  updateFlag(name: string, updates: FeatureFlagUpdate): void {
    const existing = this.flags.get(name);
    if (!existing) {
      throw new Error(`Feature flag '${name}' not found`);
    }

    this.flags.set(name, {
      ...existing,
      ...updates,
      updatedAt: new Date().toISOString(),
    });
  }

  /**
   * Remove a feature flag
   */
  removeFlag(name: string): void {
    this.flags.delete(name);
  }

  /**
   * Get a feature flag by name
   */
  getFlag(name: string): FeatureFlag | undefined {
    return this.flags.get(name);
  }

  /**
   * Get all feature flags
   */
  getAllFlags(): FeatureFlag[] {
    return Array.from(this.flags.values());
  }

  /**
   * Evaluate if a feature flag is enabled for a given context
   */
  evaluateFlag(name: string, context: FeatureContext): FeatureFlagEvaluation {
    const flag = this.flags.get(name);

    if (!flag) {
      return {
        enabled: false,
        flag: {
          name,
          description: "Flag not found",
          enabled: false,
          rolloutPercentage: 0,
          environment: [],
          userGroups: [],
          dependencies: [],
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
        reason: "Flag not found",
      };
    }

    // Check if flag is globally disabled
    if (flag.killSwitch) {
      return {
        enabled: false,
        flag,
        reason: "Kill switch activated",
      };
    }

    // Check environment
    if (
      flag.environment.length > 0 &&
      !flag.environment.includes(context.environment)
    ) {
      return {
        enabled: false,
        flag,
        reason: `Not enabled for environment: ${context.environment}`,
      };
    }

    // Check user groups
    if (flag.userGroups.length > 0) {
      const userGroups = context.userGroups || [];
      const hasMatchingGroup = flag.userGroups.some((group) =>
        userGroups.includes(group)
      );

      if (!hasMatchingGroup) {
        return {
          enabled: false,
          flag,
          reason: "User not in allowed groups",
        };
      }
    }

    // Check rollout percentage
    if (flag.rolloutPercentage < 100) {
      const userHash = this.hashUser(
        context.userId || context.requestId || "anonymous"
      );
      const rolloutBucket = (userHash % 100) + 1;

      if (rolloutBucket > flag.rolloutPercentage) {
        return {
          enabled: false,
          flag,
          reason: `Outside rollout percentage (${flag.rolloutPercentage}%)`,
        };
      }
    }

    // Check dependencies
    for (const dependency of flag.dependencies) {
      const depEvaluation = this.evaluateFlag(dependency, context);
      if (!depEvaluation.enabled) {
        return {
          enabled: false,
          flag,
          reason: `Dependency '${dependency}' not enabled: ${depEvaluation.reason}`,
        };
      }
    }

    return {
      enabled: true,
      flag,
      reason: "All conditions met",
    };
  }

  /**
   * Evaluate multiple flags for a context
   */
  evaluateFlags(
    names: string[],
    context: FeatureContext
  ): Map<string, FeatureFlagEvaluation> {
    const results = new Map<string, FeatureFlagEvaluation>();

    for (const name of names) {
      results.set(name, this.evaluateFlag(name, context));
    }

    return results;
  }

  /**
   * Get all enabled flags for a context
   */
  getEnabledFlags(context: FeatureContext): FeatureFlag[] {
    const enabledFlags: FeatureFlag[] = [];

    for (const flag of this.flags.values()) {
      const evaluation = this.evaluateFlag(flag.name, context);
      if (evaluation.enabled) {
        enabledFlags.push(flag);
      }
    }

    return enabledFlags;
  }

  /**
   * Check if a feature is enabled (convenience method)
   */
  isEnabled(name: string, context: FeatureContext): boolean {
    return this.evaluateFlag(name, context).enabled;
  }

  /**
   * Load flags from configuration
   */
  loadFromConfig(flags: FeatureFlag[]): void {
    for (const flag of flags) {
      this.registerFlag(flag);
    }
  }

  /**
   * Export flags to configuration format
   */
  exportToConfig(): FeatureFlag[] {
    return this.getAllFlags();
  }

  /**
   * Get flag statistics
   */
  getStats(): {
    total: number;
    enabled: number;
    disabled: number;
    withRollout: number;
    withDependencies: number;
  } {
    const flags = this.getAllFlags();

    return {
      total: flags.length,
      enabled: flags.filter((f) => f.enabled).length,
      disabled: flags.filter((f) => !f.enabled).length,
      withRollout: flags.filter((f) => f.rolloutPercentage < 100).length,
      withDependencies: flags.filter((f) => f.dependencies.length > 0).length,
    };
  }

  /**
   * Simple hash function for user rollout bucketing
   */
  private hashUser(userId: string): number {
    let hash = 0;
    for (let i = 0; i < userId.length; i++) {
      const char = userId.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash);
  }
}
