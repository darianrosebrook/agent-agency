#!/usr/bin/env node

/**
 * Comparison script to identify missing Tailwind class conversions in SCSS version
 * Compares Dashboard components and their children between Tailwind and SCSS versions
 * 
 * @author @darianrosebrook
 */

const fs = require('fs');
const path = require('path');

const OLD_TAILWIND_DIR = path.join(__dirname, '../../old_tailwind_version/src/components');
const SCSS_VERSION_DIR = path.join(__dirname, '../src/components');

// Components to compare (Dashboard and its children)
const COMPONENTS_TO_COMPARE = [
  'Dashboard',
  'TaskProgressChart',
  'RadialTaskProgress',
  'MultiRingProgress',
  'CodeContributionChart',
  'HexagonHeatmap',
  'ModelContributionStream',
  'TaskCompletionGauge',
  'ServerEfficiencyChart',
  'BentoPanel',
];

/**
 * Extract all className attributes and their values from a file
 */
function extractClassNames(filePath) {
  if (!fs.existsSync(filePath)) {
    return [];
  }

  const content = fs.readFileSync(filePath, 'utf-8');
  const classNames = [];
  
  // Match className="..." or className={`...`}
  const classNameRegex = /className\s*=\s*{?["'`]([^"'`]+)["'`]}?/g;
  const templateRegex = /className\s*=\s*{`([^`]+)`}/g;
  
  let match;
  
  // Regular className="..." or className={'...'}
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
  
  // Template literals className={`...`}
  while ((match = templateRegex.exec(content)) !== null) {
    const classString = match[1];
    if (classString) {
      // Extract static classes (ignore template expressions for now)
      const staticClasses = classString
        .split(/\$\{.*?\}/g) // Remove template expressions
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

/**
 * Find all Tailwind utility classes in a class string
 */
function extractTailwindClasses(classString) {
  const classes = classString.split(/\s+/).filter(c => c.trim());
  const tailwindClasses = [];
  
  // Common Tailwind patterns
  const patterns = [
    /^(p|m|px|py|pt|pb|pl|pr|mx|my|mt|mb|ml|mr|gap|space-[xy])-[0-9]+$/, // Spacing
    /^(w|h|min-w|min-h|max-w|max-h|size)-/, // Sizing
    /^(flex|grid|block|inline|hidden)/, // Display
    /^(flex-col|flex-row|flex-wrap|items-|justify-|content-|self-|place-)/, // Flexbox
    /^(grid-cols|grid-rows|col-span|row-span|auto-rows|auto-cols)/, // Grid
    /^(bg-|text-|border-|rounded)/, // Colors, borders
    /^(text-\[|bg-\[|border-\[)/, // Arbitrary values
    /^(text-|font-|leading-|tracking-|uppercase|lowercase)/, // Typography
    /^(opacity-|z-|overflow-|cursor-|pointer-events-|select-)/, // Effects
    /^(transition-|duration-|ease-|delay-|animate-)/, // Transitions
    /^(hover:|focus:|active:|disabled:)/, // Pseudo-classes
    /^(absolute|relative|fixed|sticky|static)/, // Position
    /^(top-|right-|bottom-|left-|inset-)/, // Position values
    /^(border|rounded|shadow)/, // Borders and shadows
    /^(whitespace-|break-|truncate)/, // Text utilities
    /^(shrink-|grow|basis-)/, // Flex utilities
    /^(min-|max-)/, // Min/max utilities
  ];
  
  classes.forEach(cls => {
    if (patterns.some(pattern => pattern.test(cls))) {
      tailwindClasses.push(cls);
    }
  });
  
  return tailwindClasses;
}

/**
 * Find component file in directory
 */
function findComponentFile(dir, componentName) {
  const possiblePaths = [
    path.join(dir, `${componentName}.tsx`),
    path.join(dir, `${componentName}.ts`),
    path.join(dir, componentName.toLowerCase(), `${componentName}.tsx`),
    path.join(dir, componentName.toLowerCase(), 'index.tsx'),
  ];
  
  for (const filePath of possiblePaths) {
    if (fs.existsSync(filePath)) {
      return filePath;
    }
  }
  
  // Try recursive search
  try {
    const files = fs.readdirSync(dir, { withFileTypes: true, recursive: true });
    for (const file of files) {
      if (file.isFile() && file.name === `${componentName}.tsx`) {
        return path.join(file.path, file.name);
      }
    }
  } catch (e) {
    // Ignore errors
  }
  
  return null;
}

/**
 * Check if SCSS module exists for component
 */
function findSCSSModule(componentName) {
  const possiblePaths = [
    path.join(SCSS_VERSION_DIR, `${componentName}.module.scss`),
    path.join(SCSS_VERSION_DIR, componentName.toLowerCase(), `${componentName}.module.scss`),
    path.join(SCSS_VERSION_DIR, 'dashboard', `${componentName}.module.scss`),
    path.join(SCSS_VERSION_DIR, 'compounds', `${componentName}.module.scss`),
  ];
  
  for (const filePath of possiblePaths) {
    if (fs.existsSync(filePath)) {
      return filePath;
    }
  }
  
  // Try recursive search
  try {
    const files = fs.readdirSync(SCSS_VERSION_DIR, { withFileTypes: true, recursive: true });
    for (const file of files) {
      if (file.isFile() && file.name === `${componentName}.module.scss`) {
        return path.join(file.path, file.name);
      }
    }
  } catch (e) {
    // Ignore errors
  }
  
  return null;
}

/**
 * Extract SCSS class names from module
 */
function extractSCSSClasses(scssPath) {
  if (!fs.existsSync(scssPath)) {
    return [];
  }
  
  const content = fs.readFileSync(scssPath, 'utf-8');
  const classes = [];
  
  // Match class definitions: .className { or .className,
  const classRegex = /\.([a-zA-Z][a-zA-Z0-9_-]*)\s*[,\{]/g;
  let match;
  
  while ((match = classRegex.exec(content)) !== null) {
    classes.push(match[1]);
  }
  
  return classes;
}

/**
 * Main comparison function
 */
function compareComponents() {
  const results = {
    components: {},
    summary: {
      total: 0,
      found: 0,
      missing: 0,
      missingClasses: [],
    },
  };
  
  console.log('🔍 Comparing Tailwind vs SCSS components...\n');
  
  for (const componentName of COMPONENTS_TO_COMPARE) {
    console.log(`Checking ${componentName}...`);
    
    const oldFile = findComponentFile(OLD_TAILWIND_DIR, componentName);
    const scssFile = findComponentFile(SCSS_VERSION_DIR, componentName);
    const scssModule = findSCSSModule(componentName);
    
    if (!oldFile) {
      console.log(`  ⚠️  Old Tailwind version not found: ${componentName}`);
      continue;
    }
    
    const oldClassNames = extractClassNames(oldFile);
    const scssClassNames = extractClassNames(scssFile);
    const scssModuleClasses = scssModule ? extractSCSSClasses(scssModule) : [];
    
    // Extract all Tailwind classes from old version
    const allTailwindClasses = new Set();
    oldClassNames.forEach(({ classes }) => {
      classes.forEach(cls => {
        const tailwindClasses = extractTailwindClasses(cls);
        tailwindClasses.forEach(tc => allTailwindClasses.add(tc));
      });
    });
    
    // Check which classes are used in SCSS version
    const scssContent = scssFile ? fs.readFileSync(scssFile, 'utf-8') : '';
    const scssModuleContent = scssModule ? fs.readFileSync(scssModule, 'utf-8') : '';
    
    const foundClasses = [];
    const missingClasses = [];
    
    allTailwindClasses.forEach(twClass => {
      // Check if class is referenced in SCSS file (as styles.className)
      const classNamePattern = twClass.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      const scssRefPattern = new RegExp(`styles\\.\\w+|className.*${classNamePattern}`, 'i');
      
      if (scssRefPattern.test(scssContent) || scssRefPattern.test(scssModuleContent)) {
        foundClasses.push(twClass);
      } else {
        missingClasses.push(twClass);
        results.summary.missingClasses.push({
          component: componentName,
          class: twClass,
        });
      }
    });
    
    results.components[componentName] = {
      oldFile,
      scssFile: scssFile || 'NOT FOUND',
      scssModule: scssModule || 'NOT FOUND',
      totalTailwindClasses: allTailwindClasses.size,
      foundClasses: foundClasses.length,
      missingClasses: missingClasses.length,
      missing: missingClasses,
      scssModuleClasses: scssModuleClasses.length,
    };
    
    results.summary.total += allTailwindClasses.size;
    results.summary.found += foundClasses.length;
    results.summary.missing += missingClasses.length;
    
    if (missingClasses.length > 0) {
      console.log(`  ⚠️  Missing ${missingClasses.length} Tailwind classes:`);
      missingClasses.slice(0, 10).forEach(cls => {
        console.log(`     - ${cls}`);
      });
      if (missingClasses.length > 10) {
        console.log(`     ... and ${missingClasses.length - 10} more`);
      }
    } else {
      console.log(`  ✅ All classes converted`);
    }
  }
  
  console.log('\n' + '='.repeat(60));
  console.log('SUMMARY');
  console.log('='.repeat(60));
  console.log(`Total Tailwind classes found: ${results.summary.total}`);
  console.log(`Classes found in SCSS: ${results.summary.found}`);
  console.log(`Missing classes: ${results.summary.missing}`);
  console.log(`Coverage: ${((results.summary.found / results.summary.total) * 100).toFixed(1)}%`);
  
  // Write detailed report
  const reportPath = path.join(__dirname, '../TAILWIND_SCSS_COMPARISON_REPORT.md');
  const report = generateReport(results);
  fs.writeFileSync(reportPath, report);
  console.log(`\n📄 Detailed report written to: ${reportPath}`);
  
  return results;
}

/**
 * Generate markdown report
 */
function generateReport(results) {
  let report = `# Tailwind to SCSS Conversion Comparison Report

Generated: ${new Date().toISOString()}

## Summary

- **Total Tailwind Classes**: ${results.summary.total}
- **Classes Found in SCSS**: ${results.summary.found}
- **Missing Classes**: ${results.summary.missing}
- **Coverage**: ${((results.summary.found / results.summary.total) * 100).toFixed(1)}%

## Component Details

`;

  for (const [componentName, data] of Object.entries(results.components)) {
    report += `### ${componentName}\n\n`;
    report += `- **Old Tailwind File**: \`${data.oldFile}\`\n`;
    report += `- **SCSS Component File**: \`${data.scssFile}\`\n`;
    report += `- **SCSS Module File**: \`${data.scssModule}\`\n`;
    report += `- **Total Tailwind Classes**: ${data.totalTailwindClasses}\n`;
    report += `- **Found in SCSS**: ${data.foundClasses}\n`;
    report += `- **Missing**: ${data.missingClasses}\n`;
    report += `- **SCSS Module Classes**: ${data.scssModuleClasses}\n\n`;
    
    if (data.missing.length > 0) {
      report += `#### Missing Tailwind Classes:\n\n`;
      data.missing.forEach(cls => {
        report += `- \`${cls}\`\n`;
      });
      report += '\n';
    }
  }
  
  report += `## Missing Classes by Component\n\n`;
  
  const missingByComponent = {};
  results.summary.missingClasses.forEach(({ component, class: cls }) => {
    if (!missingByComponent[component]) {
      missingByComponent[component] = [];
    }
    missingByComponent[component].push(cls);
  });
  
  for (const [component, classes] of Object.entries(missingByComponent)) {
    report += `### ${component}\n\n`;
    const uniqueClasses = [...new Set(classes)];
    uniqueClasses.forEach(cls => {
      report += `- \`${cls}\`\n`;
    });
    report += '\n';
  }
  
  return report;
}

// Run comparison
if (require.main === module) {
  compareComponents();
}

module.exports = { compareComponents };

