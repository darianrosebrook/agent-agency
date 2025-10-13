/**
 * Integration Tests for RL Pipeline
 *
 * @author @darianrosebrook
 * @module rl-pipeline-integration
 *
 * Core integration tests for the RL training pipeline covering:
 * - Debate outcome tracking
 * - Verdict quality scoring
 * - Model deployment and A/B testing
 */

import { DataCollector } from "@/benchmarking/DataCollector";
import { DebateOutcomeTracker } from "@/rl/DebateOutcomeTracker";
import { ModelDeploymentManager } from "@/rl/ModelDeploymentManager";
import { PerformanceTracker } from "@/rl/PerformanceTracker";
import { VerdictQualityScorer } from "@/rl/VerdictQualityScorer";
import {
  ArbitrationState,
  RuleCategory,
  VerdictOutcome,
  ViolationSeverity,
  type ArbitrationSession,
  type Verdict,
} from "@/types/arbitration";
import { beforeEach, describe, expect, it } from "@jest/globals";

describe("RL Pipeline Integration", () => {
  let debateTracker: DebateOutcomeTracker;
  let verdictScorer: VerdictQualityScorer;
  let deploymentManager: ModelDeploymentManager;
  let performanceTracker: PerformanceTracker;

  // Helper to create reasoning steps
  const createReasoningSteps = (descriptions: string[]) => {
    return descriptions.map((desc, idx) => ({
      step: idx + 1,
      description: desc,
      evidence: [],
      ruleReferences: [],
      confidence: 0.8,
    }));
  };

  beforeEach(() => {
    const dataCollector = new DataCollector();
    performanceTracker = new PerformanceTracker({}, dataCollector);
    debateTracker = new DebateOutcomeTracker({}, performanceTracker);
    verdictScorer = new VerdictQualityScorer();
    deploymentManager = new ModelDeploymentManager({}, performanceTracker);
  });

  describe("Debate Outcome Tracking", () => {
    it("should track arbitration session and extract training data", async () => {
      debateTracker.startTracking();
      performanceTracker.startCollection();

      const now = new Date();
      const session: ArbitrationSession = {
        id: "session-001",
        state: ArbitrationState.COMPLETED,
        violation: {
          id: "violation-001",
          ruleId: "RULE-001",
          severity: ViolationSeverity.MODERATE,
          description: "Test violation for RL training",
          evidence: ["evidence-1"],
          detectedAt: now,
          context: { test: true },
        },
        rulesEvaluated: [
          {
            id: "RULE-001",
            version: "1.0",
            category: RuleCategory.TESTING,
            title: "Test Rule",
            description: "Test rule description",
            condition: "test === true",
            severity: ViolationSeverity.MODERATE,
            waivable: false,
            requiredEvidence: [],
            precedents: [],
            effectiveDate: now,
            metadata: {},
          },
        ],
        evidence: ["evidence-1"],
        participants: ["agent-1", "agent-2"],
        precedents: [],
        startTime: now,
        verdict: {
          id: "verdict-001",
          sessionId: "session-001",
          outcome: VerdictOutcome.APPROVED,
          reasoning: createReasoningSteps([
            "Action was necessary and justified",
            "Aligns with constitutional principles",
          ]),
          rulesApplied: ["RULE-001"],
          evidence: ["evidence-1"],
          precedents: [],
          confidence: 0.85,
          issuedBy: "arbiter-001",
          issuedAt: now,
          auditLog: [],
        },
        metadata: {
          totalDurationMs: 5000,
        },
      };

      // Record outcome without debate session (arbitration-only)
      const outcome = await debateTracker.recordArbitrationOutcome(session);

      expect(outcome.id).toBeDefined();
      expect(outcome.sessionId).toBe("session-001");
      expect(outcome.participants).toHaveLength(2);
      expect(outcome.qualityScore).toBeGreaterThan(0);
      expect(outcome.metrics.complianceScore).toBe(0.85);
    });

    it("should export outcomes with quality filtering", async () => {
      debateTracker.startTracking();
      performanceTracker.startCollection();

      const now = new Date();
      // Create high-quality outcome
      const highQualitySession: ArbitrationSession = {
        id: "high-quality",
        state: ArbitrationState.COMPLETED,
        violation: {
          id: "violation-high",
          ruleId: "RULE-002",
          severity: ViolationSeverity.MINOR,
          description: "High quality test",
          evidence: ["evidence-2"],
          detectedAt: now,
          context: { test: true },
        },
        rulesEvaluated: [
          {
            id: "RULE-002",
            version: "2.0",
            category: RuleCategory.TESTING,
            title: "High Quality Rule",
            description: "High quality rule description",
            condition: "test === true",
            severity: ViolationSeverity.MINOR,
            waivable: false,
            requiredEvidence: [],
            precedents: [],
            effectiveDate: now,
            metadata: {},
          },
        ],
        evidence: ["evidence-2"],
        participants: ["agent-1"],
        precedents: [],
        startTime: now,
        verdict: {
          id: "verdict-high",
          sessionId: "high-quality",
          outcome: VerdictOutcome.APPROVED,
          reasoning: createReasoningSteps([
            "Clear reasoning",
            "Strong evidence",
            "Constitutional alignment",
          ]),
          rulesApplied: ["RULE-002"],
          evidence: ["evidence-2"],
          precedents: [],
          confidence: 0.95,
          issuedBy: "arbiter-002",
          issuedAt: now,
          auditLog: [],
        },
        metadata: {
          totalDurationMs: 3000,
        },
      };

      await debateTracker.recordArbitrationOutcome(highQualitySession);

      // Export with quality filter
      const highQualityOutcomes = debateTracker.exportOutcomes(undefined, 0.7);
      expect(highQualityOutcomes.length).toBeGreaterThanOrEqual(0);

      // Export all outcomes
      const allOutcomes = debateTracker.exportOutcomes();
      expect(allOutcomes.length).toBeGreaterThanOrEqual(
        highQualityOutcomes.length
      );
    });
  });

  describe("Verdict Quality Scoring", () => {
    it("should score verdict quality across multiple criteria", async () => {
      const now = new Date();
      const verdict: Verdict = {
        id: "verdict-002",
        sessionId: "session-002",
        outcome: VerdictOutcome.APPROVED,
        reasoning: createReasoningSteps([
          "Clear logical reasoning provided",
          "Strong evidence supports the decision",
          "Aligns with constitutional principles",
        ]),
        rulesApplied: ["RULE-003"],
        evidence: ["evidence-3", "evidence-4"],
        precedents: ["precedent-1"],
        confidence: 0.92,
        issuedBy: "arbiter-003",
        issuedAt: now,
        auditLog: [],
      };

      const qualityEval = await verdictScorer.evaluateVerdict(verdict);

      expect(qualityEval.overallScore).toBeGreaterThan(0.5);
      expect(qualityEval.criteriaScores).toBeDefined();
      expect(qualityEval.criteriaScores.reasoningClarity).toBeGreaterThan(0);
      expect(qualityEval.criteriaScores.evidenceQuality).toBeGreaterThan(0);
      expect(
        qualityEval.criteriaScores.constitutionalCompliance
      ).toBeGreaterThan(0);
      expect(qualityEval.criteriaScores.fairness).toBeGreaterThan(0);
      expect(qualityEval.criteriaScores.actionability).toBeGreaterThan(0);
      expect(qualityEval.confidence).toBeGreaterThan(0.5);
      expect(qualityEval.feedback).toHaveLength(5);
    });

    it("should provide detailed feedback for each criterion", async () => {
      const now = new Date();
      const verdict: Verdict = {
        id: "verdict-003",
        sessionId: "session-003",
        outcome: VerdictOutcome.CONDITIONAL,
        reasoning: createReasoningSteps(["Basic reasoning"]),
        rulesApplied: ["RULE-004"],
        evidence: [],
        precedents: [],
        conditions: ["Condition 1"],
        confidence: 0.6,
        issuedBy: "arbiter-004",
        issuedAt: now,
        auditLog: [],
      };

      const qualityEval = await verdictScorer.evaluateVerdict(verdict);

      expect(qualityEval.assessmentReasoning).toContain("verdict");
      qualityEval.feedback.forEach((fb) => {
        expect(fb.criterion).toBeDefined();
        expect(fb.score).toBeGreaterThanOrEqual(0);
        expect(fb.score).toBeLessThanOrEqual(1);
        expect(fb.reasoning).toBeDefined();
      });
    });
  });

  describe("Model Deployment and A/B Testing", () => {
    it("should register and manage model versions", async () => {
      const version = {
        id: "model-v1.0.0",
        modelName: "agent-model",
        version: "1.0.0",
        trainingMetrics: {
          averageReward: 0.75,
          trainingTimeMs: 10000,
          trajectoriesProcessed: 1000,
          klDivergence: 0.01,
        },
        status: "production" as const,
        deployedAt: new Date().toISOString(),
        performanceBaseline: {
          averageReward: 0.75,
          successRate: 0.85,
          latencyMs: 250,
          errorRate: 0.02,
        },
      };

      deploymentManager.registerVersion(version);
      const versions = deploymentManager.getVersions();
      expect(versions).toHaveLength(1);
      expect(versions[0].version).toBe("1.0.0");
    });

    it("should start A/B test with control and treatment", async () => {
      const controlVersion = {
        id: "model-v1.0.0",
        modelName: "agent-model",
        version: "1.0.0",
        trainingMetrics: {
          averageReward: 0.75,
          trainingTimeMs: 10000,
          trajectoriesProcessed: 1000,
          klDivergence: 0.01,
        },
        status: "production" as const,
        deployedAt: new Date().toISOString(),
      };

      const treatmentVersion = {
        id: "model-v1.1.0",
        modelName: "agent-model",
        version: "1.1.0",
        trainingMetrics: {
          averageReward: 0.78,
          trainingTimeMs: 12000,
          trajectoriesProcessed: 1200,
          klDivergence: 0.012,
        },
        status: "staging" as const,
        deployedAt: new Date().toISOString(),
      };

      deploymentManager.registerVersion(controlVersion);
      deploymentManager.registerVersion(treatmentVersion);

      const testConfig = {
        name: "model-v1.1.0-test",
        controlVersion: "model-v1.0.0",
        treatmentVersion: "model-v1.1.0",
        trafficPercentage: 10,
        durationMs: 3600000,
        minSampleSize: 100,
        promotionThresholds: {
          minSuccessRate: 0.85,
          maxLatencyIncreasePercent: 20,
          maxErrorRateIncreasePercent: 10,
          minRewardImprovement: 2,
        },
        rollbackThresholds: {
          maxSuccessRateDecreasePercent: 5,
          maxLatencyIncreasePercent: 50,
          maxErrorRatePercent: 5,
          maxRewardDecreasePercent: 10,
        },
      };

      const testId = await deploymentManager.startABTest(testConfig);
      expect(testId).toBe("model-v1.1.0-test");

      const activeTests = deploymentManager.getActiveTests();
      expect(activeTests).toHaveLength(1);
    });

    it("should evaluate A/B test and provide recommendation", async () => {
      const controlVersion = {
        id: "model-v2.0.0",
        modelName: "agent-model",
        version: "2.0.0",
        trainingMetrics: {
          averageReward: 0.8,
          trainingTimeMs: 15000,
          trajectoriesProcessed: 1500,
          klDivergence: 0.015,
        },
        status: "production" as const,
        deployedAt: new Date().toISOString(),
      };

      const treatmentVersion = {
        id: "model-v2.1.0",
        modelName: "agent-model",
        version: "2.1.0",
        trainingMetrics: {
          averageReward: 0.82,
          trainingTimeMs: 16000,
          trajectoriesProcessed: 1600,
          klDivergence: 0.014,
        },
        status: "staging" as const,
        deployedAt: new Date().toISOString(),
      };

      deploymentManager.registerVersion(controlVersion);
      deploymentManager.registerVersion(treatmentVersion);

      const testConfig = {
        name: "model-v2.1.0-test",
        controlVersion: "model-v2.0.0",
        treatmentVersion: "model-v2.1.0",
        trafficPercentage: 10,
        durationMs: 3600000,
        minSampleSize: 100,
        promotionThresholds: {
          minSuccessRate: 0.85,
          maxLatencyIncreasePercent: 20,
          maxErrorRateIncreasePercent: 10,
          minRewardImprovement: 2,
        },
        rollbackThresholds: {
          maxSuccessRateDecreasePercent: 5,
          maxLatencyIncreasePercent: 50,
          maxErrorRatePercent: 5,
          maxRewardDecreasePercent: 10,
        },
      };

      await deploymentManager.startABTest(testConfig);
      const testResult = await deploymentManager.evaluateABTest(
        "model-v2.1.0-test"
      );

      expect(testResult.testName).toBe("model-v2.1.0-test");
      expect(testResult.controlMetrics).toBeDefined();
      expect(testResult.treatmentMetrics).toBeDefined();
      expect(testResult.statisticalSignificance).toBeDefined();
      expect(testResult.recommendation).toBeDefined();
      expect(testResult.reasoning).toBeDefined();
    });

    it("should handle model rollback", async () => {
      const oldVersion = {
        id: "model-v3.0.0",
        modelName: "agent-model",
        version: "3.0.0",
        trainingMetrics: {
          averageReward: 0.75,
          trainingTimeMs: 10000,
          trajectoriesProcessed: 1000,
          klDivergence: 0.01,
        },
        status: "archived" as const,
        deployedAt: new Date(Date.now() - 172800000).toISOString(),
      };

      const currentVersion = {
        id: "model-v3.1.0",
        modelName: "agent-model",
        version: "3.1.0",
        trainingMetrics: {
          averageReward: 0.78,
          trainingTimeMs: 12000,
          trajectoriesProcessed: 1200,
          klDivergence: 0.012,
        },
        status: "production" as const,
        deployedAt: new Date().toISOString(),
      };

      deploymentManager.registerVersion(oldVersion);
      deploymentManager.registerVersion(currentVersion);

      await deploymentManager.rollback(
        "model-v3.1.0",
        "model-v3.0.0",
        "Performance degradation detected"
      );

      const versions = deploymentManager.getVersions();
      const rolledBackVersion = versions.find((v) => v.id === "model-v3.1.0");
      expect(rolledBackVersion?.status).toBe("rolled_back");
      expect(rolledBackVersion?.rolledBackTo).toBe("model-v3.0.0");

      const restoredVersion = versions.find((v) => v.id === "model-v3.0.0");
      expect(restoredVersion?.status).toBe("production");
    });
  });

  describe("Pipeline Statistics", () => {
    it("should track and report debate outcome statistics", async () => {
      debateTracker.startTracking();

      for (let i = 0; i < 3; i++) {
        const now = new Date();
        const session: ArbitrationSession = {
          id: `session-stats-${i}`,
          state: ArbitrationState.COMPLETED,
          violation: {
            id: `violation-${i}`,
            ruleId: `RULE-${i}`,
            severity: ViolationSeverity.MINOR,
            description: "Test",
            evidence: [],
            detectedAt: now,
            context: { test: true },
          },
          rulesEvaluated: [
            {
              id: `RULE-${i}`,
              version: `${i}.0`,
              category: RuleCategory.TESTING,
              title: "Test Rule",
              description: "Test rule description",
              condition: "test === true",
              severity: ViolationSeverity.MINOR,
              waivable: false,
              requiredEvidence: [],
              precedents: [],
              effectiveDate: now,
              metadata: {},
            },
          ],
          evidence: [],
          participants: ["agent-1"],
          precedents: [],
          startTime: now,
          verdict: {
            id: `verdict-${i}`,
            sessionId: `session-stats-${i}`,
            outcome: VerdictOutcome.APPROVED,
            reasoning: createReasoningSteps(["Reasoning"]),
            rulesApplied: [`RULE-${i}`],
            evidence: [],
            precedents: [],
            confidence: 0.8,
            issuedBy: "arbiter-005",
            issuedAt: now,
            auditLog: [],
          },
          metadata: {
            totalDurationMs: 1000 * (i + 1),
          },
        };

        await debateTracker.recordArbitrationOutcome(session);
      }

      const stats = debateTracker.getStats();
      expect(stats.totalOutcomes).toBe(3);
      expect(stats.averageQualityScore).toBeGreaterThan(0);
      expect(stats.isTracking).toBe(true);
    });
  });
});
