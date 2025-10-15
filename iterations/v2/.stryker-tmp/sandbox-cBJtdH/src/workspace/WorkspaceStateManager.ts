/**
 * Workspace State Manager - Main orchestrator for workspace state management
 *
 * Coordinates file watching, state snapshots, context management, and persistence.
 * Provides unified interface for workspace awareness and agent context provision.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { EventEmitter } from "events";
import { ContextManager } from "./ContextManager.js";
import { FileWatcher } from "./FileWatcher.js";
import { FileStatePersistence } from "./StatePersistence.js";
import { StateSnapshot } from "./StateSnapshot.js";
import {
  FileChange,
  FileMetadata,
  StatePersistence,
  WorkspaceContext,
  WorkspaceMetrics,
  WorkspaceSnapshot,
  WorkspaceStateConfig,
} from "./types/workspace-state.js";

export class WorkspaceStateManager extends EventEmitter {
  private config: WorkspaceStateConfig;
  private fileWatcher: FileWatcher;
  private stateSnapshot: StateSnapshot;
  private contextManager: ContextManager;
  private persistence?: StatePersistence;
  private isInitialized = false;
  private metrics: WorkspaceMetrics;

  constructor(config: WorkspaceStateConfig, persistence?: StatePersistence) {
    super();

    this.config = { ...config };

    // Create file-based persistence if enabled and no custom persistence provided
    if (config.enablePersistence && !persistence) {
      const storageDir = `.caws/workspace-state/${config.workspaceRoot
        .split("/")
        .pop()}`;
      this.persistence = new FileStatePersistence(
        storageDir,
        config.compressionLevel > 0
      );
    } else {
      this.persistence = persistence;
    }

    // Initialize components
    this.fileWatcher = new FileWatcher(
      this.config.workspaceRoot,
      this.config.watcher
    );

    this.stateSnapshot = new StateSnapshot();
    this.contextManager = new ContextManager(
      this.config.defaultContextCriteria
    );

    // Initialize metrics
    this.metrics = {
      watcher: {
        filesWatched: 0,
        eventsProcessed: 0,
        eventsPerSecond: 0,
        debounceHits: 0,
      },
      snapshots: {
        totalSnapshots: 0,
        averageSnapshotTime: 0,
        largestSnapshotSize: 0,
      },
      context: {
        requestsProcessed: 0,
        averageContextTime: 0,
        averageFilesSelected: 0,
      },
      memory: {
        heapUsed: 0,
        heapTotal: 0,
        external: 0,
      },
    };

    this.setupEventHandlers();
    this.setupMetricsTimer();
  }

  /**
   * Initialize the workspace state manager
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      // Start file watcher
      await this.fileWatcher.start();

      // Try to restore state from persistence
      if (this.persistence && this.config.enablePersistence) {
        await this.restorePersistedState();
      } else {
        // Create initial snapshot
        await this.createInitialSnapshot();
      }

      this.isInitialized = true;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(
        `Failed to initialize workspace state manager: ${message}`
      );
    }
  }

  /**
   * Shutdown the workspace state manager
   */
  async shutdown(): Promise<void> {
    if (!this.isInitialized) {
      return;
    }

    try {
      // Persist current state if enabled
      if (this.persistence && this.config.enablePersistence) {
        await this.persistCurrentState();
      }

      // Stop file watcher
      await this.fileWatcher.stop();

      this.isInitialized = false;
    } catch (error) {
      console.error("Error during workspace state manager shutdown:", error);
    }
  }

  /**
   * Get current workspace snapshot
   */
  getCurrentSnapshot(): WorkspaceSnapshot | null {
    this.ensureInitialized();
    return this.stateSnapshot.getCurrentSnapshot();
  }

  /**
   * Generate context for an agent
   */
  generateContext(
    criteria?: Partial<import("./types/workspace-state.js").ContextCriteria>,
    agentId?: string,
    taskId?: string
  ): WorkspaceContext {
    this.ensureInitialized();

    const startTime = Date.now();
    const snapshot = this.stateSnapshot.getCurrentSnapshot();

    if (!snapshot) {
      throw new Error("No workspace snapshot available");
    }

    const context = this.contextManager.generateContext(
      snapshot,
      criteria,
      agentId,
      taskId
    );

    // Update metrics
    const duration = Date.now() - startTime;
    this.metrics.context.requestsProcessed++;
    this.metrics.context.averageContextTime =
      (this.metrics.context.averageContextTime *
        (this.metrics.context.requestsProcessed - 1) +
        duration) /
      this.metrics.context.requestsProcessed;
    this.metrics.context.averageFilesSelected =
      (this.metrics.context.averageFilesSelected *
        (this.metrics.context.requestsProcessed - 1) +
        context.files.length) /
      this.metrics.context.requestsProcessed;

    this.emit("context-requested", context);

    return context;
  }

  /**
   * Generate code-specific context
   */
  generateCodeContext(
    language?: string,
    framework?: string,
    agentId?: string,
    taskId?: string
  ): WorkspaceContext {
    this.ensureInitialized();

    const snapshot = this.stateSnapshot.getCurrentSnapshot();
    if (!snapshot) {
      throw new Error("No workspace snapshot available");
    }

    return this.contextManager.generateCodeContext(
      snapshot,
      language,
      framework,
      agentId,
      taskId
    );
  }

  /**
   * Generate documentation context
   */
  generateDocumentationContext(
    agentId?: string,
    taskId?: string
  ): WorkspaceContext {
    this.ensureInitialized();

    const snapshot = this.stateSnapshot.getCurrentSnapshot();
    if (!snapshot) {
      throw new Error("No workspace snapshot available");
    }

    return this.contextManager.generateDocumentationContext(
      snapshot,
      agentId,
      taskId
    );
  }

  /**
   * Generate configuration context
   */
  generateConfigContext(agentId?: string, taskId?: string): WorkspaceContext {
    this.ensureInitialized();

    const snapshot = this.stateSnapshot.getCurrentSnapshot();
    if (!snapshot) {
      throw new Error("No workspace snapshot available");
    }

    return this.contextManager.generateConfigContext(snapshot, agentId, taskId);
  }

  /**
   * Get workspace metrics
   */
  getMetrics(): WorkspaceMetrics {
    this.metrics.watcher = this.fileWatcher.getMetrics();

    const snapshots = this.stateSnapshot.getAllSnapshots();
    this.metrics.snapshots.totalSnapshots = snapshots.length;

    if (snapshots.length > 0) {
      const totalSize = snapshots.reduce((sum, s) => sum + s.totalSize, 0);
      this.metrics.snapshots.averageSnapshotTime = totalSize / snapshots.length;

      const maxSize = Math.max(...snapshots.map((s) => s.totalSize));
      this.metrics.snapshots.largestSnapshotSize = maxSize;
    }

    // Memory metrics
    const memUsage = process.memoryUsage();
    this.metrics.memory = {
      heapUsed: memUsage.heapUsed,
      heapTotal: memUsage.heapTotal,
      external: memUsage.external,
    };

    return { ...this.metrics };
  }

  /**
   * Manually trigger snapshot creation
   */
  async createSnapshot(): Promise<WorkspaceSnapshot> {
    this.ensureInitialized();

    const snapshot = this.stateSnapshot.getCurrentSnapshot();
    if (!snapshot) {
      return await this.createInitialSnapshot();
    }

    // For now, create a new snapshot with current state
    // In a full implementation, this would scan the filesystem
    const currentSnapshot = await this.scanWorkspace();
    return this.stateSnapshot.createInitialSnapshot(currentSnapshot);
  }

  /**
   * Get list of all snapshots
   */
  getAllSnapshots(): WorkspaceSnapshot[] {
    this.ensureInitialized();
    return this.stateSnapshot.getAllSnapshots();
  }

  /**
   * Get snapshot by ID
   */
  getSnapshot(id: string): WorkspaceSnapshot | null {
    this.ensureInitialized();
    return this.stateSnapshot.getSnapshot(id);
  }

  /**
   * Prune old snapshots
   */
  pruneSnapshots(): { pruned: WorkspaceSnapshot[]; kept: WorkspaceSnapshot[] } {
    this.ensureInitialized();
    return this.stateSnapshot.pruneSnapshots(this.config.snapshotRetentionDays);
  }

  /**
   * Update context criteria
   */
  updateContextCriteria(
    criteria: Partial<import("./types/workspace-state.js").ContextCriteria>
  ): void {
    this.contextManager.updateDefaultCriteria(criteria);
  }

  /**
   * Get currently watched files
   */
  getWatchedFiles(): string[] {
    return this.fileWatcher.getWatchedFiles();
  }

  /**
   * Force change detection for a specific path
   */
  async triggerChangeDetection(path: string): Promise<void> {
    await this.fileWatcher.triggerChangeDetection(path);
  }

  /**
   * Setup event handlers
   */
  private setupEventHandlers(): void {
    // Forward file watcher events
    this.fileWatcher.on("files-changed", (changes: FileChange[]) => {
      this.handleFileChanges(changes);
    });

    this.fileWatcher.on("watcher-error", (error: Error) => {
      this.emit("watcher-error", error);
    });
  }

  /**
   * Handle file changes from watcher
   */
  private async handleFileChanges(changes: FileChange[]): Promise<void> {
    try {
      const currentSnapshot = this.stateSnapshot.getCurrentSnapshot();
      if (currentSnapshot) {
        // Create incremental snapshot
        const newSnapshot = this.stateSnapshot.createIncrementalSnapshot(
          changes,
          currentSnapshot
        );
        this.emit("snapshot-created", newSnapshot);

        // Persist if enabled
        if (this.persistence && this.config.enablePersistence) {
          await this.persistence.saveSnapshot(newSnapshot);
        }
      }
    } catch (error) {
      this.emit("persistence-error", error);
    }

    // Forward to listeners
    this.emit("files-changed", changes);
  }

  /**
   * Create initial workspace snapshot
   */
  private async createInitialSnapshot(): Promise<WorkspaceSnapshot> {
    const files = await this.scanWorkspace();
    const snapshot = this.stateSnapshot.createInitialSnapshot(files);

    // Persist if enabled
    if (this.persistence && this.config.enablePersistence) {
      await this.persistence.saveSnapshot(snapshot);
    }

    this.emit("snapshot-created", snapshot);
    return snapshot;
  }

  /**
   * Scan workspace for files
   */
  private async scanWorkspace(): Promise<FileMetadata[]> {
    // This is a simplified implementation
    // In a real system, you'd use a proper file scanning library
    const files: FileMetadata[] = [];

    // For now, return empty array - in production this would scan the actual filesystem
    // This is a placeholder to avoid implementing full file scanning in this initial version

    return files;
  }

  /**
   * Restore state from persistence
   */
  private async restorePersistedState(): Promise<void> {
    if (!this.persistence) return;

    try {
      const latestSnapshot = await this.persistence.loadLatestSnapshot();
      if (latestSnapshot) {
        // Restore snapshot to state manager
        this.stateSnapshot = new StateSnapshot();
        // Note: In a full implementation, we'd need to restore the snapshot map
        // For now, we'll create a new initial snapshot
        await this.createInitialSnapshot();
      } else {
        await this.createInitialSnapshot();
      }
    } catch (error) {
      console.warn(
        "Failed to restore persisted state, creating initial snapshot:",
        error
      );
      await this.createInitialSnapshot();
    }
  }

  /**
   * Persist current state
   */
  private async persistCurrentState(): Promise<void> {
    if (!this.persistence) return;

    const currentSnapshot = this.stateSnapshot.getCurrentSnapshot();
    if (currentSnapshot) {
      await this.persistence.saveSnapshot(currentSnapshot);
    }
  }

  /**
   * Setup periodic metrics updates
   */
  private setupMetricsTimer(): void {
    setInterval(() => {
      // Update memory metrics
      const memUsage = process.memoryUsage();
      this.metrics.memory = {
        heapUsed: memUsage.heapUsed,
        heapTotal: memUsage.heapTotal,
        external: memUsage.external,
      };
    }, 30000); // Update every 30 seconds
  }

  /**
   * Ensure manager is initialized
   */
  private ensureInitialized(): void {
    if (!this.isInitialized) {
      throw new Error("WorkspaceStateManager must be initialized before use");
    }
  }
}
