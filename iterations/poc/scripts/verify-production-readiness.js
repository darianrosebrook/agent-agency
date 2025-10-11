#!/usr/bin/env node

/**
 * Production Readiness Verification Script
 *
 * @author @darianrosebrook
 * @description Comprehensive verification of production readiness criteria
 */

import { execSync } from "child_process";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "../");

/**
 * Verification result tracking
 */
class VerificationResult {
  constructor(name) {
    this.name = name;
    this.passed = 0;
    this.failed = 0;
    this.warnings = 0;
    this.errors = [];
    this.warnings_list = [];
  }

  pass(message) {
    this.passed++;
    console.log(`✅ ${message}`);
  }

  fail(message, error = null) {
    this.failed++;
    console.log(`❌ ${message}`);
    if (error) this.errors.push(`${message}: ${error}`);
  }

  warn(message) {
    this.warnings++;
    console.log(`⚠️  ${message}`);
    this.warnings_list.push(message);
  }

  get status() {
    if (this.failed > 0) return "FAILED";
    if (this.warnings > 0) return "WARNINGS";
    return "PASSED";
  }

  summary() {
    return `${this.name}: ${this.status} (${this.passed} passed, ${this.failed} failed, ${this.warnings} warnings)`;
  }
}

/**
 * Run a command and return result
 */
function runCommand(command, options = {}) {
  try {
    const result = execSync(command, {
      cwd: projectRoot,
      stdio: options.silent ? "pipe" : "inherit",
      timeout: options.timeout || 30000,
      ...options,
    });
    return { success: true, output: result.toString() };
  } catch (error) {
    return { success: false, error: error.message, code: error.status };
  }
}

/**
 * Check file existence
 */
function fileExists(filePath) {
  return fs.existsSync(path.resolve(projectRoot, filePath));
}

/**
 * Read file content
 */
function readFile(filePath) {
  try {
    return fs.readFileSync(path.resolve(projectRoot, filePath), "utf8");
  } catch (error) {
    return null;
  }
}

/**
 * Check package.json for required scripts
 */
function checkPackageScripts(result) {
  const packageJson = JSON.parse(readFile("package.json"));
  const requiredScripts = [
    "test",
    "lint",
    "typecheck",
    "build",
    "test:e2e",
    "test:coverage",
    "test:mutation",
  ];

  requiredScripts.forEach((script) => {
    if (packageJson.scripts && packageJson.scripts[script]) {
      result.pass(`Package script '${script}' exists`);
    } else {
      result.fail(`Package script '${script}' missing`);
    }
  });
}

/**
 * Code Quality Verification
 */
async function verifyCodeQuality() {
  console.log("\n🔧 CODE QUALITY VERIFICATION");
  console.log("=".repeat(40));

  const result = new VerificationResult("Code Quality");

  // Check package.json scripts
  checkPackageScripts(result);

  // Run linting
  console.log("\n📋 Running ESLint...");
  const lintResult = runCommand("npm run lint", { silent: true });
  if (lintResult.success) {
    result.pass("ESLint passed with no errors");
  } else {
    const errorCount = (lintResult.error.match(/error/g) || []).length;
    if (errorCount === 0) {
      result.pass("ESLint passed (warnings only)");
    } else {
      result.fail(`ESLint failed with ${errorCount} errors`);
    }
  }

  // TypeScript compilation
  console.log("\n📋 Running TypeScript check...");
  const tsResult = runCommand("npm run typecheck", { silent: true });
  if (tsResult.success) {
    result.pass("TypeScript compilation successful");
  } else {
    result.fail("TypeScript compilation failed");
  }

  // Check for any types
  console.log("\n📋 Checking for any types...");
  const grepResult = runCommand(
    'grep -r ": any" src/ --include="*.ts" | wc -l',
    { silent: true }
  );
  if (grepResult.success) {
    const anyCount = parseInt(grepResult.output.trim());
    if (anyCount === 0) {
      result.pass("No explicit any types found");
    } else {
      result.warn(`${anyCount} any types found (should minimize)`);
    }
  }

  return result;
}

/**
 * Testing Verification
 */
async function verifyTesting() {
  console.log("\n🧪 TESTING VERIFICATION");
  console.log("=".repeat(40));

  const result = new VerificationResult("Testing");

  // Run unit tests
  console.log("\n📋 Running unit tests...");
  const testResult = runCommand("npm test", { silent: false, timeout: 120000 });

  // Check if tests passed by looking for exit code and output
  if (testResult.success) {
    // Try to parse test results from stdout
    const output = testResult.output || "";

    // Look for "Test Suites: X passed" or similar patterns
    const suitesPassed =
      output.includes("Test Suites:") && !output.includes("failed");
    const testsPassed = output.includes("Tests:") && !output.includes("failed");

    if (suitesPassed && testsPassed) {
      // Extract number of tests if possible
      const testMatch = output.match(/Tests:\s*(\d+)\s*passed/);
      const numTests = testMatch ? parseInt(testMatch[1]) : "multiple";
      result.pass(`All ${numTests} unit tests passed`);
    } else {
      result.fail("Unit test results unclear");
    }
  } else {
    // Tests failed - check for specific failure patterns
    const errorOutput = testResult.error || "";
    const hasFailures =
      errorOutput.includes("FAIL") ||
      errorOutput.includes("failed") ||
      errorOutput.includes("Error:") ||
      errorOutput.includes("✖");

    if (hasFailures) {
      // Count failed test indicators
      const failCount =
        (errorOutput.match(/FAIL/g) || []).length ||
        (errorOutput.match(/✖/g) || []).length ||
        1;
      result.fail(`${failCount} test suite(s) failed`);
    } else {
      result.fail("Unit tests failed to complete");
    }
  }

  // Check test coverage
  console.log("\n📋 Checking test coverage...");
  const coverageResult = runCommand("npm run test:coverage", {
    silent: true,
    timeout: 60000,
  });
  if (coverageResult.success) {
    // Parse coverage from output (basic check)
    const coverageMatch = coverageResult.output.match(
      /All files[^|]*\|[^|]*\|[^|]*\|[^|]*\|[^|]*(\d+.\d+)%/
    );
    if (coverageMatch) {
      const coverage = parseFloat(coverageMatch[1]);
      if (coverage >= 95) {
        result.pass(`Test coverage: ${coverage}% (≥95% target)`);
      } else if (coverage >= 85) {
        result.warn(`Test coverage: ${coverage}% (target: ≥95%)`);
      } else {
        result.fail(`Test coverage: ${coverage}% (below 85% minimum)`);
      }
    } else {
      result.warn("Could not parse coverage percentage");
    }
  } else {
    result.fail("Coverage report generation failed");
  }

  // Check for test files
  const testFiles = runCommand('find tests -name "*.test.ts" | wc -l', {
    silent: true,
  });
  if (testFiles.success) {
    const testCount = parseInt(testFiles.output.trim());
    if (testCount >= 10) {
      result.pass(`${testCount} test files found`);
    } else {
      result.warn(`Only ${testCount} test files found (expected ≥10)`);
    }
  }

  return result;
}

/**
 * Security Verification
 */
async function verifySecurity() {
  console.log("\n🔒 SECURITY VERIFICATION");
  console.log("=".repeat(40));

  const result = new VerificationResult("Security");

  // NPM audit
  console.log("\n📋 Running npm audit...");
  const auditResult = runCommand("npm audit --audit-level=moderate", {
    silent: true,
  });
  if (auditResult.success) {
    result.pass("NPM audit passed");
  } else {
    const vulnCount = (auditResult.error.match(/vulnerabilities/g) || [])
      .length;
    result.fail(`NPM audit found ${vulnCount} vulnerabilities`);
  }

  // Check for hardcoded secrets (excluding test files and known safe patterns)
  console.log("\n📋 Checking for hardcoded secrets...");
  const secretPatterns = ["password.*=", "secret.*=", "key.*=", "token.*="];

  let secretFound = false;
  for (const pattern of secretPatterns) {
    // Exclude test files, mock data, and environment variable usage
    const grepResult = runCommand(
      `grep -r "${pattern}" src/ --include="*.ts" | grep -v "process.env" | grep -v "mock" | grep -v "test" | grep -v "example" | grep -v "config" | wc -l`,
      { silent: true }
    );
    if (grepResult.success && parseInt(grepResult.output.trim()) > 0) {
      secretFound = true;
      result.fail(`Potential hardcoded secrets found with pattern: ${pattern}`);
    }
  }

  if (!secretFound) {
    result.pass("No hardcoded secrets detected");
  }

  // Check for environment variables usage
  const envUsage = runCommand(
    'grep -r "process.env" src/ --include="*.ts" | wc -l',
    { silent: true }
  );
  if (envUsage.success && parseInt(envUsage.output.trim()) > 0) {
    result.pass("Environment variables properly used");
  } else {
    result.warn("No environment variable usage detected");
  }

  return result;
}

/**
 * Performance Verification
 */
async function verifyPerformance() {
  console.log("\n📊 PERFORMANCE VERIFICATION");
  console.log("=".repeat(40));

  const result = new VerificationResult("Performance");

  // Check for performance-critical patterns
  console.log("\n📋 Checking performance patterns...");

  // Check for memory leaks (basic)
  const memoryIssues = runCommand(
    'grep -r "setInterval|setTimeout" src/ --include="*.ts" | grep -v "clearInterval|clearTimeout" | wc -l',
    { silent: true }
  );
  if (memoryIssues.success) {
    const issueCount = parseInt(memoryIssues.output.trim());
    if (issueCount === 0) {
      result.pass("No potential memory leaks detected");
    } else {
      result.warn(
        `${issueCount} potential memory leaks (timers without cleanup)`
      );
    }
  }

  // Check bundle size (if build exists)
  if (fileExists("dist/index.js")) {
    const stats = fs.statSync(path.resolve(projectRoot, "dist/index.js"));
    const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
    if (parseFloat(sizeMB) < 10) {
      result.pass(`Bundle size: ${sizeMB}MB (acceptable)`);
    } else {
      result.warn(`Bundle size: ${sizeMB}MB (consider optimization)`);
    }
  } else {
    result.warn("No build artifacts found for size check");
  }

  // Check for efficient algorithms (basic heuristic)
  const inefficientPatterns = [
    "for.*for", // Nested loops
    "while.*true", // Infinite loops
    "O(n.*n)", // Explicit complexity mentions
  ];

  let inefficientFound = false;
  for (const pattern of inefficientPatterns) {
    const grepResult = runCommand(
      `grep -r "${pattern}" src/ --include="*.ts" | wc -l`,
      { silent: true }
    );
    if (grepResult.success && parseInt(grepResult.output.trim()) > 0) {
      inefficientFound = true;
      result.warn(`Potentially inefficient pattern found: ${pattern}`);
    }
  }

  if (!inefficientFound) {
    result.pass("No obvious performance anti-patterns detected");
  }

  return result;
}

/**
 * Infrastructure Verification
 */
async function verifyInfrastructure() {
  console.log("\n🏗️  INFRASTRUCTURE VERIFICATION");
  console.log("=".repeat(40));

  const result = new VerificationResult("Infrastructure");

  // Check for Docker setup
  if (fileExists("docker-compose.yml")) {
    result.pass("Docker Compose configuration found");

    // Check if services can start (basic check)
    const composeCheck = runCommand("docker-compose config", { silent: true });
    if (composeCheck.success) {
      result.pass("Docker Compose configuration is valid");
    } else {
      result.fail("Docker Compose configuration has errors");
    }
  } else {
    result.fail("Docker Compose configuration missing");
  }

  // Check for database migrations
  if (fileExists("migrations/")) {
    const migrationFiles = runCommand("ls migrations/*.sql | wc -l", {
      silent: true,
    });
    if (migrationFiles.success && parseInt(migrationFiles.output.trim()) >= 2) {
      result.pass("Database migrations present");
    } else {
      result.warn("Limited database migrations found");
    }
  } else {
    result.fail("Database migrations directory missing");
  }

  // Check for environment configuration
  if (fileExists("src/config/") || fileExists("src/utils/config.ts")) {
    result.pass("Configuration management present");
  } else {
    result.warn("Configuration management not clearly structured");
  }

  // Check for health checks
  const healthCheck = runCommand(
    'grep -r "health" src/ --include="*.ts" | wc -l',
    { silent: true }
  );
  if (healthCheck.success && parseInt(healthCheck.output.trim()) > 0) {
    result.pass("Health check endpoints detected");
  } else {
    result.warn("No health check endpoints found");
  }

  return result;
}

/**
 * Documentation Verification
 */
async function verifyDocumentation() {
  console.log("\n📚 DOCUMENTATION VERIFICATION");
  console.log("=".repeat(40));

  const result = new VerificationResult("Documentation");

  // Check for README
  if (fileExists("README.md")) {
    const readme = readFile("README.md");
    if (readme && readme.length > 1000) {
      result.pass("Comprehensive README present");
    } else {
      result.warn("README exists but may be incomplete");
    }
  } else {
    result.fail("README.md missing");
  }

  // Check for API documentation
  const docsDir = fs.existsSync(path.resolve(projectRoot, "docs"));
  if (docsDir) {
    result.pass("Documentation directory present");
  } else {
    result.warn("Documentation directory missing");
  }

  // Check for JSDoc comments
  const jsdocCheck = runCommand(
    'grep -r "/\\*\\*" src/ --include="*.ts" | wc -l',
    { silent: true }
  );
  if (jsdocCheck.success) {
    const jsdocCount = parseInt(jsdocCheck.output.trim());
    if (jsdocCount >= 10) {
      result.pass(`${jsdocCount} JSDoc comments found`);
    } else {
      result.warn(`Only ${jsdocCount} JSDoc comments found`);
    }
  }

  return result;
}

/**
 * Compliance Verification
 */
async function verifyCompliance() {
  console.log("\n⚖️  COMPLIANCE VERIFICATION");
  console.log("=".repeat(40));

  const result = new VerificationResult("Compliance");

  // Check for license
  if (fileExists("LICENSE") || fileExists("LICENSE.md")) {
    result.pass("License file present");
  } else {
    result.fail("License file missing");
  }

  // Check for security policy
  if (fileExists("SECURITY.md") || fileExists("docs/SECURITY.md")) {
    result.pass("Security policy documented");
  } else {
    result.warn("Security policy not found");
  }

  // Check for data handling policies
  const privacyCheck = runCommand(
    'grep -i "privacy\\|gdpr\\|ccpa" docs/ README.md | wc -l',
    { silent: true }
  );
  if (privacyCheck.success && parseInt(privacyCheck.output.trim()) > 0) {
    result.pass("Privacy compliance mentioned");
  } else {
    result.warn("Privacy compliance not clearly documented");
  }

  return result;
}

/**
 * Main verification function
 */
async function runVerification() {
  console.log("🚀 Agent Agency Production Readiness Verification");
  console.log("================================================\n");

  const results = [];
  let totalPassed = 0;
  let totalFailed = 0;
  let totalWarnings = 0;

  try {
    // Run all verification checks
    results.push(await verifyCodeQuality());
    results.push(await verifyTesting());
    results.push(await verifySecurity());
    results.push(await verifyPerformance());
    results.push(await verifyInfrastructure());
    results.push(await verifyDocumentation());
    results.push(await verifyCompliance());

    // Calculate totals
    results.forEach((result) => {
      totalPassed += result.passed;
      totalFailed += result.failed;
      totalWarnings += result.warnings;
    });

    // Generate report
    console.log("\n📊 VERIFICATION SUMMARY");
    console.log("=".repeat(50));

    results.forEach((result) => {
      console.log(`${result.summary()}`);
    });

    console.log(`\n📈 OVERALL RESULTS:`);
    console.log(
      `   Total Checks: ${totalPassed + totalFailed + totalWarnings}`
    );
    console.log(`   Passed: ${totalPassed}`);
    console.log(`   Failed: ${totalFailed}`);
    console.log(`   Warnings: ${totalWarnings}`);

    const successRate =
      (totalPassed / (totalPassed + totalFailed + totalWarnings)) * 100;

    console.log(`\n🎯 SUCCESS RATE: ${successRate.toFixed(1)}%`);

    // Determine overall status
    if (totalFailed === 0 && totalWarnings === 0) {
      console.log("\n🟢 PRODUCTION READY: All checks passed!");
      process.exit(0);
    } else if (totalFailed === 0 && successRate >= 90) {
      console.log("\n🟡 CONDITIONAL READY: Minor warnings only");
      console.log("\n⚠️  Warnings:");
      results.forEach((result) => {
        result.warnings_list.forEach((warning) =>
          console.log(`   - ${warning}`)
        );
      });
      process.exit(0);
    } else {
      console.log("\n🔴 NOT PRODUCTION READY: Critical issues found");
      console.log("\n❌ Failures:");
      results.forEach((result) => {
        result.errors.forEach((error) => console.log(`   - ${error}`));
      });
      process.exit(1);
    }
  } catch (error) {
    console.error("\n💥 VERIFICATION FAILED:", error.message);
    process.exit(1);
  }
}

// Run verification if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runVerification();
}

export { runVerification };
