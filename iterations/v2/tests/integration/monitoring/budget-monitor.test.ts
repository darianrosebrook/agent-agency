/**
 * Integration tests for BudgetMonitor
 *
 * Tests real-time budget monitoring with file watching and threshold alerts.
 *
 * @author @darianrosebrook
 */

import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import * as fs from "fs/promises";
import * as path from "path";
import { BudgetMonitor } from "../../../src/monitoring/BudgetMonitor";
import type {
  BudgetAlert,
  BudgetUsage,
  FileChangeEvent,
} from "../../../src/monitoring/types/budget-monitor-types";
import type { WorkingSpec } from "../../../src/types/caws-types";

describe("BudgetMonitor Integration Tests", () => {
  const tempDir = path.join(__dirname, "../../temp/budget-monitor-tests");
  const projectRoot = path.join(tempDir, "project");
  let monitor: BudgetMonitor;

  const validSpec: WorkingSpec = {
    id: "TEST-BUDGET-001",
    title: "Budget Monitor Test Spec",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["src/test"],
      data_migration: false,
    },
    operational_rollback_slo: "5m",
    scope: {
      in: ["src/"],
      out: ["node_modules/", "dist/"],
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
    non_functional: {},
    contracts: [],
  };

  beforeEach(async () => {
    // Create temp directory structure
    await fs.mkdir(projectRoot, { recursive: true });
    await fs.mkdir(path.join(projectRoot, "src"), { recursive: true });
    await fs.mkdir(path.join(projectRoot, ".caws"), { recursive: true });

    // Write policy file
    const policyPath = path.join(projectRoot, ".caws", "policy.yaml");
    await fs.writeFile(
      policyPath,
      `version: "1.0.0"
risk_tiers:
  2:
    max_files: 5
    max_loc: 100
    coverage_threshold: 0.80
    mutation_threshold: 0.70
    contracts_required: true
    manual_review_required: false
`
    );
  });

  afterEach(async () => {
    if (monitor) {
      await monitor.stop();
    }

    try {
      await fs.rm(tempDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  describe("Initialization", () => {
    it("should create monitor with default config", () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
      });

      expect(monitor).toBeDefined();
    });

    it("should create monitor with custom thresholds", () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        thresholds: {
          warning: 0.3,
          critical: 0.7,
          exceeded: 0.9,
        },
      });

      expect(monitor).toBeDefined();
    });

    it("should start monitoring successfully", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false, // Disable file watching for this test
      });

      await monitor.start();

      const status = monitor.getStatus();
      expect(status.active).toBe(false); // File watching disabled
      expect(status.startedAt).toBeDefined();
    });

    it("should throw error if started twice", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: true,
      });

      await monitor.start();
      await expect(monitor.start()).rejects.toThrow("already running");
    });
  });

  describe("Budget Calculation", () => {
    it("should calculate initial budget usage", async () => {
      // Create some initial files
      await fs.writeFile(path.join(projectRoot, "src", "file1.ts"), "// File 1\n");
      await fs.writeFile(path.join(projectRoot, "src", "file2.ts"), "// File 2\n");

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      await monitor.start();
      const status = monitor.getStatus();

      expect(status.currentUsage.filesChanged).toBeGreaterThan(0);
      expect(status.currentUsage.maxFiles).toBe(5);
      expect(status.currentUsage.filesPercentage).toBeGreaterThan(0);
    });

    it("should calculate LOC correctly", async () => {
      const fileContent = Array(10).fill("console.log('test');").join("\n");
      await fs.writeFile(path.join(projectRoot, "src", "file.ts"), fileContent);

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      await monitor.start();
      const status = monitor.getStatus();

      expect(status.currentUsage.linesChanged).toBeGreaterThanOrEqual(10);
    });

    it("should respect scope.in patterns", async () => {
      // Create file inside scope
      await fs.writeFile(path.join(projectRoot, "src", "in-scope.ts"), "// In scope\n");

      // Create file outside scope
      await fs.mkdir(path.join(projectRoot, "lib"), { recursive: true });
      await fs.writeFile(
        path.join(projectRoot, "lib", "out-of-scope.ts"),
        "// Out of scope\n"
      );

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      await monitor.start();
      const status = monitor.getStatus();

      const inScopeFiles = status.currentUsage.changedFiles.filter((f) =>
        f.startsWith("src/")
      );
      const outOfScopeFiles = status.currentUsage.changedFiles.filter((f) =>
        f.startsWith("lib/")
      );

      expect(inScopeFiles.length).toBeGreaterThan(0);
      expect(outOfScopeFiles.length).toBe(0);
    });
  });

  describe("Threshold Alerts", () => {
    it("should generate warning alert at 50% threshold", async () => {
      const alerts: BudgetAlert[] = [];

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
        thresholds: {
          warning: 0.5,
          critical: 0.8,
          exceeded: 0.95,
        },
        onAlert: (alert) => {
          alerts.push(alert);
        },
      });

      // Create 3 files (60% of 5 files budget)
      await fs.writeFile(path.join(projectRoot, "src", "file1.ts"), "// File 1\n");
      await fs.writeFile(path.join(projectRoot, "src", "file2.ts"), "// File 2\n");
      await fs.writeFile(path.join(projectRoot, "src", "file3.ts"), "// File 3\n");

      await monitor.start();

      // Check that warning alert was generated
      expect(alerts.length).toBeGreaterThan(0);
      const warningAlert = alerts.find((a) => a.severity === "warning");
      expect(warningAlert).toBeDefined();
    });

    it("should generate critical alert at 80% threshold", async () => {
      const alerts: BudgetAlert[] = [];

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
        thresholds: {
          warning: 0.5,
          critical: 0.8,
          exceeded: 0.95,
        },
        onAlert: (alert) => {
          alerts.push(alert);
        },
      });

      // Create 4 files (80% of 5 files budget)
      for (let i = 1; i <= 4; i++) {
        await fs.writeFile(
          path.join(projectRoot, "src", `file${i}.ts`),
          `// File ${i}\n`
        );
      }

      await monitor.start();

      const criticalAlert = alerts.find((a) => a.severity === "critical");
      expect(criticalAlert).toBeDefined();
    });

    it("should emit budget:threshold event", async () => {
      let thresholdEmitted = false;

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
        thresholds: {
          warning: 0.5,
        },
      });

      monitor.on("budget:threshold", () => {
        thresholdEmitted = true;
      });

      // Create files to trigger threshold
      await fs.writeFile(path.join(projectRoot, "src", "file1.ts"), "// File 1\n");
      await fs.writeFile(path.join(projectRoot, "src", "file2.ts"), "// File 2\n");
      await fs.writeFile(path.join(projectRoot, "src", "file3.ts"), "// File 3\n");

      await monitor.start();

      expect(thresholdEmitted).toBe(true);
    });
  });

  describe("Event Emitters", () => {
    it("should emit budget:update event", async () => {
      let updateEmitted = false;
      let usageData: BudgetUsage | undefined;

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      monitor.on("budget:update", (usage: BudgetUsage) => {
        updateEmitted = true;
        usageData = usage;
      });

      await fs.writeFile(path.join(projectRoot, "src", "file.ts"), "// Test\n");
      await monitor.start();

      expect(updateEmitted).toBe(true);
      expect(usageData).toBeDefined();
      if (usageData) {
        expect(usageData.filesChanged).toBeGreaterThan(0);
      }
    });

    it("should emit monitor:start event", async () => {
      let startEmitted = false;

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      monitor.on("monitor:start", () => {
        startEmitted = true;
      });

      await monitor.start();

      expect(startEmitted).toBe(true);
    });

    it("should emit monitor:stop event", async () => {
      let stopEmitted = false;

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: true,
      });

      monitor.on("monitor:stop", () => {
        stopEmitted = true;
      });

      await monitor.start();
      await monitor.stop();

      expect(stopEmitted).toBe(true);
    });
  });

  describe("File Watching", () => {
    it("should detect new file creation", async () => {
      let fileChangeDetected = false;
      let changeEvent: FileChangeEvent | undefined;

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: true,
      });

      monitor.on("file:change", (event: FileChangeEvent) => {
        fileChangeDetected = true;
        changeEvent = event;
      });

      await monitor.start();

      // Wait for watcher to initialize
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Create a new file
      await fs.writeFile(path.join(projectRoot, "src", "new-file.ts"), "// New file\n");

      // Wait for file change to be detected
      await new Promise((resolve) => setTimeout(resolve, 200));

      expect(fileChangeDetected).toBe(true);
      if (changeEvent) {
        expect(changeEvent.type).toBe("add");
      }
    });

    it("should detect file modifications", async () => {
      // Create initial file
      const filePath = path.join(projectRoot, "src", "existing.ts");
      await fs.writeFile(filePath, "// Original content\n");

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: true,
      });

      let changeDetected = false;
      monitor.on("file:change", (event) => {
        if (event.type === "change") {
          changeDetected = true;
        }
      });

      await monitor.start();
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Modify the file
      await fs.writeFile(filePath, "// Modified content\n");
      await new Promise((resolve) => setTimeout(resolve, 200));

      expect(changeDetected).toBe(true);
    });
  });

  describe("Statistics", () => {
    it("should track monitoring statistics", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      await fs.writeFile(path.join(projectRoot, "src", "file1.ts"), "// File 1\n");
      await fs.writeFile(path.join(projectRoot, "src", "file2.ts"), "// File 2\n");

      await monitor.start();
      const stats = monitor.getStatistics();

      expect(stats).toBeDefined();
      expect(stats.monitoringDuration).toBeGreaterThan(0);
      expect(stats.totalFileChanges).toBeGreaterThan(0);
    });

    it("should track peak usage", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      await fs.writeFile(path.join(projectRoot, "src", "file.ts"), "// File\n");
      await monitor.start();

      const stats = monitor.getStatistics();
      expect(stats.peakFilesUsage).toBeGreaterThan(0);
    });

    it("should track alerts by severity", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
        thresholds: { warning: 0.5, critical: 0.8 },
      });

      // Create files to trigger alerts
      for (let i = 1; i <= 3; i++) {
        await fs.writeFile(
          path.join(projectRoot, "src", `file${i}.ts`),
          `// File ${i}\n`
        );
      }

      await monitor.start();
      const stats = monitor.getStatistics();

      const totalAlerts =
        stats.alertsBySeverity.info +
        stats.alertsBySeverity.warning +
        stats.alertsBySeverity.critical;

      expect(totalAlerts).toBeGreaterThan(0);
    });
  });

  describe("Recommendations", () => {
    it("should provide recommendations for high budget usage", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      // Create files to exceed 80% threshold
      for (let i = 1; i <= 4; i++) {
        await fs.writeFile(
          path.join(projectRoot, "src", `file${i}.ts`),
          `// File ${i}\n`
        );
      }

      await monitor.start();
      const recommendations = monitor.getRecommendations();

      expect(recommendations.length).toBeGreaterThan(0);
      const highPriorityRec = recommendations.find((r) => r.priority === "high");
      expect(highPriorityRec).toBeDefined();
    });

    it("should recommend splitting when budget exceeded", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      // Create files to exceed 100% budget
      for (let i = 1; i <= 6; i++) {
        await fs.writeFile(
          path.join(projectRoot, "src", `file${i}.ts`),
          `// File ${i}\n`
        );
      }

      await monitor.start();
      const recommendations = monitor.getRecommendations();

      const splitRec = recommendations.find((r) => r.type === "split");
      expect(splitRec).toBeDefined();
    });
  });

  describe("Status Management", () => {
    it("should return correct monitoring status", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      await monitor.start();
      const status = monitor.getStatus();

      expect(status.active).toBe(false); // No file watching
      expect(status.startedAt).toBeDefined();
      expect(status.totalChanges).toBeGreaterThanOrEqual(0);
      expect(status.currentUsage).toBeDefined();
    });

    it("should reset monitoring state", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      await fs.writeFile(path.join(projectRoot, "src", "file.ts"), "// File\n");
      await monitor.start();

      monitor.reset();
      const status = monitor.getStatus();

      expect(status.totalAlerts).toBe(0);
      expect(status.currentUsage.filesChanged).toBe(0);
    });
  });

  describe("Error Handling", () => {
    it("should handle invalid project root gracefully", async () => {
      monitor = new BudgetMonitor({
        projectRoot: "/nonexistent/path",
        spec: validSpec,
        useFileWatching: false,
      });

      // Should not throw
      await monitor.start();
      expect(monitor.getStatus()).toBeDefined();
    });

    it("should handle missing policy file", async () => {
      // Remove policy file
      await fs.rm(path.join(projectRoot, ".caws", "policy.yaml"), { force: true });

      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      // Should fall back to default budget
      await monitor.start();
      const status = monitor.getStatus();

      expect(status.currentUsage.maxFiles).toBeGreaterThan(0);
    });
  });

  describe("Performance", () => {
    it("should start monitoring quickly", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      const startTime = Date.now();
      await monitor.start();
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(1000); // <1s
    });

    it("should handle multiple file changes efficiently", async () => {
      monitor = new BudgetMonitor({
        projectRoot,
        spec: validSpec,
        useFileWatching: false,
      });

      // Create many files
      for (let i = 1; i <= 10; i++) {
        await fs.writeFile(
          path.join(projectRoot, "src", `file${i}.ts`),
          `// File ${i}\n`
        );
      }

      const startTime = Date.now();
      await monitor.start();
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(2000); // <2s for 10 files
    });
  });
});

