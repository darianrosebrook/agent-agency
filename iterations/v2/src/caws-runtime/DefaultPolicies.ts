/**
 * Default CAWS Constitutional Policies
 *
 * Pre-defined policies that implement the six CAWS constitutional principles.
 * These policies ensure compliance with transparency, accountability, safety,
 * fairness, privacy, and reliability requirements.
 *
 * @author @darianrosebrook
 */

import {
  ConstitutionalPolicy,
  ConstitutionalPrinciple,
  ViolationSeverity,
  RuleOperator,
} from "../types/caws-constitutional";

/**
 * Transparency Policies
 * Ensure all operations are auditable and explainable
 */
export const transparencyPolicies: ConstitutionalPolicy[] = [
  {
    id: "transparency-operation-audit",
    principle: ConstitutionalPrinciple.TRANSPARENCY,
    name: "Operation Audit Trail",
    description: "All operations must maintain a complete audit trail",
    severity: ViolationSeverity.MEDIUM,
    enabled: true,
    rules: [
      {
        id: "operation-has-id",
        condition: "operation.id",
        operator: RuleOperator.EXISTS,
        value: true,
        message: "Operations must have unique identifiers",
      },
      {
        id: "operation-has-timestamp",
        condition: "operation.timestamp",
        operator: RuleOperator.EXISTS,
        value: true,
        message: "Operations must have timestamps",
      },
    ],
  },
  {
    id: "transparency-agent-attribution",
    principle: ConstitutionalPrinciple.TRANSPARENCY,
    name: "Agent Attribution",
    description: "Operations performed by agents must be attributable",
    severity: ViolationSeverity.HIGH,
    enabled: true,
    rules: [
      {
        id: "agent-operations-have-agent-id",
        condition: "operation.agentId",
        operator: RuleOperator.EXISTS,
        value: true,
        message: "Agent operations must include agent identifier",
      },
    ],
  },
];

/**
 * Accountability Policies
 * Ensure all operations are attributable to responsible parties
 */
export const accountabilityPolicies: ConstitutionalPolicy[] = [
  {
    id: "accountability-user-attribution",
    principle: ConstitutionalPrinciple.ACCOUNTABILITY,
    name: "User Attribution",
    description: "Operations must be attributable to users",
    severity: ViolationSeverity.HIGH,
    enabled: true,
    rules: [
      {
        id: "user-operations-have-user-id",
        condition: "operation.userId",
        operator: RuleOperator.EXISTS,
        value: true,
        message: "User operations must include user identifier",
      },
    ],
  },
  {
    id: "accountability-session-tracking",
    principle: ConstitutionalPrinciple.ACCOUNTABILITY,
    name: "Session Tracking",
    description: "Operations must be associated with user sessions",
    severity: ViolationSeverity.MEDIUM,
    enabled: true,
    rules: [
      {
        id: "operations-have-session-id",
        condition: "operation.sessionId",
        operator: RuleOperator.EXISTS,
        value: true,
        message: "Operations should include session identifiers",
      },
    ],
  },
];

/**
 * Safety Policies
 * Prevent operations that could compromise system integrity
 */
export const safetyPolicies: ConstitutionalPolicy[] = [
  {
    id: "safety-no-dangerous-operations",
    principle: ConstitutionalPrinciple.SAFETY,
    name: "Dangerous Operations Prevention",
    description: "Prevent execution of inherently dangerous operations",
    severity: ViolationSeverity.CRITICAL,
    enabled: true,
    rules: [
      {
        id: "no-system-delete-operations",
        condition: "operation.type",
        operator: RuleOperator.NOT_EQUALS,
        value: "system_delete",
        message: "System deletion operations are not allowed",
      },
      {
        id: "no-system-restart-operations",
        condition: "operation.type",
        operator: RuleOperator.NOT_EQUALS,
        value: "system_restart",
        message: "System restart operations require special approval",
      },
    ],
    autoRemediation: {
      type: "block",
    },
  },
  {
    id: "safety-resource-limits",
    principle: ConstitutionalPrinciple.SAFETY,
    name: "Resource Usage Limits",
    description: "Operations must respect resource usage limits",
    severity: ViolationSeverity.HIGH,
    enabled: true,
    rules: [
      {
        id: "no-excessive-memory-usage",
        condition: "operation.payload.memoryUsage",
        operator: RuleOperator.LESS_THAN,
        value: 1000000000, // 1GB
        message: "Operations cannot request excessive memory usage",
      },
    ],
  },
];

/**
 * Fairness Policies
 * Ensure operations do not discriminate or bias against protected classes
 */
export const fairnessPolicies: ConstitutionalPolicy[] = [
  {
    id: "fairness-no-discriminatory-content",
    principle: ConstitutionalPrinciple.FAIRNESS,
    name: "Content Fairness",
    description: "Operations must not contain discriminatory content",
    severity: ViolationSeverity.HIGH,
    enabled: true,
    rules: [
      {
        id: "no-discriminatory-language",
        condition: "operation.payload.content",
        operator: RuleOperator.NOT_CONTAINS,
        value: ["hate", "discriminatory", "biased"],
        message: "Operations cannot contain discriminatory content",
      },
    ],
  },
  {
    id: "fairness-equal-treatment",
    principle: ConstitutionalPrinciple.FAIRNESS,
    name: "Equal Treatment",
    description: "Operations must treat all users equally",
    severity: ViolationSeverity.MEDIUM,
    enabled: true,
    rules: [
      {
        id: "no-user-discrimination",
        condition: "context.userId",
        operator: RuleOperator.EXISTS,
        value: true,
        message: "Operations must identify users for fairness auditing",
      },
    ],
  },
];

/**
 * Privacy Policies
 * Ensure data handling complies with privacy regulations
 */
export const privacyPolicies: ConstitutionalPolicy[] = [
  {
    id: "privacy-no-pii-logging",
    principle: ConstitutionalPrinciple.PRIVACY,
    name: "PII Protection",
    description: "Personally identifiable information must be protected",
    severity: ViolationSeverity.CRITICAL,
    enabled: true,
    rules: [
      {
        id: "no-ssn-in-payload",
        condition: "operation.payload",
        operator: RuleOperator.NOT_CONTAINS,
        value: /\d{3}-\d{2}-\d{4}/, // SSN pattern
        message: "Social Security Numbers cannot be included in operation payloads",
      },
      {
        id: "no-email-in-logs",
        condition: "operation.payload.email",
        operator: RuleOperator.NOT_EXISTS,
        value: true,
        message: "Email addresses should not be logged in operation payloads",
      },
    ],
  },
  {
    id: "privacy-data-retention",
    principle: ConstitutionalPrinciple.PRIVACY,
    name: "Data Retention Limits",
    description: "Personal data retention must comply with regulations",
    severity: ViolationSeverity.HIGH,
    enabled: true,
    rules: [
      {
        id: "reasonable-retention-period",
        condition: "operation.payload.retentionDays",
        operator: RuleOperator.LESS_THAN_OR_EQUAL,
        value: 2555, // 7 years
        message: "Data retention periods must comply with regulations",
      },
    ],
  },
];

/**
 * Reliability Policies
 * Ensure system maintains operational reliability standards
 */
export const reliabilityPolicies: ConstitutionalPolicy[] = [
  {
    id: "reliability-operation-timeouts",
    principle: ConstitutionalPrinciple.RELIABILITY,
    name: "Operation Timeouts",
    description: "Operations must complete within reasonable time limits",
    severity: ViolationSeverity.MEDIUM,
    enabled: true,
    rules: [
      {
        id: "reasonable-timeout-limits",
        condition: "operation.metadata.timeoutMs",
        operator: RuleOperator.LESS_THAN,
        value: 300000, // 5 minutes
        message: "Operation timeouts must be reasonable",
      },
    ],
  },
  {
    id: "reliability-error-handling",
    principle: ConstitutionalPrinciple.RELIABILITY,
    name: "Error Handling Requirements",
    description: "Operations must include proper error handling",
    severity: ViolationSeverity.MEDIUM,
    enabled: true,
    rules: [
      {
        id: "operations-have-retry-logic",
        condition: "operation.metadata.retryEnabled",
        operator: RuleOperator.EQUALS,
        value: true,
        message: "Critical operations should include retry logic",
      },
    ],
  },
];

/**
 * All Default CAWS Policies
 */
export const defaultCawsPolicies: ConstitutionalPolicy[] = [
  ...transparencyPolicies,
  ...accountabilityPolicies,
  ...safetyPolicies,
  ...fairnessPolicies,
  ...privacyPolicies,
  ...reliabilityPolicies,
];

/**
 * Load default policies into policy engine
 */
export function loadDefaultPolicies(
  policyEngine: { registerPolicy: (_policy: ConstitutionalPolicy) => void }
): void {
  for (const policy of defaultCawsPolicies) {
    policyEngine.registerPolicy(policy);
  }
}

/**
 * Get policies by principle
 */
export function getPoliciesByPrinciple(
  principle: ConstitutionalPrinciple
): ConstitutionalPolicy[] {
  return defaultCawsPolicies.filter((policy) => policy.principle === principle);
}

/**
 * Get policies by severity
 */
export function getPoliciesBySeverity(
  severity: ViolationSeverity
): ConstitutionalPolicy[] {
  return defaultCawsPolicies.filter((policy) => policy.severity === severity);
}
