/**
 * CAWS Constitutional Types
 *
 * Types for Constitutional AI Workflow Specification compliance
 * and runtime validation.
 *
 * @author @darianrosebrook
 */

// Re-export commonly used types
export { VerificationPriority } from "./verification";

export enum ConstitutionalPrinciple {
  _TRANSPARENCY = "transparency",
  _ACCOUNTABILITY = "accountability",
  _SAFETY = "safety",
  _FAIRNESS = "fairness",
  _PRIVACY = "privacy",
  _RELIABILITY = "reliability",
}

export enum ViolationSeverity {
  _LOW = "low",
  _MEDIUM = "medium",
  _HIGH = "high",
  _CRITICAL = "critical",
}

export enum RuleOperator {
  _EQUALS = "equals",
  _NOT_EQUALS = "not_equals",
  _CONTAINS = "contains",
  _NOT_CONTAINS = "not_contains",
  _GREATER_THAN = "greater_than",
  _LESS_THAN = "less_than",
  _GREATER_THAN_OR_EQUAL = "greater_than_or_equal",
  _LESS_THAN_OR_EQUAL = "less_than_or_equal",
  _REGEX_MATCH = "regex_match",
  _EXISTS = "exists",
  _NOT_EXISTS = "not_exists",
  _IN = "in",
  _NOT_IN = "not_in",
}

export interface ConstitutionalPolicy {
  id: string;
  principle: ConstitutionalPrinciple;
  name: string;
  description: string;
  rules: PolicyRule[];
  severity: ViolationSeverity;
  autoRemediation?: RemediationAction;
  enabled: boolean;
  metadata?: Record<string, any>;
}

export interface PolicyRule {
  id: string;
  condition: string; // JSONPath expression
  operator: RuleOperator;
  value: any;
  message: string;
  metadata?: Record<string, any>;
}

export interface RemediationAction {
  type: "alert" | "block" | "modify" | "escalate" | "log";
  parameters?: Record<string, any>;
}

export interface OperationContext {
  id: string;
  type: string;
  timestamp: Date;
  agentId?: string;
  userId?: string;
  sessionId?: string;
  payload: any;
  metadata?: Record<string, any>;
}

export interface EvaluationContext {
  agentId?: string;
  userId?: string;
  sessionId?: string;
  environment: string;
  requestId?: string;
  ipAddress?: string;
  userAgent?: string;
}

export interface ConstitutionalViolation {
  id: string;
  policyId: string;
  ruleId: string;
  principle: ConstitutionalPrinciple;
  severity: ViolationSeverity;
  message: string;
  actualValue: any;
  expectedValue: any;
  operationId: string;
  timestamp: Date;
  context: ViolationContext;
  remediation?: RemediationAction;
}

export interface ViolationContext {
  operationType: string;
  agentId?: string;
  userId?: string;
  sessionId?: string;
  environment: string;
  requestId?: string;
}

export interface ComplianceResult {
  operationId: string;
  compliant: boolean;
  violations: ConstitutionalViolation[];
  evaluations: PolicyEvaluation[];
  timestamp: Date;
  duration: number; // milliseconds
  waiverApplied?: boolean;
  waiverId?: string;
}

export interface PolicyEvaluation {
  policyId: string;
  policyName: string;
  principle: ConstitutionalPrinciple;
  compliant: boolean;
  violations: ConstitutionalViolation[];
  evaluationTime: number; // milliseconds
}

export interface WaiverRequest {
  id: string;
  policyId: string;
  operationPattern: string;
  reason: string;
  justification: string;
  requestedBy: string;
  approvedBy?: string;
  expiresAt: Date;
  status: WaiverStatus;
  createdAt: Date;
  updatedAt: Date;
  metadata?: Record<string, any>;
}

export enum WaiverStatus {
  _PENDING = "pending",
  _APPROVED = "approved",
  _REJECTED = "rejected",
  _EXPIRED = "expired",
  _REVOKED = "revoked",
}

export interface WaiverCheckResult {
  hasActiveWaiver: boolean;
  waiver?: WaiverRequest;
  expiresAt?: Date;
  remainingTime?: number; // milliseconds
}

export interface AuditResult {
  operationId: string;
  compliant: boolean;
  violations: ConstitutionalViolation[];
  recommendations: string[];
  score: number; // 0-100 compliance score
  timestamp: Date;
  auditorVersion: string;
}

export interface ConstitutionalConfig {
  enabled: boolean;
  strictMode: boolean;
  auditEnabled: boolean;
  violationResponseTimeout: number; // milliseconds
  waiverApprovalRequired: boolean;
  maxViolationsPerOperation: number;
  cacheEnabled: boolean;
  cacheTTL: number; // milliseconds
}
