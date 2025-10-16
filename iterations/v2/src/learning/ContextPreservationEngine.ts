/**
 * Context Preservation Engine
 *
 * Manages context snapshots with semantic compression and differential storage
 * for memory-efficient iteration state management in multi-turn learning.
 *
 * Priority: CRITICAL - Memory efficiency is essential for learning scalability
 * Target: 70% compression ratio, <30ms P95 restoration time
 *
 * @author @darianrosebrook
 */

import { createHash } from "crypto";
import { gunzipSync, gzipSync } from "zlib";
import type {
  ContextPreservationResult,
  ContextRestorationResult,
  ContextSnapshot,
} from "../types/learning-coordination.js";

/**
 * Context Preservation Engine Configuration
 */
export interface ContextPreservationConfig {
  enableCompression: boolean;
  enableDifferentialStorage: boolean;
  compressionLevel: number;
  maxSnapshotSizeMB: number;
  checksumValidation: boolean;
}

/**
 * Default configuration for context preservation
 */
const DEFAULT_CONFIG: ContextPreservationConfig = {
  enableCompression: true,
  enableDifferentialStorage: true,
  compressionLevel: 6,
  maxSnapshotSizeMB: 50,
  checksumValidation: true,
};

/**
 * Context Preservation Engine
 *
 * Provides semantic compression, differential storage, and fast rollback
 * for maintaining context state across learning iterations.
 */
export class ContextPreservationEngine {
  private config: ContextPreservationConfig;
  private snapshotCache: Map<string, ContextSnapshot>;
  private baseSnapshots: Map<string, string>;

  constructor(config?: Partial<ContextPreservationConfig>) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.snapshotCache = new Map();
    this.baseSnapshots = new Map();
  }

  /**
   * Create context snapshot with compression
   *
   * @param sessionId - Learning session ID
   * @param iterationNumber - Current iteration number
   * @param context - Context data to preserve
   * @param baseSnapshotId - Optional base snapshot for differential storage
   * @returns Preservation result with snapshot ID and metrics
   */
  async createSnapshot(
    sessionId: string,
    iterationNumber: number,
    context: unknown,
    baseSnapshotId?: string
  ): Promise<ContextPreservationResult> {
    const startTime = Date.now();

    try {
      const snapshotId = this.generateSnapshotId(sessionId, iterationNumber);
      const contextString = JSON.stringify(context);
      const originalSize = Buffer.byteLength(contextString, "utf-8");

      // Check size limit
      const sizeMB = originalSize / (1024 * 1024);
      if (sizeMB > this.config.maxSnapshotSizeMB) {
        return {
          snapshotId,
          success: false,
          compressionRatio: 0,
          timeMs: Date.now() - startTime,
          sizeBytes: originalSize,
          error: `Context size ${sizeMB.toFixed(2)}MB exceeds limit of ${
            this.config.maxSnapshotSizeMB
          }MB`,
        };
      }

      let compressedData: string;
      let isDiff = false;
      let actualBaseId: string | undefined;

      // Differential storage if base snapshot exists
      if (
        this.config.enableDifferentialStorage &&
        baseSnapshotId &&
        this.snapshotCache.has(baseSnapshotId)
      ) {
        const baseContext = await this.restoreSnapshot(baseSnapshotId);

        if (baseContext.success) {
          const diff = this.computeDiff(baseContext.context, context);
          compressedData = this.compressData(JSON.stringify(diff));
          isDiff = true;
          actualBaseId = baseSnapshotId;
        } else {
          compressedData = this.compressData(contextString);
        }
      } else {
        compressedData = this.compressData(contextString);
        // Store as base snapshot for future diffs
        this.baseSnapshots.set(sessionId, snapshotId);
      }

      const compressedSize = Buffer.byteLength(compressedData, "utf-8");
      const compressionRatio = 1 - compressedSize / originalSize;
      const checksum = this.computeChecksum(contextString);

      const snapshot: ContextSnapshot = {
        snapshotId,
        sessionId,
        iterationNumber,
        timestamp: new Date(),
        compressedContext: compressedData,
        compressionRatio,
        checksumMD5: checksum,
        sizeBytes: compressedSize,
        isDiff,
        basedOnSnapshotId: actualBaseId,
      };

      // Cache snapshot
      this.snapshotCache.set(snapshotId, snapshot);

      const timeMs = Date.now() - startTime;

      return {
        snapshotId,
        success: true,
        compressionRatio,
        timeMs,
        sizeBytes: compressedSize,
      };
    } catch (error) {
      return {
        snapshotId: "",
        success: false,
        compressionRatio: 0,
        timeMs: Date.now() - startTime,
        sizeBytes: 0,
        error: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  /**
   * Restore context from snapshot
   *
   * @param snapshotId - Snapshot ID to restore
   * @returns Restoration result with context and metrics
   */
  async restoreSnapshot(snapshotId: string): Promise<ContextRestorationResult> {
    const startTime = Date.now();

    try {
      const snapshot = this.snapshotCache.get(snapshotId);

      if (!snapshot) {
        const result: any = {
          success: false,
          context: null,
          timeMs: Date.now() - startTime,
          error: `Snapshot ${snapshotId} not found`,
        };

        // Include checksumValid for consistency (errors always have it as false)
        result.checksumValid = false;

        return result;
      }

      let contextString: string;

      // Handle differential snapshots
      if (snapshot.isDiff && snapshot.basedOnSnapshotId) {
        const baseResult = await this.restoreSnapshot(
          snapshot.basedOnSnapshotId
        );

        if (!baseResult.success) {
          const result: any = {
            success: false,
            context: null,
            timeMs: Date.now() - startTime,
            error: `Failed to restore base snapshot: ${baseResult.error}`,
          };

          // Include checksumValid for consistency (errors always have it as false)
          result.checksumValid = false;

          return result;
        }

        const diffString = this.decompressData(snapshot.compressedContext);
        const diff = JSON.parse(diffString);
        const restoredContext = this.applyDiff(baseResult.context, diff);
        contextString = JSON.stringify(restoredContext);
      } else {
        contextString = this.decompressData(snapshot.compressedContext);
      }

      const context = JSON.parse(contextString);

      // Validate checksum if enabled
      let checksumValid = true;
      if (this.config.checksumValidation) {
        const actualChecksum = this.computeChecksum(contextString);
        checksumValid = actualChecksum === snapshot.checksumMD5;

        if (!checksumValid) {
          return {
            success: false,
            context: null,
            timeMs: Date.now() - startTime,
            checksumValid: false, // Always include when there's a checksum issue
            error: "Checksum validation failed - context may be corrupted",
          };
        }
      }

      const timeMs = Date.now() - startTime;

      const result: any = {
        success: true,
        context,
        timeMs,
      };

      // Only include checksumValid if validation is enabled
      if (this.config.checksumValidation) {
        result.checksumValid = checksumValid;
      }

      return result;
    } catch (error) {
      return {
        success: false,
        context: null,
        timeMs: Date.now() - startTime,
        checksumValid: false, // Always include for errors
        error: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  /**
   * Get snapshot metadata without restoring full context
   *
   * @param snapshotId - Snapshot ID
   * @returns Snapshot metadata or undefined
   */
  getSnapshotMetadata(snapshotId: string): ContextSnapshot | undefined {
    return this.snapshotCache.get(snapshotId);
  }

  /**
   * Clear snapshot cache for a session
   *
   * @param sessionId - Session ID to clear
   */
  clearSession(sessionId: string): void {
    const snapshotsToDelete: string[] = [];

    for (const [snapshotId, snapshot] of this.snapshotCache.entries()) {
      if (snapshot.sessionId === sessionId) {
        snapshotsToDelete.push(snapshotId);
      }
    }

    for (const snapshotId of snapshotsToDelete) {
      this.snapshotCache.delete(snapshotId);
    }

    this.baseSnapshots.delete(sessionId);
  }

  /**
   * Get cache statistics
   *
   * @returns Cache size and compression metrics
   */
  getCacheStats(): {
    snapshotCount: number;
    totalSizeBytes: number;
    averageCompressionRatio: number;
  } {
    let totalSize = 0;
    let totalCompressionRatio = 0;
    let count = 0;

    for (const snapshot of this.snapshotCache.values()) {
      totalSize += snapshot.sizeBytes;
      totalCompressionRatio += snapshot.compressionRatio;
      count++;
    }

    return {
      snapshotCount: count,
      totalSizeBytes: totalSize,
      averageCompressionRatio: count > 0 ? totalCompressionRatio / count : 0,
    };
  }

  /**
   * Generate unique snapshot ID
   */
  private generateSnapshotId(
    sessionId: string,
    iterationNumber: number
  ): string {
    return `snapshot_${sessionId}_${iterationNumber}_${Date.now()}`;
  }

  /**
   * Compress data using gzip
   */
  private compressData(data: string): string {
    if (!this.config.enableCompression) {
      return data;
    }

    const buffer = Buffer.from(data, "utf-8");
    const compressed = gzipSync(buffer, {
      level: this.config.compressionLevel,
    });
    return compressed.toString("base64");
  }

  /**
   * Decompress data from gzip
   */
  private decompressData(compressedData: string): string {
    if (!this.config.enableCompression) {
      return compressedData;
    }

    const buffer = Buffer.from(compressedData, "base64");
    const decompressed = gunzipSync(buffer);
    return decompressed.toString("utf-8");
  }

  /**
   * Compute MD5 checksum for data integrity
   */
  private computeChecksum(data: string): string {
    return createHash("md5").update(data).digest("hex");
  }

  /**
   * Compute diff between base and current context
   *
   * Simplified diff algorithm - stores only changed values
   */
  private computeDiff(base: unknown, current: unknown): unknown {
    if (
      typeof base !== "object" ||
      base === null ||
      typeof current !== "object" ||
      current === null
    ) {
      return current;
    }

    const diff: Record<string, unknown> = {};
    const baseObj = base as Record<string, unknown>;
    const currentObj = current as Record<string, unknown>;

    // Find changed or new keys
    for (const key of Object.keys(currentObj)) {
      if (!(key in baseObj)) {
        diff[key] = currentObj[key];
      } else if (
        JSON.stringify(baseObj[key]) !== JSON.stringify(currentObj[key])
      ) {
        diff[key] = currentObj[key];
      }
    }

    // Mark deleted keys
    for (const key of Object.keys(baseObj)) {
      if (!(key in currentObj)) {
        diff[`__deleted__${key}`] = true;
      }
    }

    return diff;
  }

  /**
   * Apply diff to base context
   */
  private applyDiff(base: unknown, diff: unknown): unknown {
    if (typeof base !== "object" || base === null) {
      return diff;
    }

    if (typeof diff !== "object" || diff === null) {
      return diff;
    }

    const result = { ...base } as Record<string, unknown>;
    const diffObj = diff as Record<string, unknown>;

    for (const key of Object.keys(diffObj)) {
      if (key.startsWith("__deleted__")) {
        const originalKey = key.replace("__deleted__", "");
        delete result[originalKey];
      } else {
        result[key] = diffObj[key];
      }
    }

    return result;
  }
}
