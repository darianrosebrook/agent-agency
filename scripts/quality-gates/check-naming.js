#!/usr/bin/env node

/**
 * Quality Gate: Naming Convention Checker
 *
 * Enforces CAWS naming conventions and blocks banned modifiers.
 * Based on rules in docs/audits/v3-codebase-audit-2025-10/06-naming-violations.md
 */

import fs from "fs";
import path from "path";

const V3_PATH = path.join(process.cwd(), "iterations/v3");

// Banned modifiers that indicate duplicate/forked files
const BANNED_MODIFIERS = [
  "enhanced",
  "unified",
  "better",
  "new",
  "next",
  "final",
  "copy",
  "revamp",
  "improved",
];

// Case-insensitive regex for banned modifiers
const BANNED_REGEX = new RegExp(`\\b(${BANNED_MODIFIERS.join("|")})\\b`, "i");

// Files to check (Rust source files)
const RUST_FILES = [];

// Collect all Rust files
function collectRustFiles(dir) {
  const files = fs.readdirSync(dir);

  for (const file of files) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);

    if (stat.isDirectory() && !file.startsWith(".") && file !== "target") {
      collectRustFiles(fullPath);
    } else if (file.endsWith(".rs") && file !== "lib.rs" && file !== "mod.rs") {
      RUST_FILES.push(fullPath);
    }
  }
}

// Check naming violations
function checkNamingViolations() {
  const violations = [];

  for (const filePath of RUST_FILES) {
    const fileName = path.basename(filePath, ".rs");

    // Check for banned modifiers in filenames
    if (BANNED_REGEX.test(fileName)) {
      violations.push({
        type: "filename_banned_modifier",
        file: filePath,
        issue: `Filename contains banned modifier: ${
          fileName.match(BANNED_REGEX)[0]
        }`,
        rule: 'No duplicate "enhanced/unified/new/final" modules',
      });
    }
  }

  return violations;
}

// Check struct/impl naming consistency
function checkStructNaming() {
  const violations = [];

  for (const filePath of RUST_FILES) {
    try {
      const content = fs.readFileSync(filePath, "utf8");

      // Check for inconsistent naming patterns
      // This is a simplified check - a full AST parser would be better
      const lines = content.split("\n");

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();

        // Check for pub struct/impl with banned modifiers
        if (
          (line.startsWith("pub struct ") || line.startsWith("pub impl ")) &&
          BANNED_REGEX.test(line)
        ) {
          const match = line.match(BANNED_REGEX);
          violations.push({
            type: "struct_banned_modifier",
            file: filePath,
            line: i + 1,
            issue: `Struct/impl name contains banned modifier: ${match[0]}`,
            rule: 'No duplicate "enhanced/unified/new/final" modules',
          });
        }
      }
    } catch (error) {
      violations.push({
        type: "file_read_error",
        file: filePath,
        issue: `Could not read file: ${error.message}`,
      });
    }
  }

  return violations;
}

function main() {
  console.log("🔍 Checking naming conventions...");

  // Collect files
  collectRustFiles(V3_PATH);
  console.log(`📁 Found ${RUST_FILES.length} Rust files to check`);

  // Run checks
  const filenameViolations = checkNamingViolations();
  const structViolations = checkStructNaming();
  const allViolations = [...filenameViolations, ...structViolations];

  // Report results
  if (allViolations.length === 0) {
    console.log("✅ No naming violations found");
    process.exit(0);
  } else {
    console.log(`🚨 Found ${allViolations.length} naming violations:`);
    console.log("");

    for (const violation of allViolations) {
      console.log(`❌ ${violation.type.toUpperCase()}: ${violation.file}`);
      if (violation.line) {
        console.log(`   Line ${violation.line}: ${violation.issue}`);
      } else {
        console.log(`   ${violation.issue}`);
      }
      console.log(`   Rule: ${violation.rule}`);
      console.log("");
    }

    console.log("🔧 Fix these violations before committing.");
    console.log(
      "💡 See: docs/audits/v3-codebase-audit-2025-10/06-naming-violations.md"
    );
    process.exit(1);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { checkNamingViolations, checkStructNaming };
