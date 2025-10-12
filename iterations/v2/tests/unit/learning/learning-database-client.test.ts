/**
 * Unit Tests: LearningDatabaseClient
 *
 * Tests database client API surface, method signatures, and basic validation.
 * Comprehensive CRUD and transaction testing is covered in integration tests.
 *
 * @author @darianrosebrook
 */

import type { Pool } from "pg";
import { LearningDatabaseClient } from "../../../src/database/LearningDatabaseClient.js";

const createMockPool = (): jest.Mocked<Pool> => ({
  query: jest.fn(),
  connect: jest.fn(),
  end: jest.fn(),
  on: jest.fn(),
  removeListener: jest.fn(),
  release: jest.fn(),
  totalCount: 0,
  idleCount: 0,
  waitingCount: 0,
} as any);

describe("LearningDatabaseClient", () => {
  let mockPool: jest.Mocked<Pool>;
  let dbClient: LearningDatabaseClient;

  beforeEach(() => {
    mockPool = createMockPool();
    dbClient = new LearningDatabaseClient(mockPool);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("Initialization", () => {
    it("should create client with pool", () => {
      expect(dbClient).toBeInstanceOf(LearningDatabaseClient);
    });

    it("should expose pool for advanced operations", () => {
      const pool = dbClient.getPool();
      expect(pool).toBe(mockPool);
    });
  });

  describe("API Surface", () => {
    it("should have session CRUD methods", () => {
      expect(typeof dbClient.createSession).toBe("function");
      expect(typeof dbClient.getSession).toBe("function");
      expect(typeof dbClient.updateSession).toBe("function");
      expect(typeof dbClient.getSessionsByTask).toBe("function");
    });

    it("should have iteration CRUD methods", () => {
      expect(typeof dbClient.createIteration).toBe("function");
      expect(typeof dbClient.getIterations).toBe("function");
    });

    it("should have error pattern methods", () => {
      expect(typeof dbClient.upsertErrorPattern).toBe("function");
      expect(typeof dbClient.getErrorPatterns).toBe("function");
    });

    it("should have snapshot methods", () => {
      expect(typeof dbClient.saveSnapshot).toBe("function");
      expect(typeof dbClient.getSnapshot).toBe("function");
    });

    it("should have transaction support", () => {
      expect(typeof dbClient.transaction).toBe("function");
    });
  });

  describe("Session Methods", () => {
    it("createSession should call pool.query", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 1 });

      const session = {
        sessionId: "test",
        taskId: "task",
        agentId: "agent",
        status: "active" as any,
        config: {} as any,
        startTime: new Date(),
        iterationCount: 0,
        qualityScore: 0,
        improvementTrajectory: [],
        errorPatterns: [],
      };

      await dbClient.createSession(session);

      expect(mockPool.query).toHaveBeenCalled();
    });

    it("getSession should call pool.query with sessionId", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 0 } as any);

      const result = await dbClient.getSession("test-id");

      expect(mockPool.query).toHaveBeenCalledWith(
        expect.any(String),
        ["test-id"]
      );
      expect(result).toBeNull();
    });

    it("updateSession should call pool.query with updates", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 1 } as any);

      await dbClient.updateSession("test-id", { qualityScore: 0.9 });

      expect(mockPool.query).toHaveBeenCalled();
    });

    it("getSessionsByTask should call pool.query with taskId", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 0 } as any);

      const result = await dbClient.getSessionsByTask("task-id");

      expect(mockPool.query).toHaveBeenCalledWith(
        expect.any(String),
        ["task-id"]
      );
      expect(result).toEqual([]);
    });
  });

  describe("Iteration Methods", () => {
    it("createIteration should call pool.query", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 1 } as any);

      const iteration = {
        iterationId: "iter",
        sessionId: "sess",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap",
        errorDetected: false,
        qualityScore: 0.7,
        improvementDelta: 0.1,
        resourceUsageMB: 100,
        promptModifications: [],
      };

      await dbClient.createIteration(iteration);

      expect(mockPool.query).toHaveBeenCalled();
    });

    it("getIterations should call pool.query with sessionId", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 0 } as any);

      const result = await dbClient.getIterations("sess-id");

      expect(mockPool.query).toHaveBeenCalledWith(
        expect.any(String),
        ["sess-id"]
      );
      expect(result).toEqual([]);
    });
  });

  describe("Error Pattern Methods", () => {
    it("upsertErrorPattern should call pool.query", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 1 } as any);

      const pattern = {
        patternId: "pat",
        category: "type_error" as any,
        pattern: "TypeError",
        frequency: 1,
        confidence: 0.8,
        detectedAt: new Date(),
        remediationStrategy: "Fix",
        successRate: 0.7,
        examples: [],
      };

      await dbClient.upsertErrorPattern(pattern);

      expect(mockPool.query).toHaveBeenCalled();
    });

    it("getErrorPatterns should call pool.query", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 0 } as any);

      const result = await dbClient.getErrorPatterns();

      expect(mockPool.query).toHaveBeenCalled();
      expect(result).toEqual([]);
    });

    it("getErrorPatterns should filter by category when provided", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 0 } as any);

      const result = await dbClient.getErrorPatterns("type_error" as any);

      expect(mockPool.query).toHaveBeenCalledWith(
        expect.stringContaining("WHERE"),
        expect.arrayContaining(["type_error"])
      );
      expect(result).toEqual([]);
    });
  });

  describe("Snapshot Methods", () => {
    it("saveSnapshot should call pool.query", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 1 } as any);

      const snapshot = {
        snapshotId: "snap",
        sessionId: "sess",
        iterationNumber: 1,
        timestamp: new Date(),
        compressedContext: "data",
        compressionRatio: 0.5,
        checksumMD5: "abc",
        sizeBytes: 100,
        isDiff: false,
      };

      await dbClient.saveSnapshot(snapshot);

      expect(mockPool.query).toHaveBeenCalled();
    });

    it("getSnapshot should call pool.query with snapshotId", async () => {
      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [], rowCount: 0 } as any);

      const result = await dbClient.getSnapshot("snap-id");

      expect(mockPool.query).toHaveBeenCalledWith(
        expect.any(String),
        ["snap-id"]
      );
      expect(result).toBeNull();
    });
  });

  describe("Transaction Support", () => {
    it("transaction should acquire client and begin transaction", async () => {
      const mockClient = {
        query: jest.fn().mockResolvedValue({ rows: [], rowCount: 0 }),
        release: jest.fn(),
      };

      (mockPool.connect as jest.Mock).mockResolvedValue(mockClient);

      let executed = false;
      await dbClient.transaction(async () => {
        executed = true;
      });

      expect(executed).toBe(true);
      expect(mockClient.query).toHaveBeenCalledWith("BEGIN");
      expect(mockClient.query).toHaveBeenCalledWith("COMMIT");
      expect(mockClient.release).toHaveBeenCalled();
    });

    it("transaction should rollback on error", async () => {
      const mockClient = {
        query: jest.fn().mockResolvedValue({ rows: [], rowCount: 0 }),
        release: jest.fn(),
      };

      (mockPool.connect as jest.Mock).mockResolvedValue(mockClient);

      await expect(
        dbClient.transaction(async () => {
          throw new Error("Transaction error");
        })
      ).rejects.toThrow("Transaction error");

      expect(mockClient.query).toHaveBeenCalledWith("BEGIN");
      expect(mockClient.query).toHaveBeenCalledWith("ROLLBACK");
      expect(mockClient.release).toHaveBeenCalled();
    });

    it("transaction should release client even if rollback fails", async () => {
      const mockClient = {
        query: jest
          .fn()
          .mockResolvedValueOnce({ rows: [], rowCount: 0 }) // BEGIN
          .mockRejectedValueOnce(new Error("Rollback failed")), // ROLLBACK
        release: jest.fn(),
      };

      (mockPool.connect as jest.Mock).mockResolvedValue(mockClient);

      await expect(
        dbClient.transaction(async () => {
          throw new Error("Transaction error");
        })
      ).rejects.toThrow();

      expect(mockClient.release).toHaveBeenCalled();
    });
  });

  describe("Error Handling", () => {
    it("should handle connection failures", async () => {
      (mockPool.connect as jest.Mock).mockRejectedValue(new Error("Connection failed"));

      await expect(
        dbClient.transaction(async () => {
          // Should not execute
        })
      ).rejects.toThrow("Connection failed");
    });

    it("should propagate database errors", async () => {
      (mockPool.query as jest.Mock).mockRejectedValue(new Error("Database error"));

      const session = {
        sessionId: "test",
        taskId: "task",
        agentId: "agent",
        status: "active" as any,
        config: {} as any,
        startTime: new Date(),
        iterationCount: 0,
        qualityScore: 0,
        improvementTrajectory: [],
        errorPatterns: [],
      };

      await expect(dbClient.createSession(session)).rejects.toThrow(
        "Database error"
      );
    });
  });

  describe("Data Mapping", () => {
    it("should correctly map database rows to session objects", async () => {
      const mockRow = {
        session_id: "sess-1",
        task_id: "task-1",
        agent_id: "agent-1",
        status: "active",
        config: JSON.stringify({ maxIterations: 10 }),
        start_time: new Date(),
        end_time: null,
        iteration_count: 0,
        quality_score: 0,
        improvement_trajectory: JSON.stringify([]),
        error_patterns: JSON.stringify([]),
        final_result: null,
        learning_summary: null,
      };

      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [mockRow], rowCount: 1 });

      const result = await dbClient.getSession("sess-1");

      expect(result).not.toBeNull();
      expect(result?.sessionId).toBe("sess-1");
      expect(result?.taskId).toBe("task-1");
      expect(result?.agentId).toBe("agent-1");
    });

    it("should handle NULL values in database rows", async () => {
      const mockRow = {
        session_id: "sess-1",
        task_id: "task-1",
        agent_id: "agent-1",
        status: "active",
        config: JSON.stringify({}),
        start_time: new Date(),
        end_time: null,
        iteration_count: 0,
        quality_score: 0,
        improvement_trajectory: JSON.stringify([]),
        error_patterns: JSON.stringify([]),
        final_result: null,
        learning_summary: null,
      };

      (mockPool.query as jest.Mock).mockResolvedValue({ rows: [mockRow], rowCount: 1 });

      const result = await dbClient.getSession("sess-1");

      expect(result).not.toBeNull();
      expect(result?.endTime).toBeUndefined();
      expect(result?.finalResult).toBeUndefined();
    });
  });
});
