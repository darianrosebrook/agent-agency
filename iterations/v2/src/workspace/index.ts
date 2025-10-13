/**
 * Workspace State Management
 *
 * Provides comprehensive workspace awareness for agents including:
 * - File watching and change detection
 * - State snapshots and incremental diffs
 * - Context-aware file selection for agents
 * - Persistence and recovery
 *
 * @author @darianrosebrook
 */

// Core types
export * from "./types/workspace-state.js";

// Core components
export { ContextManager } from "./ContextManager.js";
export { FileWatcher } from "./FileWatcher.js";
export { FileStatePersistence } from "./StatePersistence.js";
export { StateSnapshot } from "./StateSnapshot.js";
export { WorkspaceStateManager } from "./WorkspaceStateManager.js";

// Default configuration
export const DEFAULT_WORKSPACE_CONFIG = {
  workspaceRoot: process.cwd(),
  watcher: {
    watchPaths: ["src", "tests", "docs"],
    ignorePatterns: [
      "**/node_modules/**",
      "**/dist/**",
      "**/build/**",
      "**/.git/**",
      "**/*.log",
      "**/coverage/**",
    ],
    debounceMs: 300,
    recursive: true,
    followSymlinks: false,
    maxFileSize: 10 * 1024 * 1024, // 10MB
    detectBinaryFiles: true,
  },
  defaultContextCriteria: {
    maxFiles: 20,
    maxSizeBytes: 1024 * 1024, // 1MB
    priorityExtensions: [".ts", ".js", ".json", ".md"],
    excludeExtensions: [".log", ".tmp", ".lock"],
    excludeDirectories: ["node_modules", "dist", "build", ".git"],
    includeBinaryFiles: false,
    relevanceKeywords: [],
    recencyWeight: 0.3,
  },
  snapshotRetentionDays: 30,
  enablePersistence: true,
  compressionLevel: 6,
} as const;
