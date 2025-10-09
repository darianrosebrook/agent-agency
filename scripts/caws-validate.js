#!/usr/bin/env node

/**
 * Shared CAWS Validation Script
 *
 * Validates CAWS compliance across all iterations in the mono-repo.
 * Ensures consistent quality standards and risk management.
 *
 * @author @darianrosebrook
 */

import { existsSync, readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT_DIR = join(__dirname, "..");

const ITERATIONS_DIR = join(ROOT_DIR, "iterations");
const AVAILABLE_ITERATIONS = ["poc", "v1"];

class CAWSValidator {
  constructor() {
    this.iterations = AVAILABLE_ITERATIONS;
  }

  /**
   * Validate CAWS compliance for specific iteration
   */
  validateIteration(iterationName) {
    console.log(
      `🔍 Validating CAWS compliance for iteration: ${iterationName}`
    );

    const iterationPath = join(ITERATIONS_DIR, iterationName);
    const workingSpecPath = join(iterationPath, ".caws", "working-spec.yaml");
    const provenancePath = join(iterationPath, ".caws", "provenance.json");

    let score = 0;
    const maxScore = 100;
    const issues = [];

    // Check working spec
    if (!existsSync(workingSpecPath)) {
      issues.push("❌ Missing .caws/working-spec.yaml");
    } else {
      score += 20;
      console.log("✅ Working spec present");

      try {
        const specContent = readFileSync(workingSpecPath, "utf8");
        // Basic validation - check for required fields
        const requiredFields = ["id:", "title:", "risk_tier:", "scope:"];
        const missingFields = requiredFields.filter(
          (field) => !specContent.includes(field)
        );

        if (missingFields.length > 0) {
          issues.push(
            `⚠️ Working spec missing fields: ${missingFields.join(", ")}`
          );
          score -= 5;
        } else {
          score += 10;
          console.log("✅ Working spec has required fields");
        }
      } catch (error) {
        issues.push("❌ Error reading working spec");
        score -= 10;
      }
    }

    // Check provenance
    if (!existsSync(provenancePath)) {
      issues.push("❌ Missing .caws/provenance.json");
    } else {
      score += 20;
      console.log("✅ Provenance tracking present");

      try {
        const provenance = JSON.parse(readFileSync(provenancePath, "utf8"));
        if (!provenance.chain || provenance.chain.length === 0) {
          issues.push("⚠️ Provenance chain is empty");
          score -= 5;
        } else {
          score += 10;
          console.log("✅ Provenance chain has entries");
        }
      } catch (error) {
        issues.push("❌ Error reading provenance");
        score -= 10;
      }
    }

    // Check package.json CAWS scripts
    const packagePath = join(iterationPath, "package.json");
    if (existsSync(packagePath)) {
      try {
        const pkg = JSON.parse(readFileSync(packagePath, "utf8"));
        const scripts = pkg.scripts || {};

        const requiredScripts = ["validate"];
        const missingScripts = requiredScripts.filter(
          (script) => !scripts[script]
        );

        if (missingScripts.length > 0) {
          issues.push(`⚠️ Missing CAWS scripts: ${missingScripts.join(", ")}`);
          score -= 5;
        } else {
          score += 10;
          console.log("✅ CAWS validation scripts present");
        }
      } catch (error) {
        issues.push("❌ Error reading package.json");
        score -= 5;
      }
    }

    // Check test coverage requirements
    const jestConfigPath = join(iterationPath, "package.json");
    if (existsSync(jestConfigPath)) {
      try {
        const pkg = JSON.parse(readFileSync(jestConfigPath, "utf8"));
        const jestConfig = pkg.jest || {};

        if (jestConfig.coverageThreshold) {
          const thresholds = jestConfig.coverageThreshold.global || {};
          const minCoverage = Math.min(
            thresholds.branches || 0,
            thresholds.functions || 0,
            thresholds.lines || 0,
            thresholds.statements || 0
          );

          if (minCoverage >= 70) {
            score += 15;
            console.log("✅ High test coverage requirements (>=70%)");
          } else if (minCoverage >= 50) {
            score += 10;
            console.log("✅ Adequate test coverage requirements (>=50%)");
          } else {
            issues.push("⚠️ Low test coverage requirements (<50%)");
            score -= 5;
          }
        } else {
          issues.push("⚠️ No test coverage thresholds defined");
          score -= 5;
        }
      } catch (error) {
        issues.push("❌ Error checking test coverage");
        score -= 5;
      }
    }

    // Risk tier specific checks
    const riskTier = this.getRiskTier(iterationName);
    if (riskTier >= 2) {
      // Higher risk tier requirements
      const mutationTest = this.checkMutationTesting(iterationPath);
      if (mutationTest) {
        score += 10;
        console.log("✅ Mutation testing configured (Risk Tier 2+)");
      } else {
        issues.push(
          "⚠️ Mutation testing not configured (required for Risk Tier 2+)"
        );
        score -= 10;
      }
    }

    // Ensure score is within bounds
    score = Math.max(0, Math.min(maxScore, score));

    console.log(
      `📊 CAWS Score: ${score}/${maxScore} (${Math.round(
        (score / maxScore) * 100
      )}%)`
    );

    if (issues.length > 0) {
      console.log("\n⚠️ Issues found:");
      issues.forEach((issue) => console.log(`  ${issue}`));
    }

    return { score, maxScore, issues, compliant: score >= 70 };
  }

  /**
   * Validate all iterations
   */
  validateAllIterations() {
    console.log("🔍 CAWS Compliance Validation Across All Iterations\n");

    const results = {};
    let totalScore = 0;
    let totalCompliant = 0;

    this.iterations.forEach((iteration) => {
      console.log(`\n--- ${iteration.toUpperCase()} ITERATION ---`);
      const result = this.validateIteration(iteration);
      results[iteration] = result;

      totalScore += result.score;
      if (result.compliant) totalCompliant++;

      console.log("");
    });

    const overallScore = totalScore / this.iterations.length;
    const overallCompliant = totalCompliant === this.iterations.length;

    console.log("=".repeat(50));
    console.log(`📊 OVERALL CAWS COMPLIANCE SUMMARY:`);
    console.log(`   Average Score: ${overallScore.toFixed(1)}/100`);
    console.log(
      `   Compliant Iterations: ${totalCompliant}/${this.iterations.length}`
    );
    console.log(
      `   Overall Status: ${
        overallCompliant ? "✅ COMPLIANT" : "❌ NON-COMPLIANT"
      }`
    );
    console.log("=".repeat(50));

    return { results, overallScore, overallCompliant };
  }

  /**
   * Get risk tier for iteration
   */
  getRiskTier(iterationName) {
    // POC is typically lower risk, main is higher risk
    return iterationName === "main" ? 2 : 1;
  }

  /**
   * Check if mutation testing is configured
   */
  checkMutationTesting(iterationPath) {
    const packagePath = join(iterationPath, "package.json");
    if (!existsSync(packagePath)) return false;

    try {
      const pkg = JSON.parse(readFileSync(packagePath, "utf8"));
      const scripts = pkg.scripts || {};
      const deps = pkg.devDependencies || {};

      // Check for mutation testing scripts and dependencies
      const hasMutationScript = Object.values(scripts).some(
        (script) => script.includes("stryker") || script.includes("mutation")
      );

      const hasMutationDep = Object.keys(deps).some(
        (dep) => dep.includes("stryker") || dep.includes("mutation")
      );

      return hasMutationScript && hasMutationDep;
    } catch {
      return false;
    }
  }
}

// Main execution
function main() {
  const args = process.argv.slice(2);
  const validator = new CAWSValidator();

  if (args.length === 0 || args[0] === "all") {
    const result = validator.validateAllIterations();
    process.exit(result.overallCompliant ? 0 : 1);
  } else {
    const iteration = args[0];
    if (!AVAILABLE_ITERATIONS.includes(iteration)) {
      console.error(`❌ Unknown iteration: ${iteration}`);
      console.log(`Available iterations: ${AVAILABLE_ITERATIONS.join(", ")}`);
      process.exit(1);
    }

    const result = validator.validateIteration(iteration);
    process.exit(result.compliant ? 0 : 1);
  }
}

main();
