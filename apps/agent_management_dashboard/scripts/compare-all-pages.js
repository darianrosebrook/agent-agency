#!/usr/bin/env node

/**
 * Comprehensive page comparison script
 * Compares all pages between Tailwind and SCSS versions
 * 
 * @author @darianrosebrook
 */

const fs = require('fs');
const path = require('path');

const OLD_TAILWIND_DIR = path.join(__dirname, '../../old_tailwind_version/src/components');
const SCSS_VERSION_DIR = path.join(__dirname, '../src');

// Pages to compare
const PAGES_TO_COMPARE = [
  { name: 'Chat', oldPath: 'Chat.tsx', newPath: 'components/chat/Chat.tsx' },
  { name: 'Projects', oldPath: 'Projects.tsx', newPath: 'components/projects/Projects.tsx' },
  { name: 'ProjectView', oldPath: 'ProjectView.tsx', newPath: 'components/projects/ProjectView.tsx' },
  { name: 'Settings', oldPath: null, newPath: 'app/settings/page.tsx' }, // May not exist in old version
  { name: 'PhaseManager', oldPath: 'PhaseManager.tsx', newPath: 'components/phase-planner/PhaseManager.tsx' },
];

// Helper functions from previous script
function extractClassNames(filePath) {
  if (!fs.existsSync(filePath)) {
    return [];
  }

  const content = fs.readFileSync(filePath, 'utf-8');
  const classNames = [];
  
  const classNameRegex = /className\s*=\s*{?["'`]([^"'`]+)["'`]}?/g;
  const templateRegex = /className\s*=\s*{`([^`]+)`}/g;
  
  let match;
  
  while ((match = classNameRegex.exec(content)) !== null) {
    const classString = match[1];
    if (classString) {
      classNames.push({
        line: content.substring(0, match.index).split('\n').length,
        classes: classString.split(/\s+/).filter(c => c.trim()),
        raw: match[0],
      });
    }
  }
  
  while ((match = templateRegex.exec(content)) !== null) {
    const classString = match[1];
    if (classString) {
      const staticClasses = classString
        .split(/\$\{.*?\}/g)
        .join(' ')
        .split(/\s+/)
        .filter(c => c.trim() && !c.includes('${'));
      
      if (staticClasses.length > 0) {
        classNames.push({
          line: content.substring(0, match.index).split('\n').length,
          classes: staticClasses,
          raw: match[0],
          isTemplate: true,
        });
      }
    }
  }
  
  return classNames;
}

function extractTailwindClasses(classString) {
  const classes = classString.split(/\s+/).filter(c => c.trim());
  const tailwindClasses = [];
  
  const patterns = [
    /^(p|m|px|py|pt|pb|pl|pr|mx|my|mt|mb|ml|mr|gap|space-[xy])-[0-9]+$/,
    /^(w|h|min-w|min-h|max-w|max-h|size)-/,
    /^(flex|grid|block|inline|hidden)/,
    /^(flex-col|flex-row|flex-wrap|items-|justify-|content-|self-|place-)/,
    /^(grid-cols|grid-rows|col-span|row-span|auto-rows|auto-cols)/,
    /^(bg-|text-|border-|rounded)/,
    /^(text-\[|bg-\[|border-\[)/,
    /^(text-|font-|leading-|tracking-|uppercase|lowercase)/,
    /^(opacity-|z-|overflow-|cursor-|pointer-events-|select-)/,
    /^(transition-|duration-|ease-|delay-|animate-)/,
    /^(hover:|focus:|active:|disabled:)/,
    /^(absolute|relative|fixed|sticky|static)/,
    /^(top-|right-|bottom-|left-|inset-)/,
    /^(border|rounded|shadow)/,
    /^(whitespace-|break-|truncate)/,
    /^(shrink-|grow|basis-)/,
    /^(min-|max-)/,
  ];
  
  classes.forEach(cls => {
    if (patterns.some(pattern => pattern.test(cls))) {
      tailwindClasses.push(cls);
    }
  });
  
  return tailwindClasses;
}

function findFile(dir, filePath) {
  const fullPath = path.join(dir, filePath);
  if (fs.existsSync(fullPath)) {
    return fullPath;
  }
  
  // Try recursive search
  try {
    const parts = filePath.split('/');
    const fileName = parts[parts.length - 1];
    const files = fs.readdirSync(dir, { withFileTypes: true, recursive: true });
    for (const file of files) {
      if (file.isFile() && file.name === fileName) {
        return path.join(file.path, file.name);
      }
    }
  } catch (e) {
    // Ignore errors
  }
  
  return null;
}

function comparePage(pageConfig) {
  const { name, oldPath, newPath } = pageConfig;
  
  const oldFile = oldPath ? findFile(OLD_TAILWIND_DIR, oldPath) : null;
  const newFile = findFile(SCSS_VERSION_DIR, newPath);
  
  if (!oldFile && oldPath) {
    return {
      name,
      status: 'not-found-old',
      oldFile: null,
      newFile: newFile || 'NOT FOUND',
    };
  }
  
  if (!newFile) {
    return {
      name,
      status: 'not-found-new',
      oldFile: oldFile || 'NOT FOUND',
      newFile: null,
    };
  }
  
  const oldClassNames = oldFile ? extractClassNames(oldFile) : [];
  const newClassNames = extractClassNames(newFile);
  
  // Extract all Tailwind classes from old version
  const allTailwindClasses = new Set();
  oldClassNames.forEach(({ classes }) => {
    classes.forEach(cls => {
      const tailwindClasses = extractTailwindClasses(cls);
      tailwindClasses.forEach(tc => allTailwindClasses.add(tc));
    });
  });
  
  // Check which classes are used in new version
  const newContent = fs.readFileSync(newFile, 'utf-8');
  
  // Check for SCSS module import
  const hasSCSSModule = /import\s+styles\s+from\s+['"].*\.module\.scss['"]/.test(newContent);
  
  // Check for remaining Tailwind classes
  const remainingTailwindClasses = [];
  newClassNames.forEach(({ classes }) => {
    classes.forEach(cls => {
      const tailwindClasses = extractTailwindClasses(cls);
      tailwindClasses.forEach(tc => {
        if (!remainingTailwindClasses.includes(tc)) {
          remainingTailwindClasses.push(tc);
        }
      });
    });
  });
  
  return {
    name,
    status: 'compared',
    oldFile,
    newFile,
    totalTailwindClasses: allTailwindClasses.size,
    remainingTailwindClasses: remainingTailwindClasses.length,
    remainingClasses: remainingTailwindClasses,
    hasSCSSModule,
    oldClassNameCount: oldClassNames.length,
    newClassNameCount: newClassNames.length,
  };
}

function compareAllPages() {
  console.log('🔍 Comparing all pages between Tailwind and SCSS versions...\n');
  
  const results = {
    pages: {},
    summary: {
      total: 0,
      compared: 0,
      notFoundOld: 0,
      notFoundNew: 0,
      withRemainingTailwind: 0,
    },
  };
  
  for (const pageConfig of PAGES_TO_COMPARE) {
    console.log(`Checking ${pageConfig.name}...`);
    const result = comparePage(pageConfig);
    results.pages[pageConfig.name] = result;
    
    if (result.status === 'compared') {
      results.summary.compared++;
      results.summary.total += result.totalTailwindClasses;
      
      if (result.remainingTailwindClasses > 0) {
        results.summary.withRemainingTailwind++;
        console.log(`  ⚠️  Found ${result.remainingTailwindClasses} remaining Tailwind classes:`);
        result.remainingClasses.slice(0, 10).forEach(cls => {
          console.log(`     - ${cls}`);
        });
        if (result.remainingClasses.length > 10) {
          console.log(`     ... and ${result.remainingClasses.length - 10} more`);
        }
      } else {
        console.log(`  ✅ All classes converted (${result.totalTailwindClasses} classes)`);
      }
      
      if (!result.hasSCSSModule) {
        console.log(`  ⚠️  No SCSS module found`);
      }
    } else if (result.status === 'not-found-old') {
      results.summary.notFoundOld++;
      console.log(`  ⚠️  Old Tailwind version not found`);
    } else if (result.status === 'not-found-new') {
      results.summary.notFoundNew++;
      console.log(`  ⚠️  SCSS version not found`);
    }
  }
  
  console.log('\n' + '='.repeat(60));
  console.log('SUMMARY');
  console.log('='.repeat(60));
  console.log(`Pages compared: ${results.summary.compared}`);
  console.log(`Pages not found (old): ${results.summary.notFoundOld}`);
  console.log(`Pages not found (new): ${results.summary.notFoundNew}`);
  console.log(`Pages with remaining Tailwind: ${results.summary.withRemainingTailwind}`);
  console.log(`Total Tailwind classes found: ${results.summary.total}`);
  
  // Write detailed report
  const reportPath = path.join(__dirname, '../ALL_PAGES_COMPARISON_REPORT.md');
  const report = generateReport(results);
  fs.writeFileSync(reportPath, report);
  console.log(`\n📄 Detailed report written to: ${reportPath}`);
  
  return results;
}

function generateReport(results) {
  let report = `# All Pages Comparison Report

Generated: ${new Date().toISOString()}

## Summary

- **Pages Compared**: ${results.summary.compared}
- **Pages Not Found (Old)**: ${results.summary.notFoundOld}
- **Pages Not Found (New)**: ${results.summary.notFoundNew}
- **Pages with Remaining Tailwind**: ${results.summary.withRemainingTailwind}
- **Total Tailwind Classes Found**: ${results.summary.total}

## Page Details

`;

  for (const [pageName, data] of Object.entries(results.pages)) {
    report += `### ${pageName}\n\n`;
    report += `- **Status**: ${data.status}\n`;
    report += `- **Old File**: \`${data.oldFile || 'NOT FOUND'}\`\n`;
    report += `- **New File**: \`${data.newFile || 'NOT FOUND'}\`\n`;
    
    if (data.status === 'compared') {
      report += `- **Total Tailwind Classes**: ${data.totalTailwindClasses}\n`;
      report += `- **Remaining Tailwind Classes**: ${data.remainingTailwindClasses}\n`;
      report += `- **Has SCSS Module**: ${data.hasSCSSModule ? 'Yes' : 'No'}\n`;
      report += `- **Old className Count**: ${data.oldClassNameCount}\n`;
      report += `- **New className Count**: ${data.newClassNameCount}\n\n`;
      
      if (data.remainingClasses && data.remainingClasses.length > 0) {
        report += `#### Remaining Tailwind Classes:\n\n`;
        data.remainingClasses.forEach(cls => {
          report += `- \`${cls}\`\n`;
        });
        report += '\n';
      }
    }
    
    report += '\n';
  }
  
  return report;
}

// Run comparison
if (require.main === module) {
  compareAllPages();
}

module.exports = { compareAllPages };

