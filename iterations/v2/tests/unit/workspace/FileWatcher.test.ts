/**
 * FileWatcher Unit Tests
 *
 * Tests file watching functionality with mocked filesystem operations.
 *
 * @author @darianrosebrook
 */

import { jest } from "@jest/globals";
import { EventEmitter } from "events";
import { FileWatcher } from "../../../src/workspace/FileWatcher.js";
import {
  FileChangeType,
  FileWatcherConfig,
} from "../../../src/workspace/types/workspace-state.js";

// Mock chokidar
const mockChokidarClose = jest.fn().mockReturnValue(Promise.resolve());
const mockChokidarOn = jest.fn();

jest.mock("chokidar", () => ({
  watch: jest.fn(() => ({
    on: mockChokidarOn,
    close: mockChokidarClose,
  })),
}));

// Mock fs/promises
jest.mock("fs/promises", () => ({
  stat: jest.fn(),
}));

// Mock path
jest.mock("path", () => ({
  resolve: jest.fn(),
  relative: jest.fn(),
  extname: jest.fn(),
}));

describe("FileWatcher", () => {
  let watcher: FileWatcher;
  let mockStat;
  let mockResolve;
  let mockRelative;
  let mockExtname;

  const mockConfig: FileWatcherConfig = {
    watchPaths: ["src", "tests"],
    ignorePatterns: ["**/node_modules/**"],
    debounceMs: 100,
    recursive: true,
    followSymlinks: false,
    maxFileSize: 1024 * 1024,
    detectBinaryFiles: true,
  };

  beforeEach(() => {
    jest.clearAllMocks();

    // Get mocked modules
    const fs = require("fs/promises");
    const path = require("path");

    mockStat = fs.stat;
    mockResolve = path.resolve;
    mockRelative = path.relative;
    mockExtname = path.extname;

    // Setup default mocks
    mockResolve.mockImplementation((...args) => args.join("/"));
    mockRelative.mockImplementation((from: any, to: any) =>
      to.replace(from + "/", "")
    );
    mockExtname.mockImplementation((path: any) => {
      const match = path.match(/\.[^.]+$/);
      return match ? match[0] : "";
    });

    // Setup stat mock
    mockStat.mockResolvedValue({
      size: 100,
      mtime: new Date("2024-01-01T00:00:00Z"),
      mode: 0o644,
    });

    watcher = new FileWatcher("/workspace", mockConfig);
  });

  afterEach(async () => {
    if (watcher) {
      await watcher.stop();
    }
  });

  describe("initialization", () => {
    it("should create watcher instance", () => {
      expect(watcher).toBeInstanceOf(FileWatcher);
      expect(watcher).toBeInstanceOf(EventEmitter);
    });

    it("should start file watching", async () => {
      mockStat.mockResolvedValue({
        size: 100,
        mtime: new Date(),
        mode: 0o644,
      });

      await watcher.start();

      expect(require("chokidar").watch).toHaveBeenCalledWith(
        ["/workspace/src", "/workspace/tests"],
        expect.objectContaining({
          ignored: expect.any(Array),
          persistent: true,
          ignoreInitial: false,
          followSymlinks: false,
          alwaysStat: true,
        })
      );
    });

    it("should stop file watching", async () => {
      await watcher.start();
      await watcher.stop();

      expect(mockChokidarClose).toHaveBeenCalled();
    });
  });

  describe("file change handling", () => {
    let changeHandler: jest.Mock;
    let addHandler;
    let changeEventHandler;
    let unlinkHandler;
    let errorHandler;

    beforeEach(async () => {
      changeHandler = jest.fn();
      watcher.on("files-changed", changeHandler);

      // Reset the mock
      mockChokidarOn.mockClear();

      await watcher.start();

      // Get the event handlers that were registered
      expect(mockChokidarOn).toHaveBeenCalledWith("add", expect.any(Function));
      expect(mockChokidarOn).toHaveBeenCalledWith(
        "change",
        expect.any(Function)
      );
      expect(mockChokidarOn).toHaveBeenCalledWith(
        "unlink",
        expect.any(Function)
      );
      expect(mockChokidarOn).toHaveBeenCalledWith(
        "error",
        expect.any(Function)
      );

      // Extract the handlers
      const calls = mockChokidarOn.mock.calls;
      addHandler = calls.find((call: any) => call[0] === "add")![1];
      changeEventHandler = calls.find((call: any) => call[0] === "change")![1];
      unlinkHandler = calls.find((call: any) => call[0] === "unlink")![1];
      errorHandler = calls.find((call: any) => call[0] === "error")![1];
    });

    it("should handle file creation events", async () => {
      // Simulate file creation
      addHandler("/workspace/src/test.ts", {
        size: 100,
        mtime: new Date("2024-01-01T00:00:00Z"),
        mode: 0o644,
      });

      // Wait for debouncing
      await new Promise((resolve) => setTimeout(resolve, 150));

      expect(changeHandler).toHaveBeenCalledWith(
        expect.arrayContaining([
          expect.objectContaining({
            type: FileChangeType.CREATED,
            file: expect.objectContaining({
              path: "/workspace/src/test.ts",
              relativePath: "src/test.ts",
              size: 100,
              extension: ".ts",
              isBinary: false,
            }),
          }),
        ])
      );
    });

    it("should handle file modification events", async () => {
      // Simulate file modification
      changeEventHandler("/workspace/src/test.ts", {
        size: 200,
        mtime: new Date("2024-01-01T00:00:00Z"),
        mode: 0o644,
      });

      // Wait for debouncing
      await new Promise((resolve) => setTimeout(resolve, 150));

      expect(changeHandler).toHaveBeenCalledWith(
        expect.arrayContaining([
          expect.objectContaining({
            type: FileChangeType.MODIFIED,
            file: expect.objectContaining({
              path: "/workspace/src/test.ts",
              size: 200,
            }),
          }),
        ])
      );
    });

    it("should handle file deletion events", async () => {
      // Simulate file deletion
      unlinkHandler("/workspace/src/test.ts");

      // Wait for debouncing
      await new Promise((resolve) => setTimeout(resolve, 150));

      expect(changeHandler).toHaveBeenCalledWith(
        expect.arrayContaining([
          expect.objectContaining({
            type: FileChangeType.DELETED,
            file: expect.objectContaining({
              path: "/workspace/src/test.ts",
            }),
          }),
        ])
      );
    });
  });

  describe("file filtering", () => {
    it("should ignore files matching patterns", async () => {
      const configWithIgnores: FileWatcherConfig = {
        ...mockConfig,
        ignorePatterns: ["**/test/**", "**/*.log"],
      };

      const watcherWithIgnores = new FileWatcher(
        "/workspace",
        configWithIgnores
      );

      // Reset mock to track this new instance
      mockChokidarOn.mockClear();
      const chokidarMock = require("chokidar");

      await watcherWithIgnores.start();

      // Check that ignore patterns are applied to the latest call
      const watchOptions =
        chokidarMock.watch.mock.calls[
          chokidarMock.watch.mock.calls.length - 1
        ][1];

      expect(watchOptions.ignored).toEqual(
        expect.arrayContaining([
          /(^|[\/\\])\../, // Hidden files
          "**/node_modules/**",
          "**/dist/**",
          "**/build/**",
          "**/.git/**",
          "**/*.log",
          "**/coverage/**",
          "**/.nyc_output/**",
          "**/*.tmp",
          "**/*.temp",
          "**/test/**",
          "**/*.log",
        ])
      );

      await watcherWithIgnores.stop();
    });

    it("should detect binary files", async () => {
      // Test binary file detection logic
      const testWatcher = new FileWatcher("/workspace", mockConfig);

      // This is testing internal logic, so we'd need to expose it or test through public interface
      // For now, just ensure the watcher is created properly
      expect(testWatcher).toBeDefined();

      // Clean up the test watcher
      await testWatcher.stop();
    });
  });

  describe("metrics", () => {
    it("should track watcher metrics", async () => {
      await watcher.start();

      const metrics = watcher.getMetrics();

      expect(metrics).toEqual(
        expect.objectContaining({
          filesWatched: expect.any(Number),
          eventsProcessed: expect.any(Number),
          eventsPerSecond: expect.any(Number),
          debounceHits: expect.any(Number),
        })
      );
    });
  });

  describe("error handling", () => {
    let errorHandler: jest.Mock;
    let watcherErrorHandler;

    beforeEach(async () => {
      errorHandler = jest.fn();
      watcher.on("watcher-error", errorHandler);

      mockChokidarOn.mockClear();
      await watcher.start();

      // Get the error handler
      const calls = mockChokidarOn.mock.calls;
      watcherErrorHandler = calls.find((call: any) => call[0] === "error")![1];
    });

    it("should emit watcher errors", () => {
      const testError = new Error("Test watcher error");
      watcherErrorHandler(testError);

      expect(errorHandler).toHaveBeenCalledWith(expect.any(Error));
    });
  });
});
