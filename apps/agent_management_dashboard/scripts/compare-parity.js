#!/usr/bin/env node

/**
 * Parity Comparison Tool
 *
 * Compares the old Tailwind version with the new SCSS modules version
 * to identify styling and interaction gaps.
 *
 * Usage: node scripts/compare-parity.js
 */

const fs = require("fs");
const path = require("path");

const OLD_VERSION_PATH = path.join(__dirname, "../../old_tailwind_version/src");
const NEW_VERSION_PATH = path.join(__dirname, "../src");

// Component mapping between old and new versions
const COMPONENT_MAP = {
  "components/Dashboard.tsx": "components/dashboard/Dashboard.tsx",
  "components/Projects.tsx": "components/projects/Projects.tsx",
  "components/ProjectView.tsx": "components/projects/ProjectView.tsx",
  "components/Chat.tsx": "components/chat/Chat.tsx",
  "components/ChatSidebar.tsx": "components/chat/ChatSidebar.tsx",
  "components/OverviewTab.tsx": "components/projects/OverviewTab.tsx",
  "components/TasksTab.tsx": "components/projects/TasksTab.tsx",
  "components/TimelineTab.tsx": "components/projects/TimelineTab.tsx",
  "components/WorkspaceTab.tsx": "components/projects/WorkspaceTab.tsx",
  "components/ManageTab.tsx": "components/projects/SettingsTab.tsx",
  "components/PhaseManager.tsx":
    "components/projects/phase-manager/PhaseManager.tsx",
};

// Extract Tailwind classes from a file
function extractTailwindClasses(content) {
  const classRegex = /className=["']([^"']+)["']/g;
  const classes = new Set();
  let match;

  while ((match = classRegex.exec(content)) !== null) {
    const classString = match[1];
    // Split by spaces and filter out empty strings
    classString.split(/\s+/).forEach((cls) => {
      if (cls.trim()) {
        classes.add(cls.trim());
      }
    });
  }

  return Array.from(classes);
}

// Extract SCSS class names from a file
function extractSCSSClasses(content) {
  const scssRegex = /\.([a-zA-Z0-9_-]+)\s*\{/g;
  const classes = new Set();
  let match;

  while ((match = scssRegex.exec(content)) !== null) {
    classes.add(match[1]);
  }

  return Array.from(classes);
}

// Extract className usage from TSX file
function extractClassNameUsage(content) {
  const classRegex = /className={([^}]+)}/g;
  const usages = [];
  let match;

  while ((match = classRegex.exec(content)) !== null) {
    usages.push(match[1].trim());
  }

  return usages;
}

// Check if a component file exists
function fileExists(filePath) {
  try {
    return fs.statSync(filePath).isFile();
  } catch {
    return false;
  }
}

// Read file content
function readFile(filePath) {
  try {
    return fs.readFileSync(filePath, "utf-8");
  } catch {
    return null;
  }
}

// Analyze a component pair
function analyzeComponent(oldPath, newPath) {
  const oldFullPath = path.join(OLD_VERSION_PATH, oldPath);
  const newFullPath = path.join(NEW_VERSION_PATH, newPath);

  const result = {
    oldPath,
    newPath,
    exists: {
      old: fileExists(oldFullPath),
      new: fileExists(newFullPath),
    },
    issues: [],
    warnings: [],
  };

  if (!result.exists.old) {
    result.issues.push(`Old component not found: ${oldPath}`);
    return result;
  }

  if (!result.exists.new) {
    result.issues.push(`New component not found: ${newPath}`);
    return result;
  }

  const oldContent = readFile(oldFullPath);
  const newContent = readFile(newFullPath);

  if (!oldContent || !newContent) {
    result.issues.push("Could not read file content");
    return result;
  }

  // Extract Tailwind classes from old version
  const oldClasses = extractTailwindClasses(oldContent);
  const oldClassNameUsages = extractClassNameUsage(oldContent);

  // Extract SCSS module class usage from new version
  const newClassNameUsages = extractClassNameUsage(newContent);

  // Find SCSS module file
  const scssModulePath = newPath.replace(".tsx", ".module.scss");
  const scssFullPath = path.join(NEW_VERSION_PATH, scssModulePath);

  if (fileExists(scssFullPath)) {
    const scssContent = readFile(scssFullPath);
    if (scssContent) {
      const scssClasses = extractSCSSClasses(scssContent);

      // Check if all className usages reference valid SCSS classes
      newClassNameUsages.forEach((usage) => {
        // Extract class names from usage (handles styles.className, cn(), etc.)
        const classMatches = usage.match(/styles\.([a-zA-Z0-9_-]+)/g);
        if (classMatches) {
          classMatches.forEach((match) => {
            const className = match.replace("styles.", "");
            if (!scssClasses.includes(className)) {
              result.warnings.push(
                `SCSS class '${className}' used but not defined in ${scssModulePath}`
              );
            }
          });
        }
      });
    } else {
      result.warnings.push(`Could not read SCSS module: ${scssModulePath}`);
    }
  } else {
    result.warnings.push(`SCSS module not found: ${scssModulePath}`);
  }

  // Check for common Tailwind patterns that might be missing
  const commonPatterns = {
    hover: oldClasses.filter((c) => c.startsWith("hover:")),
    focus: oldClasses.filter((c) => c.startsWith("focus:")),
    active: oldClasses.filter((c) => c.startsWith("active:")),
    group: oldClasses.filter((c) => c.startsWith("group")),
    transition: oldClasses.filter((c) => c.includes("transition")),
    animation: oldClasses.filter((c) => c.includes("animate")),
  };

  if (commonPatterns.hover.length > 0) {
    result.warnings.push(
      `Found ${commonPatterns.hover.length} hover states in old version - verify SCSS :hover equivalents`
    );
  }

  if (commonPatterns.transition.length > 0) {
    result.warnings.push(
      `Found ${commonPatterns.transition.length} transitions in old version - verify SCSS transition properties`
    );
  }

  return result;
}

// Main comparison function
function compareParity() {
  console.log("🔍 Starting Parity Comparison...\n");
  console.log("=".repeat(80));

  const results = [];

  for (const [oldPath, newPath] of Object.entries(COMPONENT_MAP)) {
    console.log(`\n📦 Comparing: ${oldPath} → ${newPath}`);
    const result = analyzeComponent(oldPath, newPath);
    results.push(result);

    if (result.issues.length > 0) {
      console.log("  ❌ Issues:");
      result.issues.forEach((issue) => console.log(`    - ${issue}`));
    }

    if (result.warnings.length > 0) {
      console.log("  ⚠️  Warnings:");
      result.warnings.forEach((warning) => console.log(`    - ${warning}`));
    }

    if (result.issues.length === 0 && result.warnings.length === 0) {
      console.log("  ✅ No issues found");
    }
  }

  // Summary
  console.log("\n" + "=".repeat(80));
  console.log("\n📊 Summary:\n");

  const totalComponents = results.length;
  const componentsWithIssues = results.filter(
    (r) => r.issues.length > 0
  ).length;
  const componentsWithWarnings = results.filter(
    (r) => r.warnings.length > 0
  ).length;
  const cleanComponents = results.filter(
    (r) => r.issues.length === 0 && r.warnings.length === 0
  ).length;

  console.log(`Total components compared: ${totalComponents}`);
  console.log(`Components with issues: ${componentsWithIssues}`);
  console.log(`Components with warnings: ${componentsWithWarnings}`);
  console.log(`Clean components: ${cleanComponents}`);

  // Generate detailed report
  const reportPath = path.join(__dirname, "../PARITY_COMPARISON_REPORT.md");
  let report = "# Parity Comparison Report\n\n";
  report += `Generated: ${new Date().toISOString()}\n\n`;
  report += "## Summary\n\n";
  report += `- Total components compared: ${totalComponents}\n`;
  report += `- Components with issues: ${componentsWithIssues}\n`;
  report += `- Components with warnings: ${componentsWithWarnings}\n`;
  report += `- Clean components: ${cleanComponents}\n\n`;
  report += "## Detailed Results\n\n";

  results.forEach((result) => {
    report += `### ${result.oldPath} → ${result.newPath}\n\n`;

    if (result.issues.length > 0) {
      report += "**Issues:**\n";
      result.issues.forEach((issue) => {
        report += `- ${issue}\n`;
      });
      report += "\n";
    }

    if (result.warnings.length > 0) {
      report += "**Warnings:**\n";
      result.warnings.forEach((warning) => {
        report += `- ${warning}\n`;
      });
      report += "\n";
    }

    if (result.issues.length === 0 && result.warnings.length === 0) {
      report += "✅ No issues found\n\n";
    }
  });

  fs.writeFileSync(reportPath, report);
  console.log(`\n📄 Detailed report saved to: ${reportPath}`);

  return results;
}

// Run if called directly
if (require.main === module) {
  compareParity();
}

module.exports = { compareParity, analyzeComponent };
