/**
 * Arbiter-specific CAWS types
 *
 * Extends CAWS types with arbiter orchestration metadata and result enrichment.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import type { CAWSValidationResult } from "../../caws-validator/types/validation-types.js";
import type { WorkingSpec } from "../../types/caws-types.js";

/**
 * Orchestration metadata for arbiter operations
 */
export interface OrchestrationMetadata {
  /** Task ID being validated */
  taskId?: string;
  /** Agent assigned to task */
  assignedAgent?: string;
  /** Timestamp of validation request */
  timestamp: string;
  /** Arbiter version */
  arbiterVersion: string;
  /** Session/context ID */
  sessionId?: string;
}

/**
 * Arbiter-enriched validation result
 *
 * Extends CAWS validation result with orchestration context
 */
export interface ArbiterValidationResult extends CAWSValidationResult {
  /** Orchestration metadata */
  orchestration: OrchestrationMetadata;

  /** Source working spec */
  spec: WorkingSpec;

  /** CAWS CLI version used */
  cawsVersion: string;

  /** Validation duration in milliseconds */
  durationMs: number;
}

/**
 * Policy cache entry
 */
export interface PolicyCacheEntry {
  /** Cached policy */
  policy: any; // Will be properly typed once we analyze CAWS policy structure
  /** Cache timestamp */
  cachedAt: number;
  /** TTL in milliseconds */
  ttl: number;
}

/**
 * Budget derivation request
 */
export interface BudgetDerivationRequest {
  /** Working spec to derive budget for */
  spec: WorkingSpec;
  /** Project root directory */
  projectRoot: string;
  /** Whether to apply waivers */
  applyWaivers?: boolean;
}

/**
 * Budget derivation result
 */
export interface BudgetDerivationResult {
  /** Baseline budget from policy */
  baseline: {
    max_files: number;
    max_loc: number;
  };
  /** Effective budget after waivers */
  effective: {
    max_files: number;
    max_loc: number;
  };
  /** Applied waiver IDs */
  waiversApplied: string[];
  /** Derivation timestamp */
  derivedAt: string;
  /** Policy version used */
  policyVersion: string;
}

/**
 * CAWS validation request
 */
export interface CAWSValidationRequest {
  /** Working spec to validate */
  spec: WorkingSpec;
  /** Project root directory */
  projectRoot: string;
  /** Validation options */
  options?: {
    /** Enable auto-fix for common issues */
    autoFix?: boolean;
    /** Show suggestions for improvements */
    suggestions?: boolean;
    /** Check budget compliance */
    checkBudget?: boolean;
    /** Run quality gates */
    runQualityGates?: boolean;
  };
  /** Orchestration context */
  context?: Partial<OrchestrationMetadata>;
}

/**
 * CAWS adapter configuration
 */
export interface CAWSAdapterConfig {
  /** Project root directory */
  projectRoot: string;
  /** Enable caching */
  enableCaching?: boolean;
  /** Cache TTL in milliseconds */
  cacheTTL?: number;
  /** Use temporary files for validation */
  useTemporaryFiles?: boolean;
  /** Arbiter version string */
  arbiterVersion?: string;
}

/**
 * Adapter operation result
 */
export interface AdapterOperationResult<T = any> {
  /** Operation success status */
  success: boolean;
  /** Result data */
  data?: T;
  /** Error details if failed */
  error?: {
    message: string;
    code?: string;
    details?: any;
  };
  /** Operation duration in milliseconds */
  durationMs: number;
}
