#!/usr/bin/env node

/**
 * Iteration Management Script
 *
 * Utilities for managing the mono-repo iterations structure.
 * Handles iteration creation, validation, and cross-iteration operations.
 *
 * @author @darianrosebrook
 */

import { execSync } from "child_process";
import { existsSync, readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT_DIR = join(__dirname, "..");

const ITERATIONS_DIR = join(ROOT_DIR, "iterations");
const AVAILABLE_ITERATIONS = ["poc", "v1"];

class IterationManager {
  constructor() {
    this.iterations = AVAILABLE_ITERATIONS;
  }

  /**
   * List all available iterations
   */
  listIterations() {
    console.log("Available Iterations:");
    this.iterations.forEach((iteration) => {
      const iterationPath = join(ITERATIONS_DIR, iteration);
      const exists = existsSync(iterationPath);
      const status = exists ? "✅" : "❌";
      console.log(`  ${status} ${iteration}: ${iterationPath}`);
    });
  }

  /**
   * Validate iteration structure
   */
  validateIteration(iterationName) {
    const iterationPath = join(ITERATIONS_DIR, iterationName);

    if (!existsSync(iterationPath)) {
      console.error(`❌ Iteration '${iterationName}' does not exist`);
      return false;
    }

    const requiredFiles = [
      "package.json",
      "tsconfig.json",
      "src/index.ts",
      "src/types/index.ts",
    ];

    const missingFiles = requiredFiles.filter(
      (file) => !existsSync(join(iterationPath, file))
    );

    if (missingFiles.length > 0) {
      console.error(`❌ Missing required files in ${iterationName}:`);
      missingFiles.forEach((file) => console.error(`  - ${file}`));
      return false;
    }

    console.log(`✅ Iteration '${iterationName}' structure is valid`);
    return true;
  }

  /**
   * Validate all iterations
   */
  validateAllIterations() {
    console.log("Validating all iterations...\n");
    let allValid = true;

    this.iterations.forEach((iteration) => {
      const isValid = this.validateIteration(iteration);
      if (!isValid) allValid = false;
      console.log("");
    });

    return allValid;
  }

  /**
   * Run command in specific iteration
   */
  runInIteration(iterationName, command) {
    const iterationPath = join(ITERATIONS_DIR, iterationName);

    if (!existsSync(iterationPath)) {
      console.error(`❌ Iteration '${iterationName}' does not exist`);
      return false;
    }

    try {
      console.log(`🔄 Running '${command}' in iteration '${iterationName}'`);
      execSync(command, {
        cwd: iterationPath,
        stdio: "inherit",
        env: { ...process.env, FORCE_COLOR: "1" },
      });
      return true;
    } catch (error) {
      console.error(
        `❌ Command failed in iteration '${iterationName}':`,
        error.message
      );
      return false;
    }
  }

  /**
   * Run command in all iterations
   */
  runInAllIterations(command) {
    console.log(`🔄 Running '${command}' in all iterations\n`);
    let allSuccessful = true;

    this.iterations.forEach((iteration) => {
      console.log(`--- ${iteration.toUpperCase()} ---`);
      const success = this.runInIteration(iteration, command);
      if (!success) allSuccessful = false;
      console.log("");
    });

    return allSuccessful;
  }

  /**
   * Get iteration stats
   */
  getIterationStats(iterationName) {
    const iterationPath = join(ITERATIONS_DIR, iterationName);

    if (!existsSync(iterationPath)) {
      console.error(`❌ Iteration '${iterationName}' does not exist`);
      return null;
    }

    try {
      const packageJson = JSON.parse(
        readFileSync(join(iterationPath, "package.json"), "utf8")
      );

      // Count TypeScript files
      const findCmd = `find ${iterationPath} -name "*.ts" -not -path "*/node_modules/*" -not -path "*/dist/*" | wc -l`;
      const tsFiles = parseInt(execSync(findCmd, { encoding: "utf8" }).trim());

      // Count test files
      const testCmd = `find ${iterationPath} -name "*.test.ts" -o -name "*.spec.ts" | wc -l`;
      const testFiles = parseInt(
        execSync(testCmd, { encoding: "utf8" }).trim()
      );

      return {
        name: iterationName,
        version: packageJson.version,
        description: packageJson.description,
        tsFiles,
        testFiles,
        dependencies: Object.keys(packageJson.dependencies || {}).length,
        devDependencies: Object.keys(packageJson.devDependencies || {}).length,
      };
    } catch (error) {
      console.error(
        `❌ Error reading stats for '${iterationName}':`,
        error.message
      );
      return null;
    }
  }

  /**
   * Display stats for all iterations
   */
  showStats() {
    console.log("Iteration Statistics:\n");

    this.iterations.forEach((iteration) => {
      const stats = this.getIterationStats(iteration);
      if (stats) {
        console.log(`${iteration.toUpperCase()}:`);
        console.log(`  Version: ${stats.version}`);
        console.log(`  TypeScript Files: ${stats.tsFiles}`);
        console.log(`  Test Files: ${stats.testFiles}`);
        console.log(`  Dependencies: ${stats.dependencies}`);
        console.log(`  Dev Dependencies: ${stats.devDependencies}`);
        console.log(`  Description: ${stats.description}`);
        console.log("");
      }
    });
  }

  /**
   * Show help
   */
  showHelp() {
    console.log(`
Iteration Manager v1.0

USAGE:
  node scripts/manage-iterations.js <command> [options]

COMMANDS:
  list                    List all available iterations
  validate [iteration]    Validate iteration structure (all if no iteration specified)
  run <iteration> <cmd>   Run command in specific iteration
  run-all <cmd>           Run command in all iterations
  stats                   Show statistics for all iterations
  help                    Show this help message

EXAMPLES:
  node scripts/manage-iterations.js list
  node scripts/manage-iterations.js validate poc
  node scripts/manage-iterations.js validate
  node scripts/manage-iterations.js run main "npm test"
  node scripts/manage-iterations.js run-all "npm run typecheck"
  node scripts/manage-iterations.js stats

ITERATIONS:
  poc    - Proof of Concept implementation
  main   - Production v1.0 with agentic RL features
`);
  }
}

// Main execution
function main() {
  const args = process.argv.slice(2);
  const manager = new IterationManager();

  if (args.length === 0) {
    manager.showHelp();
    process.exit(1);
  }

  const command = args[0];

  switch (command) {
    case "list":
      manager.listIterations();
      break;

    case "validate":
      if (args[1]) {
        manager.validateIteration(args[1]);
      } else {
        manager.validateAllIterations();
      }
      break;

    case "run":
      if (args.length < 3) {
        console.error("❌ Usage: run <iteration> <command>");
        process.exit(1);
      }
      const success = manager.runInIteration(args[1], args.slice(2).join(" "));
      process.exit(success ? 0 : 1);
      break;

    case "run-all":
      if (args.length < 2) {
        console.error("❌ Usage: run-all <command>");
        process.exit(1);
      }
      const allSuccess = manager.runInAllIterations(args.slice(1).join(" "));
      process.exit(allSuccess ? 0 : 1);
      break;

    case "stats":
      manager.showStats();
      break;

    case "help":
    default:
      manager.showHelp();
      break;
  }
}

main();
