#!/usr/bin/env node

/**
 * Quality Gate: Code Freeze Enforcement
 *
 * Blocks commits that add new features during crisis response.
 * Only allows bug fixes, refactoring, and critical maintenance.
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import {
  processViolations,
  getEnforcementLevel,
} from "./shared-exception-framework.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const V3_PATH = path.join(process.cwd(), "iterations/v3");

// Keywords that indicate new features (blocked during code freeze)
const NEW_FEATURE_KEYWORDS = [
  "feature",
  "add",
  "new",
  "create",
  "implement",
  "introduce",
  "enhance",
  "extend",
  "expand",
  "upgrade",
  "improve",
  "build",
  "develop",
  "launch",
];

// Allowed keywords for crisis response
const ALLOWED_KEYWORDS = [
  "fix",
  "bug",
  "refactor",
  "cleanup",
  "remove",
  "delete",
  "extract",
  "merge",
  "consolidate",
  "decompose",
  "god.object",
  "duplicate",
  "trait",
  "common",
  "quality.gate",
  "crisis",
  "emergency",
  "audit",
  "test",
  "lint",
  "format",
  "doc",
  "readme",
  "comment",
];

// Check git commit message for new features
async function checkCommitMessage() {
  // Get the commit message from git
  try {
    const { execSync } = await import("child_process");
    const commitMessage = execSync("git log -1 --pretty=%B", {
      encoding: "utf8",
    }).trim();

    const lowerMessage = commitMessage.toLowerCase();

    // Check for new feature keywords
    const hasNewFeature = NEW_FEATURE_KEYWORDS.some((keyword) =>
      lowerMessage.includes(keyword.toLowerCase())
    );

    // Check for allowed keywords
    const hasAllowed = ALLOWED_KEYWORDS.some((keyword) =>
      lowerMessage.includes(keyword.toLowerCase())
    );

    if (hasNewFeature && !hasAllowed) {
      return {
        blocked: true,
        reason: `Commit message suggests new feature: "${commitMessage}"`,
        suggestion:
          "During crisis response, only bug fixes, refactoring, and quality improvements are allowed.",
      };
    }

    return { blocked: false };
  } catch (error) {
    // If we can't get commit message, allow the commit (fail open)
    console.warn("⚠️  Could not check commit message, allowing commit");
    return { blocked: false };
  }
}

// Check for new files that might be features
async function checkNewFiles() {
  try {
    const { execSync } = await import("child_process");
    const newFiles = execSync("git diff --cached --name-only --diff-filter=A", {
      encoding: "utf8",
    })
      .trim()
      .split("\n")
      .filter((file) => file && file.endsWith(".rs"));

    const suspiciousFiles = newFiles.filter((file) => {
      const basename = path.basename(file, ".rs").toLowerCase();
      return NEW_FEATURE_KEYWORDS.some((keyword) => basename.includes(keyword));
    });

    if (suspiciousFiles.length > 0) {
      return {
        blocked: true,
        reason: `New files suggest feature addition: ${suspiciousFiles.join(
          ", "
        )}`,
        suggestion: "During crisis response, avoid adding new source files.",
      };
    }

    return { blocked: false };
  } catch (error) {
    console.warn("⚠️  Could not check new files, allowing commit");
    return { blocked: false };
  }
}

// Check for large additions that might be features
async function checkLargeAdditions() {
  try {
    const { execSync } = await import("child_process");
    const diffStats = execSync("git diff --cached --stat", {
      encoding: "utf8",
    });

    // Look for large additions
    const lines = diffStats.split("\n");
    let totalAdditions = 0;

    for (const line of lines) {
      const match = line.match(/\|.*?(\d+) insertion/);
      if (match) {
        totalAdditions += parseInt(match[1]);
      }
    }

    // Allow up to 500 lines of additions for refactoring
    if (totalAdditions > 500) {
      return {
        blocked: true,
        reason: `Large addition detected: ${totalAdditions} lines added`,
        suggestion:
          "During crisis response, keep changes under 500 lines. Split large changes.",
      };
    }

    return { blocked: false };
  } catch (error) {
    console.warn("⚠️  Could not check diff stats, allowing commit");
    return { blocked: false };
  }
}

async function main() {
  console.log("🚫 Checking code freeze compliance...");

  const commitCheck = await checkCommitMessage();
  const filesCheck = await checkNewFiles();
  const sizeCheck = await checkLargeAdditions();

  const violations = [commitCheck, filesCheck, sizeCheck].filter(
    (check) => check.blocked
  );

  if (violations.length > 0) {
    console.log(
      `🚨 CODE FREEZE VIOLATION (${violations.length}) - COMMIT BLOCKED:`
    );
    console.log("");

    for (const violation of violations) {
      console.log(`❌ ${violation.reason}`);
      console.log(`💡 ${violation.suggestion}`);
      console.log("");
    }

    console.log("🔧 Code freeze is active during crisis response.");
    console.log("📖 See docs/refactoring.md for allowed activities.");
    process.exit(1);
  } else {
    console.log("✅ Code freeze compliance check passed");
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error("❌ Code freeze check failed:", error);
    process.exit(1);
  });
}

export { checkCommitMessage, checkNewFiles, checkLargeAdditions };
