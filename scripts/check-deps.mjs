#!/usr/bin/env node

// Dependency Graph Validator
// Parses cargo metadata and enforces architectural rules
//
// Usage: node scripts/check-deps.mjs <metadata.json> < rules.txt
//
// Rules format:
// FORBID: source_pattern -> (target_pattern1, target_pattern2)
// ALLOW:  source_pattern -> target_pattern
//
// Patterns support wildcards:
// - * matches any sequence
// - ? matches any single character

import fs from "fs";

const args = process.argv.slice(2);
if (args.length < 1) {
  console.error("Usage: node check-deps.mjs <metadata.json>");
  process.exit(1);
}

const metadataFile = args[0];

// Read cargo metadata
let metadata;
try {
  const content = fs.readFileSync(metadataFile, "utf8");
  metadata = JSON.parse(content);
} catch (error) {
  console.error(
    `❌ Failed to read metadata file ${metadataFile}:`,
    error.message
  );
  process.exit(1);
}

// Read rules from stdin
let rulesText = "";
try {
  const stdin = fs.readFileSync(0, "utf8"); // Read from stdin
  rulesText = stdin;
} catch (error) {
  console.error("❌ Failed to read rules from stdin:", error.message);
  process.exit(1);
}

// Parse rules
const rules = parseRules(rulesText);

// Build dependency map
const deps = buildDependencyMap(metadata);

// Validate rules
let hasViolations = false;

console.log("🔍 Validating dependency rules...\n");

for (const rule of rules) {
  const violations = checkRule(rule, deps);
  if (violations.length > 0) {
    hasViolations = true;
    console.log(
      `❌ Rule violation: ${rule.type} ${rule.source} -> ${rule.targets.join(
        ", "
      )}`
    );
    console.log("   Found edges:");
    for (const violation of violations) {
      console.log(`     ${violation.from} -> ${violation.to}`);
    }
    console.log("");
  }
}

if (hasViolations) {
  console.log(
    "💥 Dependency violations detected! Fix the dependency graph or update rules."
  );
  process.exit(1);
} else {
  console.log("✅ All dependency rules satisfied!");
}

function parseRules(text) {
  const rules = [];
  const lines = text.trim().split("\n");

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;

    // Split by arrow and parse manually
    const parts = trimmed.split("->").map((s) => s.trim());
    if (parts.length !== 2) {
      console.warn(`⚠️  Skipping invalid rule: ${trimmed}`);
      continue;
    }

    const left = parts[0];
    const right = parts[1];

    const typeMatch = left.match(/^(FORBID|ALLOW):\s*(.+)$/);
    if (!typeMatch) {
      console.warn(`⚠️  Skipping invalid rule: ${trimmed}`);
      continue;
    }

    const [, type, source] = typeMatch;
    const targets = right.split(",").map((t) => t.trim());

    rules.push({
      type,
      source: source.trim(),
      targets,
    });
  }

  return rules;
}

function buildDependencyMap(metadata) {
  const deps = new Map();
  const workspacePackages = new Set();

  // First pass: collect workspace package names
  for (const pkg of metadata.packages) {
    if (pkg.id.startsWith("path+file://")) {
      workspacePackages.add(pkg.name);
    }
  }

  // Second pass: build dependency map for workspace packages only
  for (const pkg of metadata.packages) {
    if (!workspacePackages.has(pkg.name)) continue;

    const pkgName = pkg.name;
    const pkgDeps = new Set();

    // Collect all dependencies (normal + dev + build)
    const allDeps = [
      ...(pkg.dependencies || []),
      ...(pkg.dev_dependencies || []),
      ...(pkg.build_dependencies || []),
    ];

    for (const dep of allDeps) {
      // Only include workspace dependencies
      if (workspacePackages.has(dep.name)) {
        pkgDeps.add(dep.name);
      }
    }

    deps.set(pkgName, pkgDeps);
  }

  return deps;
}

function checkRule(rule, deps) {
  const violations = [];

  for (const [fromPkg, toPkgs] of deps) {
    if (!matchesPattern(fromPkg, rule.source)) continue;

    for (const toPkg of toPkgs) {
      const matchesAnyTarget = rule.targets.some((target) =>
        matchesPattern(toPkg, target)
      );

      if (rule.type === "FORBID" && matchesAnyTarget) {
        violations.push({ from: fromPkg, to: toPkg });
      }
      // Note: ALLOW rules are permissive and don't generate violations
      // They exist for documentation purposes only
    }
  }

  return violations;
}

function matchesPattern(text, pattern) {
  // Convert glob pattern to regex
  const regexPattern = pattern
    .replace(/\*/g, ".*") // * -> .*
    .replace(/\?/g, ".") // ? -> .
    .replace(/\//g, "\\/"); // Escape forward slashes

  const regex = new RegExp(`^${regexPattern}$`);
  return regex.test(text);
}
