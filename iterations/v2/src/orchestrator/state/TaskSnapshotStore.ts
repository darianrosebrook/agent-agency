/**
 * @fileoverview Task Snapshot Store - ARBITER-023
 *
 * Manages task execution state persistence for resumable task execution
 * and failure recovery with configurable checkpoints.
 *
 * @author @darianrosebrook
 */

import {
  SnapshotSaveRequest,
  TaskSnapshot,
  TaskSnapshotRepository,
} from "../repositories/TaskSnapshotRepository";

// Node.js types
type NodeJS_Timeout = ReturnType<typeof setTimeout>;

export interface TaskSnapshotStore {
  /**
   * Save task execution snapshot
   */
  save(request: SnapshotSaveRequest): Promise<TaskSnapshot>;

  /**
   * Restore task snapshot by ID
   */
  restore(taskId: string): Promise<TaskSnapshot | null>;

  /**
   * Delete task snapshot
   */
  delete(taskId: string): Promise<void>;

  /**
   * Update snapshot data for existing task
   */
  update(
    taskId: string,
    snapshotData: Record<string, any>
  ): Promise<TaskSnapshot>;

  /**
   * Get all snapshots for a task (version history)
   */
  getTaskHistory(taskId: string): Promise<TaskSnapshot[]>;

  /**
   * Clean up expired snapshots
   */
  cleanupExpired(): Promise<string[]>;

  /**
   * Get snapshot metadata without full data
   */
  getMetadata(
    taskId: string
  ): Promise<Omit<TaskSnapshot, "snapshotData"> | null>;

  /**
   * Check if task has active snapshot
   */
  hasSnapshot(taskId: string): Promise<boolean>;

  /**
   * Get store statistics
   */
  getStatistics(): Promise<TaskSnapshotStoreStatistics>;
}

export interface TaskSnapshotStoreStatistics {
  totalSnapshots: number;
  activeSnapshots: number;
  expiredSnapshots: number;
  totalStorageSize: number;
  lastCleanup: Date | null;
}

/**
 * Implementation of TaskSnapshotStore with PostgreSQL backing
 */
export class TaskSnapshotStoreImpl implements TaskSnapshotStore {
  private cleanupInterval: NodeJS_Timeout | null = null;
  private lastCleanup: Date | null = null;

  constructor(
    private repository: TaskSnapshotRepository,
    private config: {
      defaultTtlMs?: number;
      cleanupIntervalMs?: number;
      maxSnapshotsPerTask?: number;
    } = {}
  ) {
    this.startPeriodicCleanup();
  }

  async save(request: SnapshotSaveRequest): Promise<TaskSnapshot> {
    try {
      // Add default TTL if not specified
      const ttlMs = request.ttlMs ?? this.config.defaultTtlMs ?? 3600000; // 1 hour default

      const snapshot = await this.repository.save({
        ...request,
        ttlMs,
      });

      // Emit snapshot event
      this.emitSnapshotEvent("saved", {
        taskId: request.taskId,
        version: snapshot.snapshotVersion,
      });

      return snapshot;
    } catch (error) {
      throw new Error(
        `Failed to save snapshot for task ${request.taskId}: ${error}`
      );
    }
  }

  async restore(taskId: string): Promise<TaskSnapshot | null> {
    try {
      const snapshot = await this.repository.restore(taskId);

      if (snapshot) {
        // Emit restore event
        this.emitSnapshotEvent("restored", {
          taskId,
          version: snapshot.snapshotVersion,
        });
      }

      return snapshot;
    } catch (error) {
      throw new Error(
        `Failed to restore snapshot for task ${taskId}: ${error}`
      );
    }
  }

  async delete(taskId: string): Promise<void> {
    try {
      await this.repository.delete(taskId);

      // Emit deletion event
      this.emitSnapshotEvent("deleted", { taskId });
    } catch (error) {
      throw new Error(`Failed to delete snapshot for task ${taskId}: ${error}`);
    }
  }

  async update(
    taskId: string,
    snapshotData: Record<string, any>
  ): Promise<TaskSnapshot> {
    try {
      const snapshot = await this.repository.update(taskId, snapshotData);

      // Emit update event
      this.emitSnapshotEvent("updated", {
        taskId,
        version: snapshot.snapshotVersion,
      });

      return snapshot;
    } catch (error) {
      throw new Error(`Failed to update snapshot for task ${taskId}: ${error}`);
    }
  }

  async getTaskHistory(taskId: string): Promise<TaskSnapshot[]> {
    try {
      return await this.repository.getTaskHistory(taskId);
    } catch (error) {
      throw new Error(`Failed to get task history for ${taskId}: ${error}`);
    }
  }

  async cleanupExpired(): Promise<string[]> {
    try {
      const expiredTaskIds = await this.repository.cleanupExpired();
      this.lastCleanup = new Date();

      if (expiredTaskIds.length > 0) {
        // Emit cleanup event
        this.emitSnapshotEvent("cleanup", { expiredTaskIds });
      }

      return expiredTaskIds;
    } catch (error) {
      throw new Error(`Failed to cleanup expired snapshots: ${error}`);
    }
  }

  async getMetadata(
    taskId: string
  ): Promise<Omit<TaskSnapshot, "snapshotData"> | null> {
    try {
      return await this.repository.getMetadata(taskId);
    } catch (error) {
      throw new Error(`Failed to get metadata for task ${taskId}: ${error}`);
    }
  }

  async hasSnapshot(taskId: string): Promise<boolean> {
    try {
      const metadata = await this.getMetadata(taskId);
      return metadata !== null;
    } catch (error) {
      return false;
    }
  }

  async getStatistics(): Promise<TaskSnapshotStoreStatistics> {
    try {
      // This would require additional repository methods to get statistics
      // For now, return basic information
      return {
        totalSnapshots: 0, // Would need repository method
        activeSnapshots: 0, // Would need repository method
        expiredSnapshots: 0, // Would need repository method
        totalStorageSize: 0, // Would need repository method
        lastCleanup: this.lastCleanup,
      };
    } catch (error) {
      throw new Error(`Failed to get snapshot store statistics: ${error}`);
    }
  }

  /**
   * Save checkpoint during task execution
   */
  async saveCheckpoint(
    taskId: string,
    checkpointData: {
      stage: string;
      progress: number;
      state: Record<string, any>;
      metadata?: Record<string, any>;
    }
  ): Promise<TaskSnapshot> {
    const snapshotData = {
      checkpoint: checkpointData.stage,
      progress: checkpointData.progress,
      state: checkpointData.state,
      metadata: checkpointData.metadata || {},
      timestamp: new Date().toISOString(),
    };

    return this.save({
      taskId,
      snapshotData,
      snapshotVersion: await this.getNextVersion(taskId),
    });
  }

  /**
   * Restore from specific checkpoint
   */
  async restoreFromCheckpoint(
    taskId: string,
    checkpoint?: string
  ): Promise<TaskSnapshot | null> {
    const snapshot = await this.restore(taskId);

    if (!snapshot) {
      return null;
    }

    // If specific checkpoint requested, filter by checkpoint name
    if (checkpoint && snapshot.snapshotData.checkpoint !== checkpoint) {
      return null;
    }

    return snapshot;
  }

  /**
   * Get next version number for task
   */
  private async getNextVersion(taskId: string): Promise<number> {
    try {
      const history = await this.getTaskHistory(taskId);
      if (history.length === 0) {
        return 1;
      }

      const maxVersion = Math.max(...history.map((s) => s.snapshotVersion));
      return maxVersion + 1;
    } catch (error) {
      return 1; // Default to version 1 if history unavailable
    }
  }

  /**
   * Start periodic cleanup of expired snapshots
   */
  private startPeriodicCleanup(): void {
    const interval = this.config.cleanupIntervalMs ?? 300000; // 5 minutes default

    this.cleanupInterval = setInterval(async () => {
      try {
        await this.cleanupExpired();
      } catch (error) {
        console.error("Failed to cleanup expired snapshots:", error);
      }
    }, interval);
  }

  /**
   * Stop periodic cleanup
   */
  public stopPeriodicCleanup(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }
  }

  /**
   * Emit snapshot events (can be extended with EventEmitter)
   */
  private emitSnapshotEvent(event: string, data: any): void {
    // This can be extended to use EventEmitter for real-time notifications
    console.log(`Snapshot event: ${event}`, data);
  }

  /**
   * Create snapshot with automatic cleanup after task completion
   */
  async createWithAutoCleanup(
    taskId: string,
    snapshotData: Record<string, any>,
    completionCallback?: () => Promise<void>
  ): Promise<TaskSnapshot> {
    const snapshot = await this.save({
      taskId,
      snapshotData,
      ttlMs: 3600000, // 1 hour TTL
    });

    // Set up automatic cleanup after task completion
    if (completionCallback) {
      completionCallback().finally(() => {
        this.delete(taskId).catch((error) => {
          console.error(
            `Failed to cleanup snapshot for completed task ${taskId}:`,
            error
          );
        });
      });
    }

    return snapshot;
  }
}

