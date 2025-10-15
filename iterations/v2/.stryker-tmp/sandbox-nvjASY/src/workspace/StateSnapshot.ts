/**
 * State Snapshot - Manages workspace snapshots and change detection
 *
 * Creates efficient snapshots of workspace state with incremental diffs,
 * compression, and change tracking. Security-focused: never stores file content.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { createHash } from "crypto";
import {
  FileChange,
  FileChangeType,
  FileMetadata,
  WorkspaceSnapshot,
} from "./types/workspace-state.js";

export class StateSnapshot {
  private snapshots = new Map<string, WorkspaceSnapshot>();
  private currentSnapshot: WorkspaceSnapshot | null = null;

  /**
   * Create initial snapshot from file list
   */
  createInitialSnapshot(files: FileMetadata[]): WorkspaceSnapshot {
    const snapshot: WorkspaceSnapshot = {
      id: this.generateSnapshotId(),
      timestamp: new Date(),
      files: [...files],
      fileCount: files.length,
      totalSize: files.reduce((sum, file) => sum + file.size, 0),
      hash: this.calculateSnapshotHash(files),
    };

    this.snapshots.set(snapshot.id, snapshot);
    this.currentSnapshot = snapshot;

    return snapshot;
  }

  /**
   * Create incremental snapshot from previous snapshot and changes
   */
  createIncrementalSnapshot(
    changes: FileChange[],
    previousSnapshot: WorkspaceSnapshot
  ): WorkspaceSnapshot {
    // Start with previous snapshot's files
    const fileMap = new Map<string, FileMetadata>();
    for (const file of previousSnapshot.files) {
      fileMap.set(file.path, file);
    }

    // Apply changes
    for (const change of changes) {
      switch (change.type) {
        case FileChangeType.CREATED:
        case FileChangeType.MODIFIED:
          fileMap.set(change.file.path, change.file);
          break;
        case FileChangeType.DELETED:
          fileMap.delete(change.file.path);
          break;
        case FileChangeType.RENAMED:
          if (change.previousFile) {
            fileMap.delete(change.previousFile.path);
          }
          fileMap.set(change.file.path, change.file);
          break;
      }
    }

    const files = Array.from(fileMap.values());

    const snapshot: WorkspaceSnapshot = {
      id: this.generateSnapshotId(),
      timestamp: new Date(),
      files,
      fileCount: files.length,
      totalSize: files.reduce((sum, file) => sum + file.size, 0),
      hash: this.calculateSnapshotHash(files),
      previousSnapshotId: previousSnapshot.id,
    };

    this.snapshots.set(snapshot.id, snapshot);
    this.currentSnapshot = snapshot;

    return snapshot;
  }

  /**
   * Get current snapshot
   */
  getCurrentSnapshot(): WorkspaceSnapshot | null {
    return this.currentSnapshot;
  }

  /**
   * Get snapshot by ID
   */
  getSnapshot(id: string): WorkspaceSnapshot | null {
    return this.snapshots.get(id) || null;
  }

  /**
   * Get all snapshots
   */
  getAllSnapshots(): WorkspaceSnapshot[] {
    return Array.from(this.snapshots.values()).sort(
      (a, b) => a.timestamp.getTime() - b.timestamp.getTime()
    );
  }

  /**
   * Calculate diff between two snapshots
   */
  calculateDiff(fromId: string, toId: string): FileChange[] {
    const fromSnapshot = this.snapshots.get(fromId);
    const toSnapshot = this.snapshots.get(toId);

    if (!fromSnapshot || !toSnapshot) {
      throw new Error(`Snapshot not found: ${!fromSnapshot ? fromId : toId}`);
    }

    const changes: FileChange[] = [];
    const fromFiles = new Map(fromSnapshot.files.map((f) => [f.path, f]));
    const toFiles = new Map(toSnapshot.files.map((f) => [f.path, f]));

    // Find created and modified files
    for (const [path, toFile] of toFiles) {
      const fromFile = fromFiles.get(path);

      if (!fromFile) {
        // File was created
        changes.push({
          type: FileChangeType.CREATED,
          file: toFile,
          timestamp: toSnapshot.timestamp,
        });
      } else if (this.filesDiffer(fromFile, toFile)) {
        // File was modified
        changes.push({
          type: FileChangeType.MODIFIED,
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
          type: FileChangeType.DELETED,
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
  private filesDiffer(file1: FileMetadata, file2: FileMetadata): boolean {
    return (
      file1.size !== file2.size ||
      file1.mtime.getTime() !== file2.mtime.getTime() ||
      file1.mode !== file2.mode
    );
  }

  /**
   * Generate unique snapshot ID
   */
  private generateSnapshotId(): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 15);
    return `snapshot-${timestamp}-${random}`;
  }

  /**
   * Calculate hash of snapshot for change detection
   */
  private calculateSnapshotHash(files: FileMetadata[]): string {
    // Sort files by path for consistent hashing
    const sortedFiles = [...files].sort((a, b) => a.path.localeCompare(b.path));

    // Create hash input from file metadata (excluding content)
    const hashInput = sortedFiles
      .map(
        (file) =>
          `${file.path}:${file.size}:${file.mtime.getTime()}:${file.mode}`
      )
      .join("\n");

    return createHash("sha256").update(hashInput).digest("hex");
  }

  /**
   * Compress snapshot for storage (placeholder - would implement actual compression)
   */
  compressSnapshot(snapshot: WorkspaceSnapshot): Buffer {
    // Placeholder: In real implementation, would compress the JSON
    const jsonString = JSON.stringify(snapshot);
    return Buffer.from(jsonString, "utf8");
  }

  /**
   * Decompress snapshot from storage (placeholder)
   */
  decompressSnapshot(data: Buffer): WorkspaceSnapshot {
    // Placeholder: In real implementation, would decompress
    const jsonString = data.toString("utf8");
    const snapshot = JSON.parse(jsonString);

    // Convert date strings back to Date objects
    snapshot.timestamp = new Date(snapshot.timestamp);
    for (const file of snapshot.files) {
      file.mtime = new Date(file.mtime);
    }

    return snapshot;
  }

  /**
   * Prune old snapshots based on retention policy
   */
  pruneSnapshots(retainDays: number): {
    pruned: WorkspaceSnapshot[];
    kept: WorkspaceSnapshot[];
  } {
    const now = Date.now();
    const retainMs = retainDays * 24 * 60 * 60 * 1000;

    const allSnapshots = this.getAllSnapshots();
    const toPrune: WorkspaceSnapshot[] = [];
    const toKeep: WorkspaceSnapshot[] = [];

    for (const snapshot of allSnapshots) {
      if (now - snapshot.timestamp.getTime() > retainMs) {
        toPrune.push(snapshot);
        this.snapshots.delete(snapshot.id);
      } else {
        toKeep.push(snapshot);
      }
    }

    // Update current snapshot if it was pruned
    if (this.currentSnapshot && toPrune.includes(this.currentSnapshot)) {
      this.currentSnapshot = toKeep[toKeep.length - 1] || null;
    }

    return { pruned: toPrune, kept: toKeep };
  }

  /**
   * Get snapshot statistics
   */
  getStatistics() {
    const snapshots = this.getAllSnapshots();

    if (snapshots.length === 0) {
      return {
        totalSnapshots: 0,
        oldestSnapshot: null,
        newestSnapshot: null,
        averageFileCount: 0,
        averageTotalSize: 0,
        totalSizeIncrease: 0,
      };
    }

    const oldest = snapshots[0];
    const newest = snapshots[snapshots.length - 1];

    const totalFileCount = snapshots.reduce((sum, s) => sum + s.fileCount, 0);
    const totalSize = snapshots.reduce((sum, s) => sum + s.totalSize, 0);

    return {
      totalSnapshots: snapshots.length,
      oldestSnapshot: oldest.timestamp,
      newestSnapshot: newest.timestamp,
      averageFileCount: totalFileCount / snapshots.length,
      averageTotalSize: totalSize / snapshots.length,
      totalSizeIncrease: newest.totalSize - oldest.totalSize,
    };
  }

  /**
   * Clear all snapshots (useful for testing)
   */
  clear(): void {
    this.snapshots.clear();
    this.currentSnapshot = null;
  }
}
