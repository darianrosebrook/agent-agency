/**
 * @fileoverview Unit Tests for BudgetValidator
 * Comprehensive test suite for budget derivation and compliance checking
 * @module tests/unit/caws-validator
 */
// @ts-nocheck


import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import type {
  ChangeStats,
  WaiverDocument,
} from "../../../src/caws-validator/types/validation-types";
import { PolicyLoader } from "../../../src/caws-validator/utils/policy-loader";
import { BudgetValidator } from "../../../src/caws-validator/validation/BudgetValidator";
import { WaiverManager } from "../../../src/caws-validator/waivers/WaiverManager";
import type { CAWSPolicy, WorkingSpec } from "../../../src/types/caws-types";

describe("BudgetValidator", () => {
  let validator: BudgetValidator;
  let mockPolicyLoader: jest.Mocked<PolicyLoader>;
  let mockWaiverManager: jest.Mocked<WaiverManager>;

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

  const createMockWaiver = (
    id: string,
    delta?: { max_files?: number; max_loc?: number }
  ): WaiverDocument => ({
    id,
    title: `Test Waiver ${id}`,
    reason: "Testing",
    status: "active",
    gates: ["budget_limit"],
    expires_at: "2025-12-31",
    approvers: ["approver@test.com"],
    impact_level: "medium",
    mitigation_plan: "Monitored carefully",
    delta,
    created_at: "2025-01-01",
    created_by: "test@test.com",
  });

  const createValidSpec = (tier: number): WorkingSpec => ({
    id: "FEAT-001",
    title: "Test Feature",
    risk_tier: tier,
    mode: "feature",
    blast_radius: { modules: ["test"], data_migration: false },
    operational_rollback_slo: "5m",
    scope: { in: ["src/"], out: [] },
    invariants: ["test"],
    acceptance: [],
    non_functional: {},
    contracts: [],
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

    validator = new BudgetValidator(mockPolicyLoader, mockWaiverManager);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("deriveBudget", () => {
    it("should derive budget from policy for Tier 1", async () => {
      const spec = createValidSpec(1);
      const policy = createMockPolicy();
      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const budget = await validator.deriveBudget(spec, "/test/root");

      expect(budget.baseline.max_files).toBe(25);
      expect(budget.baseline.max_loc).toBe(1000);
      expect(budget.effective.max_files).toBe(25);
      expect(budget.effective.max_loc).toBe(1000);
      expect(budget.waiversApplied).toEqual([]);
    });

    it("should derive budget from policy for Tier 2", async () => {
      const spec = createValidSpec(2);
      const policy = createMockPolicy();
      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const budget = await validator.deriveBudget(spec, "/test/root");

      expect(budget.baseline.max_files).toBe(50);
      expect(budget.baseline.max_loc).toBe(2000);
    });

    it("should derive budget from policy for Tier 3", async () => {
      const spec = createValidSpec(3);
      const policy = createMockPolicy();
      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const budget = await validator.deriveBudget(spec, "/test/root");

      expect(budget.baseline.max_files).toBe(100);
      expect(budget.baseline.max_loc).toBe(5000);
    });

    it("should apply waivers to budget", async () => {
      const spec = createValidSpec(2);
      spec.waiver_ids = ["WV-0001"];
      const policy = createMockPolicy();
      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const waiver = createMockWaiver("WV-0001", {
        max_files: 10,
        max_loc: 500,
      });
      mockWaiverManager.loadWaiver.mockResolvedValue(waiver);
      mockWaiverManager.isWaiverValid.mockReturnValue(true);

      const budget = await validator.deriveBudget(spec, "/test/root");

      expect(budget.effective.max_files).toBe(60); // 50 + 10
      expect(budget.effective.max_loc).toBe(2500); // 2000 + 500
      expect(budget.waiversApplied).toContain("WV-0001");
    });

    it("should apply multiple waivers", async () => {
      const spec = createValidSpec(2);
      spec.waiver_ids = ["WV-0001", "WV-0002"];
      const policy = createMockPolicy();
      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      mockWaiverManager.loadWaiver.mockImplementation(async (id: string) => {
        if (id === "WV-0001") {
          return createMockWaiver("WV-0001", { max_files: 10, max_loc: 500 });
        }
        if (id === "WV-0002") {
          return createMockWaiver("WV-0002", { max_files: 5, max_loc: 250 });
        }
        return null;
      });
      mockWaiverManager.isWaiverValid.mockReturnValue(true);

      const budget = await validator.deriveBudget(spec, "/test/root");

      expect(budget.effective.max_files).toBe(65); // 50 + 10 + 5
      expect(budget.effective.max_loc).toBe(2750); // 2000 + 500 + 250
      expect(budget.waiversApplied).toEqual(["WV-0001", "WV-0002"]);
    });

    it("should skip invalid waivers", async () => {
      const spec = createValidSpec(2);
      spec.waiver_ids = ["WV-0001"];
      const policy = createMockPolicy();
      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      const waiver = createMockWaiver("WV-0001", { max_files: 10 });
      mockWaiverManager.loadWaiver.mockResolvedValue(waiver);
      mockWaiverManager.isWaiverValid.mockReturnValue(false);

      const budget = await validator.deriveBudget(spec, "/test/root");

      expect(budget.effective.max_files).toBe(50); // No waiver applied
      expect(budget.waiversApplied).toEqual([]);
    });

    it("should throw error for undefined risk tier", async () => {
      const spec = createValidSpec(5 as any);
      const policy = createMockPolicy();
      mockPolicyLoader.loadPolicy.mockResolvedValue(policy);

      await expect(validator.deriveBudget(spec, "/test/root")).rejects.toThrow(
        "Risk tier 5 not defined in policy.yaml"
      );
    });
  });

  describe("checkBudgetCompliance", () => {
    it("should pass when within budget", async () => {
      const derivedBudget = {
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        waiversApplied: [],
        derivedAt: new Date().toISOString(),
      };

      const currentStats: ChangeStats = {
        filesChanged: 30,
        linesChanged: 1500,
      };

      const compliance = await validator.checkBudgetCompliance(
        derivedBudget,
        currentStats
      );

      expect(compliance.compliant).toBe(true);
      expect(compliance.violations).toHaveLength(0);
    });

    it("should fail when files exceed budget", async () => {
      const derivedBudget = {
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        waiversApplied: [],
        derivedAt: new Date().toISOString(),
      };

      const currentStats: ChangeStats = {
        filesChanged: 60,
        linesChanged: 1500,
      };

      const compliance = await validator.checkBudgetCompliance(
        derivedBudget,
        currentStats
      );

      expect(compliance.compliant).toBe(false);
      expect(compliance.violations.length).toBe(1);
      expect(compliance.violations[0].type).toBe("max_files");
      expect(compliance.violations[0].current).toBe(60);
      expect(compliance.violations[0].limit).toBe(50);
    });

    it("should fail when LOC exceeds budget", async () => {
      const derivedBudget = {
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        waiversApplied: [],
        derivedAt: new Date().toISOString(),
      };

      const currentStats: ChangeStats = {
        filesChanged: 30,
        linesChanged: 2500,
      };

      const compliance = await validator.checkBudgetCompliance(
        derivedBudget,
        currentStats
      );

      expect(compliance.compliant).toBe(false);
      expect(compliance.violations.length).toBe(1);
      expect(compliance.violations[0].type).toBe("max_loc");
      expect(compliance.violations[0].current).toBe(2500);
      expect(compliance.violations[0].limit).toBe(2000);
    });

    it("should report multiple violations", async () => {
      const derivedBudget = {
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        waiversApplied: [],
        derivedAt: new Date().toISOString(),
      };

      const currentStats: ChangeStats = {
        filesChanged: 60,
        linesChanged: 2500,
      };

      const compliance = await validator.checkBudgetCompliance(
        derivedBudget,
        currentStats
      );

      expect(compliance.compliant).toBe(false);
      expect(compliance.violations.length).toBe(2);
      expect(compliance.violations.some((v) => v.type === "max_files")).toBe(
        true
      );
      expect(compliance.violations.some((v) => v.type === "max_loc")).toBe(
        true
      );
    });

    it("should include violation messages", async () => {
      const derivedBudget = {
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        waiversApplied: [],
        derivedAt: new Date().toISOString(),
      };

      const currentStats: ChangeStats = {
        filesChanged: 60,
        linesChanged: 1500,
      };

      const compliance = await validator.checkBudgetCompliance(
        derivedBudget,
        currentStats
      );

      expect(compliance.violations[0].message).toContain("File count");
      expect(compliance.violations[0].message).toContain("60");
      expect(compliance.violations[0].message).toContain("50");
    });
  });

  describe("generateBurnupReport", () => {
    it("should generate basic report", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        current: { filesChanged: 30, linesChanged: 1500 },
        violations: [],
        waiversApplied: [],
      };

      const report = validator.generateBurnupReport(compliance, 2);

      expect(report).toContain("Risk Tier: 2");
      expect(report).toContain("50 files");
      expect(report).toContain("2000 LOC");
      expect(report).toContain("30 files");
      expect(report).toContain("1500 LOC");
    });

    it("should show waivers in report", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 60, max_loc: 2500 },
        current: { filesChanged: 55, linesChanged: 2200 },
        violations: [],
        waiversApplied: ["WV-0001"],
      };

      const report = validator.generateBurnupReport(compliance, 2);

      expect(report).toContain("Waivers Applied: WV-0001");
      expect(report).toContain("Effective Budget: 60 files, 2500 LOC");
    });

    it("should warn when approaching limits", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        current: { filesChanged: 46, linesChanged: 1900 },
        violations: [],
        waiversApplied: [],
      };

      const report = validator.generateBurnupReport(compliance, 2);

      expect(report).toContain("WARNING: Approaching budget limits");
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

      const report = validator.generateBurnupReport(compliance, 2);

      expect(report).toContain("BUDGET EXCEEDED");
      expect(report).toContain("File count (60) exceeds budget (50)");
    });
  });

  describe("calculateUtilization", () => {
    it("should calculate file and LOC utilization", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        current: { filesChanged: 25, linesChanged: 1000 },
        violations: [],
        waiversApplied: [],
      };

      const utilization = validator.calculateUtilization(compliance);

      expect(utilization.files).toBe(50);
      expect(utilization.loc).toBe(50);
      expect(utilization.overall).toBe(50);
    });

    it("should use max of files and LOC for overall", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        current: { filesChanged: 40, linesChanged: 1000 },
        violations: [],
        waiversApplied: [],
      };

      const utilization = validator.calculateUtilization(compliance);

      expect(utilization.files).toBe(80);
      expect(utilization.loc).toBe(50);
      expect(utilization.overall).toBe(80);
    });
  });

  describe("isApproachingLimit", () => {
    it("should return true when above threshold", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        current: { filesChanged: 46, linesChanged: 1000 },
        violations: [],
        waiversApplied: [],
      };

      const approaching = validator.isApproachingLimit(compliance, 90);

      expect(approaching).toBe(true);
    });

    it("should return false when below threshold", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        current: { filesChanged: 25, linesChanged: 1000 },
        violations: [],
        waiversApplied: [],
      };

      const approaching = validator.isApproachingLimit(compliance, 90);

      expect(approaching).toBe(false);
    });

    it("should use custom threshold", () => {
      const compliance = {
        compliant: true,
        baseline: { max_files: 50, max_loc: 2000 },
        effective: { max_files: 50, max_loc: 2000 },
        current: { filesChanged: 40, linesChanged: 1000 },
        violations: [],
        waiversApplied: [],
      };

      expect(validator.isApproachingLimit(compliance, 85)).toBe(false);
      expect(validator.isApproachingLimit(compliance, 75)).toBe(true);
    });
  });
});
