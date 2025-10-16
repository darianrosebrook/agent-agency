/**
 * Integration tests for SpecFileManager
 *
 * Tests YAML conversion, file I/O, and cleanup operations.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import * as fs from "fs/promises";
import * as path from "path";
import { SpecFileManager } from "../../../src/caws-integration/utils/spec-file-manager";
import type { WorkingSpec } from "../../../src/types/caws-types";

describe("SpecFileManager Integration Tests", () => {
  const fixturesDir = path.join(__dirname, "../../fixtures/caws-integration");
  const tempDir = path.join(__dirname, "../../temp/spec-manager-tests");
  let manager: SpecFileManager;

  // Sample working spec for tests
  const sampleSpec: WorkingSpec = {
    id: "TEST-002",
    title: "Test Specification",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["src/test"],
      data_migration: false,
    },
    operational_rollback_slo: "5m",
    scope: {
      in: ["src/test/"],
      out: ["node_modules/"],
    },
    invariants: ["Test invariant"],
    acceptance: [
      {
        id: "A1",
        given: "Test condition",
        when: "Test action",
        then: "Test result",
      },
    ],
    non_functional: {
      perf: { api_p95_ms: 250 },
    },
    contracts: [],
  };

  beforeEach(async () => {
    // Create temp directory for tests
    await fs.mkdir(tempDir, { recursive: true });

    // Create manager with temp directory
    manager = new SpecFileManager({
      projectRoot: tempDir,
      useTemporaryFiles: false,
      tempDir,
    });
  });

  afterEach(async () => {
    // Clean up temp directory
    try {
      await fs.rm(tempDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  describe("YAML Conversion", () => {
    it("should convert WorkingSpec to YAML string", () => {
      const yaml = manager.specToYaml(sampleSpec);

      expect(yaml).toContain("id: TEST-002");
      expect(yaml).toContain("title: Test Specification");
      expect(yaml).toContain("risk_tier: 2");
      expect(yaml).toContain("mode: feature");
    });

    it("should parse YAML string to WorkingSpec", () => {
      const yaml = manager.specToYaml(sampleSpec);
      const parsed = manager.yamlToSpec(yaml);

      expect(parsed.id).toBe(sampleSpec.id);
      expect(parsed.title).toBe(sampleSpec.title);
      expect(parsed.risk_tier).toBe(sampleSpec.risk_tier);
      expect(parsed.mode).toBe(sampleSpec.mode);
    });

    it("should handle roundtrip conversion", () => {
      const yaml = manager.specToYaml(sampleSpec);
      const parsed = manager.yamlToSpec(yaml);
      const yaml2 = manager.specToYaml(parsed);

      expect(yaml).toBe(yaml2);
    });

    it("should throw error for invalid YAML", () => {
      expect(() => {
        manager.yamlToSpec("invalid: yaml: structure:");
      }).toThrow();
    });

    it("should throw error for incomplete spec", () => {
      const invalidYaml = "id: TEST-003\ntitle: Missing fields";

      expect(() => {
        manager.yamlToSpec(invalidYaml);
      }).toThrow("missing required fields");
    });
  });

  describe("File Operations", () => {
    it("should write spec to permanent file", async () => {
      const result = await manager.writeSpecFile(sampleSpec);

      expect(result.isTemporary).toBe(false);
      expect(result.filePath).toContain(".caws/working-spec.yaml");

      // Verify file exists
      const exists = await manager.specFileExists();
      expect(exists).toBe(true);
    });

    it("should read spec from file", async () => {
      // Write spec first
      await manager.writeSpecFile(sampleSpec);

      // Read it back
      const readSpec = await manager.readSpecFile();

      expect(readSpec.id).toBe(sampleSpec.id);
      expect(readSpec.title).toBe(sampleSpec.title);
    });

    it("should throw error when reading non-existent spec", async () => {
      await expect(manager.readSpecFile()).rejects.toThrow(
        "Working spec not found"
      );
    });

    it("should update existing spec", async () => {
      // Write initial spec
      await manager.writeSpecFile(sampleSpec);

      // Update it
      const updated = await manager.updateSpecFile({
        title: "Updated Title",
      });

      expect(updated.title).toBe("Updated Title");
      expect(updated.id).toBe(sampleSpec.id); // Other fields preserved

      // Verify persistence
      const read = await manager.readSpecFile();
      expect(read.title).toBe("Updated Title");
    });

    it("should get correct spec file path", () => {
      const specPath = manager.getSpecFilePath();

      expect(specPath).toContain(".caws");
      expect(specPath).toContain("working-spec.yaml");
      expect(path.isAbsolute(specPath)).toBe(true);
    });
  });

  describe("Temporary File Mode", () => {
    it("should write to temporary file when enabled", async () => {
      const tempManager = new SpecFileManager({
        projectRoot: tempDir,
        useTemporaryFiles: true,
        tempDir,
      });

      const result = await tempManager.writeSpecFile(sampleSpec);

      expect(result.isTemporary).toBe(true);
      expect(result.filePath).toContain(tempDir);
      expect(result.cleanup).toBeDefined();

      // Verify file exists
      const stats = await fs.stat(result.filePath);
      expect(stats.isFile()).toBe(true);
    });

    it("should cleanup temporary file", async () => {
      const tempManager = new SpecFileManager({
        projectRoot: tempDir,
        useTemporaryFiles: true,
        tempDir,
      });

      const result = await tempManager.writeSpecFile(sampleSpec);
      const tempPath = result.filePath;

      // Cleanup
      await result.cleanup!();

      // Verify file is deleted
      await expect(fs.stat(tempPath)).rejects.toThrow();
    });
  });

  describe("Backup and Restore", () => {
    it("should create backup of spec file", async () => {
      // Write initial spec
      await manager.writeSpecFile(sampleSpec);

      // Create backup
      const backupPath = await manager.backupSpecFile();

      expect(backupPath).toContain("backup");

      // Verify backup exists
      const stats = await fs.stat(backupPath);
      expect(stats.isFile()).toBe(true);

      // Cleanup backup
      await fs.unlink(backupPath);
    });

    it("should restore spec from backup", async () => {
      // Write initial spec
      await manager.writeSpecFile(sampleSpec);

      // Create backup
      const backupPath = await manager.backupSpecFile();

      // Modify spec
      await manager.updateSpecFile({ title: "Modified Title" });

      // Restore from backup
      await manager.restoreSpecFile(backupPath);

      // Verify restoration
      const restored = await manager.readSpecFile();
      expect(restored.title).toBe(sampleSpec.title);

      // Cleanup backup
      await fs.unlink(backupPath);
    });
  });

  describe("Validation", () => {
    it("should validate existing spec file", async () => {
      // Write valid spec
      await manager.writeSpecFile(sampleSpec);

      const result = await manager.validateSpecFile();

      expect(result.valid).toBe(true);
      expect(result.error).toBeUndefined();
      expect(result.spec).toBeDefined();
      expect(result.spec?.id).toBe(sampleSpec.id);
    });

    it("should return error for invalid spec", async () => {
      // Write invalid YAML directly
      const specPath = manager.getSpecFilePath();
      await fs.mkdir(path.dirname(specPath), { recursive: true });
      await fs.writeFile(specPath, "invalid: yaml: structure:", "utf-8");

      const result = await manager.validateSpecFile();

      expect(result.valid).toBe(false);
      expect(result.error).toBeDefined();
      expect(result.spec).toBeUndefined();
    });

    it("should return error for non-existent spec", async () => {
      const result = await manager.validateSpecFile();

      expect(result.valid).toBe(false);
      expect(result.error).toBeDefined();
      expect(result.error).toContain("not found");
    });
  });

  describe("Cleanup Operations", () => {
    it("should cleanup old temporary files", async () => {
      const tempManager = new SpecFileManager({
        projectRoot: tempDir,
        useTemporaryFiles: true,
        tempDir,
      });

      // Create some temporary files manually with old timestamps
      const oldTimestamp = Date.now() - 10000; // 10 seconds ago
      const tempPath1 = path.join(
        tempDir,
        `caws-spec-TEST-001-${oldTimestamp}.yaml`
      );
      const tempPath2 = path.join(
        tempDir,
        `caws-spec-TEST-002-${oldTimestamp}.yaml`
      );

      await fs.writeFile(tempPath1, manager.specToYaml(sampleSpec), "utf-8");
      await fs.writeFile(
        tempPath2,
        manager.specToYaml({ ...sampleSpec, id: "TEST-003" }),
        "utf-8"
      );

      // Verify they exist
      await expect(fs.stat(tempPath1)).resolves.toBeDefined();
      await expect(fs.stat(tempPath2)).resolves.toBeDefined();

      // Cleanup with 0ms max age (should clean all)
      const cleaned = await tempManager.cleanupTempFiles(0);

      expect(cleaned).toBeGreaterThanOrEqual(1); // At least one file cleaned

      // Verify files are deleted
      await expect(fs.stat(tempPath1)).rejects.toThrow();
      await expect(fs.stat(tempPath2)).rejects.toThrow();
    });

    it("should not cleanup recent temporary files", async () => {
      const tempManager = new SpecFileManager({
        projectRoot: tempDir,
        useTemporaryFiles: true,
        tempDir,
      });

      // Create temporary file
      const result = await tempManager.writeSpecFile(sampleSpec);
      const tempPath = result.filePath;

      // Cleanup with 1 hour max age (should not clean recent files)
      const cleaned = await tempManager.cleanupTempFiles(3600000);

      expect(cleaned).toBe(0);

      // Verify file still exists
      await expect(fs.stat(tempPath)).resolves.toBeDefined();

      // Manual cleanup
      await result.cleanup!();
    });
  });

  describe("Integration with Fixture Files", () => {
    it("should read fixture working spec", async () => {
      const fixtureManager = new SpecFileManager({
        projectRoot: fixturesDir,
        useTemporaryFiles: false,
      });

      const spec = await fixtureManager.readSpecFile();

      expect(spec.id).toBe("TEST-001");
      expect(spec.title).toBe("Test Working Specification");
      expect(spec.risk_tier).toBe(2);
    });

    it("should validate fixture working spec", async () => {
      const fixtureManager = new SpecFileManager({
        projectRoot: fixturesDir,
        useTemporaryFiles: false,
      });

      const result = await fixtureManager.validateSpecFile();

      expect(result.valid).toBe(true);
      expect(result.spec).toBeDefined();
      expect(result.spec?.id).toBe("TEST-001");
    });
  });
});
