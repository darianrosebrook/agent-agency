/**
 * Worker/Agent Type Definitions
 * 
 * Matches backend Worker model from iterations/v3/data-infrastructure/src/models.rs
 * 
 * @author @darianrosebrook
 */

/**
 * Worker capabilities structure
 * 
 * Backend stores as JSONB with flexible structure. This interface represents
 * the common fields that may be present.
 */
export interface WorkerCapabilities {
  supported_models?: string[];
  max_context_length?: number;
  features?: string[];
  rate_limits?: {
    requests_per_minute?: number;
    tokens_per_minute?: number;
  };
  [key: string]: unknown; // Allow additional fields
}

/**
 * Worker/Agent interface matching backend Worker model
 * 
 * All fields match backend exactly:
 * - created_at and updated_at: Required timestamps
 * - model_name and endpoint: Required (not nullable)
 * - capabilities: JSONB (structured object, not string array)
 */
export interface Worker {
  id: string;
  name: string;
  worker_type: string;
  specialty: string | null;
  model_name: string; // Required (backend: NOT NULL)
  endpoint: string; // Required (backend: NOT NULL)
  capabilities: WorkerCapabilities | null; // JSONB (structured object)
  performance_history: Record<string, unknown>; // JSONB
  is_active: boolean;
  created_at: string; // RFC3339, required
  updated_at: string; // RFC3339, required
}








