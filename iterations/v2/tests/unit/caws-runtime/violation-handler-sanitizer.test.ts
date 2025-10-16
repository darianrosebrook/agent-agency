/**
 * @fileoverview
 * Unit tests for ViolationHandler sanitizer functionality
 *
 * @author @darianrosebrook
 */

import {
  AlertManager,
  AuditLogger,
  ViolationHandler,
} from "@/caws-runtime/ViolationHandler";
import {
  ConstitutionalPrinciple,
  ConstitutionalViolation,
  EvaluationContext,
  OperationContext,
  ViolationContext,
  ViolationSeverity,
} from "@/types/caws-constitutional";

// Mock AlertManager and AuditLogger
class MockAlertManager implements AlertManager {
  constructor(config: any) {}

  async sendAlert(alert: any): Promise<void> {
    return Promise.resolve();
  }

  async escalateViolation(
    violation: ConstitutionalViolation,
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<void> {
    return Promise.resolve();
  }
}

class MockAuditLogger implements AuditLogger {
  constructor(config: any) {}

  async logViolation(
    violation: ConstitutionalViolation,
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<void> {
    return Promise.resolve();
  }
}

describe("ViolationHandler Sanitizer", () => {
  let violationHandler: ViolationHandler;
  let mockAlertManager: MockAlertManager;
  let mockAuditLogger: MockAuditLogger;

  beforeEach(() => {
    mockAlertManager = new MockAlertManager({
      enabled: true,
      channels: ["email", "slack"],
      recipients: ["admin@example.com"],
    });

    mockAuditLogger = new MockAuditLogger({
      enabled: true,
      logLevel: "info",
      retentionDays: 30,
    });

    violationHandler = new ViolationHandler(mockAlertManager, mockAuditLogger, {
      alertEnabled: true,
      blockEnabled: true,
      logEnabled: true,
      escalationEnabled: true,
      timeoutMs: 5000,
    });
  });

  describe("parameter sanitization", () => {
    it("should remove dangerous parameters", () => {
      const payload = {
        eval: "malicious code",
        exec: "system command",
        shell: "bash script",
        command: "rm -rf /",
        script: "alert('xss')",
        safeParam: "safe value",
      };

      // Access the private method through type assertion
      const sanitized = (violationHandler as any).sanitizeParameters(payload);

      expect(sanitized.eval).toBeUndefined();
      expect(sanitized.exec).toBeUndefined();
      expect(sanitized.shell).toBeUndefined();
      expect(sanitized.command).toBeUndefined();
      expect(sanitized.script).toBeUndefined();
      expect(sanitized.safeParam).toBe("safe value");
    });

    it("should sanitize string values", () => {
      const payload = {
        userInput: "<script>alert('xss')</script>",
        sqlQuery: "'; DROP TABLE users; --",
        safeText: "normal text",
      };

      const sanitized = (violationHandler as any).sanitizeParameters(payload);

      expect(sanitized.userInput).toBeDefined();
      expect(sanitized.userInput).not.toContain("<script>");
      expect(sanitized.sqlQuery).toBeDefined();
      expect(sanitized.sqlQuery).not.toContain("DROP TABLE");
      expect(sanitized.safeText).toBe("normal text");
    });

    it("should handle nested objects", () => {
      const payload = {
        config: {
          eval: "dangerous",
          safe: "value",
          nested: {
            exec: "command",
            normal: "text",
          },
        },
        topLevel: "safe",
      };

      const sanitized = (violationHandler as any).sanitizeParameters(payload);

      expect(sanitized.config.eval).toBeUndefined();
      expect(sanitized.config.safe).toBe("value");
      expect(sanitized.config.nested.exec).toBeUndefined();
      expect(sanitized.config.nested.normal).toBe("text");
      expect(sanitized.topLevel).toBe("safe");
    });

    it("should handle arrays", () => {
      const payload = {
        commands: ["safe command", "eval('dangerous')", "normal text"],
        configs: [{ eval: "bad", safe: "good" }, { normal: "value" }],
      };

      const sanitized = (violationHandler as any).sanitizeParameters(payload);

      expect(sanitized.commands).toBeInstanceOf(Array);
      expect(sanitized.commands[0]).toBe("safe command");
      expect(sanitized.commands[1]).not.toContain("eval");
      expect(sanitized.commands[2]).toBe("normal text");

      expect(sanitized.configs[0].eval).toBeUndefined();
      expect(sanitized.configs[0].safe).toBe("good");
      expect(sanitized.configs[1].normal).toBe("value");
    });

    it("should handle null and undefined values", () => {
      const payload = {
        nullValue: null,
        undefinedValue: undefined,
        emptyString: "",
        normalValue: "test",
      };

      const sanitized = (violationHandler as any).sanitizeParameters(payload);

      expect(sanitized.nullValue).toBeNull();
      expect(sanitized.undefinedValue).toBeUndefined();
      expect(sanitized.emptyString).toBe("");
      expect(sanitized.normalValue).toBe("test");
    });

    it("should handle non-object payloads", () => {
      expect((violationHandler as any).sanitizeParameters(null)).toBeNull();
      expect(
        (violationHandler as any).sanitizeParameters(undefined)
      ).toBeUndefined();
      expect((violationHandler as any).sanitizeParameters("string")).toBe(
        "string"
      );
      expect((violationHandler as any).sanitizeParameters(123)).toBe(123);
      expect((violationHandler as any).sanitizeParameters(true)).toBe(true);
    });
  });

  describe("string sanitization", () => {
    it("should remove script tags", () => {
      const input = "Hello <script>alert('xss')</script> world";
      const sanitized = (violationHandler as any).sanitizeString(input);

      expect(sanitized).not.toContain("<script>");
      expect(sanitized).not.toContain("alert");
      expect(sanitized).toContain("Hello");
      expect(sanitized).toContain("world");
    });

    it("should remove SQL injection patterns", () => {
      const input = "SELECT * FROM users; DROP TABLE users; --";
      const sanitized = (violationHandler as any).sanitizeString(input);

      expect(sanitized).not.toContain("DROP TABLE");
      expect(sanitized).not.toContain("--");
      expect(sanitized).toContain("SELECT");
    });

    it("should remove command injection patterns", () => {
      const input = "ls -la; rm -rf /; cat /etc/passwd";
      const sanitized = (violationHandler as any).sanitizeString(input);

      expect(sanitized).not.toContain("rm -rf");
      expect(sanitized).not.toContain("/etc/passwd");
      expect(sanitized).toContain("ls -la");
    });

    it("should preserve safe content", () => {
      const input =
        "This is a normal, safe string with numbers 123 and symbols @#$";
      const sanitized = (violationHandler as any).sanitizeString(input);

      expect(sanitized).toBe(input);
    });

    it("should handle empty strings", () => {
      const sanitized = (violationHandler as any).sanitizeString("");
      expect(sanitized).toBe("");
    });

    it("should handle very long strings", () => {
      const longString = "a".repeat(10000);
      const sanitized = (violationHandler as any).sanitizeString(longString);

      expect(sanitized).toBe(longString);
    });
  });

  describe("operation modification", () => {
    it("should apply safety modifications", () => {
      const payload = {
        dangerousAction: "delete_all_data",
        safeAction: "read_data",
        permissions: "admin",
      };

      const modified = (violationHandler as any).applySafetyModifications(
        payload
      );

      expect(modified.dangerousAction).toBeUndefined();
      expect(modified.safeAction).toBe("read_data");
      expect(modified.permissions).toBe("readonly");
    });

    it("should apply privacy modifications", () => {
      const payload = {
        personalData: "John Doe, john@example.com, 123-456-7890",
        publicData: "Public information",
        sensitiveInfo: "SSN: 123-45-6789",
      };

      const modified = (violationHandler as any).applyPrivacyModifications(
        payload
      );

      expect(modified.personalData).not.toContain("john@example.com");
      expect(modified.personalData).not.toContain("123-456-7890");
      expect(modified.publicData).toBe("Public information");
      expect(modified.sensitiveInfo).not.toContain("123-45-6789");
    });

    it("should apply reliability modifications", () => {
      const payload = {
        timeout: 0,
        retries: 100,
        batchSize: 1000000,
        normalParam: "value",
      };

      const modified = (violationHandler as any).applyReliabilityModifications(
        payload
      );

      expect(modified.timeout).toBeGreaterThan(0);
      expect(modified.retries).toBeLessThanOrEqual(10);
      expect(modified.batchSize).toBeLessThanOrEqual(1000);
      expect(modified.normalParam).toBe("value");
    });
  });

  describe("integration with violation handling", () => {
    it("should modify operations for safety violations", async () => {
      const violation: ConstitutionalViolation = {
        id: "violation-1",
        policyId: "policy-1",
        ruleId: "rule-1",
        principle: ConstitutionalPrinciple.SAFETY,
        severity: ViolationSeverity.HIGH,
        message: "Dangerous operation detected",
        actualValue: "delete_all_data",
        expectedValue: "read_only",
        operationId: "op-1",
        context: {
          operationType: "data_operation",
          environment: "production",
        } as ViolationContext,
        timestamp: new Date(),
      };

      const operation: OperationContext = {
        id: "op-1",
        type: "data_operation",
        payload: {
          action: "delete_all_data",
          target: "users",
        },
        metadata: {},
        timestamp: new Date(),
      };

      const context: EvaluationContext = {
        agentId: "agent-1",
        userId: "user-1",
        sessionId: "session-1",
        environment: "production",
        requestId: "req-1",
        ipAddress: "127.0.0.1",
        userAgent: "test-agent",
      };

      // Mock the audit logger to avoid actual logging
      jest.spyOn(mockAuditLogger, "logViolation").mockResolvedValue(undefined);

      await (violationHandler as any).modifyOperation(
        violation,
        operation,
        context
      );

      expect(mockAuditLogger.logViolation).toHaveBeenCalled();
    });

    it("should emit modification events", async () => {
      const violation: ConstitutionalViolation = {
        id: "violation-2",
        policyId: "policy-2",
        ruleId: "rule-2",
        principle: ConstitutionalPrinciple.PRIVACY,
        severity: ViolationSeverity.MEDIUM,
        message: "Privacy violation detected",
        actualValue: "personal_data",
        expectedValue: "anonymized_data",
        operationId: "op-2",
        context: {
          operationType: "data_access",
          environment: "production",
        } as ViolationContext,
        timestamp: new Date(),
      };

      const operation: OperationContext = {
        id: "op-2",
        type: "data_access",
        payload: {
          personalData: "John Doe, john@example.com",
        },
        metadata: {},
        timestamp: new Date(),
      };

      const context: EvaluationContext = {
        agentId: "agent-2",
        userId: "user-2",
        sessionId: "session-2",
        environment: "production",
        requestId: "req-2",
        ipAddress: "127.0.0.1",
        userAgent: "test-agent",
      };

      // Mock the audit logger
      jest.spyOn(mockAuditLogger, "logViolation").mockResolvedValue(undefined);

      // Listen for modification events
      const eventSpy = jest.fn();
      violationHandler.on("operation_modified", eventSpy);

      await (violationHandler as any).modifyOperation(
        violation,
        operation,
        context
      );

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          violationId: "violation-2",
          originalOperation: operation,
          modifiedOperation: expect.any(Object),
          timestamp: expect.any(String),
        })
      );
    });
  });
});
