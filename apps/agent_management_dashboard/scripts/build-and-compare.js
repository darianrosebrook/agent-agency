#!/usr/bin/env node

/**
 * Build Comparison Script
 * 
 * This script:
 * 1. Builds both the old Tailwind version and new SCSS version
 * 2. Extracts CSS from both builds
 * 3. Compares styling differences
 * 4. Generates a comprehensive report
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const OLD_VERSION_DIR = path.join(__dirname, '../../old_tailwind_version');
const NEW_VERSION_DIR = path.join(__dirname, '..');
const OUTPUT_DIR = path.join(__dirname, '../build-comparison');

// Ensure output directory exists
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

console.log('🔨 Building both applications...\n');

// Build old version (Vite)
console.log('📦 Building old Tailwind version...');
try {
  process.chdir(OLD_VERSION_DIR);
  execSync('npm run build', { stdio: 'inherit' });
  console.log('✅ Old version built successfully\n');
} catch (error) {
  console.error('❌ Failed to build old version:', error.message);
  process.exit(1);
}

// Build new version (Next.js) - skip linting for comparison
console.log('📦 Building new SCSS version...');
try {
  process.chdir(NEW_VERSION_DIR);
  // Set environment variable to skip linting
  process.env.ESLINT_NO_DEV_ERRORS = 'true';
  execSync('SKIP_ENV_VALIDATION=true next build --no-lint', { stdio: 'inherit' });
  console.log('✅ New version built successfully\n');
} catch (error) {
  console.error('❌ Failed to build new version:', error.message);
  console.log('\n⚠️  Note: Build failed due to linting errors.');
  console.log('   You may need to fix linting errors or run: SKIP_ENV_VALIDATION=true next build --no-lint\n');
  process.exit(1);
}

// Extract CSS files
console.log('📊 Extracting CSS files...\n');

const oldBuildDir = path.join(OLD_VERSION_DIR, 'build');
const newBuildDir = path.join(NEW_VERSION_DIR, '.next');

// Find CSS files in old build
const oldCssFiles = [];
if (fs.existsSync(oldBuildDir)) {
  const findCssFiles = (dir) => {
    const files = fs.readdirSync(dir);
    files.forEach(file => {
      const filePath = path.join(dir, file);
      const stat = fs.statSync(filePath);
      if (stat.isDirectory()) {
        findCssFiles(filePath);
      } else if (file.endsWith('.css')) {
        oldCssFiles.push(filePath);
      }
    });
  };
  findCssFiles(oldBuildDir);
}

// Find CSS files in new build
const newCssFiles = [];
if (fs.existsSync(newBuildDir)) {
  const findCssFiles = (dir) => {
    if (!fs.existsSync(dir)) return;
    const files = fs.readdirSync(dir);
    files.forEach(file => {
      const filePath = path.join(dir, file);
      try {
        const stat = fs.statSync(filePath);
        if (stat.isDirectory()) {
          findCssFiles(filePath);
        } else if (file.endsWith('.css')) {
          newCssFiles.push(filePath);
        }
      } catch (e) {
        // Skip if can't read
      }
    });
  };
  findCssFiles(newBuildDir);
}

console.log(`Found ${oldCssFiles.length} CSS files in old build`);
console.log(`Found ${newCssFiles.length} CSS files in new build\n`);

// Extract and compare CSS
const extractCssRules = (cssContent) => {
  const rules = new Map();
  
  // Extract class-based rules
  const classRegex = /\.([a-zA-Z0-9_-]+)\s*\{([^}]+)\}/g;
  let match;
  while ((match = classRegex.exec(cssContent)) !== null) {
    const className = match[1];
    const properties = match[2];
    rules.set(className, properties.trim());
  }
  
  // Extract utility-like rules (e.g., .rounded-lg, .p-4)
  const utilityRegex = /\.(rounded-[a-z]+|p-\d+|m-\d+|gap-\d+|text-[a-z]+|w-\d+|h-\d+|bg-\[#[a-f0-9]+\]|border-[a-z-]+)\s*\{([^}]+)\}/g;
  while ((match = utilityRegex.exec(cssContent)) !== null) {
    const utility = match[1];
    const properties = match[2];
    rules.set(utility, properties.trim());
  }
  
  return rules;
};

// Compare CSS rules
const compareCss = (oldRules, newRules) => {
  const differences = {
    missingInNew: [],
    missingInOld: [],
    different: [],
    same: []
  };
  
  // Check rules in old version
  oldRules.forEach((properties, className) => {
    if (!newRules.has(className)) {
      differences.missingInNew.push({
        className,
        properties
      });
    } else {
      const newProperties = newRules.get(className);
      if (properties !== newProperties) {
        differences.different.push({
          className,
          old: properties,
          new: newProperties
        });
      } else {
        differences.same.push(className);
      }
    }
  });
  
  // Check rules only in new version
  newRules.forEach((properties, className) => {
    if (!oldRules.has(className)) {
      differences.missingInOld.push({
        className,
        properties
      });
    }
  });
  
  return differences;
};

// Read and compare CSS files
let allOldRules = new Map();
let allNewRules = new Map();

oldCssFiles.forEach(file => {
  try {
    const content = fs.readFileSync(file, 'utf-8');
    const rules = extractCssRules(content);
    rules.forEach((props, className) => {
      allOldRules.set(className, props);
    });
  } catch (e) {
    console.warn(`⚠️  Could not read ${file}:`, e.message);
  }
});

newCssFiles.forEach(file => {
  try {
    const content = fs.readFileSync(file, 'utf-8');
    const rules = extractCssRules(content);
    rules.forEach((props, className) => {
      allNewRules.set(className, props);
    });
  } catch (e) {
    console.warn(`⚠️  Could not read ${file}:`, e.message);
  }
});

console.log(`Extracted ${allOldRules.size} CSS rules from old build`);
console.log(`Extracted ${allNewRules.size} CSS rules from new build\n`);

// Compare
const differences = compareCss(allOldRules, allNewRules);

// Generate report
const report = {
  timestamp: new Date().toISOString(),
  summary: {
    oldRules: allOldRules.size,
    newRules: allNewRules.size,
    missingInNew: differences.missingInNew.length,
    missingInOld: differences.missingInOld.length,
    different: differences.different.length,
    same: differences.same.length
  },
  differences: {
    missingInNew: differences.missingInNew.slice(0, 100), // Limit to first 100
    missingInOld: differences.missingInOld.slice(0, 100),
    different: differences.different.slice(0, 100)
  }
};

// Write report
const reportPath = path.join(OUTPUT_DIR, 'build-comparison-report.json');
fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

// Generate markdown report
const markdownReport = `# Build Comparison Report

Generated: ${new Date().toLocaleString()}

## Summary

- **Old Build CSS Rules**: ${allOldRules.size}
- **New Build CSS Rules**: ${allNewRules.size}
- **Matching Rules**: ${differences.same.length}
- **Different Rules**: ${differences.different.length}
- **Missing in New**: ${differences.missingInNew.length}
- **Missing in Old**: ${differences.missingInOld.length}

## Differences Found

### Rules Missing in New Version (${differences.missingInNew.length})

These CSS rules exist in the old Tailwind build but not in the new SCSS build:

${differences.missingInNew.slice(0, 50).map(d => `- **${d.className}**: \`${d.properties.substring(0, 100)}...\``).join('\n')}

${differences.missingInNew.length > 50 ? `\n*... and ${differences.missingInNew.length - 50} more*` : ''}

### Rules Only in New Version (${differences.missingInOld.length})

These CSS rules exist in the new SCSS build but not in the old Tailwind build:

${differences.missingInOld.slice(0, 50).map(d => `- **${d.className}**: \`${d.properties.substring(0, 100)}...\``).join('\n')}

${differences.missingInOld.length > 50 ? `\n*... and ${differences.missingInOld.length - 50} more*` : ''}

### Rules with Different Properties (${differences.different.length})

These CSS rules exist in both but have different properties:

${differences.different.slice(0, 30).map(d => `
#### ${d.className}

**Old:**
\`\`\`css
${d.old}
\`\`\`

**New:**
\`\`\`css
${d.new}
\`\`\`
`).join('\n')}

${differences.different.length > 30 ? `\n*... and ${differences.different.length - 30} more*` : ''}

## Next Steps

1. Review the differences above
2. Verify which missing rules are intentional (e.g., unused Tailwind utilities)
3. Fix any unintentional differences
4. Re-run this script to verify fixes

## Full Report

See \`build-comparison-report.json\` for the complete data.
`;

const markdownPath = path.join(OUTPUT_DIR, 'BUILD_COMPARISON_REPORT.md');
fs.writeFileSync(markdownPath, markdownReport);

console.log('📊 Comparison complete!\n');
console.log(`✅ Report saved to: ${markdownPath}`);
console.log(`✅ Full data saved to: ${reportPath}\n`);
console.log('Summary:');
console.log(`  - Matching rules: ${differences.same.length}`);
console.log(`  - Different rules: ${differences.different.length}`);
console.log(`  - Missing in new: ${differences.missingInNew.length}`);
console.log(`  - Missing in old: ${differences.missingInOld.length}\n`);

