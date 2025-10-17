/**
 * @fileoverview WorkspaceStateManager Embedding Tests
 *
 * Tests for embedding generation and semantic search in WorkspaceStateManager.
 * Mocks file system, database, and embedding service.
 *
 * @author @darianrosebrook
 */

import { jest } from "@jest/globals";
import { KnowledgeDatabaseClient } from "../../src/database/KnowledgeDatabaseClient";
import { EmbeddingService } from "../../src/embeddings/EmbeddingService";
import { WorkspaceStateManager } from "../../src/workspace/WorkspaceStateManager";
import { FileChange } from "../../src/workspace/types/workspace-state";

// Mock dependencies
jest.mock("../../src/embeddings/EmbeddingService");
jest.mock("../../src/database/KnowledgeDatabaseClient");
jest.mock("../../src/workspace/FileWatcher");
jest.mock("../../src/workspace/StateSnapshot");
jest.mock("../../src/workspace/ContextManager");

// Mock fs/promises
jest.mock("fs/promises", () => ({
  readFile: jest.fn(),
}));

const mockEmbeddingService = EmbeddingService as jest.MockedClass<
  typeof EmbeddingService
>;
const mockDbClient = KnowledgeDatabaseClient as jest.MockedClass<
  typeof KnowledgeDatabaseClient
>;

describe("WorkspaceStateManager - Embedding Integration", () => {
  let manager: WorkspaceStateManager;
  let mockEmbeddingInstance: jest.Mocked<EmbeddingService>;
  let mockDbInstance: jest.Mocked<KnowledgeDatabaseClient>;

  beforeEach(() => {
    // Clear all mocks
    jest.clearAllMocks();

    // Create mock instances
    mockEmbeddingInstance = {
      generateEmbedding: jest.fn(),
      isAvailable: jest.fn().mockResolvedValue(true),
      clearCache: jest.fn(),
      getCacheStats: jest
        .fn()
        .mockReturnValue({ size: 0, maxSize: 1000, hitRate: 0 }),
    } as any;

    mockDbInstance = {
      query: jest.fn(),
    } as any;

    // Mock constructors
    mockEmbeddingService.mockImplementation(() => mockEmbeddingInstance);
    mockDbClient.mockImplementation(() => mockDbInstance);

    // Create manager with semantic search enabled
    manager = new WorkspaceStateManager({
      workspaceRoot: "/test/workspace",
      watcher: {
        watchPaths: ["src"],
        ignorePatterns: ["node_modules/**"],
        debounceMs: 100,
        recursive: true,
      },
      defaultContextCriteria: {
        maxFiles: 10,
        maxSizeBytes: 1024 * 1024,
        priorityExtensions: [".ts", ".js"],
        excludeExtensions: [".log"],
        excludeDirectories: ["node_modules"],
        includeBinaryFiles: false,
      },
      snapshotRetentionDays: 30,
      enablePersistence: false,
      semanticSearch: {
        enabled: true,
        ollamaEndpoint: "http://localhost:11434",
        cacheSize: 1000,
        debounceMs: 500,
      },
    });
  });

  describe("initialization", () => {
    it("should initialize embedding service when semantic search is enabled", () => {
      expect(mockEmbeddingService).toHaveBeenCalledWith({
        ollamaEndpoint: "http://localhost:11434",
        cacheSize: 1000,
      });
      expect(mockDbClient).toHaveBeenCalled();
    });

    it("should not initialize embedding service when semantic search is disabled", () => {
      // Reset mocks
      jest.clearAllMocks();

      const _managerNoEmbedding = new WorkspaceStateManager({
        workspaceRoot: "/test/workspace",
        watcher: {
          watchPaths: ["src"],
          ignorePatterns: ["node_modules/**"],
          debounceMs: 100,
          recursive: true,
        },
        defaultContextCriteria: {
          maxFiles: 10,
          maxSizeBytes: 1024 * 1024,
          priorityExtensions: [".ts", ".js"],
          excludeExtensions: [".log"],
          excludeDirectories: ["node_modules"],
          includeBinaryFiles: false,
        },
        snapshotRetentionDays: 30,
        enablePersistence: false,
        semanticSearch: {
          enabled: false,
        },
      });

      expect(mockEmbeddingService).not.toHaveBeenCalled();
      expect(mockDbClient).not.toHaveBeenCalled();
    });
  });

  describe("file change embedding updates", () => {
    const mockFs = jest.mocked(await import("fs/promises"));

    beforeEach(() => {
      mockFs.readFile.mockResolvedValue('console.log("test content");');
    });

    it("should generate embedding for supported file types on change", async () => {
      const testEmbedding = new Array(768).fill(0.1);
      mockEmbeddingInstance.generateEmbedding.mockResolvedValue(testEmbedding);

      const changes: FileChange[] = [
        {
          type: "modified",
          path: "/test/workspace/src/test.ts",
          timestamp: new Date(),
        },
      ];

      // Trigger file change handling
      // Note: In real implementation, this would be called by the file watcher
      // For testing, we call the private method directly
      await (manager as any).handleEmbeddingUpdates(changes);

      // Wait for debounce
      await new Promise((resolve) => setTimeout(resolve, 600));

      expect(mockEmbeddingInstance.generateEmbedding).toHaveBeenCalledWith(
        expect.stringContaining("TypeScript source code file")
      );
      expect(mockDbInstance.query).toHaveBeenCalledWith(
        expect.stringContaining("INSERT INTO agent_capabilities_graph"),
        expect.arrayContaining([
          "system",
          "/test/workspace/src/test.ts",
          expect.stringContaining("[0.1,0.1"),
          expect.stringContaining('"source":"workspace_file"'),
        ])
      );
    });

    it("should skip embedding for unsupported file types", async () => {
      const changes: FileChange[] = [
        {
          type: "modified",
          path: "/test/workspace/src/test.log",
          timestamp: new Date(),
        },
      ];

      await (manager as any).handleEmbeddingUpdates(changes);

      // Wait for debounce
      await new Promise((resolve) => setTimeout(resolve, 600));

      expect(mockEmbeddingInstance.generateEmbedding).not.toHaveBeenCalled();
    });

    it("should skip embedding for deleted files", async () => {
      const changes: FileChange[] = [
        {
          type: "deleted",
          path: "/test/workspace/src/test.ts",
          timestamp: new Date(),
        },
      ];

      await (manager as any).handleEmbeddingUpdates(changes);

      // Wait for debounce
      await new Promise((resolve) => setTimeout(resolve, 600));

      expect(mockEmbeddingInstance.generateEmbedding).not.toHaveBeenCalled();
    });

    it("should handle embedding generation errors gracefully", async () => {
      mockEmbeddingInstance.generateEmbedding.mockRejectedValue(
        new Error("Embedding service unavailable")
      );

      const changes: FileChange[] = [
        {
          type: "modified",
          path: "/test/workspace/src/test.ts",
          timestamp: new Date(),
        },
      ];

      // Should not throw
      await (manager as any).handleEmbeddingUpdates(changes);

      // Wait for debounce
      await new Promise((resolve) => setTimeout(resolve, 600));

      // Error should be logged but not crash the process
      expect(mockEmbeddingInstance.generateEmbedding).toHaveBeenCalled();
    });
  });

  describe("text preparation for embedding", () => {
    it("should prepare appropriate context for different file types", () => {
      const managerPrivate = manager as any;

      expect(managerPrivate.getFileTypeContext("ts")).toBe(
        "TypeScript source code file"
      );
      expect(managerPrivate.getFileTypeContext("js")).toBe(
        "JavaScript source code file"
      );
      expect(managerPrivate.getFileTypeContext("md")).toBe(
        "Markdown documentation file"
      );
      expect(managerPrivate.getFileTypeContext("unknown")).toBe(
        "Source code file"
      );
    });

    it("should prepare text with context and content", () => {
      const managerPrivate = manager as any;

      const result = managerPrivate.prepareFileTextForEmbedding(
        "/workspace/src/test.ts",
        "const x = 42;"
      );

      expect(result).toContain("TypeScript source code file");
      expect(result).toContain("File: test.ts");
      expect(result).toContain("Path: /workspace/src/test.ts");
      expect(result).toContain("Content:");
      expect(result).toContain("const x = 42;");
    });

    it("should truncate long content", () => {
      const managerPrivate = manager as any;
      const longContent = "x".repeat(10000);

      const result = managerPrivate.prepareFileTextForEmbedding(
        "/workspace/test.txt",
        longContent
      );

      expect(result.length).toBeLessThan(9000); // Context + truncated content
      expect(result).toContain("...");
    });
  });

  describe("semantic context generation", () => {
    it("should generate semantic context using embedding service", async () => {
      const testEmbedding = new Array(768).fill(0.2);
      const mockResults = [
        {
          entity_id: "uuid-1",
          name: "/workspace/src/test.ts",
          relevance_score: 0.85,
          metadata: {
            file_path: "/workspace/src/test.ts",
            file_type: "ts",
            source: "workspace_file",
          },
        },
      ];

      mockEmbeddingInstance.generateEmbedding.mockResolvedValue(testEmbedding);
      mockDbInstance.query.mockResolvedValue({ rows: mockResults });

      const contextManager = (manager as any).contextManager;
      const result = await contextManager.generateSemanticContext(
        "implement user authentication",
        { maxFiles: 5 },
        mockEmbeddingInstance,
        mockDbInstance
      );

      expect(mockEmbeddingInstance.generateEmbedding).toHaveBeenCalledWith(
        "implement user authentication"
      );
      expect(mockDbInstance.query).toHaveBeenCalledWith(
        expect.stringContaining("hybrid_search"),
        expect.arrayContaining([
          expect.stringContaining("[0.2,0.2"),
          "implement user authentication",
          5,
        ])
      );
      expect(result.searchType).toBe("semantic");
      expect(result.taskDescription).toBe("implement user authentication");
    });

    it("should throw error when semantic search dependencies are missing", async () => {
      const contextManager = (manager as any).contextManager;

      await expect(
        contextManager.generateSemanticContext("test query")
      ).rejects.toThrow(
        "Semantic search requires embedding service and database client"
      );
    });
  });

  describe("shutdown", () => {
    it("should clear embedding debounce timer on shutdown", async () => {
      // Start a debounce timer
      const changes: FileChange[] = [
        {
          type: "modified",
          path: "/test/workspace/src/test.ts",
          timestamp: new Date(),
        },
      ];

      await (manager as any).handleEmbeddingUpdates(changes);

      // Should have a timer set
      expect((manager as any).embeddingDebounceTimer).toBeDefined();

      // Shutdown should clear the timer
      await manager.shutdown();

      expect((manager as any).embeddingDebounceTimer).toBeUndefined();
    });
  });
});
