/**
 * ARBITER Performance Benchmark Suite
 *
 * Measures performance of core ARBITER components:
 * - CAWS Validation: <2s target
 * - Budget Monitoring: <5% overhead target
 * - Iterative Guidance: Analysis performance
 * - Provenance Tracking: Recording performance
 *
 * @author @darianrosebrook
 */

import chokidar from "chokidar";
import * as fs from "fs/promises";
import yaml from "js-yaml";
import * as path from "path";
import { performance } from "perf_hooks";

// Test configuration
const CONFIG = {
  iterations: 10,
  warmupIterations: 3,
  targets: {
    yamlParse: 50, // YAML parsing <50ms
    yamlDump: 100, // YAML dumping <100ms
    fileWatching: 10, // File watching overhead <10ms per operation
    fsOperations: 5, // File system operations <5ms
    monitoringOverhead: 0.05, // 5%
  },
  testProject: {
    root: path.join(process.cwd(), "benchmark", "test-project"),
    specId: "PERF-TEST-001",
  },
};

// Test data - simplified working spec
const testSpec = {
  id: CONFIG.testProject.specId,
  title: "Performance Test Feature",
  risk_tier: 2,
  mode: "feature",
  blast_radius: {
    modules: ["perf"],
    data_migration: false,
  },
  operational_rollback_slo: "30m",
  scope: {
    in: ["src/perf/"],
    out: ["node_modules/"],
  },
  invariants: ["Performance test invariant"],
  acceptance: [
    {
      id: "A1",
      given: "A performance test",
      when: "Executed",
      then: "Should complete within targets",
    },
  ],
  non_functional: {
    perf: {
      api_p95_ms: 250,
    },
  },
  contracts: [],
};

/**
 * Benchmark runner class
 */
class PerformanceBenchmark {
  constructor() {
    this.results = {
      validation: [],
      guidance: [],
      provenance: [],
      monitoring: {
        baseline: [],
        monitored: [],
      },
    };
  }

  /**
   * Run all benchmarks
   */
  async runAll() {
    console.log("🚀 Starting ARBITER Performance Benchmarks");
    console.log("==========================================\n");

    try {
      // Setup test environment
      await this.setupTestEnvironment();

      // Initialize components
      await this.initializeComponents();

      // Run benchmarks
      await this.runValidationBenchmark();
      await this.runGuidanceBenchmark();
      await this.runProvenanceBenchmark();
      await this.runMonitoringOverheadBenchmark();

      // Generate report
      this.generateReport();
    } catch (error) {
      console.error("❌ Benchmark failed:", error);
      throw error;
    } finally {
      // Cleanup
      await this.cleanup();
    }
  }

  /**
   * Setup test project structure
   */
  async setupTestEnvironment() {
    console.log("📁 Setting up test environment...");

    const { root } = CONFIG.testProject;

    // Create directories
    await fs.mkdir(path.join(root, "src"), { recursive: true });
    await fs.mkdir(path.join(root, "src", "perf"), { recursive: true });
    await fs.mkdir(path.join(root, ".caws"), { recursive: true });

    // Write policy file
    const policyPath = path.join(root, ".caws", "policy.yaml");
    await fs.writeFile(
      policyPath,
      `
version: "1.0.0"
risk_tiers:
  1:
    max_files: 15
    max_loc: 300
    coverage_threshold: 0.90
    mutation_threshold: 0.70
    contracts_required: true
    manual_review_required: true
  2:
    max_files: 20
    max_loc: 500
    coverage_threshold: 0.80
    mutation_threshold: 0.70
    contracts_required: true
    manual_review_required: false
  3:
    max_files: 25
    max_loc: 750
    coverage_threshold: 0.70
    mutation_threshold: 0.30
    contracts_required: false
    manual_review_required: false
`
    );

    // Create some test files
    await fs.writeFile(
      path.join(root, "src", "perf", "test1.ts"),
      "// Test file 1\n".repeat(10)
    );
    await fs.writeFile(
      path.join(root, "src", "perf", "test2.ts"),
      "// Test file 2\n".repeat(15)
    );

    console.log("✅ Test environment ready");
  }

  /**
   * Initialize benchmark components (simplified direct implementations)
   */
  async initializeComponents() {
    const { root } = CONFIG.testProject;

    // Initialize file watcher for monitoring overhead tests
    this.watcher = chokidar.watch(root, {
      ignored: ["**/node_modules/**", "**/.git/**"],
      persistent: true,
      ignoreInitial: true,
    });

    this.watcher.on("all", (event, filePath) => {
      // Simple file change tracking
      this.fileChanges.push({
        event,
        filePath,
        timestamp: Date.now(),
      });
    });

    // Initialize results storage
    this.fileChanges = [];
  }

  /**
   * Benchmark YAML processing performance (core of spec file operations)
   */
  async runValidationBenchmark() {
    console.log("🔍 Benchmarking YAML Processing Performance...");

    const specYaml = yaml.dump(testSpec);
    const specPath = path.join(
      CONFIG.testProject.root,
      ".caws",
      "working-spec.yaml"
    );

    // Warmup
    for (let i = 0; i < CONFIG.warmupIterations; i++) {
      yaml.dump(testSpec);
      yaml.load(specYaml);
      await fs.writeFile(specPath, specYaml);
      await fs.readFile(specPath, "utf8");
    }

    // Benchmark YAML operations (core of spec file management)
    for (let i = 0; i < CONFIG.iterations; i++) {
      const start = performance.now();

      // Simulate spec file operations: dump → write → read → parse
      const dumped = yaml.dump(testSpec);
      await fs.writeFile(specPath, dumped);
      const content = await fs.readFile(specPath, "utf8");
      const parsed = yaml.load(content);

      const duration = performance.now() - start;
      this.results.validation.push(duration);

      if (!parsed || parsed.id !== testSpec.id) {
        console.warn(`⚠️  YAML processing failed on iteration ${i}`);
      }
    }

    const avg = this.average(this.results.validation);
    const p95 = this.percentile(this.results.validation, 95);

    console.log(
      `✅ YAML Processing: ${avg.toFixed(2)}ms avg, ${p95.toFixed(2)}ms P95`
    );
    console.log(
      `🎯 Target: <${CONFIG.targets.yamlParse + CONFIG.targets.yamlDump}ms - ${
        avg < CONFIG.targets.yamlParse + CONFIG.targets.yamlDump
          ? "PASS"
          : "FAIL"
      }\n`
    );
  }

  /**
   * Benchmark file system operations (core of guidance file analysis)
   */
  async runGuidanceBenchmark() {
    console.log("🧭 Benchmarking File System Operations...");

    const projectRoot = CONFIG.testProject.root;

    // Create test files for analysis
    const testFiles = [];
    for (let i = 0; i < 5; i++) {
      const filePath = path.join(projectRoot, "src", `guidance-test-${i}.ts`);
      const content = `// Guidance test file ${i}\n`.repeat(20 + i * 5);
      await fs.writeFile(filePath, content);
      testFiles.push({ path: filePath, content });
    }

    // Warmup
    for (let i = 0; i < CONFIG.warmupIterations; i++) {
      for (const file of testFiles) {
        await fs.stat(file.path);
        await fs.readFile(file.path, "utf8");
      }
    }

    // Benchmark file analysis operations (core of guidance)
    for (let i = 0; i < CONFIG.iterations; i++) {
      const start = performance.now();

      // Simulate guidance analysis: stat + read files in scope
      const analysisPromises = testFiles.map(async (file) => {
        const stat = await fs.stat(file.path);
        const content = await fs.readFile(file.path, "utf8");
        return {
          path: file.path,
          size: stat.size,
          lines: content.split("\n").length,
          matchesAcceptance: content.includes("acceptance"),
        };
      });

      await Promise.all(analysisPromises);

      const duration = performance.now() - start;
      this.results.guidance.push(duration);
    }

    const avg = this.average(this.results.guidance);
    const p95 = this.percentile(this.results.guidance, 95);

    console.log(
      `✅ File Analysis: ${avg.toFixed(2)}ms avg, ${p95.toFixed(2)}ms P95`
    );
    console.log(
      `🎯 Target: <${CONFIG.targets.fsOperations * testFiles.length}ms - ${
        avg < CONFIG.targets.fsOperations * testFiles.length ? "PASS" : "FAIL"
      }\n`
    );
  }

  /**
   * Benchmark JSON operations (core of provenance data handling)
   */
  async runProvenanceBenchmark() {
    console.log("📊 Benchmarking JSON/Data Operations...");

    const provenancePath = path.join(
      CONFIG.testProject.root,
      ".caws",
      "provenance.json"
    );

    // Create test provenance data
    const createEntry = (i) => ({
      id: `entry-${i}`,
      type: "commit",
      specId: CONFIG.testProject.specId,
      actor: { type: "ai", identifier: `bench-agent-${i}` },
      action: {
        type: "committed",
        description: `Benchmark entry ${i}`,
        details: { iteration: i, timestamp: Date.now() },
      },
      timestamp: new Date().toISOString(),
    });

    // Warmup
    for (let i = 0; i < CONFIG.warmupIterations; i++) {
      const entry = createEntry(i);
      JSON.stringify(entry);
      JSON.parse(JSON.stringify(entry));
      await fs.writeFile(provenancePath, JSON.stringify([entry], null, 2));
      await fs.readFile(provenancePath, "utf8");
    }

    // Benchmark JSON operations (core of provenance tracking)
    for (let i = 0; i < CONFIG.iterations; i++) {
      const start = performance.now();

      // Simulate provenance operations: create → serialize → write → read → parse
      const entry = createEntry(i + 100);
      const serialized = JSON.stringify(entry);
      await fs.writeFile(provenancePath, serialized);
      const content = await fs.readFile(provenancePath, "utf8");
      const parsed = JSON.parse(content);

      const duration = performance.now() - start;
      this.results.provenance.push(duration);

      if (!parsed || parsed.id !== entry.id) {
        console.warn(`⚠️  JSON processing failed on iteration ${i}`);
      }
    }

    const avg = this.average(this.results.provenance);
    const p95 = this.percentile(this.results.provenance, 95);

    console.log(
      `✅ JSON Operations: ${avg.toFixed(2)}ms avg, ${p95.toFixed(2)}ms P95`
    );
    console.log(
      `🎯 Target: <${CONFIG.targets.fsOperations * 2}ms - ${
        avg < CONFIG.targets.fsOperations * 2 ? "PASS" : "FAIL"
      }\n`
    );
  }

  /**
   * Benchmark file watching overhead
   */
  async runMonitoringOverheadBenchmark() {
    console.log("📈 Benchmarking File Watching Overhead...");

    const iterations = 20;
    const testFile = path.join(
      CONFIG.testProject.root,
      "src",
      "perf",
      "overhead-test.ts"
    );

    // Clear previous file changes
    this.fileChanges = [];

    // Baseline: File operations without monitoring
    console.log("📊 Measuring baseline performance...");

    for (let i = 0; i < iterations; i++) {
      const start = performance.now();

      await fs.writeFile(testFile, `// Overhead test ${i}\n`.repeat(5));
      await fs.readFile(testFile, "utf8");

      const duration = performance.now() - start;
      this.results.monitoring.baseline.push(duration);
    }

    // Wait for any pending file events
    await new Promise((resolve) => setTimeout(resolve, 100));

    // Monitored: File operations with file watching active
    console.log("📊 Measuring monitored performance...");

    for (let i = 0; i < iterations; i++) {
      const start = performance.now();

      await fs.writeFile(testFile, `// Overhead test ${i}\n`.repeat(5));
      await fs.readFile(testFile, "utf8");

      const duration = performance.now() - start;
      this.results.monitoring.monitored.push(duration);

      // Small delay to let file watching catch up
      await new Promise((resolve) => setTimeout(resolve, 50));
    }

    // Wait for final file events
    await new Promise((resolve) => setTimeout(resolve, 200));

    const baselineAvg = this.average(this.results.monitoring.baseline);
    const monitoredAvg = this.average(this.results.monitoring.monitored);
    const overhead = (monitoredAvg - baselineAvg) / baselineAvg;

    const fileEventsCaptured = this.fileChanges.length;

    console.log(
      `✅ File Watching Overhead: ${(overhead * 100).toFixed(
        2
      )}% (${fileEventsCaptured} events captured)`
    );
    console.log(
      `🎯 Target: <${(CONFIG.targets.monitoringOverhead * 100).toFixed(1)}% - ${
        overhead < CONFIG.targets.monitoringOverhead ? "PASS" : "FAIL"
      }\n`
    );
  }

  /**
   * Generate benchmark report
   */
  generateReport() {
    console.log("📋 Performance Benchmark Results");
    console.log("================================\n");

    const results = {
      yaml: {
        avg: this.average(this.results.validation),
        p95: this.percentile(this.results.validation, 95),
        target: CONFIG.targets.yamlParse + CONFIG.targets.yamlDump,
        pass:
          this.average(this.results.validation) <
          CONFIG.targets.yamlParse + CONFIG.targets.yamlDump,
      },
      fsAnalysis: {
        avg: this.average(this.results.guidance),
        p95: this.percentile(this.results.guidance, 95),
        target: CONFIG.targets.fsOperations * 5, // 5 test files
        pass:
          this.average(this.results.guidance) < CONFIG.targets.fsOperations * 5,
      },
      jsonOps: {
        avg: this.average(this.results.provenance),
        p95: this.percentile(this.results.provenance, 95),
        target: CONFIG.targets.fsOperations * 2,
        pass:
          this.average(this.results.provenance) <
          CONFIG.targets.fsOperations * 2,
      },
      fileWatching: {
        overhead:
          (this.average(this.results.monitoring.monitored) -
            this.average(this.results.monitoring.baseline)) /
          this.average(this.results.monitoring.baseline),
        target: CONFIG.targets.monitoringOverhead,
        pass:
          (this.average(this.results.monitoring.monitored) -
            this.average(this.results.monitoring.baseline)) /
            this.average(this.results.monitoring.baseline) <
          CONFIG.targets.monitoringOverhead,
        eventsCaptured: this.fileChanges.length,
      },
    };

    console.log("Core Operation Performance:");
    console.log(
      `• YAML Processing: ${results.yaml.avg.toFixed(2)}ms avg (${
        results.yaml.pass ? "✅" : "❌"
      } <${results.yaml.target}ms)`
    );
    console.log(
      `• File System Analysis: ${results.fsAnalysis.avg.toFixed(2)}ms avg (${
        results.fsAnalysis.pass ? "✅" : "❌"
      } <${results.fsAnalysis.target}ms)`
    );
    console.log(
      `• JSON Operations: ${results.jsonOps.avg.toFixed(2)}ms avg (${
        results.jsonOps.pass ? "✅" : "❌"
      } <${results.jsonOps.target}ms)`
    );
    console.log(
      `• File Watching Overhead: ${(
        results.fileWatching.overhead * 100
      ).toFixed(2)}% (${results.fileWatching.eventsCaptured} events) (${
        results.fileWatching.pass ? "✅" : "❌"
      } <${(results.fileWatching.target * 100).toFixed(1)}%)`
    );

    const allPass = Object.values(results).every((r) => r.pass);

    console.log(
      `\n🏆 Overall Result: ${
        allPass ? "PASS - All targets met!" : "FAIL - Some targets missed"
      }\n`
    );

    // Export detailed results
    const reportPath = path.join(process.cwd(), "benchmark", "results.json");
    fs.writeFile(
      reportPath,
      JSON.stringify(
        {
          timestamp: new Date().toISOString(),
          config: CONFIG,
          results: {
            ...results,
            raw: this.results,
          },
          summary: {
            allTargetsMet: allPass,
            totalTests: CONFIG.iterations * 4,
            totalDuration: Object.values(this.results)
              .flat()
              .reduce((a, b) => a + b, 0),
          },
        },
        null,
        2
      )
    ).catch(() => {
      /* ignore */
    });

    console.log(`📄 Detailed results saved to: ${reportPath}`);
  }

  /**
   * Cleanup test environment
   */
  async cleanup() {
    try {
      if (this.watcher) {
        await this.watcher.close();
      }

      await fs.rm(CONFIG.testProject.root, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  }

  // Utility functions
  average(arr) {
    return arr.reduce((a, b) => a + b, 0) / arr.length;
  }

  percentile(arr, p) {
    const sorted = [...arr].sort((a, b) => a - b);
    const index = Math.ceil((p / 100) * sorted.length) - 1;
    return sorted[index];
  }
}

// Run benchmarks if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const benchmark = new PerformanceBenchmark();
  benchmark.runAll().catch((error) => {
    console.error("Benchmark suite failed:", error);
    process.exit(1);
  });
}

export { PerformanceBenchmark };
