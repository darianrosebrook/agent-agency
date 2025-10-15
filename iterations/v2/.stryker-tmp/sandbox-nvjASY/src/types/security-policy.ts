/**
 * @fileoverview Security Policy Types - ARBITER-013
 *
 * Type definitions for security policies, policy evaluation, and security enforcement.
 * This file provides the core interfaces for the Security Policy Enforcer component.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


/**
 * Security policy violation severity levels
 */
export enum ViolationSeverity {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
  CRITICAL = "critical",
}

/**
 * Policy action types
 */
export enum PolicyAction {
  ALLOW = "allow",
  DENY = "deny",
  LOG = "log",
  QUARANTINE = "quarantine",
  ESCALATE = "escalate",
}

/**
 * Security policy rule definition
 */
export interface PolicyRule {
  /** Unique rule identifier */
  id: string;

  /** Rule condition (e.g., "user.role == 'admin'") */
  condition: string;

  /** Action to take when condition is met */
  action: PolicyAction;

  /** Rule priority (higher numbers = higher priority) */
  priority?: number;

  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Security policy definition
 */
export interface SecurityPolicy {
  /** Unique policy identifier */
  id: string;

  /** Human-readable policy name */
  name: string;

  /** Policy description */
  description?: string;

  /** Policy rules */
  rules: PolicyRule[];

  /** Default action when no rules match */
  defaultAction: PolicyAction;

  /** Policy severity level */
  severity: ViolationSeverity;

  /** Whether policy is enabled */
  enabled: boolean;

  /** Policy version */
  version: string;

  /** Creation timestamp */
  createdAt: Date;

  /** Last update timestamp */
  updatedAt: Date;

  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Policy evaluation result
 */
export interface PolicyDecision {
  /** Whether the action is allowed */
  allowed: boolean;

  /** Reason for the decision */
  reason?: string;

  /** Severity of the decision */
  severity: ViolationSeverity;

  /** Applied policy ID */
  policyId?: string;

  /** Applied rule ID */
  ruleId?: string;

  /** Additional metadata */
  metadata?: Record<string, any>;

  /** Timestamp of decision */
  timestamp: Date;
}

/**
 * Policy validation result
 */
export interface ValidationResult {
  /** Whether validation passed */
  valid: boolean;

  /** Validation errors */
  errors: string[];

  /** Validation warnings */
  warnings: string[];

  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Security context for policy evaluation
 */
export interface SecurityContext {
  /** Agent identifier */
  agentId: string;

  /** User identifier */
  userId: string;

  /** Tenant identifier for multi-tenancy */
  tenantId: string;

  /** Session identifier */
  sessionId: string;

  /** Granted permissions */
  permissions: string[];

  /** User roles */
  roles: string[];

  /** Security level */
  securityLevel: SecurityLevel;

  /** Authentication timestamp */
  authenticatedAt: Date;

  /** Session expiry */
  expiresAt: Date;

  /** Request metadata */
  metadata: SecurityMetadata;

  /** IP address */
  ipAddress: string;

  /** User agent */
  userAgent: string;
}

/**
 * Security levels
 */
export enum SecurityLevel {
  PUBLIC = "public",
  INTERNAL = "internal",
  CONFIDENTIAL = "confidential",
  RESTRICTED = "restricted",
  AGENT = "agent",
  TRUSTED_AGENT = "trusted_agent",
  ADMIN = "admin",
}

/**
 * Security metadata
 */
export interface SecurityMetadata {
  /** IP address */
  ipAddress?: string;

  /** User agent */
  userAgent?: string;

  /** Request source */
  source: "api" | "internal" | "test";

  /** Additional context */
  context?: Record<string, any>;
}

/**
 * Policy engine interface
 */
export interface PolicyEngine {
  /** Evaluate policy for given context and action */
  evaluatePolicy(
    context: SecurityContext,
    action: string,
    resource: string
  ): Promise<PolicyDecision>;

  /** Validate a security policy */
  validatePolicy(policy: SecurityPolicy): ValidationResult;

  /** Update a security policy */
  updatePolicy(policy: SecurityPolicy): Promise<void>;

  /** Get all active policies */
  getActivePolicies(): Promise<SecurityPolicy[]>;

  /** Get policy by ID */
  getPolicy(policyId: string): Promise<SecurityPolicy | null>;

  /** Delete a policy */
  deletePolicy(policyId: string): Promise<void>;
}

/**
 * Command validator interface
 */
export interface CommandValidator {
  /** Validate a command string */
  validateCommand(command: string, context: SecurityContext): ValidationResult;

  /** Sanitize a command string */
  sanitizeCommand(command: string): string;

  /** Check if command is dangerous */
  isDangerousCommand(command: string): boolean;

  /** Get allowed commands for context */
  getAllowedCommands(context: SecurityContext): string[];
}

/**
 * Security auditor interface
 */
export interface SecurityAuditor {
  /** Log a security event */
  logSecurityEvent(event: SecurityEvent): void;

  /** Log a policy violation */
  logPolicyViolation(violation: PolicyViolation): void;

  /** Get audit trail */
  getAuditTrail(filter: AuditFilter): Promise<AuditEntry[]>;

  /** Get security events */
  getSecurityEvents(filter: SecurityEventFilter): Promise<SecurityEvent[]>;
}

/**
 * Security event types
 */
export enum SecurityEventType {
  AUTH_SUCCESS = "auth_success",
  AUTH_FAILURE = "auth_failure",
  AUTHZ_SUCCESS = "authz_success",
  AUTHZ_FAILURE = "authz_failure",
  POLICY_VIOLATION = "policy_violation",
  SUSPICIOUS_ACTIVITY = "suspicious_activity",
  SESSION_EXPIRED = "session_expired",
  RATE_LIMIT_EXCEEDED = "rate_limit_exceeded",
  COMMAND_BLOCKED = "command_blocked",
  INPUT_VALIDATION_FAILED = "input_validation_failed",
}

/**
 * Security event
 */
export interface SecurityEvent {
  /** Event ID */
  id: string;

  /** Event type */
  type: SecurityEventType;

  /** Event timestamp */
  timestamp: Date;

  /** Security context */
  context?: SecurityContext;

  /** Resource being accessed */
  resource?: string;

  /** Action being performed */
  action?: string;

  /** Event result */
  result: "success" | "failure" | "blocked";

  /** Event details */
  details?: Record<string, any>;

  /** Event severity */
  severity: ViolationSeverity;

  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Policy violation
 */
export interface PolicyViolation {
  /** Violation ID */
  id: string;

  /** Policy ID that was violated */
  policyId: string;

  /** Rule ID that was violated */
  ruleId?: string;

  /** Security context */
  context: SecurityContext;

  /** Resource being accessed */
  resource: string;

  /** Action being performed */
  action: string;

  /** Violation timestamp */
  timestamp: Date;

  /** Violation details */
  details: Record<string, any>;

  /** Violation severity */
  severity: ViolationSeverity;

  /** Whether violation was blocked */
  blocked: boolean;
}

/**
 * Audit filter
 */
export interface AuditFilter {
  /** Start date */
  startDate?: Date;

  /** End date */
  endDate?: Date;

  /** Event types to include */
  eventTypes?: SecurityEventType[];

  /** Agent IDs to include */
  agentIds?: string[];

  /** User IDs to include */
  userIds?: string[];

  /** Tenant IDs to include */
  tenantIds?: string[];

  /** Severity levels to include */
  severities?: ViolationSeverity[];

  /** Limit number of results */
  limit?: number;

  /** Offset for pagination */
  offset?: number;
}

/**
 * Security event filter
 */
export interface SecurityEventFilter extends AuditFilter {
  /** Result types to include */
  results?: ("success" | "failure" | "blocked")[];
}

/**
 * Audit entry
 */
export interface AuditEntry {
  /** Entry ID */
  id: string;

  /** Event type */
  type: SecurityEventType;

  /** Timestamp */
  timestamp: Date;

  /** Agent ID */
  agentId?: string;

  /** User ID */
  userId?: string;

  /** Tenant ID */
  tenantId?: string;

  /** Resource */
  resource?: string;

  /** Action */
  action?: string;

  /** Result */
  result: "success" | "failure" | "blocked";

  /** Severity */
  severity: ViolationSeverity;

  /** Details */
  details: Record<string, any>;
}

/**
 * Threat detection interface
 */
export interface ThreatDetector {
  /** Detect threats in input */
  detectThreats(input: string, context: SecurityContext): ThreatDetectionResult;

  /** Get threat patterns */
  getThreatPatterns(): ThreatPattern[];

  /** Update threat patterns */
  updateThreatPatterns(patterns: ThreatPattern[]): void;
}

/**
 * Threat detection result
 */
export interface ThreatDetectionResult {
  /** Whether threats were detected */
  threatsDetected: boolean;

  /** Detected threats */
  threats: DetectedThreat[];

  /** Risk score (0-100) */
  riskScore: number;

  /** Recommendation */
  recommendation: "allow" | "block" | "quarantine" | "escalate";
}

/**
 * Detected threat
 */
export interface DetectedThreat {
  /** Threat type */
  type: string;

  /** Threat description */
  description: string;

  /** Threat severity */
  severity: ViolationSeverity;

  /** Threat pattern that matched */
  pattern: string;

  /** Confidence score (0-100) */
  confidence: number;
}

/**
 * Threat pattern
 */
export interface ThreatPattern {
  /** Pattern ID */
  id: string;

  /** Pattern name */
  name: string;

  /** Pattern regex */
  pattern: string;

  /** Threat type */
  type: string;

  /** Pattern severity */
  severity: ViolationSeverity;

  /** Whether pattern is enabled */
  enabled: boolean;
}

/**
 * Access control interface
 */
export interface AccessControl {
  /** Check if access is allowed */
  checkAccess(
    context: SecurityContext,
    resource: string,
    action: string
  ): Promise<boolean>;

  /** Grant access */
  grantAccess(
    context: SecurityContext,
    resource: string,
    action: string,
    duration?: number
  ): Promise<void>;

  /** Revoke access */
  revokeAccess(
    context: SecurityContext,
    resource: string,
    action: string
  ): Promise<void>;

  /** Get permissions for context */
  getPermissions(context: SecurityContext): Promise<string[]>;
}

/**
 * Security configuration
 */
export interface SecurityConfig {
  /** Whether security is enabled */
  enabled: boolean;

  /** Default security level */
  defaultSecurityLevel: SecurityLevel;

  /** Session timeout in milliseconds */
  sessionTimeoutMs: number;

  /** Maximum sessions per agent */
  maxSessionsPerAgent: number;

  /** Rate limiting configuration */
  rateLimits: Record<string, RateLimitConfig>;

  /** Trusted agent IDs */
  trustedAgents: string[];

  /** Admin agent IDs */
  adminAgents: string[];

  /** Whether audit logging is enabled */
  auditLogging: boolean;

  /** Security policies */
  policies: SecurityPolicyConfig;
}

/**
 * Rate limit configuration
 */
export interface RateLimitConfig {
  /** Requests per window */
  requestsPerWindow: number;

  /** Window size in milliseconds */
  windowMs: number;

  /** Block duration in milliseconds */
  blockDurationMs: number;
}

/**
 * Security policy configuration
 */
export interface SecurityPolicyConfig {
  /** Maximum task description length */
  maxTaskDescriptionLength: number;

  /** Maximum metadata size in bytes */
  maxMetadataSize: number;

  /** Allowed task types */
  allowedTaskTypes: Record<string, boolean>;

  /** Suspicious patterns to detect */
  suspiciousPatterns: RegExp[];
}

/**
 * Security error
 */
export class SecurityError extends Error {
  constructor(message: string, public code: string, public details?: any) {
    super(message);
    this.name = "SecurityError";
  }
}

/**
 * Policy validation error
 */
export class PolicyValidationError extends SecurityError {
  constructor(
    message: string,
    public policyId: string,
    public validationErrors: string[]
  ) {
    super(message, "POLICY_VALIDATION_ERROR", { policyId, validationErrors });
    this.name = "PolicyValidationError";
  }
}

/**
 * Access denied error
 */
export class AccessDeniedError extends SecurityError {
  constructor(
    message: string,
    public resource: string,
    public action: string,
    public context: SecurityContext
  ) {
    super(message, "ACCESS_DENIED", { resource, action, context });
    this.name = "AccessDeniedError";
  }
}
