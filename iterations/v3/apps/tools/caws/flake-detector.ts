#!/usr/bin/env tsx

/**
 * CAWS Flake Detection System
 *
 * Monitors test variance and quarantines intermittently failing tests.
 * This tool analyzes test run variance and identifies flaky tests for quarantine.
 *
 * @author @darianrosebrook
 */

import { readFileSync, writeFileSync, existsSync } from "fs";
import { dirname } from "path";
import { fileURLToPath } from "url";
import fs from "fs-extra";
import path from "path";
import { glob } from "glob";

interface TestResult {
  title: string;
  fullName: string;
  status: "passed" | "failed" | "pending" | "skipped";
  duration: number;
  failureMessages: string[];
}

interface TestSuiteResult {
  name: string;
  status: "passed" | "failed";
  testResults: TestResult[];
  startTime: number;
  endTime: number;
}

interface FlakeDetectionResult {
  flakyTests: string[];
  varianceScore: number;
  totalRuns: number;
  recommendations: string[];
}

interface HistoricalTestData {
  runs: TestRun[];
  quarantined: Set<string>;
  lastUpdated: string;
}

interface TestRun {
  timestamp: number;
  results: Map<string, TestResult>;
  variance: number;
}

/**
 * Flake Detection Service
 * Analyzes test run variance and identifies flaky tests
 */
class FlakeDetectionService {
  private readonly HISTORY_FILE = ".caws/flake-history.json";
  private readonly QUARANTINE_FILE = ".caws/quarantined-tests.json";
  private readonly VARIANCE_THRESHOLD = 0.05; // 5% variance threshold
  private readonly MIN_RUNS_FOR_ANALYSIS = 3;
  private readonly QUARANTINE_THRESHOLD = 0.15; // 15% flake rate triggers quarantine

  /**
   * Analyze test variance and detect flaky tests
   */
  async detectFlakes(
    currentResults: TestSuiteResult[]
  ): Promise<FlakeDetectionResult> {
    const history = this.loadHistory();
    const currentRun = this.createCurrentRun(currentResults);

    history.runs.push(currentRun);
    this.saveHistory(history);

    if (history.runs.length < this.MIN_RUNS_FOR_ANALYSIS) {
      return {
        flakyTests: [],
        varianceScore: 0,
        totalRuns: history.runs.length,
        recommendations: [
          `Need ${
            this.MIN_RUNS_FOR_ANALYSIS - history.runs.length
          } more test runs for analysis`,
        ],
      };
    }

    const flakyTests = this.identifyFlakyTests(history);
    const varianceScore = this.calculateVarianceScore(history);

    const recommendations = this.generateRecommendations(
      flakyTests,
      varianceScore
    );

    return {
      flakyTests,
      varianceScore,
      totalRuns: history.runs.length,
      recommendations,
    };
  }

  /**
   * Quarantine flaky tests
   */
  quarantineTests(testNames: string[]): void {
    const history = this.loadHistory();
    testNames.forEach((testName) => history.quarantined.add(testName));
    history.lastUpdated = new Date().toISOString();
    this.saveHistory(history);

    // Save quarantined tests list
    const quarantinedData = {
      quarantined: Array.from(history.quarantined),
      quarantinedAt: history.lastUpdated,
      reason: "Automated flake detection",
    };

    writeFileSync(
      this.QUARANTINE_FILE,
      JSON.stringify(quarantinedData, null, 2)
    );
    console.log(`🚫 Quarantined ${testNames.length} flaky tests`);
  }

  /**
   * Get currently quarantined tests
   */
  getQuarantinedTests(): string[] {
    const history = this.loadHistory();
    return Array.from(history.quarantined);
  }

  /**
   * Release tests from quarantine (manual override)
   */
  releaseFromQuarantine(testNames: string[]): void {
    const history = this.loadHistory();
    testNames.forEach((testName) => history.quarantined.delete(testName));
    history.lastUpdated = new Date().toISOString();
    this.saveHistory(history);
    console.log(`✅ Released ${testNames.length} tests from quarantine`);
  }

  private loadHistory(): HistoricalTestData {
    if (!existsSync(this.HISTORY_FILE)) {
      return {
        runs: [],
        quarantined: new Set(),
        lastUpdated: new Date().toISOString(),
      };
    }

    try {
      const data = JSON.parse(readFileSync(this.HISTORY_FILE, "utf-8"));
      return {
        runs: data.runs || [],
        quarantined: new Set(data.quarantined || []),
        lastUpdated: data.lastUpdated || new Date().toISOString(),
      };
    } catch {
      return {
        runs: [],
        quarantined: new Set(),
        lastUpdated: new Date().toISOString(),
      };
    }
  }

  private saveHistory(history: HistoricalTestData): void {
    const data = {
      runs: history.runs,
      quarantined: Array.from(history.quarantined),
      lastUpdated: history.lastUpdated,
    };
    writeFileSync(this.HISTORY_FILE, JSON.stringify(data, null, 2));
  }

  private createCurrentRun(results: TestSuiteResult[]): TestRun {
    const testMap = new Map<string, TestResult>();

    results.forEach((suite) => {
      suite.testResults.forEach((test) => {
        const key = this.getTestKey(test);
        testMap.set(key, test);
      });
    });

    const variance = this.calculateRunVariance(testMap, results);
    return {
      timestamp: Date.now(),
      results: testMap,
      variance,
    };
  }

  private getTestKey(test: TestResult): string {
    return test.fullName;
  }

  private identifyFlakyTests(history: HistoricalTestData): string[] {
    const flakyTests = new Set<string>();
    const recentRuns = history.runs.slice(-5); // Analyze last 5 runs

    // Find tests that have inconsistent results
    for (const run of recentRuns) {
      for (const [testName, result] of run.results) {
        if (result.status !== "passed") {
          // Check if this test has passed in other recent runs
          const passedInOtherRuns = recentRuns
            .filter((r) => r !== run)
            .some((r) => r.results.get(testName)?.status === "passed");

          if (passedInOtherRuns) {
            flakyTests.add(testName);
          }
        }
      }
    }

    // Check against quarantine threshold
    for (const testName of flakyTests) {
      const flakeRate = this.calculateFlakeRate(testName, recentRuns);
      if (flakeRate < this.QUARANTINE_THRESHOLD) {
        flakyTests.delete(testName);
      }
    }

    return Array.from(flakyTests);
  }

  private calculateFlakeRate(testName: string, runs: TestRun[]): number {
    const results = runs
      .map((run) => run.results.get(testName)?.status)
      .filter(Boolean);
    const failures = results.filter((status) => status !== "passed").length;
    return failures / results.length;
  }

  private calculateVarianceScore(history: HistoricalTestData): number {
    if (history.runs.length < 2) return 0;

    const recentRuns = history.runs.slice(-5);
    const varianceSum = recentRuns.reduce((sum, run) => sum + run.variance, 0);
    return varianceSum / recentRuns.length;
  }

  private calculateRunVariance(
    testMap: Map<string, TestResult>,
    _suites: TestSuiteResult[]
  ): number {
    const totalTests = testMap.size;
    const failedTests = Array.from(testMap.values()).filter(
      (t) => t.status !== "passed"
    ).length;
    return totalTests > 0 ? failedTests / totalTests : 0;
  }

  private generateRecommendations(
    flakyTests: string[],
    varianceScore: number
  ): string[] {
    const recommendations: string[] = [];

    if (flakyTests.length > 0) {
      recommendations.push(
        `Quarantine ${flakyTests.length} flaky tests for investigation`
      );
    }

    if (varianceScore > this.VARIANCE_THRESHOLD) {
      recommendations.push(
        "High test variance detected - consider test environment stability"
      );
    }

    if (varianceScore === 0) {
      recommendations.push("Excellent test stability - no flakes detected");
    }

    return recommendations;
  }
}

/**
 * CLI Interface
 */
async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.log("🔍 CAWS Flake Detection Tool");
    console.log("Usage: flake-detector.ts <command> [options]");
    console.log("");
    console.log("Commands:");
    console.log("  detect     - Analyze test variance and detect flaky tests");
    console.log("  quarantine - Quarantine specified flaky tests");
    console.log("  release    - Release tests from quarantine");
    console.log("  status     - Show current flake detection status");
    console.log("");
    console.log("Examples:");
    console.log("  flake-detector.ts detect");
    console.log('  flake-detector.ts quarantine "test name"');
    console.log('  flake-detector.ts release "test name"');
    return;
  }

  const command = args[0];
  const detector = new FlakeDetectionService();

  try {
    switch (command) {
      case "detect": {
        console.log("🔍 Analyzing test variance...");

        // 1. Test result file format support: Support multiple test result file formats
        const testResults = await parseTestResults();
        const result = await detector.detectFlakes(testResults);

        console.log(`📊 Flake Detection Results:`);
        console.log(
          `   Variance Score: ${(result.varianceScore * 100).toFixed(2)}%`
        );
        console.log(`   Total Runs Analyzed: ${result.totalRuns}`);
        console.log(`   Flaky Tests Found: ${result.flakyTests.length}`);

        if (result.flakyTests.length > 0) {
          console.log("\n🚨 Flaky Tests:");
          result.flakyTests.forEach((test) => console.log(`   - ${test}`));
        }

        result.recommendations.forEach((rec) => console.log(`💡 ${rec}`));
        break;
      }

      case "quarantine": {
        const testNames = args.slice(1);
        if (testNames.length === 0) {
          console.log("❌ Please specify test names to quarantine");
          return;
        }
        detector.quarantineTests(testNames);
        break;
      }

      case "release": {
        const testNames = args.slice(1);
        if (testNames.length === 0) {
          console.log("❌ Please specify test names to release");
          return;
        }
        detector.releaseFromQuarantine(testNames);
        break;
      }

      case "status": {
        const quarantined = detector.getQuarantinedTests();
        console.log("🚫 Currently Quarantined Tests:");
        if (quarantined.length === 0) {
          console.log("   None - all tests are active");
        } else {
          quarantined.forEach((test) => console.log(`   - ${test}`));
        }
        break;
      }

      default:
        console.log(`❌ Unknown command: ${command}`);
        process.exit(1);
    }
  } catch (error) {
    console.error("❌ Error:", error);
    process.exit(1);
  }
}

// Run CLI if this file is executed directly
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * Parse test results from various file formats
 * Supports JUnit XML, JSON (Jest/Mocha), and TAP formats
 */
async function parseTestResults(): Promise<TestSuiteResult[]> {
  const results: TestSuiteResult[] = [];

  try {
    // 2. File system integration: Integrate with file system for test result discovery
    const testResultFiles = await findTestResultFiles();

    for (const filePath of testResultFiles) {
      try {
        const content = await fs.readFile(filePath, "utf-8");
        const parsedResults = await parseTestResultFile(filePath, content);
        results.push(...parsedResults);
      } catch (error) {
        console.warn(`⚠️ Failed to parse test result file ${filePath}:`, error);
      }
    }

    console.log(`📁 Found ${testResultFiles.length} test result files`);
    console.log(`📊 Parsed ${results.length} test suite results`);
  } catch (error) {
    console.warn("⚠️ Error discovering test result files:", error);
    // Fallback to mock data if no files found
    return generateMockTestResults();
  }

  return results;
}

/**
 * Find test result files in the workspace
 */
async function findTestResultFiles(): Promise<string[]> {
  const testResultFiles: string[] = [];
  const searchPaths = [
    "test-results",
    "coverage",
    "reports",
    "target/coverage",
    "target/test-results",
    ".nyc_output",
    "junit-reports",
  ];

  // Common test result file patterns
  const patterns = [
    "**/*.xml",
    "**/junit*.xml",
    "**/test-results*.xml",
    "**/coverage*.json",
    "**/test-results*.json",
    "**/*.tap",
    "**/test-output*.tap",
  ];

  for (const searchPath of searchPaths) {
    if (await fs.pathExists(searchPath)) {
      for (const pattern of patterns) {
        try {
          const files = await glob(pattern, { cwd: searchPath });
          testResultFiles.push(...files.map((f) => path.join(searchPath, f)));
        } catch (error) {
          // Ignore glob errors for non-existent directories
        }
      }
    }
  }

  return testResultFiles;
}

/**
 * Parse a single test result file based on its format
 */
async function parseTestResultFile(
  filePath: string,
  content: string
): Promise<TestSuiteResult[]> {
  const ext = path.extname(filePath).toLowerCase();

  switch (ext) {
    case ".xml":
      return parseJUnitXML(content, filePath);
    case ".json":
      return parseJSONTestResults(content, filePath);
    case ".tap":
      return parseTAPResults(content, filePath);
    default:
      // Try to auto-detect format based on content
      if (content.trim().startsWith("<?xml")) {
        return parseJUnitXML(content, filePath);
      } else if (
        content.trim().startsWith("{") ||
        content.trim().startsWith("[")
      ) {
        return parseJSONTestResults(content, filePath);
      } else if (
        content.includes("TAP version") ||
        content.includes("ok ") ||
        content.includes("not ok ")
      ) {
        return parseTAPResults(content, filePath);
      }
      throw new Error(`Unsupported test result format for file: ${filePath}`);
  }
}

/**
 * Parse JUnit XML test results using regex-based parsing
 */
function parseJUnitXML(content: string, filePath: string): TestSuiteResult[] {
  const results: TestSuiteResult[] = [];

  try {
    // Simple regex-based XML parsing for JUnit format
    const testSuiteRegex = /<testsuite[^>]*name="([^"]*)"[^>]*>/g;
    const testCaseRegex =
      /<testcase[^>]*name="([^"]*)"[^>]*(?:time="([^"]*)")?[^>]*>/g;
    const failureRegex = /<failure[^>]*>([^<]*)<\/failure>/g;
    const errorRegex = /<error[^>]*>([^<]*)<\/error>/g;
    const skippedRegex = /<skipped[^>]*\/>/g;

    let suiteMatch;
    while ((suiteMatch = testSuiteRegex.exec(content)) !== null) {
      const suiteName = suiteMatch[1] || "Unknown Suite";
      const testResults: TestResult[] = [];

      // Find test cases within this suite
      const suiteStart = suiteMatch.index;
      const nextSuiteMatch = testSuiteRegex.exec(content);
      const suiteEnd = nextSuiteMatch ? nextSuiteMatch.index : content.length;
      testSuiteRegex.lastIndex = suiteStart; // Reset for next iteration

      const suiteContent = content.substring(suiteStart, suiteEnd);

      let testMatch;
      while ((testMatch = testCaseRegex.exec(suiteContent)) !== null) {
        const testName = testMatch[1] || "Unknown Test";
        const time = parseFloat(testMatch[2] || "0");

        // Check for failure, error, or skipped status
        const testStart = testMatch.index;
        const nextTestMatch = testCaseRegex.exec(suiteContent);
        const testEnd = nextTestMatch
          ? nextTestMatch.index
          : suiteContent.length;
        testCaseRegex.lastIndex = testStart; // Reset for next iteration

        const testContent = suiteContent.substring(testStart, testEnd);

        let status: "passed" | "failed" | "skipped" = "passed";
        let errorMessage = "";

        if (failureRegex.test(testContent)) {
          status = "failed";
          failureRegex.lastIndex = 0; // Reset regex
          const failureMatch = failureRegex.exec(testContent);
          if (failureMatch) {
            errorMessage = failureMatch[1] || "";
          }
        } else if (errorRegex.test(testContent)) {
          status = "failed";
          errorRegex.lastIndex = 0; // Reset regex
          const errorMatch = errorRegex.exec(testContent);
          if (errorMatch) {
            errorMessage = errorMatch[1] || "";
          }
        } else if (skippedRegex.test(testContent)) {
          status = "skipped";
        }

        testResults.push({
          name: testName,
          status,
          duration: time * 1000, // Convert to milliseconds
          errorMessage: errorMessage.trim(),
        });
      }

      if (testResults.length > 0) {
        results.push({
          suiteName,
          timestamp: new Date().toISOString(),
          testResults,
          totalTests: testResults.length,
          passedTests: testResults.filter((t) => t.status === "passed").length,
          failedTests: testResults.filter((t) => t.status === "failed").length,
          skippedTests: testResults.filter((t) => t.status === "skipped")
            .length,
          totalDuration: testResults.reduce((sum, t) => sum + t.duration, 0),
          source: filePath,
        });
      }
    }
  } catch (error) {
    console.warn(`⚠️ Failed to parse JUnit XML from ${filePath}:`, error);
  }

  return results;
}

/**
 * Parse JSON test results (Jest, Mocha, etc.)
 */
function parseJSONTestResults(
  content: string,
  filePath: string
): TestSuiteResult[] {
  const results: TestSuiteResult[] = [];

  try {
    const data = JSON.parse(content);

    // Handle different JSON formats
    if (Array.isArray(data)) {
      // Array of test results
      for (const item of data) {
        results.push(parseJSONTestSuite(item, filePath));
      }
    } else if (data.testResults || data.results) {
      // Jest format
      const testResults = data.testResults || data.results;
      for (const suite of testResults) {
        results.push(parseJSONTestSuite(suite, filePath));
      }
    } else if (data.suites || data.tests) {
      // Mocha format
      const suites = data.suites || [data];
      for (const suite of suites) {
        results.push(parseJSONTestSuite(suite, filePath));
      }
    } else {
      // Single test suite
      results.push(parseJSONTestSuite(data, filePath));
    }
  } catch (error) {
    console.warn(
      `⚠️ Failed to parse JSON test results from ${filePath}:`,
      error
    );
  }

  return results;
}

/**
 * Parse a single JSON test suite
 */
function parseJSONTestSuite(suite: any, filePath: string): TestSuiteResult {
  const testResults: TestResult[] = [];

  // Extract test results from various JSON formats
  const tests = suite.tests || suite.assertions || suite.testResults || [];

  for (const test of tests) {
    const testName = test.title || test.name || test.testName || "Unknown Test";
    const duration = test.duration || test.time || 0;

    let status: "passed" | "failed" | "skipped" = "passed";
    let errorMessage = "";

    if (
      test.status === "failed" ||
      test.state === "failed" ||
      test.passed === false
    ) {
      status = "failed";
      errorMessage = test.error?.message || test.failureMessages?.[0] || "";
    } else if (
      test.status === "skipped" ||
      test.state === "skipped" ||
      test.pending
    ) {
      status = "skipped";
    }

    testResults.push({
      name: testName,
      status,
      duration: typeof duration === "number" ? duration : 0,
      errorMessage: errorMessage.trim(),
    });
  }

  return {
    suiteName:
      suite.name ||
      suite.title ||
      path.basename(filePath, path.extname(filePath)),
    timestamp: suite.timestamp || new Date().toISOString(),
    testResults,
    totalTests: testResults.length,
    passedTests: testResults.filter((t) => t.status === "passed").length,
    failedTests: testResults.filter((t) => t.status === "failed").length,
    skippedTests: testResults.filter((t) => t.status === "skipped").length,
    totalDuration: testResults.reduce((sum, t) => sum + t.duration, 0),
    source: filePath,
  };
}

/**
 * Parse TAP (Test Anything Protocol) results
 */
function parseTAPResults(content: string, filePath: string): TestSuiteResult[] {
  const results: TestSuiteResult[] = [];
  const lines = content.split("\n");
  const testResults: TestResult[] = [];

  let currentTest: Partial<TestResult> = {};
  let testCounter = 0;

  for (const line of lines) {
    const trimmed = line.trim();

    if (trimmed.startsWith("ok ")) {
      // Test passed
      testCounter++;
      const testName = trimmed.substring(3).split("#")[0].trim();
      testResults.push({
        name: testName || `Test ${testCounter}`,
        status: "passed",
        duration: 0,
        errorMessage: "",
      });
    } else if (trimmed.startsWith("not ok ")) {
      // Test failed
      testCounter++;
      const testName = trimmed.substring(7).split("#")[0].trim();
      testResults.push({
        name: testName || `Test ${testCounter}`,
        status: "failed",
        duration: 0,
        errorMessage: "",
      });
    } else if (trimmed.startsWith("#")) {
      // Comment or diagnostic information
      if (currentTest.name && trimmed.includes("time=")) {
        const timeMatch = trimmed.match(/time=([0-9.]+)/);
        if (timeMatch) {
          currentTest.duration = parseFloat(timeMatch[1]) * 1000; // Convert to milliseconds
        }
      }
    }
  }

  if (testResults.length > 0) {
    results.push({
      suiteName: path.basename(filePath, path.extname(filePath)),
      timestamp: new Date().toISOString(),
      testResults,
      totalTests: testResults.length,
      passedTests: testResults.filter((t) => t.status === "passed").length,
      failedTests: testResults.filter((t) => t.status === "failed").length,
      skippedTests: testResults.filter((t) => t.status === "skipped").length,
      totalDuration: testResults.reduce((sum, t) => sum + t.duration, 0),
      source: filePath,
    });
  }

  return results;
}

/**
 * Generate mock test results for demonstration
 */
function generateMockTestResults(): TestSuiteResult[] {
  return [
    {
      suiteName: "Mock Test Suite 1",
      timestamp: new Date(Date.now() - 3600000).toISOString(),
      testResults: [
        {
          name: "test_feature_a",
          status: "passed",
          duration: 150,
          errorMessage: "",
        },
        {
          name: "test_feature_b",
          status: "failed",
          duration: 200,
          errorMessage: "Assertion failed",
        },
        {
          name: "test_feature_c",
          status: "passed",
          duration: 100,
          errorMessage: "",
        },
      ],
      totalTests: 3,
      passedTests: 2,
      failedTests: 1,
      skippedTests: 0,
      totalDuration: 450,
      source: "mock-data",
    },
  ];
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

export { FlakeDetectionService };
