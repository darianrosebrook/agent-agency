/**
 * Constitutional Runtime Tests
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import {
  ConstitutionalRuntime,
  ConstitutionalPolicyEngine,
  ViolationHandler,
  WaiverManager,
} from "../../../src/caws-runtime";
import {
  ConstitutionalPrinciple,
  ViolationSeverity,
  WaiverStatus,
  RuleOperator,
} from "../../../src/types/caws-constitutional";
import { TracingProvider } from "../../../src/observability/TracingProvider";

describe("ConstitutionalRuntime", () => {
  let runtime: ConstitutionalRuntime;
  let policyEngine: jest.Mocked<ConstitutionalPolicyEngine>;
  let violationHandler: jest.Mocked<ViolationHandler>;
  let waiverManager: jest.Mocked<WaiverManager>;
  let tracing: jest.Mocked<TracingProvider>;

  beforeEach(() => {
    policyEngine = {
      evaluateCompliance: jest.fn(),
      registerPolicy: jest.fn(),
      getPolicies: jest.fn(),
      on: jest.fn(),
    } as any;

    violationHandler = {
      handleViolations: jest.fn(),
      on: jest.fn(),
    } as any;

    waiverManager = {
      checkWaiver: jest.fn(),
      requestWaiver: jest.fn(),
      approveWaiver: jest.fn(),
      rejectWaiver: jest.fn(),
      getWaivers: jest.fn(),
      on: jest.fn(),
    } as any;

    tracing = {
      traceOperation: jest.fn(),
      on: jest.fn(),
    } as any;

    tracing.traceOperation.mockImplementation(async (name, fn) => fn());

    runtime = new ConstitutionalRuntime(
      policyEngine,
      violationHandler,
      waiverManager,
      tracing
    );
  });

  describe("initialization", () => {
    it("should initialize with default config", () => {
      expect(runtime.isEnabled()).toBe(true);
    });

    it("should setup event handlers", () => {
      expect(violationHandler.on).toHaveBeenCalledWith(
        "violation:handled",
        expect.any(Function)
      );
      expect(waiverManager.on).toHaveBeenCalledWith(
        "waiver:created",
        expect.any(Function)
      );
    });
  });

  describe("operation validation", () => {
    const validOperation = {
      id: "op-1",
      type: "task_submit",
      timestamp: new Date(),
      agentId: "agent-1",
      userId: "user-1",
      payload: { task: "test" },
    };

    const evaluationContext = {
      agentId: "agent-1",
      userId: "user-1",
      environment: "test",
    };

    it("should pass compliant operations", async () => {
      const complianceResult = {
        operationId: "op-1",
        compliant: true,
        violations: [],
        evaluations: [],
        timestamp: new Date(),
        duration: 10,
      };

      policyEngine.evaluateCompliance.mockResolvedValue(complianceResult);
      waiverManager.checkWaiver.mockResolvedValue({ hasActiveWaiver: false });

      const result = await runtime.validateOperation(validOperation, evaluationContext);

      expect(result.compliant).toBe(true);
      expect(result.violations).toHaveLength(0);
      expect(policyEngine.evaluateCompliance).toHaveBeenCalledWith(
        validOperation,
        evaluationContext
      );
    });

    it("should handle violations", async () => {
      const violation = {
        id: "v-1",
        policyId: "p-1",
        principle: ConstitutionalPrinciple.SAFETY,
        severity: ViolationSeverity.HIGH,
        message: "Safety violation",
        actualValue: "dangerous",
        expectedValue: "safe",
        operationId: "op-1",
        timestamp: new Date(),
        context: {
          operationType: "task_submit",
          agentId: "agent-1",
          userId: "user-1",
          environment: "test",
        },
      };

      const complianceResult = {
        operationId: "op-1",
        compliant: false,
        violations: [violation],
        evaluations: [],
        timestamp: new Date(),
        duration: 15,
      };

      policyEngine.evaluateCompliance.mockResolvedValue(complianceResult);
      waiverManager.checkWaiver.mockResolvedValue({ hasActiveWaiver: false });
      violationHandler.handleViolations.mockResolvedValue([]);

      const result = await runtime.validateOperation(validOperation, evaluationContext);

      expect(result.compliant).toBe(false);
      expect(result.violations).toHaveLength(1);
      expect(violationHandler.handleViolations).toHaveBeenCalledWith(
        [violation],
        validOperation,
        evaluationContext,
        5000
      );
    });

    it("should honor active waivers", async () => {
      const waiver = {
        id: "w-1",
        policyId: "p-1",
        operationPattern: "task_submit",
        reason: "Testing",
        justification: "For tests",
        requestedBy: "test-user",
        expiresAt: new Date(Date.now() + 3600000),
        status: WaiverStatus.APPROVED,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      waiverManager.checkWaiver.mockResolvedValue({
        hasActiveWaiver: true,
        waiver,
        expiresAt: waiver.expiresAt,
      });

      const result = await runtime.validateOperation(validOperation, evaluationContext);

      expect(result.compliant).toBe(true);
      expect(result.waiverApplied).toBe(true);
      expect(result.waiverId).toBe(waiver.id);
      expect(policyEngine.evaluateCompliance).not.toHaveBeenCalled();
    });

    it("should trace operations", async () => {
      const complianceResult = {
        operationId: "op-1",
        compliant: true,
        violations: [],
        evaluations: [],
        timestamp: new Date(),
        duration: 10,
      };

      policyEngine.evaluateCompliance.mockResolvedValue(complianceResult);
      waiverManager.checkWaiver.mockResolvedValue({ hasActiveWaiver: false });

      await runtime.validateOperation(validOperation, evaluationContext);

      expect(tracing.traceOperation).toHaveBeenCalledWith(
        "constitutional:validateOperation",
        expect.any(Function)
      );
    });

    it("should be disabled when configured", async () => {
      runtime.updateConfig({ enabled: false });

      const result = await runtime.validateOperation(validOperation, evaluationContext);

      expect(result.compliant).toBe(true);
      expect(result.duration).toBe(0);
      expect(policyEngine.evaluateCompliance).not.toHaveBeenCalled();
    });
  });

  describe("operation auditing", () => {
    const operation = {
      id: "op-1",
      type: "task_complete",
      timestamp: new Date(),
      agentId: "agent-1",
      payload: { result: "success" },
    };

    const result = { success: true, data: "output" };
    const context = {
      agentId: "agent-1",
      environment: "test",
    };

    it("should audit completed operations", async () => {
      const complianceResult = {
        operationId: "op-1",
        compliant: true,
        violations: [],
        evaluations: [],
        timestamp: new Date(),
        duration: 5,
      };

      policyEngine.evaluateCompliance.mockResolvedValue(complianceResult);

      const audit = await runtime.auditOperation(operation, result, context);

      expect(audit.operationId).toBe("op-1");
      expect(audit.compliant).toBe(true);
      expect(audit.score).toBe(100);
      expect(audit.violations).toHaveLength(0);
      expect(audit.recommendations).toEqual(["Operation is fully compliant with CAWS principles."]);
    });

    it("should calculate compliance scores", async () => {
      const violations = [
        {
          id: "v-1",
          policyId: "p-1",
          principle: ConstitutionalPrinciple.SAFETY,
          severity: ViolationSeverity.HIGH,
          message: "Safety violation",
          actualValue: "unsafe",
          expectedValue: "safe",
          operationId: "op-1",
          timestamp: new Date(),
          context: { operationType: "task_complete" },
        },
      ];

      const complianceResult = {
        operationId: "op-1",
        compliant: false,
        violations,
        evaluations: [],
        timestamp: new Date(),
        duration: 5,
      };

      policyEngine.evaluateCompliance.mockResolvedValue(complianceResult);

      const audit = await runtime.auditOperation(operation, result, context);

      expect(audit.compliant).toBe(false);
      expect(audit.score).toBeLessThan(100);
      expect(audit.violations).toHaveLength(1);
      expect(audit.recommendations).toContain("Consider requesting a waiver for exceptional circumstances.");
    });

    it("should skip auditing when disabled", async () => {
      runtime.updateConfig({ auditEnabled: false });

      const audit = await runtime.auditOperation(operation, result, context);

      expect(audit.operationId).toBe("op-1");
      expect(audit.compliant).toBe(true);
      expect(audit.score).toBe(100);
      expect(policyEngine.evaluateCompliance).not.toHaveBeenCalled();
    });
  });

  describe("waiver management", () => {
    it("should request waivers", async () => {
      const waiverId = "w-1";
      waiverManager.requestWaiver.mockResolvedValue(waiverId);

      const result = await runtime.requestWaiver(
        "policy-1",
        "task_*",
        "Testing waiver",
        "For automated tests",
        "test-user",
        new Date(Date.now() + 3600000)
      );

      expect(result).toBe(waiverId);
      expect(waiverManager.requestWaiver).toHaveBeenCalledWith({
        policyId: "policy-1",
        operationPattern: "task_*",
        reason: "Testing waiver",
        justification: "For automated tests",
        requestedBy: "test-user",
        expiresAt: expect.any(Date),
      });
    });

    it("should approve waivers", async () => {
      waiverManager.approveWaiver.mockResolvedValue(undefined);

      await runtime.approveWaiver("w-1", "admin");

      expect(waiverManager.approveWaiver).toHaveBeenCalledWith("w-1", "admin");
    });

    it("should reject waivers", async () => {
      waiverManager.rejectWaiver.mockResolvedValue(undefined);

      await runtime.rejectWaiver("w-1", "admin", "Not justified");

      expect(waiverManager.rejectWaiver).toHaveBeenCalledWith("w-1", "admin", "Not justified");
    });
  });

  describe("statistics", () => {
    it("should return runtime statistics", () => {
      policyEngine.getPolicies.mockReturnValue([]);
      waiverManager.getWaivers.mockReturnValue([]);

      const stats = runtime.getStats();

      expect(stats).toEqual({
        policies: 0,
        waivers: {},
      });

      expect(policyEngine.getPolicies).toHaveBeenCalled();
      expect(waiverManager.getWaivers).toHaveBeenCalled();
    });
  });

  describe("configuration", () => {
    it("should update configuration", () => {
      runtime.updateConfig({ enabled: false, strictMode: true });

      expect(runtime.isEnabled()).toBe(false);
    });

    it("should enable/disable validation", () => {
      runtime.setEnabled(false);
      expect(runtime.isEnabled()).toBe(false);

      runtime.setEnabled(true);
      expect(runtime.isEnabled()).toBe(true);
    });
  });
});

