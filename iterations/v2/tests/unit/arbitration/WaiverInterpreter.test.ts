/**
 * Unit tests for WaiverInterpreter
 *
 * Tests waiver evaluation, approval/denial logic, time-bounded waivers,
 * and expiration handling.
 */

import { WaiverInterpreter } from "@/arbitration/WaiverInterpreter";
import {
  ConstitutionalRule,
  RuleCategory,
  ViolationSeverity,
  WaiverRequest,
  WaiverStatus,
} from "@/types/arbitration";

describe("WaiverInterpreter", () => {
  let interpreter: WaiverInterpreter;

  beforeEach(() => {
    interpreter = new WaiverInterpreter();
  });

  // Helper to create a test rule
  const createRule = (
    overrides?: Partial<ConstitutionalRule>
  ): ConstitutionalRule => {
    return {
      id: "RULE-001",
      version: "1.0.0",
      category: RuleCategory.CODE_QUALITY,
      title: "Code must be linted",
      description: "All code must pass linting",
      condition: "linted === true",
      severity: ViolationSeverity.MODERATE,
      waivable: true,
      requiredEvidence: ["linter_report"],
      precedents: [],
      effectiveDate: new Date(),
      metadata: {},
      ...overrides,
    };
  };

  // Helper to create a test waiver request
  const createRequest = (overrides?: Partial<WaiverRequest>): WaiverRequest => {
    return {
      id: "waiver-1",
      ruleId: "RULE-001",
      requestedBy: "agent-1",
      justification: "Emergency fix required for production issue",
      evidence: ["evidence-1", "evidence-2"],
      requestedDuration: 24 * 60 * 60 * 1000, // 24 hours
      requestedAt: new Date(),
      context: {},
      ...overrides,
    };
  };

  describe("evaluateWaiver", () => {
    it("should approve waiver with adequate justification and evidence", async () => {
      const rule = createRule();
      const request = createRequest();

      const result = await interpreter.evaluateWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(result.shouldApprove).toBe(true);
      expect(result.reasoning).toContain("Approved");
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it("should reject waiver for non-waivable rule", async () => {
      const rule = createRule({ waivable: false });
      const request = createRequest();

      const result = await interpreter.evaluateWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(result.shouldApprove).toBe(false);
      expect(result.reasoning).toContain("not waivable");
      expect(result.confidence).toBe(1.0);
    });

    it("should reject waiver with insufficient justification", async () => {
      const rule = createRule();
      const request = createRequest({ justification: "too short" });

      const result = await interpreter.evaluateWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(result.shouldApprove).toBe(false);
      expect(result.reasoning).toContain("Insufficient justification");
    });

    it("should create conditional approval with insufficient evidence", async () => {
      const customInterpreter = new WaiverInterpreter({
        allowConditionalWaivers: true,
        minEvidenceForApproval: 3,
      });

      const rule = createRule();
      const request = createRequest({
        evidence: ["evidence-1"], // Only 1 piece
      });

      const result = await customInterpreter.evaluateWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(result.shouldApprove).toBe(true);
      expect(result.reasoning).toContain("Conditional approval");
      expect(result.conditions).toBeDefined();
      expect(result.conditions!.some((c) => c.includes("evidence"))).toBe(true);
    });

    it("should reject with insufficient evidence when conditional not allowed", async () => {
      const customInterpreter = new WaiverInterpreter({
        allowConditionalWaivers: false,
        minEvidenceForApproval: 3,
      });

      const rule = createRule();
      const request = createRequest({
        evidence: ["evidence-1"],
      });

      const result = await customInterpreter.evaluateWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(result.shouldApprove).toBe(false);
      expect(result.reasoning).toContain("Insufficient evidence");
    });

    it("should reduce duration when exceeding max", async () => {
      const customInterpreter = new WaiverInterpreter({
        maxWaiverDuration: 48 * 60 * 60 * 1000, // 48 hours
      });

      const rule = createRule();
      const request = createRequest({
        requestedDuration: 168 * 60 * 60 * 1000, // 7 days
      });

      const result = await customInterpreter.evaluateWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(result.shouldApprove).toBe(true);
      expect(result.reasoning).toContain("reduced duration");
      expect(result.recommendedDuration).toBeLessThan(
        request.requestedDuration
      );
    });

    it("should add conditions for critical rules", async () => {
      const rule = createRule({ severity: ViolationSeverity.CRITICAL });
      const request = createRequest();

      const result = await interpreter.evaluateWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(result.conditions).toBeDefined();
      expect(
        result.conditions!.some((c) => c.includes("progress reports"))
      ).toBe(true);
    });
  });

  describe("processWaiver", () => {
    it("should create approved waiver decision", async () => {
      const rule = createRule();
      const request = createRequest();

      const decision = await interpreter.processWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(decision.requestId).toBe("waiver-1");
      expect(decision.status).toBe(WaiverStatus.APPROVED);
      expect(decision.decidedBy).toBe("arbiter-1");
      expect(decision.decidedAt).toBeInstanceOf(Date);
    });

    it("should set expiration for approved waiver", async () => {
      const rule = createRule();
      const request = createRequest();

      const decision = await interpreter.processWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(decision.expiresAt).toBeInstanceOf(Date);
      expect(decision.approvedDuration).toBe(request.requestedDuration);
    });

    it("should track approved waiver as active", async () => {
      const rule = createRule();
      const request = createRequest();

      await interpreter.processWaiver(request, rule, "arbiter-1");

      expect(interpreter.isWaiverActive("RULE-001")).toBe(true);
    });

    it("should reject waiver for non-waivable rule", async () => {
      const rule = createRule({ waivable: false });
      const request = createRequest();

      const decision = await interpreter.processWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(decision.status).toBe(WaiverStatus.REJECTED);
      expect(interpreter.isWaiverActive("RULE-001")).toBe(false);
    });

    it("should set auto-revoke timestamp", async () => {
      const customInterpreter = new WaiverInterpreter({
        autoRevokeOnExpiration: true,
      });

      const rule = createRule();
      const request = createRequest();

      const decision = await customInterpreter.processWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(decision.autoRevokeAt).toBeInstanceOf(Date);
      expect(decision.autoRevokeAt).toEqual(decision.expiresAt);
    });
  });

  describe("isWaiverActive", () => {
    it("should return true for active waiver", async () => {
      const rule = createRule();
      const request = createRequest();

      await interpreter.processWaiver(request, rule, "arbiter-1");

      expect(interpreter.isWaiverActive("RULE-001")).toBe(true);
    });

    it("should return false for non-existent waiver", () => {
      expect(interpreter.isWaiverActive("RULE-999")).toBe(false);
    });

    it("should return false for expired waiver", async () => {
      const rule = createRule();
      const request = createRequest({
        requestedDuration: -1000, // Already expired
      });

      await interpreter.processWaiver(request, rule, "arbiter-1");

      expect(interpreter.isWaiverActive("RULE-001")).toBe(false);
    });

    it("should auto-revoke expired waiver when configured", async () => {
      const customInterpreter = new WaiverInterpreter({
        autoRevokeOnExpiration: true,
        minEvidenceForApproval: 1, // Ensure approval with requested duration
      });

      const rule = createRule();
      const request = createRequest({
        requestedDuration: -1000, // Already expired 1 second ago
        evidence: ["single-evidence"], // Ensure we have evidence
      });

      const decision = await customInterpreter.processWaiver(
        request,
        rule,
        "arbiter-1"
      );

      // Verify the waiver was created with correct duration
      expect(decision.approvedDuration).toBe(-1000);
      expect(decision.expiresAt).toBeDefined();

      // First call should trigger auto-revoke and return false
      expect(customInterpreter.isWaiverActive("RULE-001")).toBe(false);

      // Second call should still return false (waiver should be removed)
      expect(customInterpreter.isWaiverActive("RULE-001")).toBe(false);
      expect(customInterpreter.getActiveWaiver("RULE-001")).toBeUndefined();
    });
  });

  describe("getActiveWaiver", () => {
    it("should return active waiver", async () => {
      const rule = createRule();
      const request = createRequest();

      await interpreter.processWaiver(request, rule, "arbiter-1");

      const waiver = interpreter.getActiveWaiver("RULE-001");
      expect(waiver).toBeDefined();
      expect(waiver!.status).toBe(WaiverStatus.APPROVED);
    });

    it("should return undefined for non-existent waiver", () => {
      const waiver = interpreter.getActiveWaiver("RULE-999");
      expect(waiver).toBeUndefined();
    });

    it("should return undefined for expired waiver", async () => {
      const rule = createRule();
      const request = createRequest({
        requestedDuration: -1000,
      });

      await interpreter.processWaiver(request, rule, "arbiter-1");

      const waiver = interpreter.getActiveWaiver("RULE-001");
      expect(waiver).toBeUndefined();
    });
  });

  describe("revokeWaiver", () => {
    it("should revoke active waiver", async () => {
      const rule = createRule();
      const request = createRequest();

      await interpreter.processWaiver(request, rule, "arbiter-1");

      const success = interpreter.revokeWaiver(
        "RULE-001",
        "admin",
        "Policy violation"
      );

      expect(success).toBe(true);
      expect(interpreter.isWaiverActive("RULE-001")).toBe(false);
    });

    it("should return false for non-existent waiver", () => {
      const success = interpreter.revokeWaiver("RULE-999", "admin", "Test");
      expect(success).toBe(false);
    });

    it("should update waiver status to revoked", async () => {
      const rule = createRule();
      const request = createRequest();

      await interpreter.processWaiver(request, rule, "arbiter-1");
      interpreter.revokeWaiver("RULE-001", "admin", "Test");

      const history = interpreter.getWaiverHistory();
      const waiver = history.find((w) => w.requestId === "waiver-1");
      expect(waiver!.status).toBe(WaiverStatus.REVOKED);
    });
  });

  describe("extendWaiver", () => {
    it("should extend waiver duration", async () => {
      const rule = createRule();
      const request = createRequest();

      await interpreter.processWaiver(request, rule, "arbiter-1");

      const originalWaiver = interpreter.getActiveWaiver("RULE-001");
      const originalExpiry = originalWaiver!.expiresAt!;

      const extension = 24 * 60 * 60 * 1000; // 24 hours
      const extended = interpreter.extendWaiver(
        "RULE-001",
        extension,
        "admin",
        "Justified extension"
      );

      expect(extended).not.toBeNull();
      expect(extended!.expiresAt!.getTime()).toBeGreaterThan(
        originalExpiry.getTime()
      );
    });

    it("should return null for non-existent waiver", () => {
      const extended = interpreter.extendWaiver(
        "RULE-999",
        86400000,
        "admin",
        "Test"
      );
      expect(extended).toBeNull();
    });

    it("should reject extension exceeding max duration", async () => {
      const customInterpreter = new WaiverInterpreter({
        maxWaiverDuration: 48 * 60 * 60 * 1000, // 48 hours
      });

      const rule = createRule();
      const request = createRequest({
        requestedDuration: 24 * 60 * 60 * 1000,
      });

      await customInterpreter.processWaiver(request, rule, "arbiter-1");

      const extended = customInterpreter.extendWaiver(
        "RULE-001",
        72 * 60 * 60 * 1000, // Would exceed max
        "admin",
        "Test"
      );

      expect(extended).toBeNull();
    });
  });

  describe("getActiveWaivers", () => {
    it("should return all active waivers", async () => {
      const rule1 = createRule({ id: "RULE-001" });
      const rule2 = createRule({ id: "RULE-002" });

      await interpreter.processWaiver(
        createRequest({ id: "waiver-1", ruleId: "RULE-001" }),
        rule1,
        "arbiter-1"
      );
      await interpreter.processWaiver(
        createRequest({ id: "waiver-2", ruleId: "RULE-002" }),
        rule2,
        "arbiter-1"
      );

      const active = interpreter.getActiveWaivers();
      expect(active).toHaveLength(2);
    });

    it("should exclude expired waivers", async () => {
      const rule1 = createRule({ id: "RULE-001" });
      const rule2 = createRule({ id: "RULE-002" });

      await interpreter.processWaiver(
        createRequest({
          id: "waiver-1",
          ruleId: "RULE-001",
          requestedDuration: 86400000,
        }),
        rule1,
        "arbiter-1"
      );
      await interpreter.processWaiver(
        createRequest({
          id: "waiver-2",
          ruleId: "RULE-002",
          requestedDuration: -1000, // Expired
        }),
        rule2,
        "arbiter-1"
      );

      const active = interpreter.getActiveWaivers();
      expect(active).toHaveLength(1);
      expect(active[0].requestId).toBe("waiver-1");
    });
  });

  describe("cleanupExpiredWaivers", () => {
    it("should remove expired waivers", async () => {
      const customInterpreter = new WaiverInterpreter({
        minEvidenceForApproval: 1, // Ensure approval with provided evidence
        autoRevokeOnExpiration: false, // Don't auto-revoke so waiver stays in map
      });

      const rule = createRule();
      const request = createRequest({
        requestedDuration: -1000, // Already expired 1 second ago
        evidence: ["evidence"], // Ensure approval
      });

      await customInterpreter.processWaiver(request, rule, "arbiter-1");

      // The waiver should be expired, so getActiveWaiver returns undefined
      expect(customInterpreter.getActiveWaiver("RULE-001")).toBeUndefined();

      // But cleanupExpiredWaivers should still find and remove it
      const cleaned = customInterpreter.cleanupExpiredWaivers();
      expect(cleaned).toBe(1);
      expect(customInterpreter.isWaiverActive("RULE-001")).toBe(false);
    });

    it("should return 0 when no waivers to clean", () => {
      const cleaned = interpreter.cleanupExpiredWaivers();
      expect(cleaned).toBe(0);
    });
  });

  describe("getStatistics", () => {
    it("should track total waivers", async () => {
      const rule = createRule();

      await interpreter.processWaiver(
        createRequest({ id: "waiver-1" }),
        rule,
        "arbiter-1"
      );
      await interpreter.processWaiver(
        createRequest({ id: "waiver-2" }),
        rule,
        "arbiter-1"
      );

      const stats = interpreter.getStatistics();
      expect(stats.totalWaivers).toBe(2);
    });

    it("should count approved waivers", async () => {
      const rule = createRule();

      await interpreter.processWaiver(
        createRequest({ id: "waiver-1" }),
        rule,
        "arbiter-1"
      );

      const stats = interpreter.getStatistics();
      expect(stats.approvedCount).toBe(1);
    });

    it("should count rejected waivers", async () => {
      const rule = createRule({ waivable: false });

      await interpreter.processWaiver(
        createRequest({ id: "waiver-1" }),
        rule,
        "arbiter-1"
      );

      const stats = interpreter.getStatistics();
      expect(stats.rejectedCount).toBe(1);
    });

    it("should count revoked waivers", async () => {
      const rule = createRule();

      await interpreter.processWaiver(
        createRequest({ id: "waiver-1" }),
        rule,
        "arbiter-1"
      );
      interpreter.revokeWaiver("RULE-001", "admin", "Test");

      const stats = interpreter.getStatistics();
      expect(stats.revokedCount).toBe(1);
    });

    it("should calculate average duration", async () => {
      const rule1 = createRule({ id: "RULE-001" });
      const rule2 = createRule({ id: "RULE-002" });

      await interpreter.processWaiver(
        createRequest({
          id: "waiver-1",
          ruleId: "RULE-001",
          requestedDuration: 24 * 60 * 60 * 1000,
        }),
        rule1,
        "arbiter-1"
      );
      await interpreter.processWaiver(
        createRequest({
          id: "waiver-2",
          ruleId: "RULE-002",
          requestedDuration: 48 * 60 * 60 * 1000,
        }),
        rule2,
        "arbiter-1"
      );

      const stats = interpreter.getStatistics();
      expect(stats.averageDuration).toBeGreaterThan(0);
      expect(stats.averageDuration).toBe(((24 + 48) * 60 * 60 * 1000) / 2);
    });
  });

  describe("edge cases", () => {
    it("should handle concurrent waiver requests for same rule", async () => {
      const rule = createRule();
      const request1 = createRequest({ id: "waiver-1" });
      const request2 = createRequest({ id: "waiver-2" });

      await interpreter.processWaiver(request1, rule, "arbiter-1");

      const evaluation = await interpreter.evaluateWaiver(
        request2,
        rule,
        "arbiter-1"
      );

      expect(evaluation.shouldApprove).toBe(false);
      expect(evaluation.reasoning).toContain("Active waiver already exists");
    });

    it("should handle waiver without requested duration", async () => {
      const rule = createRule();
      const request = createRequest({
        requestedDuration: undefined as any,
      });

      const decision = await interpreter.processWaiver(
        request,
        rule,
        "arbiter-1"
      );

      expect(decision.approvedDuration).toBeGreaterThan(0);
    });
  });
});
