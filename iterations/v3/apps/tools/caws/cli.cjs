#!/usr/bin/env node
// Minimal CAWS CLI providing `init` and `scaffold` with --debug support.
// Purpose: Unblock tests that invoke `caws scaffold` and need visible errors.

const fs = require('fs');
const path = require('path');

function logDebug(enabled, ...args) {
  if (enabled) console.log('[CAWS-DEBUG]', ...args);
}

function copyDir(src, dest, debug) {
  if (!fs.existsSync(src)) {
    throw new Error(`Template directory not found: ${src}`);
  }
  if (!fs.existsSync(dest)) {
    fs.mkdirSync(dest, { recursive: true });
    logDebug(debug, 'Created directory', dest);
  }
  for (const entry of fs.readdirSync(src)) {
    const s = path.join(src, entry);
    const d = path.join(dest, entry);
    const stat = fs.statSync(s);
    if (stat.isDirectory()) {
      copyDir(s, d, debug);
    } else {
      fs.copyFileSync(s, d);
      logDebug(debug, 'Copied', s, '->', d);
    }
  }
}

function cmdInit({ projectDir, debug }) {
  const cawsDir = path.join(projectDir, '.caws');
  if (!fs.existsSync(cawsDir)) fs.mkdirSync(cawsDir, { recursive: true });
  const wsPath = path.join(cawsDir, 'working-spec.yaml');
  if (!fs.existsSync(wsPath)) {
    fs.writeFileSync(
      wsPath,
      `id: FEAT-000\n` +
        `title: "New Feature"\n` +
        `risk_tier: 2\n` +
        `mode: feature\n` +
        `change_budget: { max_files: 25, max_loc: 1000 }\n` +
        `scope: { in: ["src/"], out: ["node_modules/"] }\n` +
        `acceptance: []\n`
    );
    logDebug(debug, 'Wrote working spec at', wsPath);
  } else {
    logDebug(debug, 'Working spec already exists at', wsPath);
  }
  console.log('✅ CAWS init complete');
}

function cmdScaffold({ projectDir, debug }) {
  const templatesDir = path.join(__dirname, 'templates', 'basic');
  const targetDir = projectDir;
  const manifestPath = path.join(projectDir, '.scaffold-manifest.json');

  try {
    // Read existing manifest or create empty one
    let manifest = {};
    if (fs.existsSync(manifestPath)) {
      manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    }

    // Track scaffolded files
    if (!manifest.scaffolded) manifest.scaffolded = [];
    const scaffoldedFiles = [];

    // Copy template files and track them
    function copyAndTrack(src, dest) {
      if (!fs.existsSync(src)) {
        throw new Error(`Template directory not found: ${src}`);
      }
      if (!fs.existsSync(dest)) {
        fs.mkdirSync(dest, { recursive: true });
        logDebug(debug, 'Created directory', dest);
      }
      for (const entry of fs.readdirSync(src)) {
        const s = path.join(src, entry);
        const d = path.join(dest, entry);
        const stat = fs.statSync(s);
        if (stat.isDirectory()) {
          copyAndTrack(s, d);
        } else {
          // Check if file already exists
          if (fs.existsSync(d)) {
            logDebug(debug, 'Skipping existing file', d);
            continue;
          }
          fs.copyFileSync(s, d);
          const relativePath = path.relative(projectDir, d);
          scaffoldedFiles.push(relativePath);
          logDebug(debug, 'Scaffolded', s, '->', d);
        }
      }
    }

    copyAndTrack(templatesDir, targetDir);

    // Update manifest
    manifest.scaffolded = [...new Set([...(manifest.scaffolded || []), ...scaffoldedFiles])];
    fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
    logDebug(debug, 'Updated scaffold manifest with', scaffoldedFiles.length, 'files');

    console.log('✅ CAWS scaffold complete -', scaffoldedFiles.length, 'files scaffolded');
  } catch (err) {
    console.error('❌ CAWS scaffold failed:', err.message);
    if (debug) console.error(err.stack);
    process.exitCode = 1;
  }
}

function cmdUnsaffold({ projectDir, debug }) {
  const manifestPath = path.join(projectDir, '.scaffold-manifest.json');

  try {
    if (!fs.existsSync(manifestPath)) {
      console.log('ℹ️ No scaffold manifest found - nothing to unsaffold');
      return;
    }

    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    const scaffoldedFiles = manifest.scaffolded || [];

    let removedCount = 0;
    for (const relativePath of scaffoldedFiles) {
      const fullPath = path.join(projectDir, relativePath);
      if (fs.existsSync(fullPath)) {
        fs.unlinkSync(fullPath);
        logDebug(debug, 'Removed scaffolded file', fullPath);
        removedCount++;
      }
    }

    // Remove empty directories created by scaffolding
    function removeEmptyDirs(dir) {
      if (!fs.existsSync(dir)) return;
      const entries = fs.readdirSync(dir);
      for (const entry of entries) {
        const fullPath = path.join(dir, entry);
        if (fs.statSync(fullPath).isDirectory()) {
          removeEmptyDirs(fullPath);
        }
      }
      // Check if directory is empty after recursive removal
      if (fs.readdirSync(dir).length === 0) {
        fs.rmdirSync(dir);
        logDebug(debug, 'Removed empty scaffolded directory', dir);
      }
    }

    // Clean up empty directories in templates
    const templateDirs = ['src', 'tests', 'benches'];
    for (const templateDir of templateDirs) {
      const fullDir = path.join(projectDir, templateDir);
      if (fs.existsSync(fullDir)) {
        removeEmptyDirs(fullDir);
      }
    }

    // Remove manifest if all files were removed
    if (removedCount === scaffoldedFiles.length) {
      fs.unlinkSync(manifestPath);
      console.log('✅ CAWS unsaffold complete - all scaffolded files removed');
    } else {
      // Update manifest with remaining files
      manifest.scaffolded = scaffoldedFiles.filter(relativePath => {
        const fullPath = path.join(projectDir, relativePath);
        return fs.existsSync(fullPath);
      });
      fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
      console.log('✅ CAWS unsaffold complete -', removedCount, 'files removed,', manifest.scaffolded.length, 'files remain');
    }
  } catch (err) {
    console.error('❌ CAWS unsaffold failed:', err.message);
    if (debug) console.error(err.stack);
    process.exitCode = 1;
  }
}

function main() {
  const [, , command, ...rest] = process.argv;
  const debug = rest.includes('--debug');
  const projectDirIdx = rest.findIndex((a) => a === '--dir');
  const projectDir = projectDirIdx !== -1 && rest[projectDirIdx + 1]
    ? path.resolve(rest[projectDirIdx + 1])
    : process.cwd();

  switch (command) {
    case 'init':
      cmdInit({ projectDir, debug });
      break;
    case 'scaffold':
      cmdScaffold({ projectDir, debug });
      break;
    case 'unsaffold':
      cmdUnsaffold({ projectDir, debug });
      break;
    case 'help':
    default:
      console.log(`CAWS CLI (minimal)
Usage:
  node cli.js init [--dir <path>] [--debug]
  node cli.js scaffold [--dir <path>] [--debug]
  node cli.js unsaffold [--dir <path>] [--debug]
`);
  }
}

main();

