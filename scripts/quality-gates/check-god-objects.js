#!/usr/bin/env node

/**
 * Quality Gate: God Object Detector
 *
 * Detects files that are too large (god objects) and blocks commits that would create or worsen them.
 * Based on audit results from docs/audits/v3-codebase-audit-2025-10/03-god-objects-analysis.md
 */

import fs from "fs";
import path from "path";
import { execSync } from "child_process";
import {
  processViolations,
  getEnforcementLevel,
} from "./shared-exception-framework.js";

const V3_PATH = path.join(process.cwd(), "iterations", "v3");

// Size thresholds (lines of code)
const GOD_OBJECT_THRESHOLDS = {
  severe: 3000, // Block immediately - crisis level
  critical: 2000, // Block in CI/CD
  warning: 1750, // No warnings for files under 1750 LOC (adjusted threshold)
  target: 1500, // Long-term target - files under 1500 LOC are ideal
};

// Current known god objects (from audit)
const KNOWN_GOD_OBJECTS = [
  "council/src/intelligent_edge_case_testing.rs", // 6,348 LOC
  "observability/src/analytics_dashboard.rs", // 3,537 LOC
  "claim-extraction/src/evidence.rs", // 3,482 LOC
  "database/src/client.rs", // 3,457 LOC
  // ... other god objects from audit
];

// Files to check
const RUST_FILES = [];

// Collect staged Rust files only
function collectStagedRustFiles() {
  try {
    // Get staged files
    const stagedFiles = execSync("git diff --cached --name-only", {
      encoding: "utf8",
    })
      .trim()
      .split("\n")
      .filter((file) => file.trim() !== "");

    // Filter for Rust files
    const rustFiles = stagedFiles.filter((file) => file.endsWith(".rs"));

    // Convert to absolute paths
    for (const file of rustFiles) {
      const fullPath = path.resolve(file);
      if (fs.existsSync(fullPath)) {
        RUST_FILES.push(fullPath);
      }
    }

    console.log(`📁 Found ${rustFiles.length} staged Rust files to check`);
  } catch (error) {
    console.warn(`⚠️  Could not get staged files: ${error.message}`);
    // Fallback to checking all files if git command fails
    collectAllRustFiles();
  }
}

// Fallback: Collect all Rust files (original behavior)
function collectAllRustFiles() {
  const V3_PATH = path.join(process.cwd(), "iterations", "v3");
  collectRustFiles(V3_PATH);
}

// Collect all Rust files from directory (original function)
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

// Get file sizes
function getFileSizes() {
  const sizes = {};

  for (const filePath of RUST_FILES) {
    try {
      const content = fs.readFileSync(filePath, "utf8");
      const lines = content.split("\n").length;
      sizes[filePath] = lines;
    } catch (error) {
      console.warn(`⚠️  Could not read ${filePath}: ${error.message}`);
    }
  }

  return sizes;
}

// Check for god objects
function checkGodObjects(context = "commit") {
  // Ensure files are collected first
  if (RUST_FILES.length === 0) {
    collectStagedRustFiles();
  }

  const rawViolations = [];
  const fileSizes = getFileSizes();

  for (const [filePath, size] of Object.entries(fileSizes)) {
    const relativePath = path.relative(V3_PATH, filePath);

    if (size >= GOD_OBJECT_THRESHOLDS.severe) {
      rawViolations.push({
        type: "severe_god_object",
        file: filePath,
        relativePath,
        size,
        threshold: GOD_OBJECT_THRESHOLDS.severe,
        message: `SEVERE god object: ${size} LOC exceeds ${GOD_OBJECT_THRESHOLDS.severe} LOC limit`,
      });
    } else if (size >= GOD_OBJECT_THRESHOLDS.critical) {
      rawViolations.push({
        type: "critical_god_object",
        file: filePath,
        relativePath,
        size,
        threshold: GOD_OBJECT_THRESHOLDS.critical,
        message: `CRITICAL god object: ${size} LOC exceeds ${GOD_OBJECT_THRESHOLDS.critical} LOC limit`,
      });
    } else if (size >= GOD_OBJECT_THRESHOLDS.warning) {
      rawViolations.push({
        type: "warning_god_object",
        file: filePath,
        relativePath,
        size,
        threshold: GOD_OBJECT_THRESHOLDS.warning,
        message: `WARNING: ${size} LOC approaches god object territory (${GOD_OBJECT_THRESHOLDS.warning}+ LOC)`,
      });
    }
  }

  // Process violations with exception handling
  const result = processViolations("god_objects", rawViolations, context);

  return {
    violations: result.violations,
    warnings: result.warnings,
    enforcementLevel: result.enforcementLevel,
  };
}

// Check for new god objects (regression)
function checkGodObjectRegression() {
  // Ensure files are collected first
  if (RUST_FILES.length === 0) {
    collectStagedRustFiles();
  }

  const violations = [];
  const fileSizes = getFileSizes();

  // Check if any new files exceed critical threshold
  for (const [filePath, size] of Object.entries(fileSizes)) {
    if (size >= GOD_OBJECT_THRESHOLDS.critical) {
      const relativePath = path.relative(V3_PATH, filePath);
      const isKnown = KNOWN_GOD_OBJECTS.some((known) =>
        relativePath.includes(known)
      );

      if (!isKnown) {
        violations.push({
          type: "new_god_object",
          file: filePath,
          relativePath,
          size,
          threshold: GOD_OBJECT_THRESHOLDS.critical,
          severity: "block",
          message: `NEW god object detected: ${relativePath} (${size} LOC) exceeds ${GOD_OBJECT_THRESHOLDS.critical} LOC limit`,
        });
      }
    }
  }

  return violations;
}

function main() {
  console.log("🔍 Checking for god objects in staged files...");

  // Collect staged files
  collectStagedRustFiles();

  // Run checks
  const godObjectViolations = checkGodObjects();
  const regressionViolations = checkGodObjectRegression();
  const allViolations = [...godObjectViolations, ...regressionViolations];

  // Separate blocking vs warning violations
  const blockingViolations = allViolations.filter(
    (v) => v.severity === "block"
  );
  const warningViolations = allViolations.filter((v) => v.severity === "warn");

  // Report stats
  const fileSizes = getFileSizes();
  const severeCount = Object.values(fileSizes).filter(
    (size) => size >= GOD_OBJECT_THRESHOLDS.severe
  ).length;
  const criticalCount = Object.values(fileSizes).filter(
    (size) => size >= GOD_OBJECT_THRESHOLDS.critical
  ).length;
  const warningCount = Object.values(fileSizes).filter(
    (size) => size >= GOD_OBJECT_THRESHOLDS.warning
  ).length;

  console.log(`📊 God object stats:`);
  console.log(
    `   - ${severeCount} files > ${GOD_OBJECT_THRESHOLDS.severe} LOC (severe)`
  );
  console.log(
    `   - ${criticalCount} files > ${GOD_OBJECT_THRESHOLDS.critical} LOC (critical)`
  );
  console.log(
    `   - ${warningCount} files > ${GOD_OBJECT_THRESHOLDS.warning} LOC (warning)`
  );

  // Report warnings (non-blocking)
  if (warningViolations.length > 0) {
    console.log("");
    console.log("⚠️  WARNINGS (non-blocking):");
    for (const violation of warningViolations) {
      console.log(`   ${violation.relativePath}: ${violation.message}`);
    }
  }

  // Report blocking violations
  if (blockingViolations.length > 0) {
    console.log("");
    console.log(`🚨 BLOCKING VIOLATIONS (${blockingViolations.length}):`);
    console.log("");

    for (const violation of blockingViolations) {
      console.log(`❌ ${violation.type.toUpperCase().replace(/_/g, " ")}`);
      console.log(`   File: ${violation.relativePath}`);
      console.log(`   Size: ${violation.size} LOC`);
      console.log(`   Limit: ${violation.threshold} LOC`);
      console.log(`   Issue: ${violation.message}`);
      console.log("");
    }

    console.log("🔧 Decompose these god objects before committing.");
    console.log(
      "💡 See: docs/audits/v3-codebase-audit-2025-10/03-god-objects-analysis.md"
    );
    process.exit(1);
  } else {
    console.log("✅ No blocking god object violations");
    if (warningViolations.length === 0) {
      console.log("✅ No god object warnings");
    }
    process.exit(0);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export {
  checkGodObjects,
  checkGodObjectRegression,
  getFileSizes,
  GOD_OBJECT_THRESHOLDS,
};
