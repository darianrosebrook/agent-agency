/**
 * Artifact Sandbox for Safe Worker Filesystem Operations
 *
 * @author @darianrosebrook
 * @module orchestrator/workers/ArtifactSandbox
 *
 * Provides sandboxed filesystem access for worker threads with path validation,
 * quota enforcement, and manifest generation for artifact tracking.
 */

import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";
import { promisify } from "util";

const writeFile = promisify(fs.writeFile);
const mkdir = promisify(fs.mkdir);
const readdir = promisify(fs.readdir);
const stat = promisify(fs.stat);
const rename = promisify(fs.rename);

/**
 * Configuration for artifact sandbox.
 */
export interface ArtifactSandboxConfig {
  /**
   * Root directory for artifact storage.
   */
  rootPath: string;

  /**
   * Task ID for this sandbox instance.
   */
  taskId: string;

  /**
   * Maximum file size in bytes.
   */
  maxFileSizeBytes: number;

  /**
   * Maximum total files allowed.
   */
  maxTotalFiles: number;

  /**
   * Maximum path length allowed.
   */
  maxPathLength: number;
}

/**
 * Individual file entry in the artifact manifest.
 */
export interface ArtifactFileEntry {
  /**
   * Relative path within the artifact directory.
   */
  path: string;

  /**
   * File size in bytes.
   */
  size: number;

  /**
   * SHA256 digest of file content.
   */
  sha256: string;

  /**
   * MIME type (if detectable).
   */
  mimeType?: string;

  /**
   * Creation timestamp.
   */
  createdAt: string;
}

/**
 * Complete artifact manifest for a task.
 */
export interface ArtifactManifest {
  /**
   * Task ID this manifest belongs to.
   */
  taskId: string;

  /**
   * List of files in the artifact.
   */
  files: ArtifactFileEntry[];

  /**
   * Total size of all files in bytes.
   */
  totalSize: number;

  /**
   * Manifest creation timestamp.
   */
  createdAt: string;
}

/**
 * Custom error for artifact quota violations.
 */
export class ArtifactQuotaExceeded extends Error {
  public readonly quotaType: "size" | "files";
  
  constructor(message: string, quotaType: "size" | "files") {
    super(message);
    this.name = "ArtifactQuotaExceeded";
    this.quotaType = quotaType;
  }
}

/**
 * Custom error for invalid artifact paths.
 */
export class InvalidArtifactPath extends Error {
  public readonly path: string;
  
  constructor(message: string, path: string) {
    super(message);
    this.name = "InvalidArtifactPath";
    this.path = path;
  }
}

/**
 * Sandboxed filesystem API for worker threads.
 * Provides safe file operations with path validation and quota enforcement.
 */
export class ArtifactSandbox {
  private readonly config: ArtifactSandboxConfig;
  private readonly artifactDir: string;
  private readonly files = new Map<string, ArtifactFileEntry>();
  private totalSize = 0;

  constructor(config: ArtifactSandboxConfig) {
    this.config = config;
    this.artifactDir = path.resolve(config.rootPath, config.taskId);
  }

  /**
   * Initialize the sandbox by creating the artifact directory.
   */
  async initialize(): Promise<void> {
    try {
      await mkdir(this.artifactDir, { recursive: true });
    } catch (error) {
      throw new Error(
        `Failed to create artifact directory: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Write content to a file within the sandbox.
   *
   * @param relativePath - Path relative to artifact directory
   * @param content - File content as string or buffer
   * @throws ArtifactQuotaExceeded if quotas would be exceeded
   * @throws InvalidArtifactPath if path is invalid or escapes sandbox
   */
  async writeFile(
    relativePath: string,
    content: string | Buffer
  ): Promise<void> {
    this.validatePath(relativePath);

    const fullPath = path.join(this.artifactDir, relativePath);
    const contentBuffer = Buffer.isBuffer(content)
      ? content
      : Buffer.from(content, "utf8");

    // Check file size quota
    if (contentBuffer.length > this.config.maxFileSizeBytes) {
      throw new ArtifactQuotaExceeded(
        `File size ${contentBuffer.length} bytes exceeds limit of ${this.config.maxFileSizeBytes} bytes`,
        "size"
      );
    }

    // Check total size quota
    this.checkQuota(contentBuffer.length);

    // Ensure directory exists
    const dir = path.dirname(fullPath);
    if (dir !== this.artifactDir) {
      await mkdir(dir, { recursive: true });
    }

    // Write file
    await writeFile(fullPath, contentBuffer);

    // Generate SHA256 digest
    const sha256 = crypto
      .createHash("sha256")
      .update(contentBuffer)
      .digest("hex");

    // Detect MIME type (basic)
    const mimeType = this.detectMimeType(relativePath);

    // Track file metadata
    const fileEntry: ArtifactFileEntry = {
      path: relativePath,
      size: contentBuffer.length,
      sha256,
      mimeType,
      createdAt: new Date().toISOString(),
    };

    this.files.set(relativePath, fileEntry);
    this.totalSize += contentBuffer.length;

    console.log(
      `Artifact written: ${relativePath} (${contentBuffer.length} bytes)`
    );
  }

  /**
   * Create a directory within the sandbox.
   *
   * @param relativePath - Path relative to artifact directory
   * @throws InvalidArtifactPath if path is invalid or escapes sandbox
   */
  async mkdir(relativePath: string): Promise<void> {
    this.validatePath(relativePath);

    const fullPath = path.join(this.artifactDir, relativePath);
    await mkdir(fullPath, { recursive: true });

    console.log(`Artifact directory created: ${relativePath}`);
  }

  /**
   * List directory contents.
   *
   * @param relativePath - Path relative to artifact directory
   * @returns Array of file/directory names
   * @throws InvalidArtifactPath if path is invalid or escapes sandbox
   */
  async readdir(relativePath: string): Promise<string[]> {
    this.validatePath(relativePath);

    const fullPath = path.join(this.artifactDir, relativePath);
    return await readdir(fullPath);
  }

  /**
   * Get file/directory statistics.
   *
   * @param relativePath - Path relative to artifact directory
   * @returns File statistics
   * @throws InvalidArtifactPath if path is invalid or escapes sandbox
   */
  async stat(
    relativePath: string
  ): Promise<{ size: number; isFile: boolean; isDirectory: boolean }> {
    this.validatePath(relativePath);

    const fullPath = path.join(this.artifactDir, relativePath);
    const stats = await stat(fullPath);

    return {
      size: stats.size,
      isFile: stats.isFile(),
      isDirectory: stats.isDirectory(),
    };
  }

  /**
   * Rename a file or directory within the sandbox.
   *
   * @param oldPath - Current path relative to artifact directory
   * @param newPath - New path relative to artifact directory
   * @throws InvalidArtifactPath if either path is invalid or escapes sandbox
   */
  async rename(oldPath: string, newPath: string): Promise<void> {
    this.validatePath(oldPath);
    this.validatePath(newPath);

    const oldFullPath = path.join(this.artifactDir, oldPath);
    const newFullPath = path.join(this.artifactDir, newPath);

    await rename(oldFullPath, newFullPath);

    // Update tracking if this was a tracked file
    if (this.files.has(oldPath)) {
      const fileEntry = this.files.get(oldPath)!;
      fileEntry.path = newPath;
      this.files.delete(oldPath);
      this.files.set(newPath, fileEntry);
    }

    console.log(`Artifact renamed: ${oldPath} -> ${newPath}`);
  }

  /**
   * Generate a complete manifest of all artifacts.
   *
   * @returns Complete artifact manifest
   */
  generateManifest(): ArtifactManifest {
    return {
      taskId: this.config.taskId,
      files: Array.from(this.files.values()),
      totalSize: this.totalSize,
      createdAt: new Date().toISOString(),
    };
  }

  /**
   * Get the root path of this sandbox.
   *
   * @returns Absolute path to artifact directory
   */
  getRootPath(): string {
    return this.artifactDir;
  }

  /**
   * Generate the artifact manifest for this task.
   */
  getManifest(): ArtifactManifest {
    return {
      taskId: this.config.taskId,
      files: Array.from(this.files.values()),
      totalSize: this.totalSize,
      createdAt: new Date().toISOString(),
    };
  }

  /**
   * Validate that a path is safe and doesn't escape the sandbox.
   *
   * @param relativePath - Path to validate
   * @throws InvalidArtifactPath if path is invalid
   */
  private validatePath(relativePath: string): void {
    console.log(`Validating path: "${relativePath}"`);
    if (!relativePath || relativePath.trim() === "") {
      throw new InvalidArtifactPath("Path cannot be empty", relativePath);
    }

    // Reject absolute paths (platform-independent check)
    if (
      path.isAbsolute(relativePath) ||
      relativePath.match(/^[a-zA-Z]:/) || // Windows drive letter
      relativePath.startsWith("/") || // Unix absolute path
      relativePath.startsWith("\\")
    ) {
      // Windows absolute path
      console.log(`Rejecting absolute path: "${relativePath}"`);
      throw new InvalidArtifactPath(
        "Absolute paths are not allowed",
        relativePath
      );
    }

    // Normalize path and check for escapes
    const normalized = path.normalize(relativePath);
    const resolved = path.resolve(this.artifactDir, normalized);

    // Check for dangerous patterns AFTER normalization
    if (normalized.includes("..")) {
      throw new InvalidArtifactPath(
        "Path contains directory traversal (..)",
        relativePath
      );
    }

    // Check that resolved path is within artifact directory
    // Allow the artifact directory itself (for operations like readdir("."))
    if (
      resolved !== this.artifactDir &&
      !resolved.startsWith(this.artifactDir + path.sep)
    ) {
      throw new InvalidArtifactPath(
        "Path escapes sandbox directory",
        relativePath
      );
    }

    // Check path length
    if (normalized.length > this.config.maxPathLength) {
      throw new InvalidArtifactPath(
        `Path length ${normalized.length} exceeds limit of ${this.config.maxPathLength}`,
        relativePath
      );
    }

    // Check for null bytes (potential injection)
    if (normalized.includes("\0")) {
      throw new InvalidArtifactPath("Path contains null bytes", relativePath);
    }
  }

  /**
   * Check if adding additional bytes would exceed quota.
   *
   * @param additionalBytes - Bytes to add
   * @throws ArtifactQuotaExceeded if quota would be exceeded
   */
  private checkQuota(additionalBytes: number): void {
    // TODO: Implement comprehensive artifact quota management
    // - Track per-user, per-project, and global storage quotas
    // - Implement quota allocation and deallocation tracking
    // - Add quota soft and hard limits with warnings and enforcement
    // - Support quota inheritance and hierarchical management
    // - Implement quota monitoring and alerting
    // - Add quota usage analytics and forecasting
    // - Support dynamic quota adjustments based on usage patterns
    // - Implement quota cleanup and optimization strategies
    const newTotalSize = this.totalSize + additionalBytes;
    const maxTotalSize =
      this.config.maxFileSizeBytes * this.config.maxTotalFiles; // Rough estimate

    if (newTotalSize > maxTotalSize) {
      throw new ArtifactQuotaExceeded(
        `Total artifact size would exceed quota (${newTotalSize} > ${maxTotalSize})`,
        "size"
      );
    }

    // Check file count quota
    if (this.files.size >= this.config.maxTotalFiles) {
      throw new ArtifactQuotaExceeded(
        `File count ${this.files.size} exceeds limit of ${this.config.maxTotalFiles} files`,
        "files"
      );
    }
  }

  /**
   * Detect MIME type based on file extension.
   *
   * @param filePath - File path to analyze
   * @returns MIME type or undefined if not detectable
   */
  private detectMimeType(filePath: string): string | undefined {
    const ext = path.extname(filePath).toLowerCase();

    const mimeTypes: Record<string, string> = {
      ".json": "application/json",
      ".js": "application/javascript",
      ".ts": "application/typescript",
      ".html": "text/html",
      ".css": "text/css",
      ".md": "text/markdown",
      ".txt": "text/plain",
      ".yml": "text/yaml",
      ".yaml": "text/yaml",
      ".xml": "text/xml",
      ".csv": "text/csv",
      ".png": "image/png",
      ".jpg": "image/jpeg",
      ".jpeg": "image/jpeg",
      ".gif": "image/gif",
      ".svg": "image/svg+xml",
      ".pdf": "application/pdf",
    };

    return mimeTypes[ext];
  }
}
