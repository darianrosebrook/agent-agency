#!/usr/bin/env node

/**
 * E2E Test Runner Script
 *
 * @author @darianrosebrook
 * @description Script to run comprehensive E2E tests for Agent Agency
 */

import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "../");

/**
 * Run E2E tests with proper setup
 */
async function runE2ETests() {
  console.log("🚀 Agent Agency E2E Test Runner");
  console.log("===============================");

  try {
    // Step 1: Setup environment
    console.log("\n📋 Step 1: Setting up test environment...");
    await runCommand("npm", ["run", "test:e2e:setup"], projectRoot);

    // Step 2: Build the project
    console.log("\n🔨 Step 2: Building project...");
    await runCommand("npm", ["run", "build"], projectRoot);

    // Step 3: Run E2E tests
    console.log("\n🧪 Step 3: Running E2E tests...");
    await runCommand("npm", ["run", "test:e2e"], projectRoot);

    // Step 4: Generate reports
    console.log("\n📊 Step 4: Generating comprehensive report...");
    await runCommand("node", ["tests/e2e/test-runner.ts"], projectRoot);

    console.log("\n🎉 E2E testing completed successfully!");
  } catch (error) {
    console.error("\n❌ E2E testing failed:", error.message);
    process.exit(1);
  }
}

/**
 * Run a command and return a promise
 */
function runCommand(command, args, cwd) {
  return new Promise((resolve, reject) => {
    console.log(`Running: ${command} ${args.join(" ")}`);

    const child = spawn(command, args, {
      cwd,
      stdio: "inherit",
      shell: true,
    });

    child.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Command failed with code ${code}`));
      }
    });

    child.on("error", reject);
  });
}

/**
 * Show usage information
 */
function showUsage() {
  console.log(`
Agent Agency E2E Test Runner

Usage:
  node scripts/run-e2e-tests.js

This script will:
1. Set up the E2E test environment (Docker containers, Ollama)
2. Build the project
3. Run all E2E tests
4. Generate comprehensive reports

Prerequisites:
- Docker installed and running
- Ollama installed (optional, tests will skip AI features if not available)
- Node.js 18+

Environment Variables:
- CI=true (for CI/CD environments)
- SKIP_AI_TESTS=true (to skip AI-dependent tests)

Reports will be saved to:
- tests/e2e/artifacts/e2e-report-YYYY-MM-DD.json
- tests/e2e/artifacts/e2e-summary-YYYY-MM-DD.md
`);
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  if (process.argv.includes("--help") || process.argv.includes("-h")) {
    showUsage();
  } else {
    runE2ETests();
  }
}

export { runE2ETests };
