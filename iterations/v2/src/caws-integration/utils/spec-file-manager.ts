/**
 * Spec File Manager - WorkingSpec ↔ YAML conversion and file management
 *
 * Handles conversion between TypeScript WorkingSpec objects and YAML files,
 * manages .caws/working-spec.yaml lifecycle, and provides temporary file utilities.
 *
 * @author @darianrosebrook
 */

import * as fs from "fs/promises";
import * as yaml from "js-yaml";
import * as path from "path";
import type { WorkingSpec } from "../../types/caws-types.js";

/**
 * Configuration for spec file operations
 */
export interface SpecFileManagerConfig {
  /** Project root directory */
  projectRoot: string;
  /** Use temporary files for validation (default: true) */
  useTemporaryFiles?: boolean;
  /** Temporary directory path (default: system temp) */
  tempDir?: string;
}

/**
 * Result of spec file write operation
 */
export interface SpecFileWriteResult {
  /** Path to written file */
  filePath: string;
  /** Whether file was written to temp location */
  isTemporary: boolean;
  /** Cleanup function (only for temporary files) */
  cleanup?: () => Promise<void>;
}

/**
 * Manages WorkingSpec file operations and YAML conversion
 */
export class SpecFileManager {
  private readonly projectRoot: string;
  private readonly useTemporaryFiles: boolean;
  private readonly tempDir: string;

  constructor(config: SpecFileManagerConfig) {
    this.projectRoot = config.projectRoot;
    this.useTemporaryFiles = config.useTemporaryFiles ?? true;
    this.tempDir = config.tempDir ?? "/tmp";
  }

  /**
   * Convert WorkingSpec object to YAML string
   *
   * @param spec WorkingSpec to convert
   * @returns YAML string representation
   */
  public specToYaml(spec: WorkingSpec): string {
    return yaml.dump(spec, {
      indent: 2,
      lineWidth: 100,
      noRefs: true,
      sortKeys: false,
    });
  }

  /**
   * Parse YAML string to WorkingSpec object
   *
   * @param yamlContent YAML string to parse
   * @returns Parsed WorkingSpec object
   * @throws Error if YAML is invalid or doesn't match WorkingSpec schema
   */
  public yamlToSpec(yamlContent: string): WorkingSpec {
    try {
      const parsed = yaml.load(yamlContent) as WorkingSpec;

      // Basic validation
      if (!parsed || typeof parsed !== "object") {
        throw new Error("Invalid YAML: not an object");
      }

      if (!parsed.id || !parsed.title || !parsed.risk_tier) {
        throw new Error("Invalid WorkingSpec: missing required fields");
      }

      return parsed;
    } catch (error) {
      if (error instanceof Error) {
        throw new Error(`Failed to parse YAML: ${error.message}`);
      }
      throw new Error("Failed to parse YAML: unknown error");
    }
  }

  /**
   * Get path to .caws/working-spec.yaml in project
   *
   * @returns Absolute path to working spec file
   */
  public getSpecFilePath(): string {
    return path.join(this.projectRoot, ".caws", "working-spec.yaml");
  }

  /**
   * Check if working spec file exists
   *
   * @returns True if file exists
   */
  public async specFileExists(): Promise<boolean> {
    try {
      await fs.access(this.getSpecFilePath());
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Read working spec from .caws/working-spec.yaml
   *
   * @returns Parsed WorkingSpec object
   * @throws Error if file doesn't exist or is invalid
   */
  public async readSpecFile(): Promise<WorkingSpec> {
    const specPath = this.getSpecFilePath();

    try {
      const content = await fs.readFile(specPath, "utf-8");
      return this.yamlToSpec(content);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") {
        throw new Error(
          `Working spec not found: ${specPath}\nRun 'caws init' to create it`
        );
      }
      throw error;
    }
  }

  /**
   * Write WorkingSpec to file
   *
   * Writes to .caws/working-spec.yaml or a temporary file based on configuration.
   *
   * @param spec WorkingSpec to write
   * @returns Write result with file path and cleanup function
   */
  public async writeSpecFile(spec: WorkingSpec): Promise<SpecFileWriteResult> {
    const yamlContent = this.specToYaml(spec);

    if (this.useTemporaryFiles) {
      // Write to temporary file
      const tempPath = path.join(
        this.tempDir,
        `caws-spec-${spec.id}-${Date.now()}.yaml`
      );

      await fs.writeFile(tempPath, yamlContent, "utf-8");

      return {
        filePath: tempPath,
        isTemporary: true,
        cleanup: async () => {
          try {
            await fs.unlink(tempPath);
          } catch {
            // Ignore cleanup errors (file may already be deleted)
          }
        },
      };
    } else {
      // Write to project .caws directory
      const specPath = this.getSpecFilePath();
      const cawsDir = path.dirname(specPath);

      // Ensure .caws directory exists
      await fs.mkdir(cawsDir, { recursive: true });

      await fs.writeFile(specPath, yamlContent, "utf-8");

      return {
        filePath: specPath,
        isTemporary: false,
      };
    }
  }

  /**
   * Update existing working spec file
   *
   * Reads current spec, merges changes, and writes back.
   *
   * @param updates Partial WorkingSpec with fields to update
   * @returns Updated WorkingSpec
   */
  public async updateSpecFile(
    updates: Partial<WorkingSpec>
  ): Promise<WorkingSpec> {
    const currentSpec = await this.readSpecFile();
    const updatedSpec: WorkingSpec = {
      ...currentSpec,
      ...updates,
    };

    // Write to permanent location (not temporary)
    const originalUseTemp = this.useTemporaryFiles;
    (this as any).useTemporaryFiles = false;

    try {
      await this.writeSpecFile(updatedSpec);
      return updatedSpec;
    } finally {
      (this as any).useTemporaryFiles = originalUseTemp;
    }
  }

  /**
   * Create backup of working spec
   *
   * @returns Path to backup file
   */
  public async backupSpecFile(): Promise<string> {
    const specPath = this.getSpecFilePath();
    const backupPath = `${specPath}.backup-${Date.now()}`;

    await fs.copyFile(specPath, backupPath);

    return backupPath;
  }

  /**
   * Restore working spec from backup
   *
   * @param backupPath Path to backup file
   */
  public async restoreSpecFile(backupPath: string): Promise<void> {
    const specPath = this.getSpecFilePath();
    await fs.copyFile(backupPath, specPath);
  }

  /**
   * Validate spec file exists and is parseable
   *
   * @returns Validation result
   */
  public async validateSpecFile(): Promise<{
    valid: boolean;
    error?: string;
    spec?: WorkingSpec;
  }> {
    try {
      const spec = await this.readSpecFile();
      return {
        valid: true,
        spec,
      };
    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  /**
   * Clean up old temporary spec files
   *
   * Removes temp files older than specified age.
   *
   * @param maxAgeMs Maximum age in milliseconds (default: 1 hour)
   * @returns Number of files cleaned up
   */
  public async cleanupTempFiles(maxAgeMs = 3600000): Promise<number> {
    try {
      const files = await fs.readdir(this.tempDir);
      const specFiles = files.filter((f) => f.startsWith("caws-spec-"));

      let cleaned = 0;
      const now = Date.now();

      for (const file of specFiles) {
        const filePath = path.join(this.tempDir, file);
        try {
          const stats = await fs.stat(filePath);
          const age = now - stats.mtimeMs;

          if (age > maxAgeMs) {
            await fs.unlink(filePath);
            cleaned++;
          }
        } catch {
          // Skip files that can't be accessed
        }
      }

      return cleaned;
    } catch {
      return 0;
    }
  }
}

/**
 * Create a SpecFileManager instance with default configuration
 *
 * @param projectRoot Project root directory
 * @param useTemporaryFiles Whether to use temporary files (default: true)
 * @returns SpecFileManager instance
 */
export function createSpecFileManager(
  projectRoot: string,
  useTemporaryFiles = true
): SpecFileManager {
  return new SpecFileManager({
    projectRoot,
    useTemporaryFiles,
  });
}
