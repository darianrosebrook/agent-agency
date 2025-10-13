/**
 * State Persistence - Saves and loads workspace snapshots
 *
 * Provides file-based persistence for workspace snapshots with compression
 * and efficient storage. Implements the StatePersistence interface.
 *
 * @author @darianrosebrook
 */

import { promises as fs } from "fs";
import { join } from "path";
import {
  FileChange,
  StatePersistence as IStatePersistence,
  WorkspaceSnapshot,
} from "./types/workspace-state.js";

export class FileStatePersistence implements IStatePersistence {
  private storageDir: string;
  private compressionEnabled: boolean;

  constructor(
    storageDir: string = ".caws/workspace-state",
    compressionEnabled: boolean = true
  ) {
    this.storageDir = storageDir;
    this.compressionEnabled = compressionEnabled;
  }

  /**
   * Save a snapshot to persistent storage
   */
  async saveSnapshot(snapshot: WorkspaceSnapshot): Promise<void> {
    try {
      // Ensure storage directory exists
      await this.ensureStorageDir();

      const fileName = `${snapshot.id}.json`;
      const filePath = join(this.storageDir, fileName);

      // Serialize snapshot
      const serialized = this.serializeSnapshot(snapshot);

      // Write to file
      await fs.writeFile(filePath, serialized, "utf8");

      // Update latest snapshot pointer
      await this.updateLatestSnapshot(snapshot.id);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to save snapshot ${snapshot.id}: ${message}`);
    }
  }

  /**
   * Load the latest snapshot from storage
   */
  async loadLatestSnapshot(): Promise<WorkspaceSnapshot | null> {
    try {
      const latestId = await this.getLatestSnapshotId();
      if (!latestId) {
        return null;
      }

      return await this.loadSnapshot(latestId);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to load latest snapshot: ${message}`);
    }
  }

  /**
   * Load a specific snapshot by ID
   */
  async loadSnapshot(id: string): Promise<WorkspaceSnapshot | null> {
    try {
      const filePath = join(this.storageDir, `${id}.json`);

      // Check if file exists
      try {
        await fs.access(filePath);
      } catch {
        return null; // Snapshot doesn't exist
      }

      // Read and parse
      const data = await fs.readFile(filePath, "utf8");
      const snapshot = this.deserializeSnapshot(data);

      return snapshot;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to load snapshot ${id}: ${message}`);
    }
  }

  /**
   * List all snapshots with optional pagination
   */
  async listSnapshots(
    limit?: number,
    offset?: number
  ): Promise<WorkspaceSnapshot[]> {
    try {
      await this.ensureStorageDir();

      const files = await fs.readdir(this.storageDir);
      const snapshotFiles = files.filter(
        (f) => f.endsWith(".json") && f !== "latest.json"
      );

      // Sort by modification time (newest first)
      const sortedFiles = await Promise.all(
        snapshotFiles.map(async (file) => {
          const filePath = join(this.storageDir, file);
          const stats = await fs.stat(filePath);
          return { file, mtime: stats.mtime };
        })
      );

      sortedFiles.sort((a, b) => b.mtime.getTime() - a.mtime.getTime());

      // Apply pagination
      const startIndex = offset || 0;
      const endIndex = limit ? startIndex + limit : undefined;
      const paginatedFiles = sortedFiles.slice(startIndex, endIndex);

      // Load snapshots
      const snapshots: WorkspaceSnapshot[] = [];
      for (const { file } of paginatedFiles) {
        const id = file.replace(".json", "");
        const snapshot = await this.loadSnapshot(id);
        if (snapshot) {
          snapshots.push(snapshot);
        }
      }

      return snapshots;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to list snapshots: ${message}`);
    }
  }

  /**
   * Delete old snapshots based on retention policy
   */
  async pruneSnapshots(olderThanDays: number): Promise<number> {
    try {
      const cutoffDate = new Date();
      cutoffDate.setDate(cutoffDate.getDate() - olderThanDays);

      const allSnapshots = await this.listSnapshots();
      let deletedCount = 0;

      for (const snapshot of allSnapshots) {
        if (snapshot.timestamp < cutoffDate) {
          await this.deleteSnapshot(snapshot.id);
          deletedCount++;
        }
      }

      return deletedCount;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to prune snapshots: ${message}`);
    }
  }

  /**
   * Get changes between two snapshots
   */
  async getChangesBetweenSnapshots(
    fromId: string,
    toId: string
  ): Promise<FileChange[]> {
    const fromSnapshot = await this.loadSnapshot(fromId);
    const toSnapshot = await this.loadSnapshot(toId);

    if (!fromSnapshot || !toSnapshot) {
      throw new Error(`Snapshot not found: ${!fromSnapshot ? fromId : toId}`);
    }

    // Calculate changes (simplified - in real implementation, this would be more sophisticated)
    const changes: FileChange[] = [];

    const fromFiles = new Map(fromSnapshot.files.map((f) => [f.path, f]));
    const toFiles = new Map(toSnapshot.files.map((f) => [f.path, f]));

    // Find created and modified files
    for (const [path, toFile] of toFiles) {
      const fromFile = fromFiles.get(path);

      if (!fromFile) {
        // File was created
        changes.push({
          type: "created" as any,
          file: toFile,
          timestamp: toSnapshot.timestamp,
        });
      } else if (this.filesDiffer(fromFile, toFile)) {
        // File was modified
        changes.push({
          type: "modified" as any,
          file: toFile,
          previousFile: fromFile,
          timestamp: toSnapshot.timestamp,
        });
      }
    }

    // Find deleted files
    for (const [path, fromFile] of fromFiles) {
      if (!toFiles.has(path)) {
        changes.push({
          type: "deleted" as any,
          file: fromFile,
          timestamp: toSnapshot.timestamp,
        });
      }
    }

    return changes;
  }

  /**
   * Check if two file metadata objects represent different file states
   */
  private filesDiffer(file1: any, file2: any): boolean {
    return (
      file1.size !== file2.size ||
      file1.mtime !== file2.mtime ||
      file1.mode !== file2.mode
    );
  }

  /**
   * Serialize snapshot for storage
   */
  private serializeSnapshot(snapshot: WorkspaceSnapshot): string {
    // Create a clean copy for serialization
    const cleanSnapshot = {
      ...snapshot,
      timestamp: snapshot.timestamp.toISOString(),
      files: snapshot.files.map((file) => ({
        ...file,
        mtime: file.mtime.toISOString(),
      })),
    };

    return JSON.stringify(cleanSnapshot, null, 2);
  }

  /**
   * Deserialize snapshot from storage
   */
  private deserializeSnapshot(data: string): WorkspaceSnapshot {
    const parsed = JSON.parse(data);

    // Convert ISO strings back to Date objects
    parsed.timestamp = new Date(parsed.timestamp);
    parsed.files = parsed.files.map((file: any) => ({
      ...file,
      mtime: new Date(file.mtime),
    }));

    return parsed;
  }

  /**
   * Ensure storage directory exists
   */
  private async ensureStorageDir(): Promise<void> {
    try {
      await fs.access(this.storageDir);
    } catch {
      // Directory doesn't exist, create it
      await fs.mkdir(this.storageDir, { recursive: true });
    }
  }

  /**
   * Update the latest snapshot pointer
   */
  private async updateLatestSnapshot(snapshotId: string): Promise<void> {
    const latestPath = join(this.storageDir, "latest.json");
    const latestData = JSON.stringify({
      id: snapshotId,
      timestamp: new Date().toISOString(),
    });
    await fs.writeFile(latestPath, latestData, "utf8");
  }

  /**
   * Get the ID of the latest snapshot
   */
  private async getLatestSnapshotId(): Promise<string | null> {
    const latestPath = join(this.storageDir, "latest.json");

    try {
      const data = await fs.readFile(latestPath, "utf8");
      const latest = JSON.parse(data);
      return latest.id;
    } catch {
      return null; // No latest snapshot
    }
  }

  /**
   * Delete a snapshot file
   */
  private async deleteSnapshot(id: string): Promise<void> {
    const filePath = join(this.storageDir, `${id}.json`);
    try {
      await fs.unlink(filePath);
    } catch (error) {
      // Ignore if file doesn't exist
    }
  }

  /**
   * Get storage statistics
   */
  async getStorageStats(): Promise<{
    totalSnapshots: number;
    totalSize: number;
    oldestSnapshot?: Date;
    newestSnapshot?: Date;
  }> {
    try {
      const snapshots = await this.listSnapshots();

      if (snapshots.length === 0) {
        return { totalSnapshots: 0, totalSize: 0 };
      }

      const totalSize = snapshots.reduce((sum, s) => sum + s.totalSize, 0);
      const oldest = snapshots[snapshots.length - 1]?.timestamp;
      const newest = snapshots[0]?.timestamp;

      return {
        totalSnapshots: snapshots.length,
        totalSize,
        oldestSnapshot: oldest,
        newestSnapshot: newest,
      };
    } catch (error) {
      return { totalSnapshots: 0, totalSize: 0 };
    }
  }

  /**
   * Clear all stored snapshots (useful for testing)
   */
  async clear(): Promise<void> {
    try {
      const files = await fs.readdir(this.storageDir);
      for (const file of files) {
        if (file.endsWith(".json")) {
          await fs.unlink(join(this.storageDir, file));
        }
      }
    } catch {
      // Ignore errors during cleanup
    }
  }
}
