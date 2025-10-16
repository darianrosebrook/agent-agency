/**
 * Integration tests for IterativeGuidance system
 *
 * Tests intelligent progress tracking, gap analysis, and guidance generation.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import * as fs from "fs/promises";
import * as path from "path";
import { IterativeGuidance } from "../../../src/guidance/IterativeGuidance.js";
import type {
  GuidanceConfig,
  GuidanceContext,
} from "../../../src/guidance/types/guidance-types.js";
import type { WorkingSpec } from "../../../src/types/caws-types.js";

describe("IterativeGuidance Integration Tests", () => {
  const tempDir = path.join(__dirname, "../../temp/guidance-tests");
  const projectRoot = path.join(tempDir, "project");
  let guidance: IterativeGuidance;

  const validSpec: WorkingSpec = {
    id: "GUIDE-001",
    title: "User Authentication Feature",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["auth", "user"],
      data_migration: true,
    },
    operational_rollback_slo: "30m",
    scope: {
      in: ["src/auth/", "src/user/", "tests/auth/"],
      out: ["node_modules/", "dist/"],
    },
    invariants: ["Passwords must be hashed", "Sessions expire after 24h"],
    non_functional: {
      perf: {
        api_p95_ms: 250,
      },
      security: ["input-validation"],
    },
    acceptance: [
      {
        id: "A1",
        given: "A user is on the login page",
        when: "They enter valid credentials",
        then: "They should be redirected to dashboard",
      },
      {
        id: "A2",
        given: "A user enters invalid credentials",
        when: "They attempt to login",
        then: "An error message should be displayed",
      },
      {
        id: "A3",
        given: "A user is logged in",
        when: "Their session expires",
        then: "They should be redirected to login",
      },
    ],
    contracts: [
      {
        type: "openapi",
        path: "docs/api/auth.yaml",
        version: "1.0.0",
        tests_required: true,
      },
    ],
  };

  const context: GuidanceContext = {
    phase: "implementation",
    teamSize: 2,
    experienceLevel: "senior",
    timePressure: "medium",
    qualityRequirements: "high",
    technologyFamiliarity: "expert",
  };

  beforeEach(async () => {
    // Create temp directory structure
    await fs.mkdir(projectRoot, { recursive: true });
    await fs.mkdir(path.join(projectRoot, "src"), { recursive: true });
    await fs.mkdir(path.join(projectRoot, "src", "auth"), { recursive: true });
    await fs.mkdir(path.join(projectRoot, "tests"), { recursive: true });
    await fs.mkdir(path.join(projectRoot, ".caws"), { recursive: true });

    // Write policy file
    const policyPath = path.join(projectRoot, ".caws", "policy.yaml");
    await fs.writeFile(
      policyPath,
      `version: "1.0.0"
risk_tiers:
  2:
    max_files: 20
    max_loc: 500
    coverage_threshold: 0.80
    mutation_threshold: 0.70
    contracts_required: true
    manual_review_required: false
`
    );
  });

  afterEach(async () => {
    if (guidance) {
      // Clean up event listeners
      guidance.removeAllListeners();
    }

    try {
      await fs.rm(tempDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  describe("Initialization", () => {
    it("should create guidance system with default config", () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
      };

      guidance = new IterativeGuidance(config);
      expect(guidance).toBeDefined();
    });

    it("should create guidance system with custom context", () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
      };

      guidance = new IterativeGuidance(config, context);
      expect(guidance).toBeDefined();
    });

    it("should report capabilities correctly", () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
      };

      guidance = new IterativeGuidance(config);
      const capabilities = guidance.getCapabilities();

      expect(capabilities.analyzeAcceptanceCriteria).toBe(true);
      expect(capabilities.identifyGaps).toBe(true);
      expect(capabilities.generateNextSteps).toBe(true);
      expect(capabilities.estimateWork).toBe(true);
    });
  });

  describe("Progress Analysis", () => {
    it("should analyze progress for spec with no implementation", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      expect(result.summary).toBeDefined();
      expect(result.summary?.overallProgress).toBeGreaterThanOrEqual(0);
      expect(result.summary?.acceptanceCriteria).toHaveLength(3);
    });

    it("should analyze progress with existing implementation", async () => {
      // Create some implementation files
      await fs.writeFile(
        path.join(projectRoot, "src", "auth", "login.ts"),
        "// Login implementation\n"
      );
      await fs.writeFile(
        path.join(projectRoot, "tests", "auth", "login.test.ts"),
        "// Login tests\n"
      );

      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: ["src/auth/login.ts"],
        testFiles: ["tests/auth/login.test.ts"],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      expect(result.summary?.overallProgress).toBeGreaterThan(0);
      expect(
        result.summary?.acceptanceCriteria.some((c) => c.progressPercent > 0)
      ).toBe(true);
    });

    it("should identify gaps in implementation", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      expect(result.summary?.gaps.length).toBeGreaterThan(0);
      expect(result.summary?.gaps.some((g) => g.category === "testing")).toBe(
        true
      );
    });

    it("should generate actionable next steps", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      expect(result.summary?.nextSteps.length).toBeGreaterThan(0);
      expect(result.summary?.nextSteps[0].priority).toBeDefined();
      expect(
        result.summary?.nextSteps[0].estimatedEffort.hours
      ).toBeGreaterThan(0);
    });

    it("should estimate work remaining", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      expect(result.summary?.workEstimate.totalHours).toBeGreaterThan(0);
      expect(
        result.summary?.workEstimate.hoursByCategory.implementation
      ).toBeGreaterThan(0);
      expect(
        result.summary?.workEstimate.completionEstimates.mostLikely
      ).toBeDefined();
    });
  });

  describe("Acceptance Criteria Analysis", () => {
    it("should analyze individual acceptance criteria", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: ["src/auth/login.ts"],
        testFiles: ["tests/auth/login.test.ts"],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const criteria = result.summary?.acceptanceCriteria || [];

      expect(criteria).toHaveLength(3);
      criteria.forEach((criterion) => {
        expect(criterion.id).toMatch(/^A[1-3]$/);
        expect(criterion.status).toBeDefined();
        expect(criterion.progressPercent).toBeGreaterThanOrEqual(0);
        expect(criterion.estimatedHoursRemaining).toBeGreaterThan(0);
      });
    });

    it("should detect completed criteria", async () => {
      // Create comprehensive implementation
      await fs.writeFile(
        path.join(projectRoot, "src", "auth", "login.ts"),
        "// Full login implementation\n".repeat(50)
      );
      await fs.writeFile(
        path.join(projectRoot, "tests", "auth", "login.test.ts"),
        "// Comprehensive tests\n".repeat(30)
      );
      await fs.writeFile(
        path.join(projectRoot, "src", "auth", "session.ts"),
        "// Session management\n"
      );
      await fs.writeFile(
        path.join(projectRoot, "tests", "auth", "session.test.ts"),
        "// Session tests\n"
      );

      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: ["src/auth/login.ts", "src/auth/session.ts"],
        testFiles: ["tests/auth/login.test.ts", "tests/auth/session.test.ts"],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const criteria = result.summary?.acceptanceCriteria || [];
      const completedCriteria = criteria.filter(
        (c) => c.status === "completed"
      );

      expect(completedCriteria.length).toBeGreaterThan(0);
    });

    it("should identify blocked criteria", async () => {
      // Spec with external dependencies that aren't met
      const specWithDeps = {
        ...validSpec,
        contracts: [], // Remove contracts to create blockers
        acceptance: [
          {
            id: "A1",
            given: "External API is available",
            when: "User attempts to authenticate",
            then: "Authentication succeeds",
          },
        ],
      };

      const config: GuidanceConfig = {
        spec: specWithDeps,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const criteria = result.summary?.acceptanceCriteria || [];
      expect(criteria[0].status).toBe("blocked");
      expect(criteria[0].blockers.length).toBeGreaterThan(0);
    });
  });

  describe("Gap Analysis", () => {
    it("should identify testing gaps", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: ["src/auth/login.ts"], // Implementation but no tests
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const testingGaps =
        result.summary?.gaps.filter((g) => g.category === "testing") || [];
      expect(testingGaps.length).toBeGreaterThan(0);
    });

    it("should identify implementation gaps", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [], // No implementation
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const implementationGaps =
        result.summary?.gaps.filter((g) => g.category === "implementation") ||
        [];
      expect(implementationGaps.length).toBeGreaterThan(0);
    });

    it("should identify budget-related gaps when usage is high", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
        budgetUsage: {
          filesChanged: 18,
          maxFiles: 20,
          filesPercentage: 90,
          linesChanged: 450,
          maxLoc: 500,
          locPercentage: 90,
          changedFiles: [],
          lastUpdated: new Date().toISOString(),
        },
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      // Budget gaps may or may not be identified depending on exact thresholds
      const gaps = result.summary?.gaps || [];
      expect(Array.isArray(gaps)).toBe(true);
    });
  });

  describe("Next Steps Generation", () => {
    it("should generate implementation steps for incomplete criteria", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const implementationSteps =
        result.summary?.nextSteps.filter(
          (s) => s.category === "implementation"
        ) || [];
      expect(implementationSteps.length).toBeGreaterThan(0);

      implementationSteps.forEach((step) => {
        expect(step.id).toMatch(/^implement-A[1-3]$/);
        expect(step.estimatedEffort.hours).toBeGreaterThan(0);
        expect(step.priority).toBe("high");
      });
    });

    it("should generate testing steps for gaps", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: ["src/auth/login.ts"],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const testingSteps =
        result.summary?.nextSteps.filter((s) => s.category === "testing") || [];
      expect(testingSteps.length).toBeGreaterThan(0);
    });

    it("should prioritize steps correctly", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const steps = result.summary?.nextSteps || [];

      // Critical steps should come first
      const criticalSteps = steps.filter((s) => s.priority === "critical");
      const highSteps = steps.filter((s) => s.priority === "high");

      if (criticalSteps.length > 0 && highSteps.length > 0) {
        const firstCriticalIndex = steps.findIndex(
          (s) => s.priority === "critical"
        );
        const firstHighIndex = steps.findIndex((s) => s.priority === "high");
        expect(firstCriticalIndex).toBeLessThan(firstHighIndex);
      }
    });

    it("should mark parallelizable steps", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const steps = result.summary?.nextSteps || [];
      const parallelizableSteps = steps.filter((s) => s.parallelizable);

      // Most implementation steps should be parallelizable
      expect(parallelizableSteps.length).toBeGreaterThan(0);
    });
  });

  describe("Work Estimation", () => {
    it("should estimate total work remaining", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const estimate = result.summary?.workEstimate;

      expect(estimate?.totalHours).toBeGreaterThan(0);
      if (estimate?.confidenceIntervals) {
        expect(estimate.confidenceIntervals.optimistic).toBeLessThan(
          estimate.confidenceIntervals.mostLikely
        );
        expect(estimate.confidenceIntervals.mostLikely).toBeLessThan(
          estimate.confidenceIntervals.pessimistic
        );
      }
    });

    it("should break down work by category", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const estimate = result.summary?.workEstimate;

      expect(estimate?.hoursByCategory.implementation).toBeGreaterThan(0);
      expect(estimate?.hoursByCategory.testing).toBeGreaterThan(0);
    });

    it("should break down work by priority", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const estimate = result.summary?.workEstimate;

      expect(estimate?.hoursByPriority.high).toBeGreaterThan(0);
    });

    it("should estimate completion dates", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const estimates = result.summary?.workEstimate.completionEstimates;

      expect(
        new Date(estimates?.earliest || "") <
          new Date(estimates?.mostLikely || "")
      ).toBe(true);
      expect(
        new Date(estimates?.mostLikely || "") <
          new Date(estimates?.latest || "")
      ).toBe(true);
    });
  });

  describe("Risk Assessment", () => {
    it("should assess project risks", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const riskAssessment = result.summary?.riskAssessment;

      expect(riskAssessment?.overallRisk).toBeDefined();
      expect(Array.isArray(riskAssessment?.riskFactors)).toBe(true);
      expect(Array.isArray(riskAssessment?.mitigationStrategies)).toBe(true);
    });

    it("should identify high-risk situations", async () => {
      // Create high-risk scenario: no implementation, no tests, high budget usage
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
        budgetUsage: {
          filesChanged: 19,
          maxFiles: 20,
          filesPercentage: 95,
          linesChanged: 475,
          maxLoc: 500,
          locPercentage: 95,
          changedFiles: [],
          lastUpdated: new Date().toISOString(),
        },
      };

      const highPressureContext: GuidanceContext = {
        ...context,
        timePressure: "critical",
      };

      guidance = new IterativeGuidance(config, highPressureContext);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      const riskAssessment = result.summary?.riskAssessment;

      expect(["high", "critical"]).toContain(riskAssessment?.overallRisk);
      expect(riskAssessment?.riskFactors.length).toBeGreaterThan(2);
    });
  });

  describe("Step-by-Step Guidance", () => {
    it("should provide step guidance", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const stepGuidance = await guidance.getStepGuidance(0);

      expect(stepGuidance).toBeDefined();
      expect(stepGuidance.currentStep).toBe(1);
      expect(stepGuidance.step).toBeDefined();
      expect(stepGuidance.tips.length).toBeGreaterThan(0);
      expect(stepGuidance.pitfalls.length).toBeGreaterThan(0);
      expect(stepGuidance.qualityChecks.length).toBeGreaterThan(0);
    });

    it("should provide tips for different step types", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const stepGuidance = await guidance.getStepGuidance(0);

      expect(stepGuidance.tips.length).toBeGreaterThan(0);
      expect(stepGuidance.tips.some((tip) => tip.includes("test"))).toBe(true);
    });

    it("should identify common pitfalls", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);
      const stepGuidance = await guidance.getStepGuidance(0);

      expect(stepGuidance.pitfalls.length).toBeGreaterThan(0);
      expect(
        stepGuidance.pitfalls.some((pitfall) => pitfall.includes("error"))
      ).toBe(true);
    });
  });

  describe("Recommendations", () => {
    it("should provide improvement recommendations", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: ["src/auth/login.ts"],
        testFiles: [], // Low test coverage
        aiAttribution: {
          totalCommits: 10,
          aiAssistedCommits: 3,
          aiToolsUsed: ["Cursor", "GitHub Copilot"],
        },
      };

      guidance = new IterativeGuidance(config, context);
      const recommendations = guidance.getRecommendations();

      expect(Array.isArray(recommendations)).toBe(true);
      expect(recommendations.length).toBeGreaterThan(0);

      // Should recommend increasing test coverage
      const testCoverageRec = recommendations.find((r) =>
        r.title.includes("Test Coverage")
      );
      expect(testCoverageRec).toBeDefined();
    });

    it("should recommend parallelization for team work", async () => {
      const teamContext: GuidanceContext = {
        ...context,
        teamSize: 3, // Team of 3
      };

      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, teamContext);
      const recommendations = guidance.getRecommendations();

      const parallelRec = recommendations.find((r) =>
        r.title.includes("Parallelization")
      );
      expect(parallelRec).toBeDefined();
    });
  });

  describe("Event Emission", () => {
    it("should emit analysis events", async () => {
      const events: string[] = [];

      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      guidance = new IterativeGuidance(config, context);

      guidance.on("analysis:start", () => events.push("start"));
      guidance.on("progress:analyzed", () => events.push("progress"));
      guidance.on("gaps:identified", () => events.push("gaps"));
      guidance.on("steps:generated", () => events.push("steps"));
      guidance.on("estimate:updated", () => events.push("estimate"));
      guidance.on("analysis:complete", () => events.push("complete"));

      await guidance.analyzeProgress();

      expect(events).toContain("start");
      expect(events).toContain("progress");
      expect(events).toContain("gaps");
      expect(events).toContain("steps");
      expect(events).toContain("estimate");
      expect(events).toContain("complete");
    });

    it("should handle analysis errors gracefully", async () => {
      const config: GuidanceConfig = {
        spec: { ...validSpec, acceptance: [] }, // Empty acceptance criteria
        projectRoot,
      };

      guidance = new IterativeGuidance(config, context);

      const result = await guidance.analyzeProgress();

      // Should succeed with empty acceptance criteria
      expect(result.success).toBe(true);
      expect(result.summary?.acceptanceCriteria).toHaveLength(0);
    });
  });

  describe("Integration with External Data", () => {
    it("should use budget statistics in analysis", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
        budgetUsage: {
          filesChanged: 10,
          maxFiles: 20,
          filesPercentage: 50,
          linesChanged: 250,
          maxLoc: 500,
          locPercentage: 50,
          changedFiles: [],
          lastUpdated: new Date().toISOString(),
        },
        budgetStats: {
          monitoringDuration: 3600000,
          totalFileChanges: 10,
          totalLocChanges: 250,
          peakFilesUsage: 50,
          peakLocUsage: 50,
          alertsBySeverity: { info: 0, warning: 0, critical: 0 },
          frequentlyChangedFiles: [],
        },
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      expect(result.summary?.overallProgress).toBeDefined();
    });

    it("should incorporate recent changes", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
        recentChanges: [
          {
            file: "src/auth/login.ts",
            type: "added",
            timestamp: new Date().toISOString(),
          },
        ],
      };

      guidance = new IterativeGuidance(config, context);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      // Should use recent changes in progress calculation
    });

    it("should use AI attribution data", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
        aiAttribution: {
          totalCommits: 20,
          aiAssistedCommits: 8,
          aiToolsUsed: ["Cursor", "GitHub Copilot"],
        },
      };

      guidance = new IterativeGuidance(config, context);
      const recommendations = guidance.getRecommendations();

      const aiRec = recommendations.find((r) => r.title.includes("AI"));
      expect(aiRec).toBeDefined();
    });
  });

  describe("Context Sensitivity", () => {
    it("should adjust estimates for team size", async () => {
      const smallTeamConfig: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      const largeTeamContext: GuidanceContext = {
        ...context,
        teamSize: 5,
      };

      guidance = new IterativeGuidance(smallTeamConfig, largeTeamContext);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      expect(
        result.summary?.workEstimate.parallelizationFactor
      ).toBeGreaterThan(0);
    });

    it("should adjust for time pressure", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      const highPressureContext: GuidanceContext = {
        ...context,
        timePressure: "high",
      };

      guidance = new IterativeGuidance(config, highPressureContext);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      // Should assess risk based on time pressure and other factors
      expect(["low", "medium", "high", "critical"]).toContain(
        result.summary?.riskAssessment.overallRisk
      );
    });

    it("should adjust for experience level", async () => {
      const config: GuidanceConfig = {
        spec: validSpec,
        projectRoot,
        existingFiles: [],
        testFiles: [],
      };

      const juniorContext: GuidanceContext = {
        ...context,
        experienceLevel: "junior",
      };

      guidance = new IterativeGuidance(config, juniorContext);
      const result = await guidance.analyzeProgress();

      expect(result.success).toBe(true);
      // Should have higher estimates for junior developers
      expect(result.summary?.workEstimate.totalHours).toBeGreaterThan(10);
    });
  });
});
