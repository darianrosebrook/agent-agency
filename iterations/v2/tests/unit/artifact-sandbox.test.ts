/**
 * Unit Tests for Artifact Sandbox Security and Path Validation
 *
 * @author @darianrosebrook
 */

import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import * as fs from "fs";
import * as path from "path";
import {
  ArtifactQuotaExceeded,
  ArtifactSandbox,
  InvalidArtifactPath,
} from "../../src/orchestrator/workers/ArtifactSandbox.js";

describe("Artifact Sandbox Security", () => {
  const testRoot = path.join(__dirname, "..", "fixtures", "artifacts");
  const taskId = "test-task-123";

  const sandboxConfig = {
    rootPath: testRoot,
    taskId,
    maxFileSizeBytes: 1024 * 1024, // 1MB
    maxTotalFiles: 10,
    maxPathLength: 100,
  };

  let sandbox: ArtifactSandbox;

  beforeEach(async () => {
    // Clean up any existing test artifacts
    if (fs.existsSync(testRoot)) {
      fs.rmSync(testRoot, { recursive: true, force: true });
    }
    fs.mkdirSync(testRoot, { recursive: true });

    sandbox = new ArtifactSandbox(sandboxConfig);
    await sandbox.initialize();
  });

  afterEach(() => {
    // Clean up test artifacts
    if (fs.existsSync(testRoot)) {
      fs.rmSync(testRoot, { recursive: true, force: true });
    }
  });

  describe("Path Validation", () => {
    it("should reject paths that escape the sandbox", async () => {
      const escapePaths = [
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32",
        "/absolute/path/outside",
        "C:\\absolute\\windows\\path",
        "../../../../escape.txt",
        "./../escape.txt",
        "../escape.txt",
      ];

      for (const escapePath of escapePaths) {
        await expect(
          sandbox.writeFile(escapePath, "malicious content")
        ).rejects.toThrow(InvalidArtifactPath);
      }
    });

    it("should reject paths with null bytes", async () => {
      const nullBytePaths = ["file.txt\0evil", "file\0.txt", "file.txt\x00"];

      for (const nullPath of nullBytePaths) {
        await expect(sandbox.writeFile(nullPath, "content")).rejects.toThrow(
          InvalidArtifactPath
        );
      }
    });

    it("should reject paths that are too long", async () => {
      const longPath = "a".repeat(sandboxConfig.maxPathLength + 1);
      await expect(sandbox.writeFile(longPath, "content")).rejects.toThrow(
        InvalidArtifactPath
      );
    });

    it("should reject empty paths", async () => {
      await expect(sandbox.writeFile("", "content")).rejects.toThrow(
        InvalidArtifactPath
      );
      await expect(sandbox.writeFile("   ", "content")).rejects.toThrow(
        InvalidArtifactPath
      );
    });

    it("should accept valid relative paths", async () => {
      const validPaths = [
        "file.txt",
        "subdir/file.txt",
        "deep/nested/path/file.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
        "file123.txt",
        "file.name.with.dots.txt",
        "valid/../escape.txt",
      ];

      for (const validPath of validPaths) {
        await expect(
          sandbox.writeFile(validPath, "content")
        ).resolves.toBeUndefined();
      }
    });

    it("should normalize paths correctly", async () => {
      // These should all resolve to the same normalized path
      const equivalentPaths = ["subdir/../file.txt", "./file.txt", "file.txt"];

      for (const equivalentPath of equivalentPaths) {
        await expect(
          sandbox.writeFile(equivalentPath, "content")
        ).resolves.toBeUndefined();
      }

      // Should only create one file
      const files = await sandbox.readdir(".");
      expect(files.filter((f) => f === "file.txt")).toHaveLength(1);
    });
  });

  describe("Quota Enforcement", () => {
    it("should reject files larger than maxFileSizeBytes", async () => {
      const largeContent = "x".repeat(sandboxConfig.maxFileSizeBytes + 1);
      await expect(
        sandbox.writeFile("large.txt", largeContent)
      ).rejects.toThrow(ArtifactQuotaExceeded);
    });

    it("should accept files at the size limit", async () => {
      const exactSizeContent = "x".repeat(sandboxConfig.maxFileSizeBytes);
      await expect(
        sandbox.writeFile("exact.txt", exactSizeContent)
      ).resolves.toBeUndefined();
    });

    it("should enforce total file count limit", async () => {
      // Write up to the limit
      for (let i = 0; i < sandboxConfig.maxTotalFiles; i++) {
        await sandbox.writeFile(`file${i}.txt`, `content ${i}`);
      }

      // Next file should be rejected
      await expect(
        sandbox.writeFile("overflow.txt", "content")
      ).rejects.toThrow(ArtifactQuotaExceeded);
    });

    it("should track cumulative size across files", async () => {
      // This test assumes a reasonable total size limit based on individual file limits
      // In practice, the total size limit is derived from individual limits
      const fileSize = 100 * 1024; // 100KB each
      const fileCount = 5;

      for (let i = 0; i < fileCount; i++) {
        const content = "x".repeat(fileSize);
        await sandbox.writeFile(`file${i}.txt`, content);
      }

      // Verify manifest reflects cumulative size
      const manifest = sandbox.generateManifest();
      expect(manifest.totalSize).toBe(fileSize * fileCount);
    });
  });

  describe("File Operations", () => {
    it("should create directories automatically", async () => {
      await sandbox.writeFile("deep/nested/path/file.txt", "content");

      // Should be able to read the directory
      const dirs = await sandbox.readdir("deep/nested/path");
      expect(dirs).toContain("file.txt");

      // Should be able to read parent directories
      const nested = await sandbox.readdir("deep/nested");
      expect(nested).toContain("path");

      const deep = await sandbox.readdir("deep");
      expect(deep).toContain("nested");
    });

    it("should support reading directories", async () => {
      await sandbox.writeFile("file1.txt", "content1");
      await sandbox.writeFile("file2.txt", "content2");
      await sandbox.writeFile("subdir/file3.txt", "content3");

      const rootFiles = await sandbox.readdir(".");
      expect(rootFiles).toContain("file1.txt");
      expect(rootFiles).toContain("file2.txt");
      expect(rootFiles).toContain("subdir");

      const subdirFiles = await sandbox.readdir("subdir");
      expect(subdirFiles).toContain("file3.txt");
    });

    it("should support file statistics", async () => {
      const content = "test content";
      await sandbox.writeFile("test.txt", content);

      const stats = await sandbox.stat("test.txt");
      expect(stats.size).toBe(Buffer.from(content).length);
      expect(stats.isFile).toBe(true);
      expect(stats.isDirectory).toBe(false);
    });

    it("should support directory statistics", async () => {
      await sandbox.mkdir("testdir");

      const stats = await sandbox.stat("testdir");
      expect(stats.size).toBeGreaterThanOrEqual(0);
      expect(stats.isFile).toBe(false);
      expect(stats.isDirectory).toBe(true);
    });

    it("should support renaming files", async () => {
      await sandbox.writeFile("oldname.txt", "content");

      // Verify old file exists
      let stats = await sandbox.stat("oldname.txt");
      expect(stats.isFile).toBe(true);

      // Rename
      await sandbox.rename("oldname.txt", "newname.txt");

      // Verify old file is gone
      await expect(sandbox.stat("oldname.txt")).rejects.toThrow();

      // Verify new file exists
      stats = await sandbox.stat("newname.txt");
      expect(stats.isFile).toBe(true);
    });

    it("should support renaming directories", async () => {
      await sandbox.writeFile("olddir/file.txt", "content");

      // Rename directory
      await sandbox.rename("olddir", "newdir");

      // Verify new structure
      const stats = await sandbox.stat("newdir");
      expect(stats.isDirectory).toBe(true);

      const files = await sandbox.readdir("newdir");
      expect(files).toContain("file.txt");
    });
  });

  describe("Manifest Generation", () => {
    it("should generate correct manifest for single file", async () => {
      const content = "test content";
      const filePath = "test.txt";
      await sandbox.writeFile(filePath, content);

      const manifest = sandbox.generateManifest();

      expect(manifest.taskId).toBe(taskId);
      expect(manifest.files).toHaveLength(1);
      expect(manifest.totalSize).toBe(Buffer.from(content).length);

      const fileEntry = manifest.files[0];
      expect(fileEntry.path).toBe(filePath);
      expect(fileEntry.size).toBe(Buffer.from(content).length);
      expect(fileEntry.sha256).toBeDefined();
      expect(fileEntry.sha256.length).toBe(64); // SHA256 hex length
      expect(fileEntry.createdAt).toBeDefined();
    });

    it("should generate manifest with correct SHA256 digests", async () => {
      const content = "test content for hashing";
      await sandbox.writeFile("hash-test.txt", content);

      const manifest = sandbox.generateManifest();
      const fileEntry = manifest.files[0];

      // Verify SHA256 is correct by recalculating
      const crypto = await import("crypto");
      const expectedHash = crypto
        .createHash("sha256")
        .update(content)
        .digest("hex");

      expect(fileEntry.sha256).toBe(expectedHash);
    });

    it("should include MIME types for known extensions", async () => {
      const testFiles = [
        {
          path: "test.json",
          content: '{"test": true}',
          mime: "application/json",
        },
        {
          path: "test.js",
          content: "console.log('test');",
          mime: "application/javascript",
        },
        { path: "test.html", content: "<html></html>", mime: "text/html" },
        { path: "test.css", content: "body { color: red; }", mime: "text/css" },
        { path: "test.unknown", content: "unknown content", mime: undefined },
      ];

      for (const { path, content, mime } of testFiles) {
        await sandbox.writeFile(path, content);
      }

      const manifest = sandbox.generateManifest();

      for (const file of manifest.files) {
        const expectedMime = testFiles.find(
          (tf) => tf.path === file.path
        )?.mime;
        expect(file.mimeType).toBe(expectedMime);
      }
    });

    it("should track multiple files correctly", async () => {
      const files = [
        { path: "file1.txt", content: "content 1" },
        { path: "file2.txt", content: "content 2" },
        { path: "subdir/file3.txt", content: "content 3" },
      ];

      for (const { path, content } of files) {
        await sandbox.writeFile(path, content);
      }

      const manifest = sandbox.generateManifest();

      expect(manifest.files).toHaveLength(3);
      expect(manifest.totalSize).toBe(
        files.reduce((sum, f) => sum + Buffer.from(f.content).length, 0)
      );

      // Verify all files are in manifest
      const manifestPaths = manifest.files.map((f) => f.path).sort();
      const expectedPaths = files.map((f) => f.path).sort();
      expect(manifestPaths).toEqual(expectedPaths);
    });

    it("should update manifest after file operations", async () => {
      // Create file
      await sandbox.writeFile("test.txt", "original content");
      let manifest = sandbox.generateManifest();
      expect(manifest.files).toHaveLength(1);
      expect(manifest.totalSize).toBe("original content".length);

      // Rename file
      await sandbox.rename("test.txt", "renamed.txt");
      manifest = sandbox.generateManifest();
      expect(manifest.files).toHaveLength(1);
      expect(manifest.files[0].path).toBe("renamed.txt");

      // Add another file
      await sandbox.writeFile("another.txt", "more content");
      manifest = sandbox.generateManifest();
      expect(manifest.files).toHaveLength(2);
      expect(manifest.totalSize).toBe(
        "original content".length + "more content".length
      );
    });
  });

  describe("Error Handling", () => {
    it("should handle filesystem errors gracefully", async () => {
      // Try to read non-existent file
      await expect(sandbox.readdir("nonexistent")).rejects.toThrow();

      // Try to stat non-existent file
      await expect(sandbox.stat("nonexistent.txt")).rejects.toThrow();

      // Try to rename non-existent file
      await expect(
        sandbox.rename("nonexistent.txt", "newname.txt")
      ).rejects.toThrow();
    });

    it("should provide descriptive error messages", async () => {
      try {
        await sandbox.writeFile("../../../escape.txt", "content");
        throw new Error("Expected InvalidArtifactPath to be thrown");
      } catch (error: any) {
        expect(error).toBeInstanceOf(InvalidArtifactPath);
        expect(error.message).toContain("directory traversal");
        expect(error.path).toBe("../../../escape.txt");
      }
    });

    it("should maintain sandbox integrity after errors", async () => {
      // Try invalid operation
      await expect(
        sandbox.writeFile("../../../escape.txt", "content")
      ).rejects.toThrow();

      // Should still be able to do valid operations
      await expect(
        sandbox.writeFile("valid.txt", "content")
      ).resolves.toBeUndefined();

      const manifest = sandbox.generateManifest();
      expect(manifest.files).toHaveLength(1);
      expect(manifest.files[0].path).toBe("valid.txt");
    });
  });
});
