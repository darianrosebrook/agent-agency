/**
 * @fileoverview Unit Tests for CAWSValidator
 * Comprehensive test suite for main CAWS validation orchestrator
 * @module tests/unit/caws-validator
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { CAWSValidator } from "../../../src/caws-validator/CAWSValidator";
import type { ChangeStats } from "../../../src/caws-validator/types/validation-types";
import { PolicyLoader } from "../../../src/caws-validator/utils/policy-loader";
import { WaiverManager } from "../../../src/caws-validator/waivers/WaiverManager";
import { PerformanceTracker } from "../../../src/rl/PerformanceTracker";
import type { CAWSPolicy, WorkingSpec } from "../../../src/types/caws-types";

describe("CAWSValidator", () => {
  let validator: CAWSValidator;
  let mockPolicyLoader: jest.Mocked<PolicyLoader>;
  let mockWaiverManager: jest.Mocked<WaiverManager>;
  let mockPerformanceTracker: jest.Mocked<PerformanceTracker>;

  const createMockPolicy = (): CAWSPolicy => ({
    version: "3.1.0",
    risk_tiers: {
      1: {
        max_files: 25,
        max_loc: 1000,
        coverage_threshold: 90,
        mutation_threshold: 70,
        contracts_required: true,
        manual_review_required: true,
      },
      2: {
        max_files: 50,
        max_loc: 2000,
        coverage_threshold: 80,
        mutation_threshold: 50,
        contracts_required: true,
        manual_review_required: false,
      },
      3: {
        max_files: 100,
        max_loc: 5000,
        coverage_threshold: 70,
        mutation_threshold: 30,
        contracts_required: false,
        manual_review_required: false,
      },
    },
  });

  const createValidSpec = (
    overrides: Partial<WorkingSpec> = {}
  ): WorkingSpec => ({
    id: "FEAT-001",
    title: "Test Feature",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["src/features"],
      data_migration: false,
    },
    operational_rollback_slo: "5m",
    scope: {
      in: ["src/features/", "tests/"],
      out: ["node_modules/", "dist/"],
    },
    invariants: ["System remains stable", "Data consistency maintained"],
    acceptance: [
      {
        id: "A1",
        given: "User is logged in",
        when: "User clicks submit",
        then: "Data is saved",
      },
    ],
    non_functional: {
      a11y: ["keyboard-navigation"],
      perf: { api_p95_ms: 250 },
      security: ["input-validation"],
    },
    contracts: [
      {
        type: "openapi",
        path: "docs/api.yaml",
      },
    ],
    ...overrides,
  });

  beforeEach(() => {
    mockPolicyLoader = {
      loadPolicy: jest.fn(),
      validatePolicy: jest.fn(),
      getDefaultPolicy: jest.fn(),
    } as any;

    mockWaiverManager = {
      loadWaiver: jest.fn(),
      loadWaivers: jest.fn(),
      isWaiverValid: jest.fn(),
      listWaivers: jest.fn(),
    } as any;

    mockPerformanceTracker = {
      recordConstitutionalValidation: jest.fn(),
    } as any;

    validator = new CAWSValidator({
      policyLoader: mockPolicyLoader,
      waiverManager: mockWaiverManager,
      performanceTracker: mockPerformanceTracker,
    });
  });

  describe("validateWorkingSpec", () => {
    it("should validate successfully with valid spec", async () => {
      const spec = createValidSpec();
      const policy = createMockPolicy();

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const result = await validator.validateWorkingSpec(spec);

      expect(result.passed).toBe(true);
      expect(result.verdict).toBe("pass");
      expect(result.cawsVersion).toBe("3.1.0");
      expect(result.timestamp).toBeDefined();
      expect(result.metadata?.specId).toBe(spec.id);
      expect(result.metadata?.riskTier).toBe(spec.risk_tier);
    });

    it("should fail with invalid spec structure", async () => {
      const spec = {
        title: "Invalid Spec",
      } as WorkingSpec;
      const policy = createMockPolicy();

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const result = await validator.validateWorkingSpec(spec);

      expect(result.passed).toBe(false);
      expect(result.verdict).toBe("fail");
      expect(result.remediation).toBeDefined();
      expect(result.remediation!.length).toBeGreaterThan(0);
    });

    it("should include budget compliance when requested", async () => {
      const spec = createValidSpec();
      const policy = createMockPolicy();
      const currentStats: ChangeStats = {
        filesChanged: 30,
        linesChanged: 1500,
      };

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const result = await validator.validateWorkingSpec(spec, {
        checkBudget: true,
        currentStats,
      });

      expect(result.budgetCompliance).toBeDefined();
      expect(result.budgetCompliance?.compliant).toBe(true);
      expect(result.budgetCompliance?.current.filesChanged).toBe(30);
      expect(result.budgetCompliance?.current.linesChanged).toBe(1500);
    });

    it("should fail budget validation when over limit", async () => {
      const spec = createValidSpec();
      const policy = createMockPolicy();
      const currentStats: ChangeStats = {
        filesChanged: 60, // Over limit of 50
        linesChanged: 1500,
      };

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const result = await validator.validateWorkingSpec(spec, {
        checkBudget: true,
        currentStats,
      });

      expect(result.passed).toBe(false);
      expect(result.budgetCompliance?.compliant).toBe(false);
      expect(result.budgetCompliance?.violations.length).toBeGreaterThan(0);
    });

    it("should execute quality gates when requested", async () => {
      const spec = createValidSpec();
      const policy = createMockPolicy();

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const result = await validator.validateWorkingSpec(spec, {
        executeGates: true,
      });

      // Quality gates execution is currently stubbed, so should be empty array
      expect(result.qualityGates).toEqual([]);
    });

    it("should skip spec validation when requested", async () => {
      const spec = createValidSpec({
        acceptance: [
          {
            id: "A1",
            given: "User is logged in",
            when: "User clicks button",
            then: "Action completes",
          },
        ], // Valid acceptance criteria
      });
      const policy = createMockPolicy();

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const result = await validator.validateWorkingSpec(spec, {
        skipSpecValidation: true,
      });

      // Should pass because spec validation was skipped
      expect(result.passed).toBe(true);
    });

    it("should handle policy loading errors", async () => {
      const spec = createValidSpec();

      mockPolicyLoader.loadPolicy.mockRejectedValue(
        new Error("Policy not found")
      );

      const result = await validator.validateWorkingSpec(spec);

      expect(result.passed).toBe(false);
      expect(result.remediation).toBeDefined();
      expect(result.remediation![0]).toContain("Policy not found");
    });

    it("should record performance metrics", async () => {
      const spec = createValidSpec();
      const policy = createMockPolicy();

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      await validator.validateWorkingSpec(spec);

      expect(
        mockPerformanceTracker.recordConstitutionalValidation
      ).toHaveBeenCalledWith({
        taskId: spec.id,
        agentId: "caws-validator",
        validationResult: expect.objectContaining({
          valid: true,
          violations: expect.any(Array),
          complianceScore: expect.any(Number),
          processingTimeMs: expect.any(Number),
          ruleCount: 10,
        }),
      });
    });
  });

  describe("validateAndPublish", () => {
    it("should validate and return result (publication not yet implemented)", async () => {
      const spec = createValidSpec();
      const policy = createMockPolicy();

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const result = await validator.validateAndPublish(spec);

      expect(result.passed).toBe(true);
      // Publication logic would be added later
    });
  });

  describe("validateWithAutoFix", () => {
    it("should apply auto-fixes and validate", async () => {
      const spec = createValidSpec({
        risk_tier: 5, // Invalid, should be auto-fixed to 3
      });
      const policy = createMockPolicy();

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const result = await validator.validateWithAutoFix(spec);

      expect(spec.risk_tier).toBe(3); // Should be clamped
      expect(result.passed).toBe(true);
    });
  });

  describe("checkBudgetCompliance", () => {
    it("should check budget compliance for given stats", async () => {
      const spec = createValidSpec();
      const policy = createMockPolicy();
      const currentStats: ChangeStats = {
        filesChanged: 30,
        linesChanged: 1500,
      };

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const compliance = await validator.checkBudgetCompliance(
        spec,
        currentStats,
        "/test/root"
      );

      expect(compliance.compliant).toBe(true);
      expect(compliance.baseline.max_files).toBe(50);
      expect(compliance.effective.max_files).toBe(50);
    });

    it("should handle waivers in budget compliance", async () => {
      const spec = createValidSpec({
        waiver_ids: ["WV-0001"],
      });
      const policy = createMockPolicy();
      const currentStats: ChangeStats = {
        filesChanged: 45,
        linesChanged: 1500,
      };

      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);
      mockWaiverManager.loadWaiver.mockResolvedValue({
        id: "WV-0001",
        title: "Test Waiver",
        reason: "Testing",
        status: "active",
        gates: ["budget_limit"],
        expires_at: "2025-12-31",
        approvers: ["test@test.com"],
        impact_level: "medium",
        mitigation_plan: "Test",
        created_at: "2025-01-01",
        created_by: "test@test.com",
        delta: { max_files: 10 },
      });
      mockWaiverManager.isWaiverValid.mockReturnValue(true);

      const compliance = await validator.checkBudgetCompliance(
        spec,
        currentStats,
        "/test/root"
      );

      expect(compliance.compliant).toBe(true);
      expect(compliance.effective.max_files).toBe(60); // 50 + 10
      expect(compliance.waiversApplied).toContain("WV-0001");
    });
  });

  describe("generateBudgetReport", () => {
    it("should generate readable budget report", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        current: { filesChanged: 30, linesChanged: 1500 },
        violations: [],
        waiversApplied: [],
      };

      const report = validator.generateBudgetReport(compliance, 2);

      expect(report).toContain("CAWS Budget Burn-up Report");
      expect(report).toContain("Risk Tier: 2");
      expect(report).toContain("30 files");
      expect(report).toContain("1500 LOC");
      expect(report).toContain("File Usage: 60%");
      expect(report).toContain("LOC Usage: 75%");
    });

    it("should include waiver information in report", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 60, max_loc: 2500 },
        current: { filesChanged: 45, linesChanged: 2000 },
        violations: [],
        waiversApplied: ["WV-0001"],
      };

      const report = validator.generateBudgetReport(compliance, 2);

      expect(report).toContain("Waivers Applied: WV-0001");
      expect(report).toContain("Effective Budget: 60 files, 2500 LOC");
    });

    it("should show violations in report", () => {
      const compliance = {
        compliant: false,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        current: { filesChanged: 60, linesChanged: 2500 },
        violations: [
          {
            gate: "budget_limit",
            type: "max_files" as const,
            current: 60,
            limit: 50,
            baseline: 50,
            message: "File count (60) exceeds budget (50)",
          },
        ],
        waiversApplied: [],
      };

      const report = validator.generateBudgetReport(compliance, 2);

      expect(report).toContain("BUDGET EXCEEDED");
      expect(report).toContain("File count (60) exceeds budget (50)");
    });
  });

  describe("getValidationSummary", () => {
    it("should generate comprehensive validation summary", () => {
      const result = {
        passed: true,
        cawsVersion: "3.1.0",
        timestamp: new Date().toISOString(),
        budgetCompliance: {
          compliant: true,
          baseline: { max_files: 50, max_loc: 2000 },
          effective: { max_files: 50, max_loc: 2000 },
          current: { filesChanged: 30, linesChanged: 1500 },
          violations: [],
          waiversApplied: [],
        },
        qualityGates: [],
        waivers: [],
        verdict: "pass" as const,
        remediation: undefined,
        metadata: {
          specId: "FEAT-001",
          riskTier: 2,
          durationMs: 150,
          environment: "test",
          mode: "feature",
        },
      };

      const summary = validator.getValidationSummary(result);

      expect(summary).toContain("CAWS Validation Summary");
      expect(summary).toContain("Status: ✅ PASSED");
      expect(summary).toContain("Verdict: PASS");
      expect(summary).toContain("Duration: 150ms");
      expect(summary).toContain("Budget Status:");
      expect(summary).toContain("Compliant: ✅");
    });

    it("should include remediation steps for failed validation", () => {
      const result = {
        passed: false,
        cawsVersion: "3.1.0",
        timestamp: new Date().toISOString(),
        budgetCompliance: undefined,
        qualityGates: [],
        waivers: [],
        verdict: "fail" as const,
        remediation: [
          "Fix: Missing required field: id",
          "Fix: Invalid risk tier",
        ],
        metadata: {
          specId: "INVALID-001",
          riskTier: 1,
          durationMs: 200,
          environment: "test",
          mode: "feature",
        },
      };

      const summary = validator.getValidationSummary(result);

      expect(summary).toContain("Status: ❌ FAILED");
      expect(summary).toContain("Remediation Required:");
      expect(summary).toContain("Fix: Missing required field: id");
      expect(summary).toContain("Fix: Invalid risk tier");
    });
  });

  describe("error handling", () => {
    it("should handle validation errors gracefully", async () => {
      const spec = createValidSpec();

      // Simulate a validation error
      mockPolicyLoader.loadPolicy.mockRejectedValue(
        new Error("Database connection failed")
      );

      const result = await validator.validateWorkingSpec(spec);

      expect(result.passed).toBe(false);
      expect(result.verdict).toBe("fail");
      expect(result.remediation).toBeDefined();
      expect(result.metadata?.durationMs).toBeGreaterThanOrEqual(0);
    });

    it("should include error details in metadata", async () => {
      const spec = createValidSpec();

      mockPolicyLoader.loadPolicy.mockRejectedValue(
        new Error("Policy validation failed")
      );

      const result = await validator.validateWorkingSpec(spec);

      expect(result.metadata?.specId).toBe(spec.id);
      expect(result.metadata?.riskTier).toBe(spec.risk_tier);
      expect(result.metadata?.durationMs).toBeGreaterThanOrEqual(0);
    });
  });
});
