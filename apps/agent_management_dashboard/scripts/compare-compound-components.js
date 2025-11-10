#!/usr/bin/env node

/**
 * Compound Components Comparison Script
 * 
 * Compares compound components between old Tailwind version and new SCSS version,
 * mapping Tailwind classes to SCSS equivalents.
 * 
 * Usage: node scripts/compare-compound-components.js [ComponentName]
 */

const fs = require('fs');
const path = require('path');

// Component name from command line argument
const componentName = process.argv[2];

if (!componentName) {
  console.error('Usage: node scripts/compare-compound-components.js ComponentName');
  process.exit(1);
}

const oldVersionPath = path.join(__dirname, '../../old_tailwind_version/src/components');
const newVersionPath = path.join(__dirname, '../src/components/compounds');

// Handle special cases for file locations
const componentFileMap = {
  'ImageWithFallback': {
    old: path.join(oldVersionPath, 'figma/ImageWithFallback.tsx'),
    new: path.join(newVersionPath, 'ImageWithFallback.tsx')
  }
};

function getComponentPaths(name) {
  if (componentFileMap[name]) {
    return componentFileMap[name];
  }
  
  return {
    old: path.join(oldVersionPath, `${name}.tsx`),
    new: path.join(newVersionPath, `${name}.tsx`)
  };
}

function extractTailwindClasses(content) {
  const classes = new Set();
  
  // Match className="..." or className='...'
  const classNameRegex = /className\s*=\s*["']([^"']+)["']/g;
  let match;
  
  while ((match = classNameRegex.exec(content)) !== null) {
    const classString = match[1];
    // Split by spaces and filter out empty strings
    const classList = classString.split(/\s+/).filter(c => c.trim());
    classList.forEach(cls => {
      // Filter out template literals and function calls
      if (!cls.includes('${') && !cls.includes('`') && cls.trim()) {
        classes.add(cls.trim());
      }
    });
  }
  
  // Also match template literals with cn() or similar
  const cnRegex = /cn\([^)]*["']([^"']+)["']/g;
  while ((match = cnRegex.exec(content)) !== null) {
    const classString = match[1];
    classString.split(/\s+/).forEach(cls => {
      if (cls.trim()) classes.add(cls.trim());
    });
  }
  
  return Array.from(classes).sort();
}

function extractSCSSClasses(content) {
  const classes = new Set();
  
  // Match styles.className or styles['className']
  const stylesRegex = /styles\.(\w+)/g;
  let match;
  
  while ((match = stylesRegex.exec(content)) !== null) {
    classes.add(match[1]);
  }
  
  return Array.from(classes).sort();
}

function readSCSSModule(componentName) {
  const scssPath = path.join(newVersionPath, `${componentName}.module.scss`);
  
  if (!fs.existsSync(scssPath)) {
    return null;
  }
  
  return fs.readFileSync(scssPath, 'utf-8');
}

function parseSCSSProperties(scssContent) {
  const properties = {};
  
  // Match class definitions and their properties
  const classRegex = /\.(\w+)\s*\{([^}]+)\}/g;
  let match;
  
  while ((match = classRegex.exec(scssContent)) !== null) {
    const className = match[1];
    const propertiesBlock = match[2];
    
    // Extract individual properties
    const props = {};
    const propRegex = /(\w+(?:-\w+)*)\s*:\s*([^;]+);/g;
    let propMatch;
    
    while ((propMatch = propRegex.exec(propertiesBlock)) !== null) {
      props[propMatch[1]] = propMatch[2].trim();
    }
    
    properties[className] = props;
  }
  
  return properties;
}

function mapTailwindToSCSS(tailwindClass) {
  // Common Tailwind to SCSS mappings
  const mappings = {
    // Spacing
    'p-0': 'padding: $spacing-0',
    'p-1': 'padding: $spacing-1',
    'p-2': 'padding: $spacing-2',
    'p-3': 'padding: $spacing-3',
    'p-4': 'padding: $spacing-4',
    'p-5': 'padding: $spacing-5',
    'p-6': 'padding: $spacing-6',
    'p-8': 'padding: $spacing-8',
    'px-4': 'padding-inline: $spacing-4',
    'py-2': 'padding-block: $spacing-2',
    'm-0': 'margin: $spacing-0',
    'mb-2': 'margin-bottom: $spacing-2',
    'mb-4': 'margin-bottom: $spacing-4',
    'mb-6': 'margin-bottom: $spacing-6',
    'mb-8': 'margin-bottom: $spacing-8',
    'ml-12': 'margin-left: $spacing-12',
    'gap-2': 'gap: $spacing-2',
    'gap-3': 'gap: $spacing-3',
    'gap-4': 'gap: $spacing-4',
    
    // Colors
    'bg-gray-900': 'background-color: $color-gray-900',
    'bg-gray-800': 'background-color: $color-gray-800',
    'bg-gray-700': 'background-color: $color-gray-700',
    'bg-gray-100': 'background-color: $color-gray-100',
    'bg-zinc-800': 'background-color: $color-zinc-800',
    'bg-zinc-700': 'background-color: $color-zinc-700',
    'bg-[#111111]': 'background-color: $color-gray-900',
    'bg-[#1a1a1a]': 'background-color: $color-dark-bg-primary',
    'text-white': 'color: $color-white',
    'text-gray-400': 'color: $color-gray-400',
    'text-gray-500': 'color: $color-gray-500',
    'text-gray-600': 'color: $color-gray-600',
    'text-gray-700': 'color: $color-gray-700',
    'text-zinc-300': 'color: $color-zinc-300',
    'text-zinc-400': 'color: $color-zinc-400',
    
    // Layout
    'flex': 'display: flex',
    'flex-col': 'flex-direction: column',
    'items-center': 'align-items: center',
    'justify-between': 'justify-content: space-between',
    'justify-center': 'justify-content: center',
    'relative': 'position: relative',
    'absolute': 'position: absolute',
    'w-full': 'width: 100%',
    'h-full': 'height: 100%',
    'size-full': 'width: 100%; height: 100%',
    
    // Borders
    'border': 'border: 1px solid',
    'border-[#cacaca]': 'border-color: $color-gray-300',
    'rounded-lg': 'border-radius: $spacing-lg',
    'rounded-[12px]': 'border-radius: 0.75rem',
    'rounded-full': 'border-radius: 9999px',
    
    // Typography
    'text-sm': 'font-size: $font-size-sm',
    'text-xs': 'font-size: $font-size-xs',
    'font-medium': 'font-weight: $font-weight-medium',
    'font-semibold': 'font-weight: $font-weight-semibold',
  };
  
  return mappings[tailwindClass] || `Unknown: ${tailwindClass}`;
}

function compareComponent(componentName) {
  const paths = getComponentPaths(componentName);
  
  console.log(`\n=== ${componentName} Comparison ===\n`);
  
  // Check if files exist
  if (!fs.existsSync(paths.old)) {
    console.log(`⚠️  Old version not found: ${paths.old}`);
    return;
  }
  
  if (!fs.existsSync(paths.new)) {
    console.log(`⚠️  New version not found: ${paths.new}`);
    return;
  }
  
  // Read files
  const oldContent = fs.readFileSync(paths.old, 'utf-8');
  const newContent = fs.readFileSync(paths.new, 'utf-8');
  
  // Extract classes
  const tailwindClasses = extractTailwindClasses(oldContent);
  const scssModuleClasses = extractSCSSClasses(newContent);
  
  // Read SCSS module
  const scssModuleContent = readSCSSModule(componentName);
  const scssProperties = scssModuleContent ? parseSCSSProperties(scssModuleContent) : {};
  
  console.log('Old Version (Tailwind):');
  console.log(`  File: ${paths.old}`);
  console.log(`  Classes found: ${tailwindClasses.length}`);
  if (tailwindClasses.length > 0) {
    tailwindClasses.forEach(cls => {
      const scssEquivalent = mapTailwindToSCSS(cls);
      console.log(`    - ${cls} → ${scssEquivalent}`);
    });
  }
  
  console.log('\nNew Version (SCSS):');
  console.log(`  File: ${paths.new}`);
  console.log(`  SCSS module classes found: ${scssModuleClasses.length}`);
  if (scssModuleClasses.length > 0) {
    scssModuleClasses.forEach(cls => {
      const props = scssProperties[cls] || {};
      const propsStr = Object.keys(props).length > 0 
        ? Object.entries(props).map(([k, v]) => `${k}: ${v}`).join('; ')
        : 'No properties';
      console.log(`    - .${cls} → ${propsStr}`);
    });
  }
  
  // Check for Tailwind classes in new version
  const tailwindInNew = extractTailwindClasses(newContent);
  if (tailwindInNew.length > 0) {
    console.log('\n⚠️  WARNING: Tailwind classes found in new version:');
    tailwindInNew.forEach(cls => {
      console.log(`    - ${cls}`);
    });
  }
  
  // Parity assessment
  console.log('\nParity Status:');
  if (tailwindInNew.length === 0 && scssModuleClasses.length > 0) {
    console.log('  ✅ COMPLETE - All classes converted to SCSS');
  } else if (tailwindInNew.length > 0) {
    console.log('  ⚠️  INCOMPLETE - Tailwind classes still present in new version');
  } else {
    console.log('  ⚠️  REVIEW NEEDED - No SCSS classes found');
  }
  
  console.log('\n' + '='.repeat(50));
}

// Run comparison
compareComponent(componentName);

