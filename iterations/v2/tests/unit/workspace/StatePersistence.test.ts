/**
 * StatePersistence Unit Tests
 *
 * Tests file-based persistence functionality for workspace snapshots.
 *
 * @author @darianrosebrook
 */

import { jest } from "@jest/globals";
import { tmpdir } from "os";
import { join } from "path";
import { FileStatePersistence } from "../../../src/workspace/StatePersistence.js";
import { FileMetadata } from "../../../src/workspace/types/workspace-state.js";

// Mock fs/promises
jest.mock("fs", () => ({
  promises: {
    access: jest.fn(),
    mkdir: jest.fn(),
    readdir: jest.fn(),
    readFile: jest.fn(),
    writeFile: jest.fn(),
    stat: jest.fn(),
    unlink: jest.fn(),
  },
}));

describe("FileStatePersistence", () => {
  let persistence: FileStatePersistence;
  let mockFs: any;
  let tempDir: string;

  const mockSnapshot = {
    id: "test-snapshot-123",
    timestamp: new Date("2024-01-01T10:00:00Z"),
    files: [
      {
        path: "/workspace/src/main.ts",
        relativePath: "src/main.ts",
        size: 1024,
        mtime: new Date("2024-01-01T10:00:00Z"),
        mode: 0o644,
        isBinary: false,
        extension: ".ts",
        mimeType: "application/typescript",
      },
    ] as FileMetadata[],
    fileCount: 1,
    totalSize: 1024,
    hash: "test-hash",
  };

  beforeEach(() => {
    mockFs = require("fs").promises;
    tempDir = join(tmpdir(), "workspace-state-test");
    persistence = new FileStatePersistence(tempDir, false);

    // Reset all mocks
    jest.clearAllMocks();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("initialization", () => {
    it("should create persistence instance", () => {
      expect(persistence).toBeDefined();
    });

    it("should create storage directory when saving", async () => {
      mockFs.access.mockRejectedValue(new Error("Directory not found"));
      mockFs.mkdir.mockResolvedValue(undefined);
      mockFs.writeFile.mockResolvedValue(undefined);

      await persistence.saveSnapshot(mockSnapshot);

      expect(mockFs.mkdir).toHaveBeenCalledWith(tempDir, { recursive: true });
    });
  });

  describe("saving snapshots", () => {
    beforeEach(() => {
      mockFs.access.mockResolvedValue(undefined);
      mockFs.writeFile.mockResolvedValue(undefined);
    });

    it("should save snapshot to file", async () => {
      await persistence.saveSnapshot(mockSnapshot);

      expect(mockFs.writeFile).toHaveBeenCalledWith(
        join(tempDir, "test-snapshot-123.json"),
        expect.stringContaining('"id": "test-snapshot-123"'),
        "utf8"
      );
    });

    it("should update latest snapshot pointer", async () => {
      await persistence.saveSnapshot(mockSnapshot);

      expect(mockFs.writeFile).toHaveBeenNthCalledWith(
        2, // Second call is for latest.json
        join(tempDir, "latest.json"),
        expect.stringContaining('"id":"test-snapshot-123"'),
        "utf8"
      );
    });
  });

  describe("loading snapshots", () => {
    beforeEach(() => {
      mockFs.access.mockResolvedValue(undefined);
    });

    it("should load snapshot from file", async () => {
      const serialized = JSON.stringify({
        ...mockSnapshot,
        timestamp: mockSnapshot.timestamp.toISOString(),
        files: mockSnapshot.files.map((f) => ({
          ...f,
          mtime: f.mtime.toISOString(),
        })),
      });

      mockFs.readFile.mockResolvedValue(serialized);

      const loaded = await persistence.loadSnapshot("test-snapshot-123");

      expect(loaded).toBeDefined();
      expect(loaded!.id).toBe("test-snapshot-123");
      expect(loaded!.timestamp).toEqual(mockSnapshot.timestamp);
      expect(loaded!.files).toHaveLength(1);
    });

    it("should return null for non-existent snapshot", async () => {
      mockFs.access.mockRejectedValue(new Error("File not found"));

      const loaded = await persistence.loadSnapshot("non-existent");

      expect(loaded).toBeNull();
    });

    it("should load latest snapshot", async () => {
      // Mock latest.json
      mockFs.readFile
        .mockResolvedValueOnce(JSON.stringify({ id: "test-snapshot-123" }))
        .mockResolvedValueOnce(
          JSON.stringify({
            ...mockSnapshot,
            timestamp: mockSnapshot.timestamp.toISOString(),
            files: mockSnapshot.files.map((f) => ({
              ...f,
              mtime: f.mtime.toISOString(),
            })),
          })
        );

      const loaded = await persistence.loadLatestSnapshot();

      expect(loaded).toBeDefined();
      expect(loaded!.id).toBe("test-snapshot-123");
    });

    it("should return null when no latest snapshot exists", async () => {
      mockFs.readFile.mockRejectedValue(new Error("File not found"));

      const loaded = await persistence.loadLatestSnapshot();

      expect(loaded).toBeNull();
    });
  });

  describe("listing snapshots", () => {
    beforeEach(() => {
      mockFs.access.mockResolvedValue(undefined);
      mockFs.readdir.mockResolvedValue([
        "test-snapshot-123.json",
        "test-snapshot-456.json",
        "latest.json",
      ]);

      mockFs.stat.mockImplementation((filePath: string) => {
        const is123 = filePath.includes("123");
        return Promise.resolve({
          mtime: new Date(
            is123 ? "2024-01-02T10:00:00Z" : "2024-01-01T10:00:00Z"
          ),
        });
      });
    });

    it("should list snapshots ordered by recency", async () => {
      mockFs.readFile.mockImplementation((filePath: string) => {
        if (filePath.includes("123")) {
          return Promise.resolve(
            JSON.stringify({
              ...mockSnapshot,
              id: "test-snapshot-123",
              timestamp: "2024-01-02T10:00:00Z",
            })
          );
        } else {
          return Promise.resolve(
            JSON.stringify({
              ...mockSnapshot,
              id: "test-snapshot-456",
              timestamp: "2024-01-01T10:00:00Z",
            })
          );
        }
      });

      const snapshots = await persistence.listSnapshots();

      expect(snapshots).toHaveLength(2);
      expect(snapshots[0].id).toBe("test-snapshot-123"); // Most recent first
      expect(snapshots[1].id).toBe("test-snapshot-456");
    });

    it("should respect limit parameter", async () => {
      const snapshots = await persistence.listSnapshots(1);

      expect(snapshots).toHaveLength(1);
    });

    it("should respect offset parameter", async () => {
      const snapshots = await persistence.listSnapshots(10, 1);

      expect(snapshots).toHaveLength(1);
    });
  });

  describe("pruning snapshots", () => {
    it("should prune old snapshots", async () => {
      const oldSnapshot = {
        ...mockSnapshot,
        id: "old-snapshot",
        timestamp: new Date(Date.now() - 40 * 24 * 60 * 60 * 1000), // 40 days ago
      };

      const newSnapshot = {
        ...mockSnapshot,
        id: "new-snapshot",
        timestamp: new Date(), // Today
      };

      // Mock listSnapshots to return both snapshots
      jest
        .spyOn(persistence, "listSnapshots")
        .mockResolvedValue([newSnapshot, oldSnapshot]);
      mockFs.unlink.mockResolvedValue(undefined);

      const deletedCount = await persistence.pruneSnapshots(30); // Keep 30 days

      expect(deletedCount).toBe(1);
      expect(mockFs.unlink).toHaveBeenCalledWith(
        join(tempDir, "old-snapshot.json")
      );
    });
  });

  describe("storage stats", () => {
    it("should return storage statistics", async () => {
      jest
        .spyOn(persistence, "listSnapshots")
        .mockResolvedValue([mockSnapshot]);

      const stats = await persistence.getStorageStats();

      expect(stats.totalSnapshots).toBe(1);
      expect(stats.totalSize).toBe(1024);
      expect(stats.oldestSnapshot).toEqual(mockSnapshot.timestamp);
      expect(stats.newestSnapshot).toEqual(mockSnapshot.timestamp);
    });

    it("should handle empty storage", async () => {
      jest.spyOn(persistence, "listSnapshots").mockResolvedValue([]);

      const stats = await persistence.getStorageStats();

      expect(stats.totalSnapshots).toBe(0);
      expect(stats.totalSize).toBe(0);
    });
  });

  describe("clearing storage", () => {
    it("should clear all stored snapshots", async () => {
      mockFs.readdir.mockResolvedValue([
        "snapshot1.json",
        "snapshot2.json",
        "latest.json",
        "other-file.txt",
      ]);

      mockFs.unlink.mockResolvedValue(undefined);

      await persistence.clear();

      expect(mockFs.unlink).toHaveBeenCalledTimes(3); // Only JSON files
      expect(mockFs.unlink).toHaveBeenCalledWith(
        join(tempDir, "snapshot1.json")
      );
      expect(mockFs.unlink).toHaveBeenCalledWith(
        join(tempDir, "snapshot2.json")
      );
      expect(mockFs.unlink).toHaveBeenCalledWith(join(tempDir, "latest.json"));
    });
  });
});
