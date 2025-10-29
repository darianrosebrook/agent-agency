#!/usr/bin/env node

/**
 * Quality Gates Runner
 *
 * Runs all quality gates and blocks commits if any critical violations are found.
 * Part of the crisis response - prevents further codebase degradation.
 *
 * Usage:
 *   node scripts/quality-gates/run-quality-gates.js [--ci] [--fix]
 *
 * Options:
 *   --ci     Run in CI mode (stricter, no interactive fixes)
 *   --fix    Attempt automatic fixes for some violations
 */

import path from "path";

// Import quality gate modules
import {
  checkNamingViolations,
  checkSymbolNaming,
  collectSourceFiles,
  getEnforcementLevel,
} from "./check-naming.js";
import { checkDuplicationRegression } from "./check-duplication.js";
import {
  checkGodObjects,
  checkGodObjectRegression,
} from "./check-god-objects.js";
import {
  checkCommitMessage,
  checkNewFiles,
  checkLargeAdditions,
} from "./check-code-freeze.js";
import { execSync } from "child_process";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const CI_MODE = process.argv.includes("--ci");
const FIX_MODE = process.argv.includes("--fix");

class QualityGateRunner {
  constructor() {
    this.violations = [];
    this.warnings = [];
  }

  async runAllGates() {
    console.log("🚦 Running Quality Gates - Crisis Response Mode");
    console.log("=".repeat(50));

    try {
      // Gate 1: Naming Conventions
      console.log("\n🔤 Checking naming conventions...");
      await this.runNamingGate();

      // Gate 1.5: Code Freeze (Crisis Response)
      console.log("\n🚫 Checking code freeze compliance...");
      await this.runCodeFreezeGate();

      // Gate 2: Duplication Prevention
      console.log("\n📋 Checking duplication...");
      await this.runDuplicationGate();

      // Gate 3: God Object Prevention
      console.log("\n🏗️  Checking god objects...");
      await this.runGodObjectGate();

      // Gate 4: Documentation Quality
      console.log("\n📋 Checking documentation quality...");
      await this.runDocumentationQualityGate();

      // Report results
      this.reportResults();
    } catch (error) {
      console.error("❌ Quality gates failed:", error.message);
      process.exit(1);
    }
  }

  async runNamingGate() {
    try {
      // Collect source files first
      const sourceFiles = [];
      collectSourceFiles(
        path.join(process.cwd(), "iterations/v3"),
        sourceFiles
      );

      const filenameResults = checkNamingViolations(sourceFiles);
      const symbolViolations = checkSymbolNaming(sourceFiles);
      const allViolations = [
        ...filenameResults.violations,
        ...symbolViolations,
      ];
      const allWarnings = filenameResults.warnings;

      // Report warnings
      if (allWarnings.length > 0) {
        console.log(`   ℹ️  ${allWarnings.length} approved exceptions in use`);
        for (const warning of allWarnings) {
          console.log(`      📋 ${warning.file}: ${warning.reason}`);
        }
      }

      // Handle violations based on enforcement level
      const enforcementLevel = getEnforcementLevel();

      if (allViolations.length > 0) {
        console.log(
          `   🔧 Enforcement level: ${enforcementLevel.toUpperCase()}`
        );

        for (const violation of allViolations) {
          const severity = violation.severity || enforcementLevel;

          // Only add to violations if severity requires blocking
          if (severity === "fail" || severity === "block") {
            this.violations.push({
              gate: "naming",
              type: violation.type,
              message: violation.issue,
              file: violation.file,
              line: violation.line,
              rule: violation.rule,
              severity: severity,
              suggestion: violation.suggestion,
            });
          } else {
            // Warning level - add to warnings instead
            this.warnings.push({
              gate: "naming",
              type: violation.type,
              message: violation.issue,
              file: violation.file,
              line: violation.line,
              rule: violation.rule,
              suggestion: violation.suggestion,
            });
          }
        }

        if (enforcementLevel === "warning") {
          console.log(
            `   ⚠️  ${allViolations.length} naming warnings (commit allowed)`
          );
        } else {
          console.log(
            `   🚫 ${allViolations.length} naming violations (${enforcementLevel} mode)`
          );
        }
      } else {
        console.log("   ✅ No problematic naming patterns found");
      }
    } catch (error) {
      this.violations.push({
        gate: "naming",
        type: "error",
        message: error.message,
      });
    }
  }

  async runCodeFreezeGate() {
    try {
      const commitCheck = await checkCommitMessage();
      const filesCheck = await checkNewFiles();
      const sizeCheck = await checkLargeAdditions();

      const violations = [commitCheck, filesCheck, sizeCheck].filter(
        (check) => check.blocked
      );

      if (violations.length > 0) {
        for (const violation of violations) {
          this.violations.push({
            gate: "code_freeze",
            type: "crisis_response_violation",
            message: violation.reason,
            suggestion: violation.suggestion,
          });
        }
      } else {
        console.log("   ✅ Code freeze compliance check passed");
      }
    } catch (error) {
      this.violations.push({
        gate: "code_freeze",
        type: "error",
        message: `Code freeze check failed: ${error.message}`,
      });
    }
  }

  async runDuplicationGate() {
    try {
      const { checkDuplicationRegression } = await import(
        "./check-duplication.js"
      );

      // Get context for enforcement level
      const context = process.env.CAWS_ENFORCEMENT_CONTEXT || "commit";

      const duplicationResults = checkDuplicationRegression(context);

      // Report warnings (approved exceptions)
      if (duplicationResults.warnings.length > 0) {
        console.log(
          `   ℹ️  ${duplicationResults.warnings.length} approved exceptions in use`
        );
        for (const warning of duplicationResults.warnings) {
          console.log(
            `      📋 ${warning.violation.file}: ${warning.exception.reason}`
          );
        }
      }

      // Handle violations based on enforcement level
      const enforcementLevel = duplicationResults.enforcementLevel;

      if (duplicationResults.violations.length > 0) {
        console.log(
          `   🔧 Enforcement level: ${enforcementLevel.toUpperCase()}`
        );

        for (const violation of duplicationResults.violations) {
          const severity = violation.severity || enforcementLevel;

          // Only add to violations if severity requires blocking
          if (severity === "fail" || severity === "block") {
            this.violations.push({
              gate: "duplication",
              type: violation.type,
              message: violation.issue,
              file: violation.relativePath,
              threshold: violation.threshold,
              current: violation.current,
              severity: severity,
            });
          } else {
            // Warning level - add to warnings instead
            this.warnings.push({
              gate: "duplication",
              type: violation.type,
              message: violation.issue,
              file: violation.relativePath,
              threshold: violation.threshold,
              current: violation.current,
            });
          }
        }

        if (enforcementLevel === "warning") {
          console.log(
            `   ⚠️  ${duplicationResults.violations.length} duplication warnings (commit allowed)`
          );
        } else {
          console.log(
            `   🚫 ${duplicationResults.violations.length} duplication violations (${enforcementLevel} mode)`
          );
        }
      } else {
        console.log("   ✅ No duplication violations found");
      }
    } catch (error) {
      this.violations.push({
        gate: "duplication",
        type: "error",
        message: `Duplication check failed: ${error.message}`,
      });
    }
  }

  async runGodObjectGate() {
    try {
      const { checkGodObjects, checkGodObjectRegression } = await import(
        "./check-god-objects.js"
      );

      // Get context for enforcement level
      const context = process.env.CAWS_ENFORCEMENT_CONTEXT || "commit";

      const godObjectResults = checkGodObjects(context);
      const regressionViolations = checkGodObjectRegression();

      const allViolations = [
        ...godObjectResults.violations,
        ...regressionViolations,
      ];
      const allWarnings = godObjectResults.warnings;

      // Report warnings (approved exceptions)
      if (allWarnings.length > 0) {
        console.log(`   ℹ️  ${allWarnings.length} approved exceptions in use`);
        for (const warning of allWarnings) {
          console.log(
            `      📋 ${warning.violation.file}: ${warning.exception.reason}`
          );
        }
      }

      // Handle violations based on enforcement level
      const enforcementLevel = godObjectResults.enforcementLevel;

      if (allViolations.length > 0) {
        console.log(
          `   🔧 Enforcement level: ${enforcementLevel.toUpperCase()}`
        );

        for (const violation of allViolations) {
          const severity = violation.severity || enforcementLevel;

          // Only add to violations if severity requires blocking
          if (severity === "fail" || severity === "block") {
            this.violations.push({
              gate: "god_objects",
              type: violation.type,
              message: violation.message,
              file: violation.relativePath,
              size: violation.size,
              severity: severity,
            });
          } else {
            // Warning level - add to warnings instead
            this.warnings.push({
              gate: "god_objects",
              type: violation.type,
              message: violation.message,
              file: violation.relativePath,
              size: violation.size,
            });
          }
        }

        if (enforcementLevel === "warning") {
          console.log(
            `   ⚠️  ${allViolations.length} god object warnings (commit allowed)`
          );
        } else {
          console.log(
            `   🚫 ${allViolations.length} god object violations (${enforcementLevel} mode)`
          );
        }
      } else {
        console.log("   ✅ No god object violations found");
      }
    } catch (error) {
      this.violations.push({
        gate: "god_objects",
        type: "error",
        message: `God object check failed: ${error.message}`,
      });
    }
  }

  async runDocumentationQualityGate() {
    try {
      const __filename = fileURLToPath(import.meta.url);
      const __dirname = dirname(__filename);
      const projectRoot = join(__dirname, "..", "..");
      const docLinterPath = join(
        projectRoot,
        "scripts",
        "doc-quality-linter.py"
      );

      // Get context for enforcement level
      const context = process.env.CAWS_ENFORCEMENT_CONTEXT || "commit";

      // Run the documentation quality linter
      const output = execSync(
        `python3 "${docLinterPath}" --path "${projectRoot}" --format json --exit-code`,
        {
          encoding: "utf8",
          maxBuffer: 100 * 1024 * 1024, // 100MB buffer
          stdio: ["pipe", "pipe", "pipe"], // Capture stderr for error handling
        }
      );

      // Parse the JSON output
      const issues = JSON.parse(output);

      if (issues.length > 0) {
        // Convert issues to violations format for shared framework
        const rawViolations = issues.map((issue) => ({
          type: issue.severity === "error" ? "documentation_error" : "documentation_warning",
          file: issue.file,
          line: issue.line,
          message: issue.message,
          rule: issue.rule,
          suggested_fix: issue.suggested_fix,
        }));

        // Import shared framework
        const { processViolations } = await import("./shared-exception-framework.js");
        
        // Process violations with exception handling
        const result = processViolations("documentation", rawViolations, context);

        // Report warnings (approved exceptions)
        if (result.warnings.length > 0) {
          console.log(`   ℹ️  ${result.warnings.length} approved exceptions in use`);
          for (const warning of result.warnings) {
            console.log(`      📋 ${warning.violation.file}: ${warning.exception.reason}`);
          }
        }

        // Handle violations based on enforcement level
        const enforcementLevel = result.enforcementLevel;
        
        console.log(`   🔧 Enforcement level: ${enforcementLevel.toUpperCase()}`);
        
        for (const violation of result.violations) {
          const severity = violation.severity || enforcementLevel;
          
          // Only add to violations if severity requires blocking
          if (severity === "fail" || severity === "block") {
            this.violations.push({
              gate: "documentation_quality",
              type: violation.type,
              message: violation.message,
              file: violation.file,
              line: violation.line,
              rule: violation.rule,
              suggested_fix: violation.suggested_fix,
              severity: severity,
            });
          } else {
            // Warning level - add to warnings instead
            this.warnings.push({
              gate: "documentation_quality",
              type: violation.type,
              message: violation.message,
              file: violation.file,
              line: violation.line,
              rule: violation.rule,
              suggested_fix: violation.suggested_fix,
            });
          }
        }
        
        if (enforcementLevel === "warning") {
          console.log(`   ⚠️  ${result.violations.length} documentation warnings (commit allowed)`);
        } else {
          console.log(`   🚫 ${result.violations.length} documentation violations (${enforcementLevel} mode)`);
        }
      } else {
        console.log("   ✅ No documentation quality issues found");
      }
    } catch (error) {
      // Check if it's an exit code error (issues found) or a real error
      if (error.status === 1) {
        // This means the linter found issues and exited with code 1
        // The output should contain the JSON with issues
        try {
          const output = error.stdout || error.stderr || "";
          if (output.trim()) {
            const issues = JSON.parse(output);

            if (issues.length > 0) {
              // Get context for enforcement level
              const context = process.env.CAWS_ENFORCEMENT_CONTEXT || "commit";
              
              // Convert issues to violations format for shared framework
              const rawViolations = issues.map((issue) => ({
                type: issue.severity === "error" ? "documentation_error" : "documentation_warning",
                file: issue.file,
                line: issue.line,
                message: issue.message,
                rule: issue.rule,
                suggested_fix: issue.suggested_fix,
              }));

              // Import shared framework
              const { processViolations } = await import("./shared-exception-framework.js");
              
              // Process violations with exception handling
              const result = processViolations("documentation", rawViolations, context);

              // Report warnings (approved exceptions)
              if (result.warnings.length > 0) {
                console.log(`   ℹ️  ${result.warnings.length} approved exceptions in use`);
                for (const warning of result.warnings) {
                  console.log(`      📋 ${warning.violation.file}: ${warning.exception.reason}`);
                }
              }

              // Handle violations based on enforcement level
              const enforcementLevel = result.enforcementLevel;
              
              console.log(`   🔧 Enforcement level: ${enforcementLevel.toUpperCase()}`);
              
              for (const violation of result.violations) {
                const severity = violation.severity || enforcementLevel;
                
                // Only add to violations if severity requires blocking
                if (severity === "fail" || severity === "block") {
                  this.violations.push({
                    gate: "documentation_quality",
                    type: violation.type,
                    message: violation.message,
                    file: violation.file,
                    line: violation.line,
                    rule: violation.rule,
                    suggested_fix: violation.suggested_fix,
                    severity: severity,
                  });
                } else {
                  // Warning level - add to warnings instead
                  this.warnings.push({
                    gate: "documentation_quality",
                    type: violation.type,
                    message: violation.message,
                    file: violation.file,
                    line: violation.line,
                    rule: violation.rule,
                    suggested_fix: violation.suggested_fix,
                  });
                }
              }
              
              if (enforcementLevel === "warning") {
                console.log(`   ⚠️  ${result.violations.length} documentation warnings (commit allowed)`);
              } else {
                console.log(`   🚫 ${result.violations.length} documentation violations (${enforcementLevel} mode)`);
              }
            }
          }
        } catch (parseError) {
          // If we can't parse the output, treat as a general error
          this.violations.push({
            gate: "documentation_quality",
            type: "error",
            message: `Documentation quality check failed: ${error.message}`,
          });
        }
      } else {
        // Real error (Python not found, script missing, etc.)
        this.violations.push({
          gate: "documentation_quality",
          type: "error",
          message: `Documentation quality check failed: ${error.message}`,
        });
      }
    }
  }

  reportResults() {
    console.log("\n" + "=".repeat(50));
    console.log("📊 QUALITY GATES RESULTS");
    console.log("=".repeat(50));

    // Report warnings
    if (this.warnings.length > 0) {
      console.log(`\n⚠️  WARNINGS (${this.warnings.length}):`);
      for (const warning of this.warnings) {
        console.log(`   ${warning.file || "General"}: ${warning.message}`);
      }
    }

    // Report violations
    if (this.violations.length > 0) {
      console.log(
        `\n🚨 VIOLATIONS (${this.violations.length}) - COMMIT BLOCKED:`
      );
      console.log("");

      for (const violation of this.violations) {
        console.log(
          `❌ ${violation.gate.toUpperCase()}: ${violation.type.toUpperCase()}`
        );
        console.log(`   ${violation.message}`);
        if (violation.file) {
          console.log(`   File: ${violation.file}`);
        }
        if (violation.size) {
          console.log(`   Size: ${violation.size} LOC`);
        }
        if (violation.details) {
          console.log(
            `   Details: ${JSON.stringify(violation.details, null, 2)}`
          );
        }
        console.log("");
      }

      console.log("🔧 Fix these critical violations before committing.");
      console.log("📖 See docs/refactoring.md for crisis response plan.");
      process.exit(1);
    } else {
      console.log("\n✅ ALL QUALITY GATES PASSED");
      console.log("🎉 Commit allowed - quality maintained!");
      process.exit(0);
    }
  }
}

// Main execution
async function main() {
  if (CI_MODE) {
    console.log("🔧 Running in CI mode - strict enforcement");
  }

  if (FIX_MODE) {
    console.log("🔧 Running in fix mode - will attempt automatic fixes");
  }

  const runner = new QualityGateRunner();
  await runner.runAllGates();
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error("💥 Quality gates crashed:", error);
    process.exit(1);
  });
}

export default QualityGateRunner;
