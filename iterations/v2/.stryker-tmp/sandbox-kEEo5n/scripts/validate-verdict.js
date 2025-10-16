#!/usr/bin/env node
// @ts-nocheck

/**
 * CAWS Verdict Validator CLI
 *
 * Validates CAWS verdicts for constitutional compliance before ingestion.
 *
 * Usage:
 *   node scripts/validate-verdict.js <verdict-file> [spec-file]
 *   node scripts/validate-verdict.js --help
 *
 * @author @darianrosebrook
 */

const fs = require("fs");
const path = require("path");

// Import the validator (this would be compiled to JS in production)
let VerdictValidator;
try {
  VerdictValidator =
    require("../dist/tools/caws/verdict-validator").VerdictValidator;
} catch (e) {
  console.error(
    "VerdictValidator not found. Please compile TypeScript first: npm run build"
  );
  process.exit(1);
}

async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0 || args.includes("--help") || args.includes("-h")) {
    showHelp();
    return;
  }

  const verdictFile = args[0];
  const specFile = args[1];

  if (!verdictFile) {
    console.error("Error: Verdict file is required");
    showHelp();
    process.exit(1);
  }

  if (!fs.existsSync(verdictFile)) {
    console.error(`Error: Verdict file not found: ${verdictFile}`);
    process.exit(1);
  }

  if (specFile && !fs.existsSync(specFile)) {
    console.error(`Error: Spec file not found: ${specFile}`);
    process.exit(1);
  }

  try {
    const validator = new VerdictValidator();
    const result = await validator.validateVerdictFile(verdictFile, specFile);

    console.log("🔍 CAWS Verdict Validation Results");
    console.log("=====================================");
    console.log(`Verdict File: ${verdictFile}`);
    console.log(`Spec File: ${specFile || "Not provided"}`);
    console.log(`Validation Time: ${result.metadata.validationTime}ms`);
    console.log(`Result: ${result.valid ? "✅ VALID" : "❌ INVALID"}`);
    console.log("");

    if (result.errors.length > 0) {
      console.log("🚫 Errors:");
      result.errors.forEach((error, index) => {
        console.log(`  ${index + 1}. ${error.code}: ${error.message}`);
        if (error.field) {
          console.log(`     Field: ${error.field}`);
        }
        if (error.details) {
          console.log(
            `     Details: ${JSON.stringify(error.details, null, 2)}`
          );
        }
      });
      console.log("");
    }

    if (result.warnings.length > 0) {
      console.log("⚠️  Warnings:");
      result.warnings.forEach((warning, index) => {
        console.log(`  ${index + 1}. ${warning.code}: ${warning.message}`);
        if (warning.field) {
          console.log(`     Field: ${warning.field}`);
        }
        if (warning.suggestion) {
          console.log(`     Suggestion: ${warning.suggestion}`);
        }
      });
      console.log("");
    }

    if (result.valid) {
      console.log(
        "🎉 Verdict is constitutionally compliant and ready for ingestion!"
      );
      console.log("");
      console.log("Constitutional Context:");
      console.log(
        `  Spec Hash: ${result.verdict.constitutionalContext.specHash}`
      );
      console.log(
        `  Clause Citations: ${result.verdict.constitutionalContext.clauseCitations.length}`
      );
      console.log(`  Governance Metrics:`);
      console.log(
        `    Waiver Rate: ${(
          result.verdict.constitutionalContext.governanceMetrics.waiverRate *
          100
        ).toFixed(1)}%`
      );
      console.log(
        `    Gate Integrity: ${(
          result.verdict.constitutionalContext.governanceMetrics.gateIntegrity *
          100
        ).toFixed(1)}%`
      );
      console.log(
        `    Budget Compliance: ${(
          result.verdict.constitutionalContext.governanceMetrics
            .budgetCompliance * 100
        ).toFixed(1)}%`
      );
      console.log(
        `    Evidence Completeness: ${(
          result.verdict.constitutionalContext.governanceMetrics
            .evidenceCompleteness * 100
        ).toFixed(1)}%`
      );
    } else {
      console.log(
        "❌ Verdict validation failed. Please address the errors above."
      );
      process.exit(1);
    }
  } catch (error) {
    console.error("Validation failed:", error.message);
    process.exit(1);
  }
}

function showHelp() {
  console.log(`
CAWS Verdict Validator CLI

Validates CAWS verdicts for constitutional compliance before benchmark data ingestion.

USAGE:
  node scripts/validate-verdict.js <verdict-file> [spec-file]

ARGUMENTS:
  verdict-file    Path to the CAWS verdict YAML/JSON file
  spec-file       Optional path to the .caws/working-spec.yaml for hash validation

EXAMPLES:
  # Validate verdict with spec hash check
  node scripts/validate-verdict.js results/verdict-001.yaml .caws/working-spec.yaml

  # Validate verdict without spec file
  node scripts/validate-verdict.js results/verdict-001.yaml

  # Show this help
  node scripts/validate-verdict.js --help

VALIDATION CHECKS:
  ✅ Schema compliance (required fields, structure)
  ✅ Cryptographic signature verification (ed25519)
  ✅ Spec hash integrity (SHA-256 of working-spec.yaml)
  ✅ Clause citations (CAWS:Section.Subsection format)
  ✅ Governance metrics (waiver rate, gate integrity, etc.)
  ✅ Waiver lifecycle (expiry, approval status)
  ✅ Provenance chain (immutable hash chain)
  ✅ Business logic rules (pass/fail consistency)

EXIT CODES:
  0  Verdict is valid and constitutionally compliant
  1  Verdict validation failed or file errors

For more information, see: docs/1-core-orchestration/theory.md
`);
}

if (require.main === module) {
  main().catch((error) => {
    console.error("Unexpected error:", error);
    process.exit(1);
  });
}

module.exports = { main };
