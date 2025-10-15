#!/usr/bin/env node

/**
 * Arbiter Progress Monitoring Script
 *
 * Monitors arbiter progress against acceptance criteria and reports metrics.
 *
 * @author @darianrosebrook
 */

import { execSync } from "child_process";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = join(__dirname, "..");

// ANSI colors
const colors = {
  reset: "\x1b[0m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  red: "\x1b[31m",
  blue: "\x1b[34m",
  gray: "\x1b[90m",
};

function log(msg, color = "reset") {
  console.log(`${colors[color]}${msg}${colors.reset}`);
}

function section(title) {
  console.log("\n" + "=".repeat(60));
  log(title, "blue");
  console.log("=".repeat(60));
}

function countFiles(pattern) {
  try {
    const result = execSync(
      `find ${projectRoot}/src ${projectRoot}/tests -name '${pattern}' 2>/dev/null | wc -l`,
      { encoding: "utf-8" }
    );
    return parseInt(result.trim()) || 0;
  } catch {
    return 0;
  }
}

function runCommand(cmd) {
  try {
    execSync(cmd, { cwd: projectRoot, encoding: "utf-8", stdio: "pipe" });
    return { success: true, output: "" };
  } catch (error) {
    return { success: false, output: error.stdout || error.message };
  }
}

function getTestResults() {
  const result = runCommand("npm test -- --passWithNoTests 2>&1");

  if (!result.success) {
    const output = result.output;
    const passMatch = output.match(/(\d+) passing/);
    const failMatch = output.match(/(\d+) failing/);

    return {
      passing: passMatch ? parseInt(passMatch[1]) : 0,
      failing: failMatch ? parseInt(failMatch[1]) : 0,
      total:
        (passMatch ? parseInt(passMatch[1]) : 0) +
        (failMatch ? parseInt(failMatch[1]) : 0),
    };
  }

  return { passing: 0, failing: 0, total: 0 };
}

function getLintErrors() {
  const result = runCommand("npm run lint 2>&1");
  if (result.success) return 0;

  const errorMatch = result.output.match(/(\d+) error/);
  return errorMatch ? parseInt(errorMatch[1]) : 0;
}

function getTypeCheckErrors() {
  const result = runCommand("npm run typecheck 2>&1");
  if (result.success) return 0;

  const errorMatch = result.output.match(/Found (\d+) error/);
  return errorMatch ? parseInt(errorMatch[1]) : 0;
}

function checkAcceptanceCriteria() {
  const specs = [
    { file: ".caws/working-spec-system.yaml", criteria: 8 },
    { file: ".caws/working-spec-e2e-text.yaml", criteria: 5 },
    { file: ".caws/working-spec-e2e-code.yaml", criteria: 5 },
    { file: ".caws/working-spec-e2e-tokens.yaml", criteria: 5 },
    { file: ".caws/working-spec-integration.yaml", criteria: 6 },
    { file: ".caws/working-spec-documentation.yaml", criteria: 5 },
  ];

  const totalCriteria = specs.reduce((sum, spec) => sum + spec.criteria, 0);

  return { total: totalCriteria, met: 0 }; // Will be determined by test results
}

function monitorProgress() {
  section("BOT-POC ARBITER PROGRESS MONITOR");

  log(`Project Root: ${projectRoot}`, "gray");
  log(`Timestamp: ${new Date().toISOString()}`, "gray");

  // File counts
  section("File Creation Progress");
  const srcFiles = countFiles("*.ts");
  const testFiles = countFiles("*.test.ts");
  const e2eFiles = countFiles("*.e2e.test.ts");

  log(`Source files (.ts): ${srcFiles}`, srcFiles > 0 ? "green" : "yellow");
  log(
    `Test files (.test.ts): ${testFiles}`,
    testFiles > 0 ? "green" : "yellow"
  );
  log(
    `E2E test files (.e2e.test.ts): ${e2eFiles}`,
    e2eFiles >= 3 ? "green" : "yellow"
  );

  // Test results
  section("Test Execution Status");
  const tests = getTestResults();
  log(
    `Tests passing: ${tests.passing}/${tests.total}`,
    tests.passing === tests.total && tests.total > 0 ? "green" : "yellow"
  );
  if (tests.failing > 0) {
    log(`Tests failing: ${tests.failing}`, "red");
  }

  // Quality gates
  section("Quality Gates");
  const lintErrors = getLintErrors();
  const typeErrors = getTypeCheckErrors();

  log(`Linting errors: ${lintErrors}`, lintErrors === 0 ? "green" : "red");
  log(`TypeScript errors: ${typeErrors}`, typeErrors === 0 ? "green" : "red");

  // Acceptance criteria
  section("Acceptance Criteria");
  const acceptance = checkAcceptanceCriteria();
  log(
    `Criteria met: ${acceptance.met}/${acceptance.total} (${Math.round(
      (acceptance.met / acceptance.total) * 100
    )}%)`,
    acceptance.met === acceptance.total ? "green" : "yellow"
  );

  // Overall status
  section("Overall Status");
  const allGreen =
    srcFiles > 0 &&
    testFiles > 0 &&
    e2eFiles >= 3 &&
    tests.passing === tests.total &&
    tests.total > 0 &&
    lintErrors === 0 &&
    typeErrors === 0;

  if (allGreen) {
    log(
      "✅ All quality gates PASSING - Ready for completion validation",
      "green"
    );
  } else if (srcFiles === 0) {
    log(
      "⏳ INITIALIZATION PHASE - Waiting for arbiter to start building",
      "yellow"
    );
  } else {
    log("🚧 IN PROGRESS - Arbiter is actively building", "yellow");
  }

  // Recommendations
  section("Next Steps");
  if (srcFiles === 0) {
    log("→ Run: npm install", "blue");
    log("→ Run: npm run validate:all", "blue");
    log("→ Begin implementing E2E-001 (Text Transformation)", "blue");
  } else if (lintErrors > 0 || typeErrors > 0) {
    log("→ Fix linting and TypeScript errors", "blue");
  } else if (tests.failing > 0) {
    log("→ Fix failing tests", "blue");
  } else if (e2eFiles < 3) {
    log("→ Complete all 3 E2E test scenarios", "blue");
  } else {
    log("→ Generate documentation and completion report", "blue");
  }

  console.log("\n");
}

// Run monitoring
monitorProgress();
