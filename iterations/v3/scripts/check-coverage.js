#!/usr/bin/env node
// Parse lcov.info and enforce a minimum branch coverage threshold.
const fs = require('fs');
const path = require('path');

const lcovPath = path.join(__dirname, '..', 'target', 'coverage', 'lcov.info');
const min = parseFloat(process.env.COVERAGE_MIN || '0.80');

if (!fs.existsSync(lcovPath)) {
  console.error(`[coverage] Missing lcov.info at ${lcovPath}`);
  process.exit(2);
}

const data = fs.readFileSync(lcovPath, 'utf8');
let brHit = 0;
let brMiss = 0;
for (const line of data.split('\n')) {
  if (line.startsWith('BRH:')) brHit += parseInt(line.slice(4), 10) || 0;
  if (line.startsWith('BRF:')) brMiss += parseInt(line.slice(4), 10) || 0;
}
const total = brHit + brMiss;
const pct = total > 0 ? brHit / total : 0;
console.log(`[coverage] Branch coverage: ${(pct*100).toFixed(2)}% (min ${(min*100).toFixed(0)}%)`);
if (pct + 1e-9 < min) {
  console.error(`[coverage] FAIL: ${(pct*100).toFixed(2)}% < ${(min*100).toFixed(0)}%`);
  process.exit(1);
}
console.log('[coverage] PASS');

