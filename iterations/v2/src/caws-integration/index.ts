/**
 * CAWS Integration Layer
 *
 * Exports all CAWS adapters, utilities, and types for arbiter integration.
 *
 * @author @darianrosebrook
 */

// Adapters
export {
  CAWSValidationAdapter,
  createCAWSValidationAdapter,
} from "./adapters/CAWSValidationAdapter.js";

export {
  CAWSPolicyAdapter,
  createCAWSPolicyAdapter,
} from "./adapters/CAWSPolicyAdapter.js";

// Utilities
export {
  SpecFileManager,
  createSpecFileManager,
  type SpecFileManagerConfig,
  type SpecFileWriteResult,
} from "./utils/spec-file-manager.js";

// Types
export type {
  AdapterOperationResult,
  ArbiterValidationResult,
  BudgetDerivationRequest,
  BudgetDerivationResult,
  CAWSAdapterConfig,
  CAWSValidationRequest,
  OrchestrationMetadata,
  PolicyCacheEntry,
} from "./types/arbiter-caws-types.js";
