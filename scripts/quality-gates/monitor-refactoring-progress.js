#!/usr/bin/env node

/**
 * Refactoring Progress Monitor
 *
 * Tracks crisis response progress against Week 1 targets:
 * - God object decomposition (11 → 0 files >3,000 LOC)
 * - Duplicate reduction (658+ → <329 struct names, 69 → <7 filenames)
 * - Quality gate compliance
 * - Overall crisis metrics
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const V3_PATH = path.join(__dirname, "../../iterations/v3");

// Crisis targets (Week 1 Emergency Stabilization)
const CRISIS_TARGETS = {
  god_objects_severe: {
    current: 11,
    target: 0,
    description: "Files >3,000 LOC",
  },
  god_objects_critical: {
    current: null,
    target: 0,
    description: "Files >2,000 LOC",
  },
  duplicate_structs: {
    current: 658,
    target: 329,
    description: "Duplicate struct/trait names",
  },
  duplicate_filenames: {
    current: 69,
    target: 7,
    description: "Duplicate filenames",
  },
  quality_gates: {
    current: "passing",
    target: "passing",
    description: "Quality gates status",
  },
};

class RefactoringMonitor {
  constructor() {
    this.metrics = {};
    this.lastUpdate = null;
  }

  async collectMetrics() {
    console.log("📊 Collecting refactoring crisis metrics...");

    // God object metrics
    const godObjects = await this.getGodObjectMetrics();
    this.metrics.god_objects = godObjects;

    // Duplication metrics
    const duplication = await this.getDuplicationMetrics();
    this.metrics.duplication = duplication;

    // Quality gate status
    const qualityStatus = await this.getQualityGateStatus();
    this.metrics.quality_gates = qualityStatus;

    // Overall progress
    this.metrics.overall_progress = this.calculateOverallProgress();
    this.lastUpdate = new Date().toISOString();

    return this.metrics;
  }

  async getGodObjectMetrics() {
    try {
      // Import the god object checker
      const { getFileSizes, GOD_OBJECT_THRESHOLDS } = await import(
        "./check-god-objects.js"
      );

      const fileSizes = getFileSizes();
      const severe = Object.values(fileSizes).filter(
        (size) => size > GOD_OBJECT_THRESHOLDS.severe
      ).length;
      const critical = Object.values(fileSizes).filter(
        (size) => size > GOD_OBJECT_THRESHOLDS.critical
      ).length;
      const warning = Object.values(fileSizes).filter(
        (size) => size > GOD_OBJECT_THRESHOLDS.warning
      ).length;

      return {
        severe: {
          count: severe,
          target: 0,
          status: severe === 0 ? "✅" : "🚨",
        },
        critical: {
          count: critical,
          target: 0,
          status: critical === 0 ? "✅" : "⚠️",
        },
        warning: {
          count: warning,
          target: "<5",
          status: warning < 5 ? "✅" : "⚠️",
        },
        largest_files: this.getLargestFiles(fileSizes, 5),
      };
    } catch (error) {
      return { error: error.message };
    }
  }

  async getDuplicationMetrics() {
    try {
      // Import duplication checker
      const { getDuplicateFilenames, getDuplicateStructs } = await import(
        "./check-duplication.js"
      );

      const filenames = getDuplicateFilenames();
      const structs = getDuplicateStructs();

      const filenameCount = Object.keys(filenames).length;
      const structCount = Object.keys(structs).length;

      return {
        filenames: {
          count: filenameCount,
          target: CRISIS_TARGETS.duplicate_filenames.target,
          status:
            filenameCount <= CRISIS_TARGETS.duplicate_filenames.target
              ? "✅"
              : "🚨",
          breakdown: filenames,
        },
        structs: {
          count: structCount,
          target: CRISIS_TARGETS.duplicate_structs.target,
          status:
            structCount <= CRISIS_TARGETS.duplicate_structs.target
              ? "✅"
              : "🚨",
          breakdown: structs,
        },
      };
    } catch (error) {
      return { error: error.message };
    }
  }

  async getQualityGateStatus() {
    try {
      // Run quality gates in a subprocess to check status
      const { spawn } = await import("child_process");
      const { promisify } = await import("util");

      return new Promise((resolve) => {
        const child = spawn(
          "node",
          ["scripts/quality-gates/run-quality-gates.js", "--ci"],
          {
            stdio: "pipe",
            cwd: path.join(__dirname, "../.."),
          }
        );

        let stdout = "";
        let stderr = "";

        child.stdout.on("data", (data) => {
          stdout += data.toString();
        });
        child.stderr.on("data", (data) => {
          stderr += data.toString();
        });

        child.on("close", (code) => {
          resolve({
            status: code === 0 ? "passing" : "failing",
            exit_code: code,
            output: stdout,
            errors: stderr,
          });
        });
      });
    } catch (error) {
      return { error: error.message };
    }
  }

  getLargestFiles(fileSizes, limit = 5) {
    return Object.entries(fileSizes)
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([file, size]) => ({
        file: path.relative(V3_PATH, file),
        size,
      }));
  }

  calculateOverallProgress() {
    const metrics = this.metrics;

    if (!metrics.god_objects || !metrics.duplication) {
      return { error: "Insufficient metrics for progress calculation" };
    }

    const godObjectProgress = metrics.god_objects.severe?.count === 0 ? 100 : 0;
    const duplicationProgress = Math.max(
      0,
      Math.min(
        100,
        50 *
          (CRISIS_TARGETS.duplicate_filenames.target /
            Math.max(metrics.duplication.filenames?.count || 1, 1)) +
          50 *
            (CRISIS_TARGETS.duplicate_structs.target /
              Math.max(metrics.duplication.structs?.count || 1, 1))
      )
    );

    const qualityProgress =
      metrics.quality_gates?.status === "passing" ? 100 : 0;

    const overall = Math.round(
      (godObjectProgress + duplicationProgress + qualityProgress) / 3
    );

    return {
      overall_percentage: overall,
      breakdown: {
        god_objects: godObjectProgress,
        duplication: Math.round(duplicationProgress),
        quality_gates: qualityProgress,
      },
      status:
        overall >= 80
          ? "🟢 GOOD"
          : overall >= 50
          ? "🟡 NEEDS WORK"
          : "🔴 CRITICAL",
    };
  }

  generateReport() {
    const report = {
      timestamp: this.lastUpdate,
      crisis_targets: CRISIS_TARGETS,
      current_metrics: this.metrics,
      summary: {
        status: this.metrics.overall_progress?.status || "UNKNOWN",
        progress_percentage:
          this.metrics.overall_progress?.overall_percentage || 0,
        critical_issues: this.getCriticalIssues(),
        recommendations: this.getRecommendations(),
      },
    };

    return report;
  }

  getCriticalIssues() {
    const issues = [];

    if (this.metrics.god_objects?.severe?.count > 0) {
      issues.push(
        `${this.metrics.god_objects.severe.count} severe god objects (>3,000 LOC)`
      );
    }

    if (
      this.metrics.duplication?.structs?.count >
      CRISIS_TARGETS.duplicate_structs.target
    ) {
      issues.push(
        `${this.metrics.duplication.structs.count} duplicate struct names (target: ${CRISIS_TARGETS.duplicate_structs.target})`
      );
    }

    if (
      this.metrics.duplication?.filenames?.count >
      CRISIS_TARGETS.duplicate_filenames.target
    ) {
      issues.push(
        `${this.metrics.duplication.filenames.count} duplicate filenames (target: ${CRISIS_TARGETS.duplicate_filenames.target})`
      );
    }

    if (this.metrics.quality_gates?.status !== "passing") {
      issues.push("Quality gates failing");
    }

    return issues;
  }

  getRecommendations() {
    const recs = [];

    if (this.metrics.god_objects?.severe?.count > 0) {
      recs.push("Decompose severe god objects (>3,000 LOC) immediately");
    }

    if (
      this.metrics.duplication?.structs?.count >
      CRISIS_TARGETS.duplicate_structs.target
    ) {
      recs.push("Extract common traits for duplicate struct names");
    }

    if (
      this.metrics.duplication?.filenames?.count >
      CRISIS_TARGETS.duplicate_filenames.target
    ) {
      recs.push("Consolidate duplicate filename implementations");
    }

    if (this.metrics.quality_gates?.status !== "passing") {
      recs.push("Fix quality gate violations before proceeding");
    }

    if (recs.length === 0) {
      recs.push("Proceed to Week 2: Foundation Establishment");
    }

    return recs;
  }

  displayReport() {
    const report = this.generateReport();

    console.log("🚨 AGENT AGENCY V3 - CRISIS RESPONSE MONITOR");
    console.log("=".repeat(50));
    console.log(`📅 Last Update: ${report.timestamp}`);
    console.log(
      `📊 Overall Progress: ${report.summary.status} (${report.summary.progress_percentage}%)`
    );
    console.log("");

    console.log("🎯 CRISIS TARGETS (Week 1):");
    console.log(
      `   God Objects >3K LOC: ${CRISIS_TARGETS.god_objects_severe.current} → ${CRISIS_TARGETS.god_objects_severe.target}`
    );
    console.log(
      `   Duplicate Structs: ${CRISIS_TARGETS.duplicate_structs.current}+ → ${CRISIS_TARGETS.duplicate_structs.target}`
    );
    console.log(
      `   Duplicate Filenames: ${CRISIS_TARGETS.duplicate_filenames.current} → ${CRISIS_TARGETS.duplicate_filenames.target}`
    );
    console.log("");

    if (this.metrics.god_objects && !this.metrics.god_objects.error) {
      console.log("🏗️  GOD OBJECT STATUS:");
      console.log(
        `   Severe (>3K LOC): ${this.metrics.god_objects.severe.status} ${this.metrics.god_objects.severe.count} files`
      );
      console.log(
        `   Critical (>2K LOC): ${this.metrics.god_objects.critical.status} ${this.metrics.god_objects.critical.count} files`
      );
      console.log(
        `   Warning (>1.5K LOC): ${this.metrics.god_objects.warning.status} ${this.metrics.god_objects.warning.count} files`
      );
      console.log("   Largest files:");
      this.metrics.god_objects.largest_files?.forEach(({ file, size }) => {
        console.log(`     - ${file}: ${size} LOC`);
      });
      console.log("");
    }

    if (this.metrics.duplication && !this.metrics.duplication.error) {
      console.log("📋 DUPLICATION STATUS:");
      console.log(
        `   Filenames: ${this.metrics.duplication.filenames.status} ${this.metrics.duplication.filenames.count} duplicates`
      );
      console.log(
        `   Structs: ${this.metrics.duplication.structs.status} ${this.metrics.duplication.structs.count} duplicates`
      );
      console.log("");
    }

    if (this.metrics.quality_gates && !this.metrics.quality_gates.error) {
      console.log("🚦 QUALITY GATES STATUS:");
      console.log(
        `   Status: ${
          this.metrics.quality_gates.status === "passing"
            ? "✅ PASSING"
            : "❌ FAILING"
        }`
      );
      console.log("");
    }

    if (report.summary.critical_issues.length > 0) {
      console.log("🚨 CRITICAL ISSUES:");
      report.summary.critical_issues.forEach((issue) => {
        console.log(`   ❌ ${issue}`);
      });
      console.log("");
    }

    console.log("💡 RECOMMENDATIONS:");
    report.summary.recommendations.forEach((rec) => {
      console.log(`   • ${rec}`);
    });
    console.log("");

    console.log("📖 See docs/refactoring.md for detailed crisis response plan");
  }
}

async function main() {
  const monitor = new RefactoringMonitor();

  try {
    await monitor.collectMetrics();
    monitor.displayReport();

    // Save report to file for CI/CD
    const report = monitor.generateReport();
    const reportPath = path.join(
      __dirname,
      "../../docs-status/refactoring-progress-report.json"
    );
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    console.log(
      `💾 Report saved to: docs-status/refactoring-progress-report.json`
    );
  } catch (error) {
    console.error("❌ Refactoring monitor failed:", error);
    process.exit(1);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export default RefactoringMonitor;
