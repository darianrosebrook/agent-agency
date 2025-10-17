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
  try {
    copyDir(templatesDir, targetDir, debug);
    console.log('✅ CAWS scaffold complete');
  } catch (err) {
    console.error('❌ CAWS scaffold failed:', err.message);
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
    case 'help':
    default:
      console.log(`CAWS CLI (minimal)
Usage:
  node cli.js init [--dir <path>] [--debug]
  node cli.js scaffold [--dir <path>] [--debug]
`);
  }
}

main();

