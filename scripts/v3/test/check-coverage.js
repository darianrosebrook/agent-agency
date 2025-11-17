#!/usr/bin/env node
// Parse lcov.info and enforce minimum line and branch coverage thresholds.
// Tier 2 requirements: 80% line coverage, 90% branch coverage
const fs = require('fs');
const path = require('path');

// Support both relative paths (from scripts/v3/test) and absolute paths
const lcovPath = process.env.LCOV_PATH || 
  (fs.existsSync(path.join(__dirname, '..', 'target', 'coverage', 'lcov.info')) 
    ? path.join(__dirname, '..', 'target', 'coverage', 'lcov.info')
    : path.join(__dirname, '..', '..', 'iterations', 'v3', 'target', 'coverage', 'lcov.info'));
const lineMin = parseFloat(process.env.LINE_COVERAGE_MIN || process.env.COVERAGE_MIN || '0.80');
const branchMin = parseFloat(process.env.BRANCH_COVERAGE_MIN || '0.90');

if (!fs.existsSync(lcovPath)) {
  console.error(`[coverage] Missing lcov.info at ${lcovPath}`);
  process.exit(2);
}

const data = fs.readFileSync(lcovPath, 'utf8');

// Parse line coverage
let lineHit = 0;
let lineMiss = 0;
// Parse branch coverage
let brHit = 0;
let brMiss = 0;

for (const line of data.split('\n')) {
  if (line.startsWith('LH:')) lineHit += parseInt(line.slice(3), 10) || 0;
  if (line.startsWith('LF:')) lineMiss += parseInt(line.slice(3), 10) || 0;
  if (line.startsWith('BRH:')) brHit += parseInt(line.slice(4), 10) || 0;
  if (line.startsWith('BRF:')) brMiss += parseInt(line.slice(4), 10) || 0;
}

const lineTotal = lineHit + lineMiss;
const linePct = lineTotal > 0 ? lineHit / lineTotal : 0;

const branchTotal = brHit + brMiss;
const branchPct = branchTotal > 0 ? brHit / branchTotal : 0;

console.log(`[coverage] Line coverage: ${(linePct*100).toFixed(2)}% (min ${(lineMin*100).toFixed(0)}%)`);
console.log(`[coverage] Branch coverage: ${(branchPct*100).toFixed(2)}% (min ${(branchMin*100).toFixed(0)}%)`);

let failed = false;

if (linePct + 1e-9 < lineMin) {
  console.error(`[coverage] FAIL: Line coverage ${(linePct*100).toFixed(2)}% < ${(lineMin*100).toFixed(0)}%`);
  failed = true;
}

if (branchPct + 1e-9 < branchMin) {
  console.error(`[coverage] FAIL: Branch coverage ${(branchPct*100).toFixed(2)}% < ${(branchMin*100).toFixed(0)}%`);
  failed = true;
}

if (failed) {
  process.exit(1);
}

console.log('[coverage] PASS - Both line and branch coverage thresholds met');

