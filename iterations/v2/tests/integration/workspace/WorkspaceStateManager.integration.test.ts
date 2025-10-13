/**
 * WorkspaceStateManager Integration Tests
 *
 * Tests the WorkspaceStateManager and its integration with FileWatcher,
 * StateSnapshot, and ContextManager components.
 *
 * @author @darianrosebrook
 */

import { jest } from "@jest/globals";
import { WorkspaceStateManager } from "../../../src/workspace/WorkspaceStateManager.js";
import { FileMetadata } from "../../../src/workspace/types/workspace-state.js";

// Mock external dependencies
jest.mock("chokidar", () => ({
  watch: jest.fn(() => ({
    on: jest.fn(),
    close: jest.fn().mockReturnValue(Promise.resolve()),
  })),
}));

jest.mock("fs/promises", () => ({
  stat: jest.fn(),
}));

jest.mock("path", () => ({
  resolve: jest.fn((...args: string[]) => args.join("/")),
  relative: jest.fn((from: string, to: string) => to.replace(from + "/", "")),
  extname: jest.fn((path: string) => {
    const match = path.match(/\.[^.]+$/);
    return match ? match[0] : "";
  }),
}));

describe("WorkspaceStateManager Integration", () => {
  let manager: WorkspaceStateManager;

  // Mock file data for testing
  const mockFiles: FileMetadata[] = [
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
    {
      path: "/workspace/src/utils.ts",
      relativePath: "src/utils.ts",
      size: 512,
      mtime: new Date("2024-01-02T10:00:00Z"),
      mode: 0o644,
      isBinary: false,
      extension: ".ts",
      mimeType: "application/typescript",
    },
    {
      path: "/workspace/README.md",
      relativePath: "README.md",
      size: 2048,
      mtime: new Date("2024-01-03T10:00:00Z"),
      mode: 0o644,
      isBinary: false,
      extension: ".md",
      mimeType: "text/markdown",
    },
    {
      path: "/workspace/package.json",
      relativePath: "package.json",
      size: 256,
      mtime: new Date("2024-01-04T10:00:00Z"),
      mode: 0o644,
      isBinary: false,
      extension: ".json",
      mimeType: "application/json",
    },
  ];

  beforeEach(() => {
    // Create manager with test configuration
    manager = new WorkspaceStateManager({
      workspaceRoot: "/workspace",
      watcher: {
        watchPaths: ["src", "tests"],
        ignorePatterns: ["**/node_modules/**"],
        debounceMs: 50, // Faster for tests
        recursive: true,
        followSymlinks: false,
        maxFileSize: 1024 * 1024,
        detectBinaryFiles: true,
      },
      defaultContextCriteria: {
        maxFiles: 10,
        maxSizeBytes: 1024 * 1024,
        priorityExtensions: [".ts", ".js", ".json", ".md"],
        excludeExtensions: [".log", ".tmp"],
        excludeDirectories: ["node_modules", "dist"],
        includeBinaryFiles: false,
        relevanceKeywords: [],
        recencyWeight: 0.3,
      },
      snapshotRetentionDays: 30,
      enablePersistence: false, // Disable for tests
      compressionLevel: 6,
    });

    // Mock the file scanning to return our test files
    jest.spyOn(manager as any, "scanWorkspace").mockResolvedValue(mockFiles);
  });

  afterEach(async () => {
    if (manager) {
      await manager.shutdown();
    }
    jest.clearAllMocks();
  });

  describe("initialization", () => {
    it("should initialize successfully", async () => {
      await manager.initialize();

      expect(manager.getCurrentSnapshot()).toBeDefined();
      const snapshot = manager.getCurrentSnapshot()!;
      expect(snapshot.files).toHaveLength(4);
      expect(snapshot.fileCount).toBe(4);
      expect(snapshot.totalSize).toBe(1024 + 512 + 2048 + 256); // 3840
    });

    it("should create initial snapshot on startup", async () => {
      const snapshotPromise = new Promise((resolve) => {
        manager.once("snapshot-created", resolve);
      });

      await manager.initialize();

      const snapshot = await snapshotPromise;
      expect(snapshot).toBeDefined();
      expect((snapshot as any).files).toHaveLength(4);
    });
  });

  describe("context generation", () => {
    beforeEach(async () => {
      await manager.initialize();
    });

    it("should generate default context", () => {
      const context = manager.generateContext();

      expect(context.files).toBeDefined();
      expect(context.files.length).toBeGreaterThan(0);
      expect(context.files.length).toBeLessThanOrEqual(10);
      expect(context.totalSize).toBeGreaterThan(0);
      expect(context.criteria).toBeDefined();
      expect(context.timestamp).toBeInstanceOf(Date);
      expect(context.relevanceScores.size).toBe(context.files.length);
    });

    it("should generate code context with language filter", () => {
      const context = manager.generateCodeContext("typescript");

      expect(context.files).toBeDefined();
      expect(context.files.length).toBeGreaterThan(0);

      // Should prioritize .ts files
      const tsFiles = context.files.filter((f) => f.extension === ".ts");
      expect(tsFiles.length).toBeGreaterThan(0);
    });

    it("should generate documentation context", () => {
      const context = manager.generateDocumentationContext();

      expect(context.files).toBeDefined();

      // Should include README.md
      const hasMarkdown = context.files.some((f) => f.extension === ".md");
      expect(hasMarkdown).toBe(true);
    });

    it("should generate config context", () => {
      const context = manager.generateConfigContext();

      expect(context.files).toBeDefined();

      // Should include package.json
      const hasJson = context.files.some((f) => f.extension === ".json");
      expect(hasJson).toBe(true);
    });

    it("should respect custom criteria", () => {
      const customCriteria = {
        maxFiles: 2,
        priorityExtensions: [".md"],
        relevanceKeywords: ["readme"],
      };

      const context = manager.generateContext(customCriteria);

      expect(context.files.length).toBeLessThanOrEqual(2);
      expect(context.criteria.maxFiles).toBe(2);
    });
  });

  describe("metrics", () => {
    beforeEach(async () => {
      await manager.initialize();
    });

    it("should provide metrics", () => {
      const metrics = manager.getMetrics();

      expect(metrics).toBeDefined();
      expect(metrics.watcher).toBeDefined();
      expect(metrics.snapshots).toBeDefined();
      expect(metrics.context).toBeDefined();
      expect(metrics.memory).toBeDefined();

      // Should have snapshot data
      expect(metrics.snapshots.totalSnapshots).toBeGreaterThan(0);
    });

    it("should track context requests", () => {
      const initialMetrics = manager.getMetrics();
      const initialRequests = initialMetrics.context.requestsProcessed;

      manager.generateContext();

      const updatedMetrics = manager.getMetrics();
      expect(updatedMetrics.context.requestsProcessed).toBe(
        initialRequests + 1
      );
    });
  });

  describe("snapshot management", () => {
    beforeEach(async () => {
      await manager.initialize();
    });

    it("should list snapshots", () => {
      const snapshots = manager.getAllSnapshots();

      expect(snapshots).toBeDefined();
      expect(snapshots.length).toBeGreaterThan(0);
      expect(snapshots[0]).toHaveProperty("id");
      expect(snapshots[0]).toHaveProperty("timestamp");
      expect(snapshots[0]).toHaveProperty("files");
    });

    it("should get current snapshot", () => {
      const snapshot = manager.getCurrentSnapshot();

      expect(snapshot).toBeDefined();
      expect(snapshot!.files).toHaveLength(4);
    });

    it("should get snapshot by id", () => {
      const current = manager.getCurrentSnapshot()!;
      const retrieved = manager.getSnapshot(current.id);

      expect(retrieved).toEqual(current);
    });
  });

  describe("configuration", () => {
    beforeEach(async () => {
      await manager.initialize();
    });

    it("should update context criteria", () => {
      const newCriteria = {
        maxFiles: 5,
        recencyWeight: 0.5,
      };

      manager.updateContextCriteria(newCriteria);

      const context = manager.generateContext();
      expect(context.criteria.maxFiles).toBe(5);
      expect(context.criteria.recencyWeight).toBe(0.5);
    });
  });

  describe("error handling", () => {
    it("should throw error when not initialized", () => {
      expect(() => {
        manager.generateContext();
      }).toThrow("WorkspaceStateManager must be initialized before use");
    });

    it("should handle shutdown gracefully", async () => {
      await manager.initialize();
      await manager.shutdown();

      // Should not throw on subsequent shutdown
      await expect(manager.shutdown()).resolves.toBeUndefined();
    });
  });
});
