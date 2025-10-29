#!/usr/bin/env node

/**
 * Quality Gate: Naming Convention Checker
 *
 * Enforces CAWS naming conventions and blocks banned modifiers that indicate duplication.
 * Focuses on functional duplication prevention, not Rust naming conventions.
 * Based on rules in .cursor/rules/03-naming-and-refactor.mdc
 */

import fs from "fs";
import path from "path";

const V3_PATH = path.join(process.cwd(), "iterations/v3");
const EXCEPTION_CONFIG_PATH = path.join(
  process.cwd(),
  ".caws",
  "naming-exceptions.json"
);

// Banned modifiers that indicate duplicate/forked files (functional duplication)
const BANNED_MODIFIERS = [
  "enhanced",
  "unified",
  "simplified",
  "better",
  "new",
  "next",
  "final",
  "copy",
  "revamp",
  "improved",
];

// Rust convention files that are expected and should not be flagged
const RUST_CONVENTION_FILES = ["lib.rs", "mod.rs", "main.rs", "Cargo.toml"];

// Case-insensitive regex for banned modifiers
const BANNED_REGEX = new RegExp(`\\b(${BANNED_MODIFIERS.join("|")})\\b`, "i");

// Load exception configuration
function loadExceptionConfig() {
  try {
    if (fs.existsSync(EXCEPTION_CONFIG_PATH)) {
      const content = fs.readFileSync(EXCEPTION_CONFIG_PATH, "utf8");
      return JSON.parse(content);
    }
  } catch (error) {
    console.warn(`⚠️  Could not load naming exceptions: ${error.message}`);
  }

  return {
    exceptions: [],
    enforcement_levels: {
      commit: "warning",
      push: "block",
      ci: "fail",
    },
  };
}

// Check if a file matches an exception pattern
function isException(filePath, modifier) {
  const config = loadExceptionConfig();
  const now = new Date();

  for (const exception of config.exceptions) {
    // Convert glob pattern to regex
    let pattern = exception.file_pattern;

    // Handle ** patterns (match any path)
    if (pattern.startsWith("**/")) {
      pattern = pattern.substring(3); // Remove **/
      const regex = new RegExp(`.*/${pattern.replace(/\*/g, "[^/]*")}$`);
      if (
        regex.test(filePath) &&
        exception.modifier.toLowerCase() === modifier.toLowerCase()
      ) {
        // Check if exception is still valid
        const expiresAt = new Date(exception.expires_at);
        if (expiresAt > now) {
          return {
            valid: true,
            exception: exception,
          };
        } else {
          return {
            valid: false,
            reason: "expired",
            exception: exception,
          };
        }
      }
    } else {
      // Handle other patterns
      const regex = new RegExp(
        `^${pattern.replace(/\*\*/g, ".*").replace(/\*/g, "[^/]*")}$`
      );
      if (
        regex.test(filePath) &&
        exception.modifier.toLowerCase() === modifier.toLowerCase()
      ) {
        // Check if exception is still valid
        const expiresAt = new Date(exception.expires_at);
        if (expiresAt > now) {
          return {
            valid: true,
            exception: exception,
          };
        } else {
          return {
            valid: false,
            reason: "expired",
            exception: exception,
          };
        }
      }
    }
  }

  return { valid: false };
}

// Get enforcement level based on context
function getEnforcementLevel() {
  const config = loadExceptionConfig();
  const context = process.env.CAWS_ENFORCEMENT_CONTEXT || "commit";
  return config.enforcement_levels[context] || "warning";
}

// Files to check (all source files)
const SOURCE_FILES = [];

// File extensions to check
const SOURCE_EXTENSIONS = [
  ".rs",
  ".ts",
  ".tsx",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
  ".mts",
  ".cts",
  ".py",
  ".go",
  ".java",
  ".cpp",
  ".c",
  ".h",
];

// Convention files that are expected and should not be flagged
const CONVENTION_FILES = [
  "lib.rs",
  "mod.rs",
  "main.rs",
  "Cargo.toml",
  "index.ts",
  "index.js",
];

// Collect all source files
function collectSourceFiles(dir, sourceFiles = SOURCE_FILES) {
  const files = fs.readdirSync(dir);

  for (const file of files) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);

    if (
      stat.isDirectory() &&
      !file.startsWith(".") &&
      file !== "target" &&
      file !== "node_modules"
    ) {
      collectSourceFiles(fullPath, sourceFiles);
    } else if (
      SOURCE_EXTENSIONS.some((ext) => file.endsWith(ext)) &&
      !CONVENTION_FILES.includes(file)
    ) {
      sourceFiles.push(fullPath);
    }
  }
}

// Check naming violations (excluding convention files)
function checkNamingViolations(sourceFiles = SOURCE_FILES) {
  const violations = [];
  const warnings = [];
  const enforcementLevel = getEnforcementLevel();

  for (const filePath of sourceFiles) {
    const fileName = path.basename(filePath);
    const fileNameWithoutExt = path.basename(filePath, path.extname(filePath));

    // Skip convention files
    if (CONVENTION_FILES.includes(fileName)) {
      continue;
    }

    // Check for banned modifiers in filenames
    if (BANNED_REGEX.test(fileNameWithoutExt)) {
      const matchedModifier = fileNameWithoutExt.match(BANNED_REGEX)[0];
      const exceptionCheck = isException(filePath, matchedModifier);

      if (exceptionCheck.valid) {
        // Valid exception - log for transparency
        warnings.push({
          type: "exception_used",
          file: filePath,
          issue: `Using approved exception for "${matchedModifier}" modifier`,
          reason: exceptionCheck.exception.reason,
          approved_by: exceptionCheck.exception.approved_by,
          expires_at: exceptionCheck.exception.expires_at,
        });
        continue;
      }

      if (exceptionCheck.reason === "expired") {
        // Expired exception - treat as violation
        violations.push({
          type: "expired_exception",
          file: filePath,
          issue: `Exception expired for "${matchedModifier}" modifier`,
          rule: "Expired naming exceptions must be renewed or file renamed",
          severity: enforcementLevel,
          original_exception: exceptionCheck.exception,
        });
        continue;
      }

      // No valid exception - create violation
      const violation = {
        type: "filename_banned_modifier",
        file: filePath,
        issue: `Filename contains banned modifier: ${matchedModifier}`,
        rule: 'No duplicate "enhanced/unified/new/final" modules - indicates functional duplication',
        severity: enforcementLevel,
        suggestion: `Consider renaming to a purpose-first name. If this is legitimate architectural consolidation, add an exception to .caws/naming-exceptions.json`,
      };

      violations.push(violation);
    }
  }

  return { violations, warnings };
}

// Check symbol naming consistency
function checkSymbolNaming(sourceFiles = SOURCE_FILES) {
  const violations = [];

  for (const filePath of sourceFiles) {
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
  console.log(
    "🔍 Checking for problematic naming patterns (functional duplication indicators)..."
  );

  // Collect files
  collectSourceFiles(V3_PATH);
  console.log(`📁 Found ${SOURCE_FILES.length} source files to check`);

  // Run checks
  const filenameResults = checkNamingViolations();
  const symbolViolations = checkSymbolNaming();

  const allViolations = [...filenameResults.violations, ...symbolViolations];
  const allWarnings = filenameResults.warnings;

  // Report results
  if (allViolations.length === 0 && allWarnings.length === 0) {
    console.log("✅ No problematic naming patterns found");
    console.log(
      "ℹ️  Convention files (lib.rs, mod.rs, index.ts, etc.) are expected and not flagged"
    );
    process.exit(0);
  } else {
    // Report warnings first
    if (allWarnings.length > 0) {
      console.log(`ℹ️  ${allWarnings.length} approved exceptions in use:`);
      for (const warning of allWarnings) {
        console.log(`   📋 ${warning.file}`);
        console.log(`      ${warning.issue}`);
        console.log(`      Reason: ${warning.reason}`);
        console.log(`      Approved by: ${warning.approved_by}`);
        console.log(`      Expires: ${warning.expires_at}`);
        console.log("");
      }
    }

    // Report violations
    if (allViolations.length > 0) {
      console.log(
        `🚨 Found ${allViolations.length} problematic naming patterns:`
      );
      console.log("");

      for (const violation of allViolations) {
        const severityIcon =
          violation.severity === "fail"
            ? "💥"
            : violation.severity === "block"
            ? "🚫"
            : "⚠️";

        console.log(
          `${severityIcon} ${violation.type.toUpperCase()}: ${violation.file}`
        );
        if (violation.line) {
          console.log(`   Line ${violation.line}: ${violation.issue}`);
        } else {
          console.log(`   ${violation.issue}`);
        }
        console.log(`   Rule: ${violation.rule}`);
        if (violation.suggestion) {
          console.log(`   💡 ${violation.suggestion}`);
        }
        console.log("");
      }

      const enforcementLevel = getEnforcementLevel();
      console.log(`🔧 Enforcement level: ${enforcementLevel.toUpperCase()}`);

      if (enforcementLevel === "fail") {
        console.log("💥 CI/CD will fail - fix violations immediately");
        process.exit(1);
      } else if (enforcementLevel === "block") {
        console.log("🚫 Push blocked - fix violations before pushing");
        process.exit(1);
      } else {
        console.log("⚠️  Warning mode - commit allowed but fix recommended");
        process.exit(0);
      }
    } else {
      console.log("✅ All violations resolved (warnings only)");
      process.exit(0);
    }
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export {
  checkNamingViolations,
  checkSymbolNaming,
  collectSourceFiles,
  loadExceptionConfig,
  isException,
  getEnforcementLevel,
};
