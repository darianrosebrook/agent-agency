#!/usr/bin/env node

/**
 * @fileoverview Test Analysis Module
 * Provides test result parsing and analysis for CAWS dashboard
 * @author @darianrosebrook
 */

const fs = require("fs");
const path = require("path");

/**
 * Parse test results from various formats
 * @param {string} resultPath - Path to test results directory
 * @returns {Array} Array of parsed test run data
 */
function parseTestResults(resultPath) {
  const testRuns = [];

  try {
    // Try to find and parse test result files
    const files = fs.readdirSync(resultPath, { recursive: true });

    for (const file of files) {
      if (typeof file === "string") {
        const filePath = path.join(resultPath, file);

        // Parse JUnit XML results
        if (file.endsWith(".xml") && file.includes("junit")) {
          const junitData = parseJUnitXML(filePath);
          if (junitData) testRuns.push(junitData);
        }

        // Parse cargo test output logs
        if (file.includes("cargo-test") || file.includes("test-output")) {
          const cargoData = parseCargoTestOutput(filePath);
          if (cargoData) testRuns.push(cargoData);
        }
      }
    }

    return testRuns;
  } catch (error) {
    console.error(
      "CAWS: Error parsing test results from " + resultPath + ":",
      error.message
    );
    return [];
  }
}

/**
 * Parse JUnit XML test results
 * @param {string} filePath - Path to JUnit XML file
 * @returns {Object|null} Parsed test run data
 */
function parseJUnitXML(filePath) {
  try {
    const xmlContent = fs.readFileSync(filePath, "utf8");
    // Simple XML parsing for test results
    const testSuiteMatch = xmlContent.match(
      /testsuite[^>]*tests="(\d+)"[^>]*failures="(\d+)"/
    );
    if (testSuiteMatch) {
      const totalTests = parseInt(testSuiteMatch[1]);
      const failures = parseInt(testSuiteMatch[2]);
      return {
        framework: "junit",
        total_tests: totalTests,
        passed: totalTests - failures,
        failed: failures,
        skipped: 0,
        duration_ms: 0,
        timestamp: new Date().toISOString(),
      };
    }
  } catch (error) {
    // Failed to parse JUnit XML
  }
  return null;
}

/**
 * Parse cargo test output
 * @param {string} filePath - Path to cargo test output file
 * @returns {Object|null} Parsed test run data
 */
function parseCargoTestOutput(filePath) {
  try {
    const output = fs.readFileSync(filePath, "utf8");

    // Parse cargo test summary
    const testMatch = output.match(/test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored/);
    if (testMatch) {
      const result = testMatch[1];
      const passed = parseInt(testMatch[2]);
      const failed = parseInt(testMatch[3]);
      const ignored = parseInt(testMatch[4]);

      return {
        framework: "cargo",
        total_tests: passed + failed + ignored,
        passed,
        failed,
        skipped: ignored,
        duration_ms: 0,
        timestamp: new Date().toISOString(),
      };
    }
  } catch (error) {
    // Failed to parse cargo output
  }
  return null;
}

/**
 * Analyze test execution history
 * @param {Array} testHistory - Historical test execution data
 * @returns {Object} Analysis results
 */
function analyzeTestExecutionHistory(testHistory) {
  if (!testHistory || testHistory.length === 0) {
    return {
      total_runs: 0,
      average_pass_rate: 0,
      failure_trends: [],
      flaky_tests: [],
    };
  }

  const totalRuns = testHistory.length;
  const averagePassRate = testHistory.reduce((sum, run) => {
    return sum + (run.passed / (run.passed + run.failed));
  }, 0) / totalRuns;

  // Simple failure trend analysis
  const failureTrends = testHistory.map(run => ({
    timestamp: run.timestamp,
    failure_rate: run.failed / (run.passed + run.failed),
  }));

  return {
    total_runs: totalRuns,
    average_pass_rate: averagePassRate,
    failure_trends: failureTrends,
    flaky_tests: [], // Would be calculated by detectFlakyTests
  };
}

module.exports = {
  parseTestResults,
  parseJUnitXML,
  parseCargoTestOutput,
  analyzeTestExecutionHistory
};
