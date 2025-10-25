#!/usr/bin/env node

/**
 * Quality Gate: Duplication Checker
 *
 * Detects duplicate filenames and struct names that violate CAWS quality standards.
 * Based on audit results from docs/audits/v3-codebase-audit-2025-10/02-duplication-report.md
 */

import fs from "fs";
import path from "path";

const V3_PATH = path.join(process.cwd(), "iterations/v3");

// Thresholds for blocking commits
const DUPLICATE_FILENAME_THRESHOLD = 69; // Current count - block if it increases
const DUPLICATE_STRUCT_THRESHOLD = 658; // Current count - block if it increases

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

// Get filename duplicates
function getDuplicateFilenames() {
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
    if (count > 1) {
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

// Check for regression (increase in duplication)
function checkDuplicationRegression() {
  const violations = [];

  // Check filename duplicates
  const filenameDuplicates = getDuplicateFilenames();
  const filenameCount = Object.keys(filenameDuplicates).length;

  if (filenameCount > DUPLICATE_FILENAME_THRESHOLD) {
    violations.push({
      type: "filename_duplication_regression",
      issue: `Duplicate filenames increased from ${DUPLICATE_FILENAME_THRESHOLD} to ${filenameCount}`,
      details: filenameDuplicates,
      threshold: DUPLICATE_FILENAME_THRESHOLD,
      current: filenameCount,
    });
  }

  // Check struct duplicates
  const structDuplicates = getDuplicateStructs();
  const structCount = Object.keys(structDuplicates).length;

  if (structCount > DUPLICATE_STRUCT_THRESHOLD) {
    violations.push({
      type: "struct_duplication_regression",
      issue: `Duplicate struct/trait names increased from ${DUPLICATE_STRUCT_THRESHOLD} to ${structCount}`,
      details: structDuplicates,
      threshold: DUPLICATE_STRUCT_THRESHOLD,
      current: structCount,
    });
  }

  return violations;
}

function main() {
  console.log("🔍 Checking for duplication violations...");

  // Collect files
  collectRustFiles(V3_PATH);
  console.log(`📁 Found ${RUST_FILES.length} Rust files to check`);

  // Check for regression
  const violations = checkDuplicationRegression();

  // Also get current duplication stats for reporting
  const filenameDuplicates = getDuplicateFilenames();
  const structDuplicates = getDuplicateStructs();

  console.log(`📊 Current duplication stats:`);
  console.log(
    `   - ${Object.keys(filenameDuplicates).length} duplicate filenames`
  );
  console.log(
    `   - ${Object.keys(structDuplicates).length} duplicate struct/trait names`
  );

  if (violations.length === 0) {
    console.log("✅ No duplication regression detected");
    process.exit(0);
  } else {
    console.log(`🚨 Duplication regression detected!`);

    for (const violation of violations) {
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
      "🔧 Duplication must not increase. Fix these issues before committing."
    );
    console.log(
      "💡 See: docs/audits/v3-codebase-audit-2025-10/02-duplication-report.md"
    );
    process.exit(1);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export {
  getDuplicateFilenames,
  getDuplicateStructs,
  checkDuplicationRegression,
};
