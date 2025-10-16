#!/usr/bin/env node

/**
 * Bulk fix unused variable warnings by prefixing with underscore
 *
 * Usage: node scripts/fix-unused-vars.js [--dry-run] [--filter="pattern"] [--input=lint-output.txt]
 *
 * First run: npm run lint > lint-output.txt 2>&1
 * Then: node scripts/fix-unused-vars.js --input=lint-output.txt --dry-run
 *
 * @author @darianrosebrook
 */

import fs from "fs";

const DRY_RUN = process.argv.includes("--dry-run");
const FILTER = process.argv
  .find((arg) => arg.startsWith("--filter="))
  ?.split("=")[1];
const INPUT_FILE =
  process.argv.find((arg) => arg.startsWith("--input="))?.split("=")[1] ||
  "lint-output.txt";

console.log("🔧 Bulk fixing unused variable warnings...\n");

if (DRY_RUN) {
  console.log("🚫 DRY RUN MODE - No files will be modified\n");
}

console.log(`📋 Reading lint warnings from ${INPUT_FILE}...`);

if (!fs.existsSync(INPUT_FILE)) {
  console.error(`❌ Input file not found: ${INPUT_FILE}`);
  console.log("💡 First run: npm run lint > lint-output.txt 2>&1");
  process.exit(1);
}

const eslintOutput = fs.readFileSync(INPUT_FILE, "utf8");
console.log(`Raw output length: ${eslintOutput.length}`);

console.log("🔍 Parsing warnings...");

// Parse the ESLint output to extract file paths and line numbers
const warnings = [];
const lines = eslintOutput.split("\n");
let currentFile = null;

for (const line of lines) {
  // Check if this is a file path line (starts with / and ends with .ts)
  if (line.startsWith("/") && (line.endsWith(".ts") || line.endsWith(".js"))) {
    currentFile = line.trim();
    continue;
  }

  // Match pattern: LINE_NUMBER:COLUMN   warning  'VAR_NAME' is defined but never used. Allowed unused args must match /^_/u no-unused-vars
  const match = line.match(
    /^\s+(\d+):\d+\s+warning\s+'([^']+)'\s+is\s+(?:defined|assigned a value)\s+but\s+never\s+used/
  );

  if (match && currentFile) {
    const [, lineNum, varName] = match;

    // Skip if filter doesn't match
    if (FILTER && !currentFile.includes(FILTER)) {
      continue;
    }

    // Skip already prefixed variables
    if (varName.startsWith("_")) {
      continue;
    }

    warnings.push({
      file: currentFile,
      line: parseInt(lineNum),
      variable: varName,
    });
  }
}

console.log(`📊 Found ${warnings.length} unused variables to fix\n`);

// Group by file for batch processing
const fileGroups = warnings.reduce((acc, warning) => {
  if (!acc[warning.file]) {
    acc[warning.file] = [];
  }
  acc[warning.file].push(warning);
  return acc;
}, {});

let totalFixed = 0;
let filesProcessed = 0;

// Process each file
for (const [filePath, fileWarnings] of Object.entries(fileGroups)) {
  if (!fs.existsSync(filePath)) {
    console.log(`⚠️  File not found: ${filePath}`);
    continue;
  }

  console.log(`🔧 Processing ${filePath} (${fileWarnings.length} fixes)...`);

  let content = fs.readFileSync(filePath, "utf8");
  const originalContent = content;
  let fileFixed = 0;

  // Process each warning for this file in reverse line order
  // to avoid offset issues when modifying multiple lines
  fileWarnings
    .sort((a, b) => b.line - a.line) // Process from bottom to top
    .forEach(({ line, variable }) => {
      const lines = content.split("\n");

      if (line > lines.length) {
        console.log(`  ⚠️  Line ${line} out of bounds for ${variable}`);
        return;
      }

      const targetLine = lines[line - 1]; // 0-based indexing

      // Find the variable in the line and replace it
      // This is a simple regex replacement - may need refinement for complex cases
      const escapedVar = variable.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const pattern = new RegExp(
        `\\b${escapedVar}\\b(?=\\s*[,:;=]|\\s*\\))`,
        "g"
      );

      const newLine = targetLine.replace(pattern, `_${variable}`);

      if (newLine !== targetLine) {
        lines[line - 1] = newLine;
        content = lines.join("\n");
        fileFixed++;
        console.log(
          `  ✅ Fixed "${variable}" -> "_${variable}" on line ${line}`
        );
      } else {
        console.log(
          `  ❌ Could not fix "${variable}" on line ${line} (pattern not found)`
        );
      }
    });

  // Write back if changes were made
  if (content !== originalContent && !DRY_RUN) {
    fs.writeFileSync(filePath, content, "utf8");
    filesProcessed++;
  } else if (DRY_RUN && content !== originalContent) {
    console.log(`  🚫 DRY RUN: Would have modified ${filePath}`);
  }

  totalFixed += fileFixed;
  console.log(`  📈 Fixed ${fileFixed} variables in ${filePath}\n`);
}

console.log("📊 Summary:");
console.log(`   Files processed: ${filesProcessed}`);
console.log(`   Variables fixed: ${totalFixed}`);
console.log(`   Total warnings: ${warnings.length}`);

if (DRY_RUN) {
  console.log("\n🔄 Run without --dry-run to apply changes");
} else {
  console.log("\n✅ Bulk fix complete!");
  console.log('🔍 Run "npm run lint" to verify the fixes');
}
