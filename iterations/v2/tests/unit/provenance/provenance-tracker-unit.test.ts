/**
 * Unit tests for ProvenanceTracker
 *
 * Tests core functionality with mocked dependencies.
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { ProvenanceTracker } from "../../../src/provenance/ProvenanceTracker.js";
import type {
  AIAttribution,
  ProvenanceTrackerConfig,
} from "../../../src/provenance/types/provenance-types.js";
import type { WorkingSpec } from "../../../src/types/caws-types.js";

describe("ProvenanceTracker Unit Tests", () => {
  let tracker: ProvenanceTracker;
  let mockStorage: any;

  const validSpec: WorkingSpec = {
    id: "UNIT-001",
    title: "Unit Test Spec",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["test"],
      data_migration: false,
    },
    operational_rollback_slo: "15m",
    scope: {
      in: ["src/"],
      out: ["node_modules/"],
    },
    invariants: ["Test invariant"],
    acceptance: [
      {
        id: "A1",
        given: "Test",
        when: "Test",
        then: "Test",
      },
    ],
    contracts: [],
    non_functional: {},
  };

  const config: ProvenanceTrackerConfig = {
    projectRoot: "/test/project",
    spec: validSpec,
    enableAIAttribution: false, // Disable for most tests
    storage: {
      type: "memory",
    },
  };

  beforeEach(() => {
    // Create mock storage
    mockStorage = {
      storeEntry: jest
        .fn<() => Promise<void>>()
        .mockResolvedValue(undefined as any),
      storeAttribution: jest
        .fn<() => Promise<void>>()
        .mockResolvedValue(undefined as any),
      getProvenanceChain: jest.fn<() => Promise<any>>().mockResolvedValue({
        spec: validSpec,
        entries: [],
        currentHash: "test-hash",
        metadata: {},
      } as any),
      storeProvenanceChain: jest
        .fn<() => Promise<void>>()
        .mockResolvedValue(undefined as any),
      getAttributions: jest
        .fn<() => Promise<any[]>>()
        .mockResolvedValue([] as any),
      cleanup: jest.fn<() => Promise<any>>().mockResolvedValue({
        entriesRemoved: 0,
        attributionsRemoved: 0,
      } as any),
    };

    tracker = new ProvenanceTracker(config);
    // Replace storage with mock
    (tracker as any).storage = mockStorage;
  });

  describe("Initialization", () => {
    it("should create tracker with valid config", () => {
      expect(tracker).toBeDefined();
      expect(tracker).toBeInstanceOf(ProvenanceTracker);
    });

    it("should report correct capabilities", () => {
      const capabilities = tracker.getCapabilities();

      expect(capabilities.trackAIAttribution).toBe(false);
      expect(capabilities.trackHumanContributions).toBe(true);
      expect(capabilities.integrateWithCAWS).toBe(false);
      expect(capabilities.verifyIntegrity).toBe(false);
      expect(capabilities.generateReports).toBe(true);
    });

    it("should report capabilities with AI attribution enabled", () => {
      const aiConfig = { ...config, enableAIAttribution: true };
      const aiTracker = new ProvenanceTracker(aiConfig);

      const capabilities = aiTracker.getCapabilities();
      expect(capabilities.trackAIAttribution).toBe(true);
    });
  });

  describe("Entry Recording", () => {
    it("should record a basic provenance entry", async () => {
      const entry = await tracker.recordEntry(
        "commit",
        validSpec.id,
        { type: "human", identifier: "test@example.com" },
        { type: "committed", description: "Test commit" }
      );

      expect(entry).toBeDefined();
      expect(entry.type).toBe("commit");
      expect(entry.specId).toBe(validSpec.id);
      expect(entry.actor.type).toBe("human");
      expect(entry.action.description).toBe("Test commit");
      expect(mockStorage.storeEntry).toHaveBeenCalledTimes(1);
    });

    it("should generate unique IDs for entries", async () => {
      const entry1 = await tracker.recordEntry(
        "commit",
        validSpec.id,
        { type: "human", identifier: "user1" },
        { type: "committed", description: "Entry 1" }
      );

      const entry2 = await tracker.recordEntry(
        "commit",
        validSpec.id,
        { type: "human", identifier: "user2" },
        { type: "committed", description: "Entry 2" }
      );

      expect(entry1.id).toBeDefined();
      expect(entry2.id).toBeDefined();
      expect(entry1.id).not.toBe(entry2.id);
    });

    it("should include timestamp in entries", async () => {
      const before = Date.now();
      const entry = await tracker.recordEntry(
        "commit",
        validSpec.id,
        { type: "human", identifier: "test" },
        { type: "committed", description: "Test" }
      );
      const after = Date.now();
      const entryTime = new Date(entry.timestamp).getTime();

      expect(entry.timestamp).toBeDefined();
      expect(entryTime).toBeGreaterThanOrEqual(before);
      expect(entryTime).toBeLessThanOrEqual(after);
    });

    it("should accept optional commit hash", async () => {
      const entry = await tracker.recordEntry(
        "commit",
        validSpec.id,
        { type: "human", identifier: "test" },
        { type: "committed", description: "Test" },
        { commitHash: "abc123" }
      );

      expect(entry.commitHash).toBe("abc123");
    });

    it("should accept optional affected files", async () => {
      const affectedFiles = [
        {
          path: "src/test.ts",
          changeType: "modified" as const,
          linesChanged: 10,
        },
      ];

      const entry = await tracker.recordEntry(
        "commit",
        validSpec.id,
        { type: "human", identifier: "test" },
        { type: "committed", description: "Test" },
        { affectedFiles }
      );

      expect(entry.affectedFiles).toEqual(affectedFiles);
    });

    it("should accept optional quality metrics", async () => {
      const qualityMetrics = { testCoverage: 85, lintErrors: 0 };

      const entry = await tracker.recordEntry(
        "validation",
        validSpec.id,
        { type: "ai", identifier: "validator" },
        { type: "validated", description: "Test" },
        { qualityMetrics }
      );

      expect(entry.qualityMetrics).toBeDefined();
      expect(entry.qualityMetrics?.testCoverage).toBe(85);
    });
  });

  describe("AI Attribution Recording", () => {
    it("should record AI attribution manually", async () => {
      const attribution = await tracker.recordAIAttribution(
        "cursor-composer",
        "1.0.0",
        [{ file: "test.ts", startLine: 1, endLine: 10 }],
        "high"
      );

      expect(attribution).toBeDefined();
      expect(attribution.toolType).toBe("cursor-composer");
      expect(attribution.toolVersion).toBe("1.0.0");
      expect(attribution.confidence).toBe("high");
      expect(mockStorage.storeAttribution).toHaveBeenCalledTimes(1);
    });

    it("should generate unique IDs for attributions", async () => {
      const attr1 = await tracker.recordAIAttribution("cursor-composer");
      const attr2 = await tracker.recordAIAttribution("github-copilot");

      expect(attr1.id).toBeDefined();
      expect(attr2.id).toBeDefined();
      expect(attr1.id).not.toBe(attr2.id);
    });

    it("should use default medium confidence if not specified", async () => {
      const attribution = await tracker.recordAIAttribution("cursor-composer");

      expect(attribution.confidence).toBe("medium");
    });

    it("should accept optional metadata", async () => {
      const metadata = { source: "test", version: "1.0" };
      const attribution = await tracker.recordAIAttribution(
        "cursor-composer",
        undefined,
        undefined,
        "high",
        metadata
      );

      expect(attribution.metadata).toEqual(metadata);
    });
  });

  describe("Provenance Chain Management", () => {
    it("should retrieve provenance chain", async () => {
      const chain = await tracker.getProvenanceChain(validSpec.id);

      expect(chain).toBeDefined();
      expect(chain!.spec.id).toBe(validSpec.id);
      expect(mockStorage.getProvenanceChain).toHaveBeenCalledWith(validSpec.id);
    });

    it("should return null for non-existent chain", async () => {
      mockStorage.getProvenanceChain.mockResolvedValue(null);

      const chain = await tracker.getProvenanceChain("NON-EXISTENT");

      expect(chain).toBeNull();
    });
  });

  describe("AI Attribution Statistics", () => {
    it("should calculate AI attribution stats", async () => {
      const mockAttributions: AIAttribution[] = [
        {
          id: "1",
          toolType: "cursor-composer",
          confidence: "high",
          timestamp: new Date().toISOString(),
        },
        {
          id: "2",
          toolType: "github-copilot",
          confidence: "medium",
          timestamp: new Date().toISOString(),
        },
        {
          id: "3",
          toolType: "cursor-composer",
          confidence: "high",
          timestamp: new Date().toISOString(),
        },
      ];

      mockStorage.getAttributions.mockResolvedValue(mockAttributions);

      const stats = await tracker.getAIAttributionStats();

      expect(stats.total).toBe(3);
      expect(stats.byToolType["cursor-composer"]).toBe(2);
      expect(stats.byToolType["github-copilot"]).toBe(1);
      expect(stats.byConfidence.high).toBe(2);
      expect(stats.byConfidence.medium).toBe(1);
    });

    it("should handle empty attribution stats", async () => {
      mockStorage.getAttributions.mockResolvedValue([]);

      const stats = await tracker.getAIAttributionStats();

      expect(stats.total).toBe(0);
      expect(stats.topTools).toHaveLength(0);
    });

    it("should filter attributions by date range", async () => {
      const start = "2025-01-01T00:00:00Z";
      const end = "2025-12-31T23:59:59Z";

      await tracker.getAIAttributionStats(start, end);

      expect(mockStorage.getAttributions).toHaveBeenCalledWith(start, end);
    });
  });

  describe("Report Generation", () => {
    beforeEach(() => {
      mockStorage.getProvenanceChain.mockResolvedValue({
        spec: validSpec,
        entries: [
          {
            id: "1",
            type: "commit",
            specId: validSpec.id,
            timestamp: new Date().toISOString(),
            actor: { type: "human", identifier: "dev1" },
            action: { type: "committed", description: "Test" },
          },
        ],
        currentHash: "test-hash",
        metadata: {},
      });
    });

    it("should generate summary report", async () => {
      const report = await tracker.generateReport(validSpec.id, "summary");

      expect(report).toBeDefined();
      expect(report.type).toBe("summary");
      expect(report.spec.id).toBe(validSpec.id);
      expect(report.id).toBeDefined();
      expect(report.generatedAt).toBeDefined();
    });

    it("should generate detailed report", async () => {
      const report = await tracker.generateReport(validSpec.id, "detailed");

      expect(report.type).toBe("detailed");
    });

    it("should generate compliance report", async () => {
      const report = await tracker.generateReport(validSpec.id, "compliance");

      expect(report.type).toBe("compliance");
    });

    it("should generate audit report", async () => {
      const report = await tracker.generateReport(validSpec.id, "audit");

      expect(report.type).toBe("audit");
    });

    it("should filter report by period", async () => {
      const period = {
        start: "2025-01-01T00:00:00Z",
        end: "2025-12-31T23:59:59Z",
      };

      const report = await tracker.generateReport(
        validSpec.id,
        "summary",
        period
      );

      expect(report.period).toEqual(period);
    });
  });

  describe("Pattern Analysis", () => {
    it("should analyze patterns for empty chain", async () => {
      mockStorage.getProvenanceChain.mockResolvedValue({
        spec: validSpec,
        entries: [],
        currentHash: "test-hash",
        metadata: {},
      });

      const analysis = await tracker.analyzePatterns(validSpec.id);

      expect(analysis).toBeDefined();
      expect(analysis.aiVsHumanBalance).toBe(0);
      expect(analysis.productivityTrends).toEqual([]);
      expect(analysis.qualityCorrelation.aiUsage).toBe(0);
    });

    it("should analyze patterns with entries", async () => {
      mockStorage.getProvenanceChain.mockResolvedValue({
        spec: validSpec,
        entries: [
          {
            id: "1",
            type: "commit",
            specId: validSpec.id,
            timestamp: new Date().toISOString(),
            actor: { type: "human", identifier: "dev1" },
            action: { type: "committed", description: "Human commit" },
          },
          {
            id: "2",
            type: "commit",
            specId: validSpec.id,
            timestamp: new Date().toISOString(),
            actor: { type: "human", identifier: "dev2" },
            action: { type: "committed", description: "AI commit" },
            aiAttributions: [
              {
                id: "a1",
                toolType: "cursor-composer",
                confidence: "high",
                timestamp: new Date().toISOString(),
              },
            ],
          },
        ],
        currentHash: "test-hash",
        metadata: {},
      });

      const analysis = await tracker.analyzePatterns(validSpec.id);

      expect(analysis.aiVsHumanBalance).toBeGreaterThan(0);
      expect(analysis.qualityCorrelation).toBeDefined();
    });

    it("should return empty analysis for non-existent chain", async () => {
      mockStorage.getProvenanceChain.mockResolvedValue(null);

      const analysis = await tracker.analyzePatterns(validSpec.id);

      expect(analysis.aiVsHumanBalance).toBe(0);
    });
  });

  describe("Integrity Verification", () => {
    it("should verify integrity of valid chain", async () => {
      mockStorage.getProvenanceChain.mockResolvedValue({
        spec: validSpec,
        entries: [
          {
            id: "1",
            type: "commit",
            specId: validSpec.id,
            timestamp: new Date().toISOString(),
            actor: { type: "human", identifier: "dev1" },
            action: { type: "committed", description: "Test" },
          },
        ],
        currentHash: "valid-hash",
        metadata: {},
      });

      const result = await tracker.verifyIntegrity(validSpec.id);

      expect(result).toBeDefined();
      expect(result.verified).toBe(true);
    });

    it("should return unverified for non-existent chain", async () => {
      mockStorage.getProvenanceChain.mockResolvedValue(null);

      const result = await tracker.verifyIntegrity(validSpec.id);

      expect(result.verified).toBe(false);
      expect(result.issues).toContain("No provenance chain found");
    });
  });

  // Note: Cleanup operations are handled by storage layer directly

  describe("Event Emission", () => {
    it("should emit entry:added event when recording entry", async () => {
      const listener = jest.fn();
      tracker.on("entry:added", listener);

      await tracker.recordEntry(
        "commit",
        validSpec.id,
        { type: "human", identifier: "test" },
        { type: "committed", description: "Test" }
      );

      expect(listener).toHaveBeenCalledTimes(1);
      expect(listener.mock.calls[0][0]).toMatchObject({
        type: "commit",
        specId: validSpec.id,
      });
    });

    it("should emit attribution:recorded event when recording attribution", async () => {
      const listener = jest.fn();
      tracker.on("attribution:recorded", listener);

      await tracker.recordAIAttribution("cursor-composer");

      expect(listener).toHaveBeenCalledTimes(1);
      expect(listener.mock.calls[0][0]).toMatchObject({
        toolType: "cursor-composer",
      });
    });

    it("should emit report:generated event when generating report", async () => {
      const listener = jest.fn();
      tracker.on("report:generated", listener);

      mockStorage.getProvenanceChain.mockResolvedValue({
        id: validSpec.id,
        entries: [],
        currentHash: "test-hash",
        metadata: {},
      });

      await tracker.generateReport(validSpec.id);

      expect(listener).toHaveBeenCalledTimes(1);
      expect(listener.mock.calls[0][0]).toMatchObject({
        specId: validSpec.id,
        type: "summary",
      });
    });
  });

  describe("Error Handling", () => {
    it("should propagate storage errors", async () => {
      mockStorage.storeEntry.mockRejectedValue(new Error("Storage error"));

      await expect(
        tracker.recordEntry(
          "commit",
          validSpec.id,
          { type: "human", identifier: "test" },
          { type: "committed", description: "Test" }
        )
      ).rejects.toThrow("Storage error");
    });

    it("should handle attribution storage errors", async () => {
      mockStorage.storeAttribution.mockRejectedValue(
        new Error("Attribution error")
      );

      await expect(
        tracker.recordAIAttribution("cursor-composer")
      ).rejects.toThrow("Attribution error");
    });
  });

  describe("Lifecycle Management", () => {
    it("should stop tracker cleanly", () => {
      expect(() => tracker.stop()).not.toThrow();
    });

    it("should allow removing all listeners", () => {
      tracker.on("entry:added", jest.fn());
      expect(() => tracker.removeAllListeners()).not.toThrow();
    });
  });
});
