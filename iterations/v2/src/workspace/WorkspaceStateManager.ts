/**
 * Workspace State Manager - Main orchestrator for workspace state management
 *
 * Coordinates file watching, state snapshots, context management, and persistence.
 * Provides unified interface for workspace awareness and agent context provision.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { KnowledgeDatabaseClient } from "../database/KnowledgeDatabaseClient.js";
import { EmbeddingService } from "../embeddings/EmbeddingService.js";
import { LoggerFactory } from "../logging/StructuredLogger.js";
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
  private embeddingService?: EmbeddingService;
  private dbClient?: KnowledgeDatabaseClient;
  private isInitialized = false;
  private metrics: WorkspaceMetrics;
  private metricsTimer: ReturnType<typeof setInterval> | null = null;
  private embeddingDebounceTimer?: ReturnType<typeof setTimeout>;
  private logger = LoggerFactory.createWorkspaceLogger();

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

    // Initialize semantic search components if enabled
    if (this.config.semanticSearch?.enabled) {
      this.embeddingService = new EmbeddingService({
        ollamaEndpoint:
          this.config.semanticSearch.ollamaEndpoint || "http://localhost:11434",
        cacheSize: this.config.semanticSearch.cacheSize || 1000,
      });

      // Initialize database client for embedding storage
      // Note: This assumes KnowledgeDatabaseClient can be instantiated with default config
      // In a real implementation, this would be passed in or configured properly
      this.dbClient = new KnowledgeDatabaseClient(/* config */);
    }

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
      // Clear metrics timer
      if (this.metricsTimer) {
        clearInterval(this.metricsTimer);
        this.metricsTimer = null;
      }

      // Clear embedding debounce timer
      if (this.embeddingDebounceTimer) {
        clearTimeout(this.embeddingDebounceTimer);
        this.embeddingDebounceTimer = undefined;
      }

      // Persist current state if enabled
      if (this.persistence && this.config.enablePersistence) {
        await this.persistCurrentState();
      }

      // Stop file watcher
      await this.fileWatcher.stop();

      // Shutdown embedding service
      if (this.embeddingService) {
        await this.embeddingService.shutdown();
      }

      this.isInitialized = false;
    } catch (error) {
      this.logger.error("Error during workspace state manager shutdown", {
        operation: "shutdown",
        error: error as Error,
      });
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
   * Get recent changes with optional filtering
   */
  async getRecentChanges(
    options: {
      maxAge?: number; // Maximum age in milliseconds
      maxCount?: number; // Maximum number of changes to return
      agentId?: string; // Filter by agent ID
    } = {}
  ): Promise<FileChange[]> {
    this.ensureInitialized();

    // For now, return empty array as this is a placeholder implementation
    // In a full implementation, this would query the file watcher's change history
    // and filter based on the provided options

    const {
      maxAge: _maxAge = 24 * 60 * 60 * 1000,
      maxCount: _maxCount = 100,
      agentId: _agentId,
    } = options;

    // This is a placeholder - in a real implementation, we'd:
    // 1. Get changes from the file watcher
    // 2. Filter by timestamp (within maxAge)
    // 3. Filter by agentId if provided
    // 4. Limit to maxCount
    // 5. Return the filtered results

    return [];
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

    // Handle embedding updates for semantic search
    if (
      this.config.semanticSearch?.enabled &&
      this.embeddingService &&
      this.dbClient
    ) {
      await this.handleEmbeddingUpdates(changes);
    }

    // Forward to listeners
    this.emit("files-changed", changes);
  }

  /**
   * Handle embedding updates for changed files
   */
  private async handleEmbeddingUpdates(changes: FileChange[]): Promise<void> {
    if (!this.embeddingService || !this.dbClient) {
      return;
    }

    // Debounce embedding updates to avoid excessive API calls
    const debounceMs = this.config.semanticSearch?.debounceMs || 1000;

    if (this.embeddingDebounceTimer) {
      clearTimeout(this.embeddingDebounceTimer);
    }

    this.embeddingDebounceTimer = setTimeout(async () => {
      try {
        // Filter changes to files we want to embed
        const relevantChanges = changes.filter((change) =>
          this.shouldGenerateEmbedding(change)
        );

        if (relevantChanges.length === 0) {
          return;
        }

        // Process embedding updates
        await Promise.all(
          relevantChanges.map((change) => this.updateFileEmbedding(change))
        );

        this.emit("embeddings-updated", relevantChanges.length);
      } catch (error) {
        this.emit("embedding-error", error);
      }
    }, debounceMs);
  }

  /**
   * Determine if a file change should trigger embedding generation
   */
  private shouldGenerateEmbedding(change: FileChange): boolean {
    // Only process added/modified files (not deleted)
    if (change.type === "deleted") {
      return false;
    }

    // Check file extension
    const ext = change.path.split(".").pop()?.toLowerCase();
    const supportedExtensions = [
      "ts",
      "js",
      "tsx",
      "jsx",
      "py",
      "java",
      "cpp",
      "c",
      "h",
      "hpp",
      "md",
      "txt",
      "json",
      "yaml",
      "yml",
    ];

    return ext ? supportedExtensions.includes(ext) : false;
  }

  /**
   * Update embedding for a single file
   */
  private async updateFileEmbedding(change: FileChange): Promise<void> {
    if (!this.embeddingService || !this.dbClient) {
      return;
    }

    try {
      // Read file content
      const fs = await import("fs/promises");
      const content = await fs.readFile(change.path, "utf-8");

      // Prepare text for embedding
      const textForEmbedding = this.prepareFileTextForEmbedding(
        change.path,
        content
      );

      // Generate embedding
      const embedding = await this.embeddingService.generateEmbedding(
        textForEmbedding
      );

      // Store in database using existing agent_capabilities_graph table
      await this.dbClient.query(
        `
        INSERT INTO agent_capabilities_graph (
          agent_id, capability_type, capability_name, canonical_name,
          embedding, confidence, metadata
        ) VALUES ($1, 'TECHNOLOGY', $2, $2, $3, 1.0, $4)
        ON CONFLICT (agent_id, canonical_name)
        DO UPDATE SET embedding = EXCLUDED.embedding, last_updated = NOW()
      `,
        [
          "system",
          change.path,
          `[${embedding.join(",")}]`,
          JSON.stringify({
            source: "workspace_file",
            file_type: change.path.split(".").pop(),
            last_modified: change.timestamp || new Date().toISOString(),
          }),
        ]
      );
    } catch (error) {
      // Log error but don't throw - embedding failures shouldn't break file watching
      console.error(`Failed to update embedding for ${change.path}:`, error);
    }
  }

  /**
   * Prepare file text for embedding generation
   */
  private prepareFileTextForEmbedding(
    filePath: string,
    content: string
  ): string {
    const fileName = filePath.split("/").pop() || filePath;
    const extension = filePath.split(".").pop() || "";

    // Add context about file type
    const context = this.getFileTypeContext(extension);

    // Limit content size to avoid token limits
    const maxContentLength = 8000; // Leave room for context
    const truncatedContent =
      content.length > maxContentLength
        ? content.substring(0, maxContentLength) + "..."
        : content;

    return `${context}\n\nFile: ${fileName}\nPath: ${filePath}\n\nContent:\n${truncatedContent}`;
  }

  /**
   * Get context string based on file type
   */
  private getFileTypeContext(extension: string): string {
    const contexts: Record<string, string> = {
      ts: "TypeScript source code file",
      js: "JavaScript source code file",
      tsx: "TypeScript React component file",
      jsx: "JavaScript React component file",
      py: "Python source code file",
      java: "Java source code file",
      cpp: "C++ source code file",
      c: "C source code file",
      h: "C/C++ header file",
      hpp: "C++ header file",
      md: "Markdown documentation file",
      txt: "Plain text file",
      json: "JSON data file",
      yaml: "YAML configuration file",
      yml: "YAML configuration file",
    };

    return contexts[extension] || "Source code file";
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
    // Clear any existing timer first to prevent multiple timers
    if (this.metricsTimer) {
      clearInterval(this.metricsTimer);
      this.metricsTimer = null;
    }

    this.metricsTimer = setInterval(() => {
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
