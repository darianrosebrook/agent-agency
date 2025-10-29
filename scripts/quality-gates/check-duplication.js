#!/usr/bin/env node

/**
 * Quality Gate: Duplication Checker
 *
 * Detects duplicate filenames and struct names that violate CAWS quality standards.
 * Based on audit results from docs/audits/v3-codebase-audit-2025-10/02-duplication-report.md
 */

import fs from "fs";
import path from "path";
import {
  processViolations,
  getEnforcementLevel,
} from "./shared-exception-framework.js";

const V3_PATH = path.join(process.cwd(), "iterations", "v3");

// Thresholds for blocking commits - Focus on functional duplication
const DUPLICATE_STRUCT_THRESHOLD = 692; // Current count - block if it increases
const DUPLICATE_FUNCTION_THRESHOLD = 250; // Adjusted threshold for duplicate function names (excluding expected patterns)
const DUPLICATE_TRAIT_THRESHOLD = 100; // New threshold for duplicate trait names

// Expected architectural patterns that are normal to duplicate across modules
const EXPECTED_ARCHITECTURAL_PATTERNS = [
  // Core Rust patterns - these are expected to be duplicated
  "new",
  "config",
  "with_config",
  "update",
  "update_config",
  "reset",
  "stats",
  "get_stats",
  "validate",
  "build",
  "from_string",
  "as_str",
  "is_healthy",
  "len",
  "is_empty",
  "get",
  "set",
  "clone",
  "default",
  "from",
  "into",
  "try_from",
  "to_string",
  "fmt",
  "debug",
  "serialize",
  "deserialize",
  "hash",
  "eq",
  "partial_eq",
  "ord",
  "partial_ord",

  // Domain-specific patterns that are expected across modules
  "record_success",
  "record_failure",
  "calculate",
  "get_metrics",
  "summary",
  "get_summary",
  "register",
  "analyze",
  "train",
  "predict",
  "fit",
  "search",
  "metrics",
  "register_handler",
  "get_statistics",
  "merge",
  "observe",
  "count",
  "pool",
  "insert",
  "load",
  "encode",
  "decode",
  "success",
  "failure",
  "get_state",
  "reset_stats",
  "get_policy",
  "finalize",
  "digest",
  "data",
  "kind",
  "try_from_value",
  "from_extension",
  "state",
  "metadata",
  "size",
  "can_ingest",
  "validate_changeset",
  "add_stage",
  "create_default_slos",
  "new_with_memory",
  "empty",
  "error_count",
  "with_timeout",
  "manager",
  "cosine_similarity",
  "requires_human_intervention",
  "is_retryable",
  "category",
  "with_context",
];

// Rust convention files that are expected to be duplicated
const RUST_CONVENTION_FILES = ["lib.rs", "mod.rs", "main.rs", "Cargo.toml"];

// Files to check
const RUST_FILES = [];

// Collect all Rust files
function collectRustFiles(dir) {
  const files = fs.readdirSync(dir);

  for (const file of files) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);

    if (stat.isDirectory() && !file.startsWith(".") && file !== "target") {
      collectRustFiles(fullPath);
    } else if (file.endsWith(".rs")) {
      RUST_FILES.push(fullPath);
    }
  }
}

// Get problematic filename duplicates (excluding Rust conventions)
function getProblematicFilenameDuplicates() {
  // Ensure files are collected first
  if (RUST_FILES.length === 0) {
    collectRustFiles(V3_PATH);
  }

  const filenameCounts = {};

  for (const filePath of RUST_FILES) {
    const filename = path.basename(filePath);
    filenameCounts[filename] = (filenameCounts[filename] || 0) + 1;
  }

  const duplicates = {};
  for (const [filename, count] of Object.entries(filenameCounts)) {
    // Only flag non-convention files that are duplicated
    if (count > 1 && !RUST_CONVENTION_FILES.includes(filename)) {
      duplicates[filename] = count;
    }
  }

  return duplicates;
}

// Get struct/impl name duplicates
function getDuplicateStructs() {
  // Ensure files are collected first
  if (RUST_FILES.length === 0) {
    collectRustFiles(V3_PATH);
  }

  const structCounts = {};

  for (const filePath of RUST_FILES) {
    try {
      const content = fs.readFileSync(filePath, "utf8");
      const lines = content.split("\n");

      for (const line of lines) {
        const trimmed = line.trim();

        // Match pub struct Name or pub impl Name patterns
        const structMatch = trimmed.match(/^pub (struct|impl) (\w+)/);
        if (structMatch) {
          const name = structMatch[2];
          structCounts[name] = (structCounts[name] || 0) + 1;
        }

        // Also check trait names
        const traitMatch = trimmed.match(/^pub trait (\w+)/);
        if (traitMatch) {
          const name = traitMatch[1];
          structCounts[name] = (structCounts[name] || 0) + 1;
        }
      }
    } catch (error) {
      // Skip files that can't be read
      console.warn(`⚠️  Could not read ${filePath}: ${error.message}`);
    }
  }

  const duplicates = {};
  for (const [name, count] of Object.entries(structCounts)) {
    if (count > 1) {
      duplicates[name] = count;
    }
  }

  return duplicates;
}

// Get function name duplicates (new focus on functional duplication)
function getDuplicateFunctions() {
  if (RUST_FILES.length === 0) {
    collectRustFiles(V3_PATH);
  }

  const functionCounts = {};

  for (const filePath of RUST_FILES) {
    try {
      const content = fs.readFileSync(filePath, "utf8");
      const lines = content.split("\n");

      for (const line of lines) {
        const trimmed = line.trim();

        // Match pub fn function_name patterns
        const functionMatch = trimmed.match(/^pub fn (\w+)/);
        if (functionMatch) {
          const name = functionMatch[1];
          functionCounts[name] = (functionCounts[name] || 0) + 1;
        }
      }
    } catch (error) {
      console.warn(`⚠️  Could not read ${filePath}: ${error.message}`);
    }
  }

  const duplicates = {};
  for (const [name, count] of Object.entries(functionCounts)) {
    // Only count as problematic duplication if it's not an expected architectural pattern
    if (count > 1 && !EXPECTED_ARCHITECTURAL_PATTERNS.includes(name)) {
      duplicates[name] = count;
    }
  }

  return duplicates;
}

// Get trait name duplicates separately
function getDuplicateTraits() {
  if (RUST_FILES.length === 0) {
    collectRustFiles(V3_PATH);
  }

  const traitCounts = {};

  for (const filePath of RUST_FILES) {
    try {
      const content = fs.readFileSync(filePath, "utf8");
      const lines = content.split("\n");

      for (const line of lines) {
        const trimmed = line.trim();

        // Match pub trait Name patterns
        const traitMatch = trimmed.match(/^pub trait (\w+)/);
        if (traitMatch) {
          const name = traitMatch[1];
          traitCounts[name] = (traitCounts[name] || 0) + 1;
        }
      }
    } catch (error) {
      console.warn(`⚠️  Could not read ${filePath}: ${error.message}`);
    }
  }

  const duplicates = {};
  for (const [name, count] of Object.entries(traitCounts)) {
    if (count > 1) {
      duplicates[name] = count;
    }
  }

  return duplicates;
}

// Check for regression (increase in functional duplication)
function checkDuplicationRegression(context = "commit") {
  const rawViolations = [];

  // Check problematic filename duplicates (excluding Rust conventions)
  const filenameDuplicates = getProblematicFilenameDuplicates();
  const filenameCount = Object.keys(filenameDuplicates).length;

  // Only flag if there are excessive non-convention duplicates
  // Note: Having multiple manager.rs, types.rs, config.rs files across crates is normal architecture
  if (filenameCount > 100) {
    // Very high threshold - architectural duplication is expected and good
    rawViolations.push({
      type: "problematic_filename_duplication",
      file: "multiple",
      relativePath: "multiple",
      issue: `Excessive duplicate filenames (excluding Rust conventions): ${filenameCount}`,
      details: filenameDuplicates,
      threshold: 100,
      current: filenameCount,
    });
  }

  // Check struct duplicates (CRITICAL - this is the real problem)
  const structDuplicates = getDuplicateStructs();
  const structCount = Object.keys(structDuplicates).length;

  if (structCount > DUPLICATE_STRUCT_THRESHOLD) {
    rawViolations.push({
      type: "struct_duplication_regression",
      file: "multiple",
      relativePath: "multiple",
      issue: `Duplicate struct names increased from ${DUPLICATE_STRUCT_THRESHOLD} to ${structCount}`,
      details: structDuplicates,
      threshold: DUPLICATE_STRUCT_THRESHOLD,
      current: structCount,
    });
  }

  // Check function duplicates (NEW - functional duplication focus)
  const functionDuplicates = getDuplicateFunctions();
  const functionCount = Object.keys(functionDuplicates).length;

  if (functionCount > DUPLICATE_FUNCTION_THRESHOLD) {
    rawViolations.push({
      type: "function_duplication_regression",
      file: "multiple",
      relativePath: "multiple",
      issue: `Problematic duplicate function names (excluding expected patterns): ${functionCount}`,
      details: functionDuplicates,
      threshold: DUPLICATE_FUNCTION_THRESHOLD,
      current: functionCount,
    });
  }

  // Check trait duplicates (NEW - functional duplication focus)
  const traitDuplicates = getDuplicateTraits();
  const traitCount = Object.keys(traitDuplicates).length;

  if (traitCount > DUPLICATE_TRAIT_THRESHOLD) {
    rawViolations.push({
      type: "trait_duplication_regression",
      file: "multiple",
      relativePath: "multiple",
      issue: `Duplicate trait names increased from ${DUPLICATE_TRAIT_THRESHOLD} to ${traitCount}`,
      details: traitDuplicates,
      threshold: DUPLICATE_TRAIT_THRESHOLD,
      current: traitCount,
    });
  }

  // Process violations with exception handling
  const result = processViolations("duplication", rawViolations, context);

  return {
    violations: result.violations,
    warnings: result.warnings,
    enforcementLevel: result.enforcementLevel,
  };
}

function main() {
  console.log("🔍 Checking for functional duplication violations...");

  // Collect files
  collectRustFiles(V3_PATH);
  console.log(`📁 Found ${RUST_FILES.length} Rust files to check`);

  // Check for regression
  const context = process.env.CAWS_ENFORCEMENT_CONTEXT || "commit";
  const results = checkDuplicationRegression(context);

  // Get current duplication stats for reporting
  const problematicFilenameDuplicates = getProblematicFilenameDuplicates();
  const structDuplicates = getDuplicateStructs();
  const functionDuplicates = getDuplicateFunctions();
  const traitDuplicates = getDuplicateTraits();

  console.log(`📊 Current functional duplication stats:`);
  console.log(
    `   - ${
      Object.keys(problematicFilenameDuplicates).length
    } problematic duplicate filenames (excluding Rust conventions)`
  );
  console.log(
    `   - ${
      Object.keys(structDuplicates).length
    } duplicate struct names (CRITICAL)`
  );
  console.log(
    `   - ${Object.keys(functionDuplicates).length} duplicate function names`
  );
  console.log(
    `   - ${Object.keys(traitDuplicates).length} duplicate trait names`
  );

  // Show Rust convention duplicates for context (not violations)
  const allFilenameDuplicates = {};
  for (const filePath of RUST_FILES) {
    const filename = path.basename(filePath);
    allFilenameDuplicates[filename] =
      (allFilenameDuplicates[filename] || 0) + 1;
  }
  const rustConventionDuplicates = {};
  for (const [filename, count] of Object.entries(allFilenameDuplicates)) {
    if (count > 1 && RUST_CONVENTION_FILES.includes(filename)) {
      rustConventionDuplicates[filename] = count;
    }
  }

  if (Object.keys(rustConventionDuplicates).length > 0) {
    console.log(
      `   - ${
        Object.keys(rustConventionDuplicates).length
      } Rust convention duplicates (expected):`
    );
    for (const [filename, count] of Object.entries(rustConventionDuplicates)) {
      console.log(`     - ${filename}: ${count} occurrences (Rust convention)`);
    }
  }

  // Report warnings (approved exceptions)
  if (results.warnings.length > 0) {
    console.log(`   ℹ️  ${results.warnings.length} approved exceptions in use`);
    for (const warning of results.warnings) {
      console.log(
        `      📋 ${warning.violation.file}: ${warning.exception.reason}`
      );
    }
  }

  if (results.violations.length === 0) {
    console.log("✅ No functional duplication regression detected");
    process.exit(0);
  } else {
    console.log(`🚨 Functional duplication regression detected!`);
    console.log(
      `   🔧 Enforcement level: ${results.enforcementLevel.toUpperCase()}`
    );

    for (const violation of results.violations) {
      console.log("");
      console.log(`❌ ${violation.type.toUpperCase().replace(/_/g, " ")}`);
      console.log(`   Issue: ${violation.issue}`);
      console.log(
        `   Threshold: ${violation.threshold}, Current: ${violation.current}`
      );

      if (violation.details && Object.keys(violation.details).length <= 10) {
        console.log("   Details:");
        for (const [name, count] of Object.entries(violation.details)) {
          console.log(`     - ${name}: ${count} occurrences`);
        }
      } else if (violation.details) {
        console.log(
          `   Details: ${
            Object.keys(violation.details).length
          } duplicates (too many to list)`
        );
      }
    }

    console.log("");
    console.log(
      "🔧 Functional duplication must not increase. Focus on consolidating duplicate business logic."
    );
    console.log("💡 See: docs/refactoring.md for consolidation strategies");

    if (results.enforcementLevel === "warning") {
      console.log("⚠️  Warning mode - commit allowed but review required");
      process.exit(0);
    } else {
      console.log(`🚫 ${results.enforcementLevel} mode - action blocked`);
      process.exit(1);
    }
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export {
  getProblematicFilenameDuplicates,
  getDuplicateStructs,
  getDuplicateFunctions,
  getDuplicateTraits,
  checkDuplicationRegression,
};
