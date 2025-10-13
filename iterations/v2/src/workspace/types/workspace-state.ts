/**
 * Core types for Workspace State Manager
 *
 * Manages workspace context, tracks file changes, and maintains state across agent sessions.
 * Security: File content is never stored in state - only metadata for tracking and context selection.
 *
 * @author @darianrosebrook
 */

/**
 * File metadata without content (security: never store file content)
 */
export interface FileMetadata {
  /** Absolute path to file */
  path: string;
  /** Relative path from workspace root */
  relativePath: string;
  /** File size in bytes */
  size: number;
  /** Last modified timestamp */
  mtime: Date;
  /** File permissions (octal) */
  mode: number;
  /** Whether file is binary (excluded from content-based operations) */
  isBinary: boolean;
  /** File extension (lowercased) */
  extension: string;
  /** MIME type if detectable */
  mimeType?: string;
}

/**
 * File change types
 */
export enum FileChangeType {
  CREATED = "created",
  MODIFIED = "modified",
  DELETED = "deleted",
  RENAMED = "renamed",
}

/**
 * Represents a single file change
 */
export interface FileChange {
  /** Type of change */
  type: FileChangeType;
  /** File metadata at time of change */
  file: FileMetadata;
  /** Previous metadata (for modified/renamed/deleted) */
  previousFile?: FileMetadata;
  /** Timestamp of change detection */
  timestamp: Date;
  /** Agent that made the change (if known) */
  agentId?: string;
  /** Task context for the change */
  taskId?: string;
}

/**
 * Workspace snapshot - point-in-time view of workspace state
 */
export interface WorkspaceSnapshot {
  /** Unique snapshot ID */
  id: string;
  /** Timestamp when snapshot was taken */
  timestamp: Date;
  /** Files in workspace at snapshot time */
  files: FileMetadata[];
  /** Total number of files */
  fileCount: number;
  /** Total size of all files (bytes) */
  totalSize: number;
  /** Hash of snapshot for change detection */
  hash: string;
  /** Previous snapshot ID (for incremental diffs) */
  previousSnapshotId?: string;
}

/**
 * Context selection criteria for agents
 */
export interface ContextCriteria {
  /** Maximum number of files to include */
  maxFiles: number;
  /** Maximum total size in bytes */
  maxSizeBytes: number;
  /** File extensions to prioritize (e.g., ['.ts', '.js', '.json']) */
  priorityExtensions: string[];
  /** File extensions to exclude */
  excludeExtensions: string[];
  /** Directories to exclude from context */
  excludeDirectories: string[];
  /** Whether to include binary files (usually false) */
  includeBinaryFiles: boolean;
  /** Task-specific keywords for relevance scoring */
  relevanceKeywords: string[];
  /** Recency weight (how much to favor recently modified files) */
  recencyWeight: number;
}

/**
 * Selected context for an agent
 */
export interface WorkspaceContext {
  /** Files selected for context */
  files: FileMetadata[];
  /** Total size of selected files */
  totalSize: number;
  /** Selection criteria used */
  criteria: ContextCriteria;
  /** Relevance scores for each file (0-1) */
  relevanceScores: Map<string, number>;
  /** Timestamp when context was generated */
  timestamp: Date;
  /** Agent requesting context */
  agentId?: string;
  /** Task context */
  taskId?: string;
}

/**
 * Configuration for file watching
 */
export interface FileWatcherConfig {
  /** Paths to watch (relative to workspace root) */
  watchPaths: string[];
  /** Patterns to ignore */
  ignorePatterns: string[];
  /** Debounce time for rapid changes (ms) */
  debounceMs: number;
  /** Whether to watch subdirectories recursively */
  recursive: boolean;
  /** Whether to follow symlinks */
  followSymlinks: boolean;
  /** Maximum file size to track (bytes) */
  maxFileSize: number;
  /** Whether to detect binary files */
  detectBinaryFiles: boolean;
}

/**
 * Workspace state manager configuration
 */
export interface WorkspaceStateConfig {
  /** Root directory of workspace */
  workspaceRoot: string;
  /** File watcher configuration */
  watcher: FileWatcherConfig;
  /** Default context criteria */
  defaultContextCriteria: ContextCriteria;
  /** Snapshot retention policy */
  snapshotRetentionDays: number;
  /** Whether to persist state to database */
  enablePersistence: boolean;
  /** State compression level (0-9) */
  compressionLevel: number;
}

/**
 * State persistence operations
 */
export interface StatePersistence {
  /** Save snapshot to persistent storage */
  saveSnapshot(snapshot: WorkspaceSnapshot): Promise<void>;
  /** Load latest snapshot from storage */
  loadLatestSnapshot(): Promise<WorkspaceSnapshot | null>;
  /** Load snapshot by ID */
  loadSnapshot(id: string): Promise<WorkspaceSnapshot | null>;
  /** List all snapshots with pagination */
  listSnapshots(limit?: number, offset?: number): Promise<WorkspaceSnapshot[]>;
  /** Delete old snapshots based on retention policy */
  pruneSnapshots(olderThanDays: number): Promise<number>;
  /** Get changes between two snapshots */
  getChangesBetweenSnapshots(
    fromId: string,
    toId: string
  ): Promise<FileChange[]>;
}

/**
 * Events emitted by workspace state manager
 */
export interface WorkspaceStateEvents {
  /** Emitted when files change */
  "files-changed": (changes: FileChange[]) => void;
  /** Emitted when snapshot is created */
  "snapshot-created": (snapshot: WorkspaceSnapshot) => void;
  /** Emitted when context is requested */
  "context-requested": (context: WorkspaceContext) => void;
  /** Emitted on watcher errors */
  "watcher-error": (error: Error) => void;
  /** Emitted on persistence errors */
  "persistence-error": (error: Error) => void;
}

/**
 * Performance metrics for workspace operations
 */
export interface WorkspaceMetrics {
  /** File watching metrics */
  watcher: {
    filesWatched: number;
    eventsProcessed: number;
    eventsPerSecond: number;
    debounceHits: number;
  };
  /** Snapshot metrics */
  snapshots: {
    totalSnapshots: number;
    averageSnapshotTime: number;
    largestSnapshotSize: number;
  };
  /** Context metrics */
  context: {
    requestsProcessed: number;
    averageContextTime: number;
    averageFilesSelected: number;
  };
  /** Memory usage */
  memory: {
    heapUsed: number;
    heapTotal: number;
    external: number;
  };
}

/**
 * Error types for workspace state operations
 */
export class WorkspaceStateError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly operation: string
  ) {
    super(message);
    this.name = "WorkspaceStateError";
  }
}

export class FileWatcherError extends WorkspaceStateError {
  constructor(message: string, public readonly watchPath: string) {
    super(message, "FILE_WATCHER_ERROR", "file-watching");
  }
}

export class StatePersistenceError extends WorkspaceStateError {
  constructor(message: string, public readonly snapshotId?: string) {
    super(message, "STATE_PERSISTENCE_ERROR", "state-persistence");
  }
}

export class ContextSelectionError extends WorkspaceStateError {
  constructor(
    message: string,
    public readonly criteria: Partial<ContextCriteria>
  ) {
    super(message, "CONTEXT_SELECTION_ERROR", "context-selection");
  }
}
