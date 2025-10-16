/**
 * Memory System Types
 *
 * Types and interfaces for the memory management system.
 *
 * @author @darianrosebrook
 */

export interface ContextualMemory {
  memoryId: string;
  relevanceScore: number;
  contextMatch: {
    similarityScore: number;
    keywordMatches: string[];
    semanticMatches: string[];
    temporalAlignment: number;
  };
  content: {
    taskType: string;
    outcome: string;
    lessons: string[];
  };
  tenantId?: string;
  timestamp?: Date;
}

export interface FederatedInsights {
  insights: ContextualMemory[];
  confidence: number;
  sourceTenants: string[];
  aggregationMethod: "weighted" | "consensus" | "hybrid";
  privacyPreserved: boolean;
}

export interface TaskContext {
  type: string;
  description?: string;
  metadata?: Record<string, any>;
}

export interface TenantConfig {
  isolationLevel: "strict" | "shared" | "federated";
  maxMemorySize: number;
  allowFederation: boolean;
  auditLogging?: boolean;
}

export interface TenantAccessResult {
  allowed: boolean;
  reason: string;
  restrictions?: string[];
}

export interface TenantIsolator {
  validateTenantAccess(
    _tenantId: string,
    _operation: string,
    _resource: string
  ): Promise<TenantAccessResult>;
}
