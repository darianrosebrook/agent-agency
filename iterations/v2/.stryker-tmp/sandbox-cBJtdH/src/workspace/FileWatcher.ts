/**
 * File Watcher - Monitors workspace for file changes
 *
 * Uses chokidar for cross-platform file watching with efficient event handling,
 * debouncing, and ignore patterns. Security-focused: never reads file content.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import chokidar, { FSWatcher } from "chokidar";
import { EventEmitter } from "events";
import { stat } from "fs/promises";
import { extname, relative, resolve } from "path";
import {
  FileChange,
  FileChangeType,
  FileMetadata,
  FileWatcherConfig,
  FileWatcherError,
  WorkspaceMetrics,
} from "./types/workspace-state.js";

export class FileWatcher extends EventEmitter {
  private watcher: FSWatcher | null = null;
  private config: FileWatcherConfig;
  private workspaceRoot: string;
  private debounceTimer: ReturnType<typeof setInterval> | null = null;
  private pendingChanges = new Map<string, Partial<FileChange>>();
  private metrics: WorkspaceMetrics["watcher"] = {
    filesWatched: 0,
    eventsProcessed: 0,
    eventsPerSecond: 0,
    debounceHits: 0,
  };
  private eventCount = 0;
  private lastMetricsTime = Date.now();

  constructor(workspaceRoot: string, config: FileWatcherConfig) {
    super();
    this.workspaceRoot = resolve(workspaceRoot);
    this.config = { ...config };
    this.setupMetricsTimer();
  }

  /**
   * Start watching files
   */
  async start(): Promise<void> {
    try {
      if (this.watcher) {
        await this.stop();
      }

      const watchPaths = this.config.watchPaths.map((path) =>
        resolve(this.workspaceRoot, path)
      );

      this.watcher = chokidar.watch(watchPaths, {
        ignored: this.createIgnorePatterns(),
        persistent: true,
        ignoreInitial: false,
        followSymlinks: this.config.followSymlinks,
        usePolling: false, // Use native OS events when possible
        interval: 100, // Polling interval if needed
        binaryInterval: 300, // Slower for binary files
        alwaysStat: true,
        depth: this.config.recursive ? undefined : 0,
        awaitWriteFinish: {
          stabilityThreshold: 100,
          pollInterval: 50,
        },
      });

      this.setupEventHandlers();
      this.metrics.filesWatched = watchPaths.length;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new FileWatcherError(
        `Failed to start file watcher: ${message}`,
        this.workspaceRoot
      );
    }
  }

  /**
   * Stop watching files
   */
  async stop(): Promise<void> {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }

    if (this.watcher) {
      await this.watcher.close();
      this.watcher = null;
    }

    this.pendingChanges.clear();
  }

  /**
   * Get current metrics
   */
  getMetrics(): WorkspaceMetrics["watcher"] {
    return { ...this.metrics };
  }

  /**
   * Create ignore patterns for chokidar
   */
  private createIgnorePatterns(): Array<string | RegExp> {
    const patterns: Array<string | RegExp> = [
      // Common ignore patterns
      /(^|[\/\\])\../, // Hidden files/directories
      "**/node_modules/**",
      "**/dist/**",
      "**/build/**",
      "**/.git/**",
      "**/*.log",
      "**/coverage/**",
      "**/.nyc_output/**",
      "**/*.tmp",
      "**/*.temp",
    ];

    // Add user-specified patterns
    for (const pattern of this.config.ignorePatterns) {
      if (pattern.startsWith("/") && pattern.endsWith("/")) {
        // Regex pattern
        try {
          patterns.push(new RegExp(pattern.slice(1, -1)));
        } catch {
          // Invalid regex, skip
        }
      } else {
        // Glob pattern
        patterns.push(pattern);
      }
    }

    return patterns;
  }

  /**
   * Setup chokidar event handlers
   */
  private setupEventHandlers(): void {
    if (!this.watcher) return;

    this.watcher.on("add", (path, stats) => {
      this.handleFileEvent(path, stats, FileChangeType.CREATED);
    });

    this.watcher.on("change", (path, stats) => {
      this.handleFileEvent(path, stats, FileChangeType.MODIFIED);
    });

    this.watcher.on("unlink", (path) => {
      this.handleFileEvent(path, undefined, FileChangeType.DELETED);
    });

    this.watcher.on("error", (error) => {
      this.emit(
        "watcher-error",
        new FileWatcherError(
          `File watcher error: ${error.message}`,
          this.workspaceRoot
        )
      );
    });
  }

  /**
   * Handle file system events
   */
  private async handleFileEvent(
    path: string,
    stats: any,
    changeType: FileChangeType
  ): Promise<void> {
    try {
      const filePath = resolve(path);
      const relativePath = relative(this.workspaceRoot, filePath);

      // Skip if file should be ignored
      if (this.shouldIgnoreFile(relativePath)) {
        return;
      }

      // Skip if file is too large
      if (stats && stats.size > this.config.maxFileSize) {
        return;
      }

      const metadata = await this.createFileMetadata(
        filePath,
        relativePath,
        stats
      );

      const change: Partial<FileChange> = {
        type: changeType,
        file: metadata,
        timestamp: new Date(),
      };

      // Store pending change for debouncing
      this.pendingChanges.set(filePath, change);
      this.scheduleDebouncedEmit();

      this.metrics.eventsProcessed++;
      this.eventCount++;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.emit(
        "watcher-error",
        new FileWatcherError(`Failed to handle file event: ${message}`, path)
      );
    }
  }

  /**
   * Create file metadata from path and stats
   */
  private async createFileMetadata(
    absolutePath: string,
    relativePath: string,
    stats?: any
  ): Promise<FileMetadata> {
    let fileStats = stats;

    if (!fileStats) {
      try {
        fileStats = await stat(absolutePath);
      } catch {
        // File may have been deleted, use minimal metadata
        return {
          path: absolutePath,
          relativePath,
          size: 0,
          mtime: new Date(),
          mode: 0,
          isBinary: false,
          extension: extname(absolutePath).toLowerCase(),
        };
      }
    }

    const extension = extname(absolutePath).toLowerCase();
    const isBinary = this.isBinaryFile(extension, fileStats.size);

    return {
      path: absolutePath,
      relativePath,
      size: fileStats.size,
      mtime: new Date(fileStats.mtime),
      mode: fileStats.mode,
      isBinary,
      extension,
      mimeType: this.detectMimeType(extension),
    };
  }

  /**
   * Determine if file is binary based on extension and size heuristics
   */
  private isBinaryFile(extension: string, size: number): boolean {
    if (!this.config.detectBinaryFiles) {
      return false;
    }

    // Known binary extensions
    const binaryExtensions = [
      ".jpg",
      ".jpeg",
      ".png",
      ".gif",
      ".bmp",
      ".ico",
      ".svg",
      ".mp3",
      ".mp4",
      ".avi",
      ".mov",
      ".wmv",
      ".flv",
      ".pdf",
      ".doc",
      ".docx",
      ".xls",
      ".xlsx",
      ".ppt",
      ".pptx",
      ".zip",
      ".rar",
      ".7z",
      ".tar",
      ".gz",
      ".exe",
      ".dll",
      ".so",
      ".dylib",
      ".db",
      ".sqlite",
      ".sqlite3",
    ];

    if (binaryExtensions.includes(extension)) {
      return true;
    }

    // Size heuristic: files over 1MB are likely binary
    if (size > 1024 * 1024) {
      return true;
    }

    return false;
  }

  /**
   * Detect MIME type from extension
   */
  private detectMimeType(extension: string): string | undefined {
    const mimeTypes: Record<string, string> = {
      ".txt": "text/plain",
      ".json": "application/json",
      ".js": "application/javascript",
      ".ts": "application/typescript",
      ".tsx": "application/typescript",
      ".jsx": "application/javascript",
      ".html": "text/html",
      ".css": "text/css",
      ".md": "text/markdown",
      ".yaml": "application/yaml",
      ".yml": "application/yaml",
      ".xml": "application/xml",
      ".py": "text/x-python",
      ".java": "text/x-java-source",
      ".c": "text/x-csrc",
      ".cpp": "text/x-c++src",
      ".h": "text/x-chdr",
      ".hpp": "text/x-c++hdr",
      ".rs": "text/rust",
      ".go": "text/x-go",
      ".php": "application/x-php",
      ".rb": "text/x-ruby",
      ".sh": "application/x-shellscript",
      ".sql": "application/sql",
    };

    return mimeTypes[extension];
  }

  /**
   * Check if file should be ignored
   */
  private shouldIgnoreFile(relativePath: string): boolean {
    // Check against ignore patterns
    for (const pattern of this.config.ignorePatterns) {
      if (typeof pattern === "string") {
        // Simple glob matching (simplified)
        if (
          relativePath.includes(pattern.replace("**/", "").replace("/*", ""))
        ) {
          return true;
        }
      }
    }

    // Check file size
    // Note: We can't check size here without stats, so this is handled in handleFileEvent

    return false;
  }

  /**
   * Schedule debounced emission of changes
   */
  private scheduleDebouncedEmit(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.metrics.debounceHits++;
    }

    this.debounceTimer = setTimeout(() => {
      this.emitPendingChanges();
    }, this.config.debounceMs);
  }

  /**
   * Emit all pending changes
   */
  private emitPendingChanges(): void {
    if (this.pendingChanges.size === 0) return;

    const changes: FileChange[] = Array.from(this.pendingChanges.values()).map(
      (change) => change as FileChange
    );

    this.pendingChanges.clear();
    this.debounceTimer = null;

    if (changes.length > 0) {
      this.emit("files-changed", changes);
    }
  }

  /**
   * Setup periodic metrics calculation
   */
  private setupMetricsTimer(): void {
    setInterval(() => {
      const now = Date.now();
      const timeDiff = (now - this.lastMetricsTime) / 1000; // seconds

      if (timeDiff > 0) {
        this.metrics.eventsPerSecond = this.eventCount / timeDiff;
        this.eventCount = 0;
        this.lastMetricsTime = now;
      }
    }, 5000); // Update every 5 seconds
  }

  /**
   * Get list of currently watched files
   */
  getWatchedFiles(): string[] {
    return this.watcher ? Object.keys(this.watcher.getWatched()) : [];
  }

  /**
   * Manually trigger change detection for a path
   */
  async triggerChangeDetection(path: string): Promise<void> {
    const absolutePath = resolve(this.workspaceRoot, path);
    try {
      const stats = await stat(absolutePath);
      this.handleFileEvent(absolutePath, stats, FileChangeType.MODIFIED);
    } catch {
      // File doesn't exist, treat as deletion
      this.handleFileEvent(absolutePath, undefined, FileChangeType.DELETED);
    }
  }
}
