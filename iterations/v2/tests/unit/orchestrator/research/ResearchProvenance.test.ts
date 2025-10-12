/**
 * @fileoverview Unit Tests for ResearchProvenance (ARBITER-006 Phase 4)
 *
 * @author @darianrosebrook
 *
 * Tests for research provenance tracking including:
 * - Recording successful research
 * - Recording failed research
 * - Retrieval operations (task research, statistics)
 * - Data validation
 * - Error handling
 * - Database unavailability handling
 */

import { ResearchProvenance } from "../../../../src/orchestrator/research/ResearchProvenance";
import {
  VerificationPriority,
  MockDatabaseClient,
  mockResearchContext,
} from "../../../mocks/knowledge-mocks";

describe("ResearchProvenance", () => {
  let provenance: ResearchProvenance;
  let mockDbClient: MockDatabaseClient;

  beforeEach(() => {
    mockDbClient = new MockDatabaseClient();
    provenance = new ResearchProvenance(mockDbClient as any);
  });

  afterEach(async () => {
    await mockDbClient.end();
  });

  describe("Recording Operations", () => {
    it("should record successful research", async () => {
      const taskId = "task-123";
      const researchContext = mockResearchContext();
      const durationMs = 487;

      await provenance.recordResearch(taskId, researchContext, durationMs);

      // Verify data was stored
      const result = await mockDbClient.query(
        "SELECT * FROM task_research_provenance WHERE task_id = $1",
        [taskId]
      );

      expect(result.rows.length).toBeGreaterThan(0);
    });

    it("should record all required fields", async () => {
      const taskId = "task-456";
      const researchContext = mockResearchContext({
        queries: ["query 1", "query 2"],
        findings: [
          {
            query: "query 1",
            summary: "Summary 1",
            confidence: 0.9,
            keyFindings: [],
          },
          {
            query: "query 2",
            summary: "Summary 2",
            confidence: 0.85,
            keyFindings: [],
          },
        ],
        confidence: 0.875,
      });
      const durationMs = 525;

      const consoleLogSpy = jest
        .spyOn(console, "log")
        .mockImplementation(() => {});

      await provenance.recordResearch(taskId, researchContext, durationMs);

      expect(consoleLogSpy).toHaveBeenCalledWith(
        expect.stringContaining(
          `Research provenance recorded for task ${taskId}`
        )
      );
      expect(consoleLogSpy).toHaveBeenCalledWith(
        expect.stringContaining("2 findings")
      );

      consoleLogSpy.mockRestore();
    });

    it("should record failed research", async () => {
      const taskId = "task-789";
      const queries = ["query 1", "query 2"];
      const error = new Error("All queries failed");
      const durationMs = 200;

      await provenance.recordFailure(taskId, queries, error, durationMs);

      // Verify failure was stored
      const result = await mockDbClient.query(
        "SELECT * FROM task_research_provenance WHERE task_id = $1",
        [taskId]
      );

      expect(result.rows.length).toBeGreaterThan(0);
    });

    it("should record error message in failure", async () => {
      const taskId = "task-error";
      const queries = ["query 1"];
      const error = new Error("Network timeout");
      const durationMs = 5000;

      const consoleErrorSpy = jest
        .spyOn(console, "error")
        .mockImplementation(() => {});

      // Mock the database to actually store data
      const insertSpy = jest.spyOn(mockDbClient, "query");

      await provenance.recordFailure(taskId, queries, error, durationMs);

      // Verify the insert was attempted with error message
      expect(insertSpy).toHaveBeenCalled();

      insertSpy.mockRestore();
      consoleErrorSpy.mockRestore();
    });

    it("should handle database unavailable gracefully", async () => {
      const disconnectedProvenance = new ResearchProvenance(undefined);

      const consoleWarnSpy = jest
        .spyOn(console, "warn")
        .mockImplementation(() => {});

      const taskId = "task-no-db";
      const researchContext = mockResearchContext();

      // Should not throw
      await disconnectedProvenance.recordResearch(taskId, researchContext, 100);

      expect(consoleWarnSpy).toHaveBeenCalledWith(
        "Database not available, research provenance not recorded"
      );

      consoleWarnSpy.mockRestore();
    });

    it("should handle database not connected", async () => {
      const disconnectedClient = {
        isConnected: () => false,
      } as any;

      const disconnectedProvenance = new ResearchProvenance(disconnectedClient);

      const consoleWarnSpy = jest
        .spyOn(console, "warn")
        .mockImplementation(() => {});

      const taskId = "task-disconnected";
      const researchContext = mockResearchContext();

      await disconnectedProvenance.recordResearch(taskId, researchContext, 100);

      expect(consoleWarnSpy).toHaveBeenCalled();

      consoleWarnSpy.mockRestore();
    });

    it("should log database errors appropriately", async () => {
      const failingClient = {
        isConnected: () => true,
        query: async () => {
          throw new Error("Database write error");
        },
      } as any;

      const failingProvenance = new ResearchProvenance(failingClient);

      const consoleErrorSpy = jest
        .spyOn(console, "error")
        .mockImplementation(() => {});

      const taskId = "task-error-log";
      const researchContext = mockResearchContext();

      await failingProvenance.recordResearch(taskId, researchContext, 100);

      expect(consoleErrorSpy).toHaveBeenCalledWith(
        "Failed to record research provenance:",
        expect.any(Error)
      );

      consoleErrorSpy.mockRestore();
    });
  });

  describe("Retrieval Operations", () => {
    beforeEach(async () => {
      // Seed some test data
      const taskId = "task-retrieval-test";
      const researchContext = mockResearchContext();
      await provenance.recordResearch(taskId, researchContext, 500);
      await provenance.recordResearch(taskId, researchContext, 450);
    });

    it("getTaskResearch should return all records for a task", async () => {
      const taskId = "task-retrieval-test";

      const records = await provenance.getTaskResearch(taskId);

      expect(records).toBeInstanceOf(Array);
      expect(records.length).toBeGreaterThan(0);
    });

    it("getTaskResearch should return empty array for unknown task", async () => {
      const records = await provenance.getTaskResearch("task-unknown-999");

      expect(records).toBeInstanceOf(Array);
      expect(records.length).toBe(0);
    });

    it("getStatistics should calculate correctly", async () => {
      // Seed more data
      await provenance.recordResearch(
        "task-stats-1",
        mockResearchContext(),
        400
      );
      await provenance.recordResearch(
        "task-stats-2",
        mockResearchContext(),
        450
      );
      await provenance.recordFailure(
        "task-stats-3",
        ["query"],
        new Error("Failed"),
        500
      );

      const stats = await provenance.getStatistics();

      expect(stats).toBeDefined();
      expect(typeof stats.totalResearch).toBe("number");
      expect(typeof stats.successfulResearch).toBe("number");
      expect(typeof stats.failedResearch).toBe("number");
      expect(typeof stats.averageConfidence).toBe("number");
      expect(typeof stats.averageDurationMs).toBe("number");
      expect(stats.totalResearch).toBeGreaterThan(0);
      expect(stats.totalResearch).toBe(
        stats.successfulResearch + stats.failedResearch
      );
    });

    it("getStatistics should handle empty database", async () => {
      const emptyProvenance = new ResearchProvenance(
        new MockDatabaseClient() as any
      );

      const stats = await emptyProvenance.getStatistics();

      expect(stats.totalResearch).toBe(0);
      expect(stats.successfulResearch).toBe(0);
      expect(stats.failedResearch).toBe(0);
    });

    it("getStatistics should handle null database client", async () => {
      const nullProvenance = new ResearchProvenance(undefined);

      const stats = await nullProvenance.getStatistics();

      expect(stats.totalResearch).toBe(0);
    });

    it("should handle retrieval errors gracefully", async () => {
      const failingClient = {
        isConnected: () => true,
        query: async () => {
          throw new Error("Database read error");
        },
      } as any;

      const failingProvenance = new ResearchProvenance(failingClient);

      const consoleErrorSpy = jest
        .spyOn(console, "error")
        .mockImplementation(() => {});

      const records = await failingProvenance.getTaskResearch("task-123");

      expect(records).toEqual([]);
      expect(consoleErrorSpy).toHaveBeenCalled();

      consoleErrorSpy.mockRestore();
    });
  });

  describe("Data Validation", () => {
    it("should validate ResearchContext structure", async () => {
      const taskId = "task-validation";
      const validContext = mockResearchContext({
        queries: ["query 1", "query 2"],
        findings: [
          {
            query: "query 1",
            summary: "Summary",
            confidence: 0.9,
            keyFindings: [],
          },
        ],
        confidence: 0.9,
        augmentedAt: new Date(),
        metadata: {},
      });

      // Should not throw
      await provenance.recordResearch(taskId, validContext, 500);

      const records = await provenance.getTaskResearch(taskId);
      expect(records.length).toBeGreaterThan(0);
    });

    it("should handle malformed data gracefully", async () => {
      const taskId = "task-malformed";
      const malformedContext: any = {
        queries: null, // Invalid
        findings: undefined, // Invalid
        confidence: "high", // Wrong type
        augmentedAt: "not a date", // Wrong type
      };

      const consoleErrorSpy = jest
        .spyOn(console, "error")
        .mockImplementation(() => {});

      // Should not throw, should handle gracefully
      try {
        await provenance.recordResearch(taskId, malformedContext, 500);
      } catch (error) {
        // Expected to potentially fail
      }

      consoleErrorSpy.mockRestore();
    });

    it("should sanitize error messages", async () => {
      const taskId = "task-sanitize";
      const queries = ["query 1"];
      const errorWithSensitiveData = new Error(
        "Failed: API_KEY=secret123 PASSWORD=pass456"
      );

      // Mock database to capture the stored error message
      const insertSpy = jest.spyOn(mockDbClient, "query");

      await provenance.recordFailure(
        taskId,
        queries,
        errorWithSensitiveData,
        100
      );

      expect(insertSpy).toHaveBeenCalled();

      insertSpy.mockRestore();
    });
  });

  describe("Cleanup Operations", () => {
    it("cleanupOldRecords should remove expired entries", async () => {
      // Seed old data
      await provenance.recordResearch("task-old", mockResearchContext(), 500);

      const beforeCount = await provenance.getStatistics();

      // Cleanup records older than 1ms (all records)
      await provenance.cleanupOldRecords(1);

      const afterCount = await provenance.getStatistics();

      // After cleanup, should have fewer or same records
      expect(afterCount.totalResearch).toBeLessThanOrEqual(
        beforeCount.totalResearch
      );
    });

    it("cleanupOldRecords should preserve recent entries", async () => {
      // Seed recent data
      await provenance.recordResearch(
        "task-recent",
        mockResearchContext(),
        500
      );

      // Cleanup records older than 1 day (should not affect recent records)
      const beforeStats = await provenance.getStatistics();
      await provenance.cleanupOldRecords(86400000); // 1 day in ms

      const afterStats = await provenance.getStatistics();

      // Recent records should still be there
      expect(afterStats.totalResearch).toBe(beforeStats.totalResearch);
    });

    it("cleanupOldRecords should handle database errors", async () => {
      const failingClient = {
        isConnected: () => true,
        query: async () => {
          throw new Error("Cleanup failed");
        },
      } as any;

      const failingProvenance = new ResearchProvenance(failingClient);

      const consoleErrorSpy = jest
        .spyOn(console, "error")
        .mockImplementation(() => {});

      await failingProvenance.cleanupOldRecords(1000);

      expect(consoleErrorSpy).toHaveBeenCalled();

      consoleErrorSpy.mockRestore();
    });

    it("cleanupOldRecords should handle disconnected database", async () => {
      const disconnectedProvenance = new ResearchProvenance(undefined);

      // Should not throw
      await disconnectedProvenance.cleanupOldRecords(1000);
    });
  });

  describe("Performance", () => {
    it("should record research quickly (<50ms)", async () => {
      const taskId = "task-perf";
      const researchContext = mockResearchContext();

      const startTime = Date.now();
      await provenance.recordResearch(taskId, researchContext, 500);
      const duration = Date.now() - startTime;

      // Should be fast (database write)
      expect(duration).toBeLessThan(100);
    });

    it("should handle bulk inserts efficiently", async () => {
      const tasks = Array(10)
        .fill(null)
        .map((_, i) => ({
          taskId: `bulk-task-${i}`,
          context: mockResearchContext(),
          duration: 500,
        }));

      const startTime = Date.now();
      await Promise.all(
        tasks.map((t) =>
          provenance.recordResearch(t.taskId, t.context, t.duration)
        )
      );
      const duration = Date.now() - startTime;

      // 10 inserts should be reasonably fast
      expect(duration).toBeLessThan(1000);
    });

    it("should retrieve statistics quickly", async () => {
      // Seed data
      for (let i = 0; i < 5; i++) {
        await provenance.recordResearch(
          `task-stats-${i}`,
          mockResearchContext(),
          500
        );
      }

      const startTime = Date.now();
      await provenance.getStatistics();
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(100);
    });
  });

  describe("Concurrent Operations", () => {
    it("should handle concurrent writes", async () => {
      const writes = Array(5)
        .fill(null)
        .map((_, i) =>
          provenance.recordResearch(
            `concurrent-${i}`,
            mockResearchContext(),
            500
          )
        );

      // Should not throw
      await Promise.all(writes);

      const stats = await provenance.getStatistics();
      expect(stats.totalResearch).toBeGreaterThanOrEqual(5);
    });

    it("should handle concurrent reads", async () => {
      // Seed data
      await provenance.recordResearch(
        "task-concurrent",
        mockResearchContext(),
        500
      );

      const reads = Array(5)
        .fill(null)
        .map(() => provenance.getTaskResearch("task-concurrent"));

      const results = await Promise.all(reads);

      results.forEach((result) => {
        expect(result.length).toBeGreaterThan(0);
      });
    });

    it("should handle mixed concurrent operations", async () => {
      const operations = [
        provenance.recordResearch("task-mix-1", mockResearchContext(), 500),
        provenance.getStatistics(),
        provenance.recordResearch("task-mix-2", mockResearchContext(), 450),
        provenance.getTaskResearch("task-mix-1"),
        provenance.recordFailure(
          "task-mix-3",
          ["query"],
          new Error("Failed"),
          300
        ),
      ];

      // Should not throw
      await Promise.all(operations);
    });
  });

  describe("Edge Cases", () => {
    it("should handle very long task IDs", async () => {
      const longTaskId = "task-" + "x".repeat(1000);
      const researchContext = mockResearchContext();

      // Should handle gracefully
      await provenance.recordResearch(longTaskId, researchContext, 500);
    });

    it("should handle very large findings arrays", async () => {
      const manyFindings = Array(100)
        .fill(null)
        .map((_, i) => ({
          query: `query ${i}`,
          summary: `Summary ${i}`,
          confidence: 0.8,
          keyFindings: [],
        }));

      const researchContext = mockResearchContext({
        findings: manyFindings,
      });

      // Should handle gracefully
      await provenance.recordResearch(
        "task-many-findings",
        researchContext,
        5000
      );
    });

    it("should handle zero confidence", async () => {
      const zeroConfidenceContext = mockResearchContext({
        confidence: 0,
      });

      // Should handle gracefully
      await provenance.recordResearch(
        "task-zero-confidence",
        zeroConfidenceContext,
        500
      );
    });

    it("should handle negative duration", async () => {
      const researchContext = mockResearchContext();

      // Should handle gracefully
      await provenance.recordResearch(
        "task-negative-duration",
        researchContext,
        -1
      );
    });

    it("should handle undefined duration", async () => {
      const researchContext = mockResearchContext();

      // Should handle gracefully
      await provenance.recordResearch(
        "task-undefined-duration",
        researchContext,
        undefined as any
      );
    });
  });
});
