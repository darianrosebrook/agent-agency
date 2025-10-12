/**
 * Constitutional Policy Engine
 *
 * Evaluates operations against CAWS constitutional policies.
 * Detects violations and determines compliance status.
 *
 * @author @darianrosebrook
 */

import {
  ConstitutionalPolicy,
  PolicyRule,
  RuleOperator,
  ConstitutionalPrinciple,
  ViolationSeverity,
  OperationContext,
  EvaluationContext,
  ConstitutionalViolation,
  PolicyEvaluation,
  ComplianceResult,
} from "../types/caws-constitutional";

export class ConstitutionalPolicyEngine {
  private policies: Map<string, ConstitutionalPolicy> = new Map();

  /**
   * Register a constitutional policy
   */
  registerPolicy(policy: ConstitutionalPolicy): void {
    this.policies.set(policy.id, policy);
  }

  /**
   * Unregister a policy
   */
  unregisterPolicy(policyId: string): boolean {
    return this.policies.delete(policyId);
  }

  /**
   * Get all registered policies
   */
  getPolicies(): ConstitutionalPolicy[] {
    return Array.from(this.policies.values());
  }

  /**
   * Get policy by ID
   */
  getPolicy(policyId: string): ConstitutionalPolicy | undefined {
    return this.policies.get(policyId);
  }

  /**
   * Evaluate operation compliance against all policies
   */
  async evaluateCompliance(
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<ComplianceResult> {
    const startTime = Date.now();
    const violations: ConstitutionalViolation[] = [];
    const evaluations: PolicyEvaluation[] = [];

    // Evaluate against each enabled policy
    for (const policy of this.policies.values()) {
      if (!policy.enabled) continue;

      const evaluationStart = Date.now();
      const evaluation = await this.evaluatePolicy(policy, operation, context);
      const evaluationTime = Date.now() - evaluationStart;

      evaluations.push({
        ...evaluation,
        evaluationTime,
      });

      if (!evaluation.compliant) {
        violations.push(...evaluation.violations);
      }
    }

    const duration = Date.now() - startTime;

    return {
      operationId: operation.id,
      compliant: violations.length === 0,
      violations,
      evaluations,
      timestamp: new Date(),
      duration,
    };
  }

  /**
   * Evaluate operation against a single policy
   */
  private async evaluatePolicy(
    policy: ConstitutionalPolicy,
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<Omit<PolicyEvaluation, "evaluationTime">> {
    const violations: ConstitutionalViolation[] = [];

    for (const rule of policy.rules) {
      const violation = this.evaluateRule(rule, operation, context, policy);
      if (violation) {
        violations.push(violation);
      }
    }

    return {
      policyId: policy.id,
      policyName: policy.name,
      principle: policy.principle,
      compliant: violations.length === 0,
      violations,
    };
  }

  /**
   * Evaluate a single rule
   */
  private evaluateRule(
    rule: PolicyRule,
    operation: OperationContext,
    context: EvaluationContext,
    policy: ConstitutionalPolicy
  ): ConstitutionalViolation | null {
    try {
      // Extract value from operation using JSONPath-like expression
      const actualValue = this.extractValue(rule.condition, {
        operation,
        context,
      });

      // Evaluate the condition
      const compliant = this.evaluateCondition(
        rule.operator,
        actualValue,
        rule.value
      );

      if (!compliant) {
        return {
          id: `violation-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          policyId: policy.id,
          ruleId: rule.id,
          principle: policy.principle,
          severity: policy.severity,
          message: rule.message,
          actualValue,
          expectedValue: rule.value,
          operationId: operation.id,
          timestamp: new Date(),
          context: {
            operationType: operation.type,
            agentId: context.agentId,
            userId: context.userId,
            sessionId: context.sessionId,
            environment: context.environment,
            requestId: context.requestId,
          },
          remediation: policy.autoRemediation,
        };
      }

      return null;
    } catch (error) {
      // If rule evaluation fails, treat as violation
      return {
        id: `violation-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        policyId: policy.id,
        ruleId: rule.id,
        principle: policy.principle,
        severity: ViolationSeverity.MEDIUM,
        message: `Rule evaluation failed: ${error instanceof Error ? error.message : "Unknown error"}`,
        actualValue: null,
        expectedValue: rule.value,
        operationId: operation.id,
        timestamp: new Date(),
        context: {
          operationType: operation.type,
          agentId: context.agentId,
          userId: context.userId,
          sessionId: context.sessionId,
          environment: context.environment,
          requestId: context.requestId,
        },
      };
    }
  }

  /**
   * Extract value from object using JSONPath-like expression
   */
  private extractValue(path: string, root: any): any {
    if (!path) return root;

    const parts = path.split('.');
    let current = root;

    for (const part of parts) {
      if (current && typeof current === 'object') {
        // Handle array access like "items[0]"
        const arrayMatch = part.match(/^([^[\]]+)\[(\d+)\]$/);
        if (arrayMatch) {
          const [, prop, index] = arrayMatch;
          current = current[prop];
          if (Array.isArray(current)) {
            current = current[parseInt(index)];
          } else {
            return undefined;
          }
        } else {
          current = current[part];
        }
      } else {
        return undefined;
      }
    }

    return current;
  }

  /**
   * Evaluate condition based on operator
   */
  private evaluateCondition(
    operator: RuleOperator,
    actualValue: any,
    expectedValue: any
  ): boolean {
    switch (operator) {
      case RuleOperator.EQUALS:
        return actualValue === expectedValue;

      case RuleOperator.NOT_EQUALS:
        return actualValue !== expectedValue;

      case RuleOperator.CONTAINS:
        if (Array.isArray(actualValue)) {
          return actualValue.includes(expectedValue);
        }
        if (typeof actualValue === 'string') {
          return actualValue.includes(String(expectedValue));
        }
        return false;

      case RuleOperator.NOT_CONTAINS:
        if (Array.isArray(actualValue)) {
          return !actualValue.includes(expectedValue);
        }
        if (typeof actualValue === 'string') {
          return !actualValue.includes(String(expectedValue));
        }
        return true;

      case RuleOperator.GREATER_THAN:
        return Number(actualValue) > Number(expectedValue);

      case RuleOperator.LESS_THAN:
        return Number(actualValue) < Number(expectedValue);

      case RuleOperator.GREATER_THAN_OR_EQUAL:
        return Number(actualValue) >= Number(expectedValue);

      case RuleOperator.LESS_THAN_OR_EQUAL:
        return Number(actualValue) <= Number(expectedValue);

      case RuleOperator.EXISTS:
        return actualValue !== undefined && actualValue !== null;

      case RuleOperator.NOT_EXISTS:
        return actualValue === undefined || actualValue === null;

      case RuleOperator.IN:
        if (Array.isArray(expectedValue)) {
          return expectedValue.includes(actualValue);
        }
        return false;

      case RuleOperator.NOT_IN:
        if (Array.isArray(expectedValue)) {
          return !expectedValue.includes(actualValue);
        }
        return true;

      case RuleOperator.REGEX_MATCH:
        if (typeof actualValue === 'string' && typeof expectedValue === 'string') {
          return new RegExp(expectedValue).test(actualValue);
        }
        return false;

      default:
        return false;
    }
  }

  /**
   * Clear all policies
   */
  clearPolicies(): void {
    this.policies.clear();
  }

  /**
   * Enable/disable policy
   */
  setPolicyEnabled(policyId: string, enabled: boolean): boolean {
    const policy = this.policies.get(policyId);
    if (policy) {
      policy.enabled = enabled;
      return true;
    }
    return false;
  }

  /**
   * Get policies by principle
   */
  getPoliciesByPrinciple(principle: ConstitutionalPrinciple): ConstitutionalPolicy[] {
    return Array.from(this.policies.values()).filter(
      (policy) => policy.principle === principle
    );
  }

  /**
   * Get policies by severity
   */
  getPoliciesBySeverity(severity: ViolationSeverity): ConstitutionalPolicy[] {
    return Array.from(this.policies.values()).filter(
      (policy) => policy.severity === severity
    );
  }
}
