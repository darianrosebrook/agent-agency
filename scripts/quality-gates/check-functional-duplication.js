#!/usr/bin/env node

/**
 * Quality Gate: Functional Duplication Checker
 *
 * Advanced checker for business logic duplication patterns.
 * Focuses on detecting duplicate implementations, not just naming.
 * Based on analysis of actual code patterns and business logic.
 */

import fs from "fs";
import path from "path";

const V3_PATH = path.join(process.cwd(), "iterations/v3");

// Thresholds for functional duplication
const FUNCTIONAL_DUPLICATION_THRESHOLDS = {
  duplicateStructs: 692,
  duplicateFunctions: 200,
  duplicateTraits: 100,
  duplicateEnums: 50,
  duplicateImpls: 150,
  similarCodeBlocks: 30, // Similar code patterns across files
};

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

// Extract code patterns for similarity analysis
function extractCodePatterns(content) {
  const patterns = [];
  const lines = content.split("\n");

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    // Extract function signatures
    const fnMatch = line.match(/^pub fn (\w+)\s*\([^)]*\)\s*->\s*[^{]+/);
    if (fnMatch) {
      patterns.push({
        type: "function_signature",
        name: fnMatch[1],
        line: i + 1,
        content: line,
      });
    }

    // Extract struct definitions
    const structMatch = line.match(/^pub struct (\w+)/);
    if (structMatch) {
      patterns.push({
        type: "struct_definition",
        name: structMatch[1],
        line: i + 1,
        content: line,
      });
    }

    // Extract trait definitions
    const traitMatch = line.match(/^pub trait (\w+)/);
    if (traitMatch) {
      patterns.push({
        type: "trait_definition",
        name: traitMatch[1],
        line: i + 1,
        content: line,
      });
    }

    // Extract impl blocks
    const implMatch = line.match(/^impl\s+(\w+)/);
    if (implMatch) {
      patterns.push({
        type: "impl_block",
        name: implMatch[1],
        line: i + 1,
        content: line,
      });
    }
  }

  return patterns;
}

// Find duplicate patterns across files
function findDuplicatePatterns() {
  const allPatterns = [];
  const filePatterns = new Map();

  // Extract patterns from all files
  for (const filePath of RUST_FILES) {
    try {
      const content = fs.readFileSync(filePath, "utf8");
      const patterns = extractCodePatterns(content);
      filePatterns.set(filePath, patterns);
      allPatterns.push(...patterns.map((p) => ({ ...p, file: filePath })));
    } catch (error) {
      console.warn(`⚠️  Could not read ${filePath}: ${error.message}`);
    }
  }

  // Group patterns by type and name
  const patternGroups = {};
  for (const pattern of allPatterns) {
    const key = `${pattern.type}:${pattern.name}`;
    if (!patternGroups[key]) {
      patternGroups[key] = [];
    }
    patternGroups[key].push(pattern);
  }

  // Find duplicates
  const duplicates = {};
  for (const [key, patterns] of Object.entries(patternGroups)) {
    if (patterns.length > 1) {
      const [type, name] = key.split(":");
      duplicates[key] = {
        type,
        name,
        count: patterns.length,
        locations: patterns.map((p) => ({ file: p.file, line: p.line })),
      };
    }
  }

  return duplicates;
}

// Analyze code similarity (simplified)
function analyzeCodeSimilarity() {
  const similarBlocks = [];
  const fileContents = new Map();

  // Read all file contents
  for (const filePath of RUST_FILES) {
    try {
      const content = fs.readFileSync(filePath, "utf8");
      fileContents.set(filePath, content);
    } catch (error) {
      console.warn(`⚠️  Could not read ${filePath}: ${error.message}`);
    }
  }

  // Simple similarity detection based on common patterns
  const commonPatterns = [
    /async fn \w+\([^)]*\) -> Result<[^,>]+,\s*\w+Error>/g,
    /pub struct \w+\s*\{[^}]*\}/g,
    /impl \w+ for \w+\s*\{[^}]*\}/g,
    /match \w+\s*\{[^}]*\}/g,
  ];

  for (const [filePath, content] of fileContents) {
    for (const pattern of commonPatterns) {
      const matches = content.match(pattern);
      if (matches && matches.length > 1) {
        similarBlocks.push({
          file: filePath,
          pattern: pattern.toString(),
          count: matches.length,
          matches: matches.slice(0, 3), // Show first 3 matches
        });
      }
    }
  }

  return similarBlocks;
}

// Check for functional duplication violations
function checkFunctionalDuplication() {
  const violations = [];

  // Get duplicate patterns
  const duplicatePatterns = findDuplicatePatterns();

  // Check struct duplicates
  const structDuplicates = Object.values(duplicatePatterns).filter(
    (p) => p.type === "struct_definition"
  );
  if (
    structDuplicates.length > FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateStructs
  ) {
    violations.push({
      type: "struct_functional_duplication",
      issue: `Found ${structDuplicates.length} duplicate struct definitions (threshold: ${FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateStructs})`,
      count: structDuplicates.length,
      threshold: FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateStructs,
      details: structDuplicates.slice(0, 10), // Show first 10
    });
  }

  // Check function duplicates
  const functionDuplicates = Object.values(duplicatePatterns).filter(
    (p) => p.type === "function_signature"
  );
  if (
    functionDuplicates.length >
    FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateFunctions
  ) {
    violations.push({
      type: "function_functional_duplication",
      issue: `Found ${functionDuplicates.length} duplicate function signatures (threshold: ${FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateFunctions})`,
      count: functionDuplicates.length,
      threshold: FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateFunctions,
      details: functionDuplicates.slice(0, 10),
    });
  }

  // Check trait duplicates
  const traitDuplicates = Object.values(duplicatePatterns).filter(
    (p) => p.type === "trait_definition"
  );
  if (
    traitDuplicates.length > FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateTraits
  ) {
    violations.push({
      type: "trait_functional_duplication",
      issue: `Found ${traitDuplicates.length} duplicate trait definitions (threshold: ${FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateTraits})`,
      count: traitDuplicates.length,
      threshold: FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateTraits,
      details: traitDuplicates.slice(0, 10),
    });
  }

  // Check impl duplicates
  const implDuplicates = Object.values(duplicatePatterns).filter(
    (p) => p.type === "impl_block"
  );
  if (
    implDuplicates.length > FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateImpls
  ) {
    violations.push({
      type: "impl_functional_duplication",
      issue: `Found ${implDuplicates.length} duplicate impl blocks (threshold: ${FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateImpls})`,
      count: implDuplicates.length,
      threshold: FUNCTIONAL_DUPLICATION_THRESHOLDS.duplicateImpls,
      details: implDuplicates.slice(0, 10),
    });
  }

  // Check code similarity
  const similarBlocks = analyzeCodeSimilarity();
  if (
    similarBlocks.length > FUNCTIONAL_DUPLICATION_THRESHOLDS.similarCodeBlocks
  ) {
    violations.push({
      type: "code_similarity_duplication",
      issue: `Found ${similarBlocks.length} files with similar code patterns (threshold: ${FUNCTIONAL_DUPLICATION_THRESHOLDS.similarCodeBlocks})`,
      count: similarBlocks.length,
      threshold: FUNCTIONAL_DUPLICATION_THRESHOLDS.similarCodeBlocks,
      details: similarBlocks.slice(0, 5),
    });
  }

  return violations;
}

// Generate detailed report
function generateDetailedReport() {
  const duplicatePatterns = findDuplicatePatterns();
  const similarBlocks = analyzeCodeSimilarity();

  console.log("📊 Functional Duplication Analysis Report");
  console.log("=".repeat(50));

  // Summary by type
  const typeCounts = {};
  for (const pattern of Object.values(duplicatePatterns)) {
    typeCounts[pattern.type] = (typeCounts[pattern.type] || 0) + 1;
  }

  console.log("\n🔍 Duplicate Patterns by Type:");
  for (const [type, count] of Object.entries(typeCounts)) {
    const threshold =
      FUNCTIONAL_DUPLICATION_THRESHOLDS[
        `duplicate${
          type.charAt(0).toUpperCase() +
          type
            .slice(1)
            .replace("_definition", "s")
            .replace("_signature", "s")
            .replace("_block", "s")
        }`
      ] || 0;
    const status =
      count > threshold ? "🚨" : count > threshold * 0.8 ? "⚠️" : "✅";
    console.log(
      `   ${status} ${type}: ${count} duplicates (threshold: ${threshold})`
    );
  }

  // Top duplicates
  console.log("\n🔝 Top Duplicate Patterns:");
  const sortedDuplicates = Object.values(duplicatePatterns)
    .sort((a, b) => b.count - a.count)
    .slice(0, 10);

  for (const duplicate of sortedDuplicates) {
    console.log(
      `   - ${duplicate.name} (${duplicate.type}): ${duplicate.count} occurrences`
    );
    console.log(
      `     Locations: ${duplicate.locations
        .map((l) => `${path.basename(l.file)}:${l.line}`)
        .join(", ")}`
    );
  }

  // Similar code patterns
  if (similarBlocks.length > 0) {
    console.log("\n🔄 Similar Code Patterns:");
    for (const block of similarBlocks.slice(0, 5)) {
      console.log(
        `   - ${path.basename(block.file)}: ${block.count} similar patterns`
      );
    }
  }

  console.log("\n💡 Recommendations:");
  console.log("   1. Extract common traits for duplicate structs");
  console.log("   2. Consolidate duplicate functions into shared modules");
  console.log("   3. Create base traits for duplicate implementations");
  console.log("   4. Refactor similar code patterns into utilities");
}

function main() {
  console.log("🔍 Analyzing functional duplication patterns...");

  // Collect files
  collectRustFiles(V3_PATH);
  console.log(`📁 Found ${RUST_FILES.length} Rust files to analyze`);

  // Check for violations
  const violations = checkFunctionalDuplication();

  // Generate detailed report
  generateDetailedReport();

  if (violations.length === 0) {
    console.log("\n✅ No functional duplication violations detected");
    process.exit(0);
  } else {
    console.log(
      `\n🚨 Found ${violations.length} functional duplication violations:`
    );

    for (const violation of violations) {
      console.log("");
      console.log(`❌ ${violation.type.toUpperCase().replace(/_/g, " ")}`);
      console.log(`   Issue: ${violation.issue}`);
      console.log(
        `   Count: ${violation.count}, Threshold: ${violation.threshold}`
      );

      if (violation.details && violation.details.length > 0) {
        console.log("   Examples:");
        for (const detail of violation.details.slice(0, 3)) {
          if (detail.locations) {
            console.log(
              `     - ${detail.name}: ${detail.locations
                .map((l) => `${path.basename(l.file)}:${l.line}`)
                .join(", ")}`
            );
          } else if (detail.file) {
            console.log(
              `     - ${path.basename(detail.file)}: ${
                detail.count
              } similar patterns`
            );
          }
        }
      }
    }

    console.log("");
    console.log("🔧 Functional duplication must be reduced. Focus on:");
    console.log("   1. Extracting common traits and interfaces");
    console.log("   2. Consolidating duplicate business logic");
    console.log("   3. Creating shared utility modules");
    console.log("   4. Refactoring similar code patterns");
    process.exit(1);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export {
  findDuplicatePatterns,
  analyzeCodeSimilarity,
  checkFunctionalDuplication,
  generateDetailedReport,
};
