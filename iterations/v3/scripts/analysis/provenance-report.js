#!/usr/bin/env node
// Emits a slim provenance report for CI artifacts.
const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const chainPath = path.join(root, '.caws', 'provenance', 'chain.json');
const outDir = path.join(root, 'target', 'provenance');
const outPath = path.join(outDir, 'report.json');
const limit = parseInt(process.env.PROVENANCE_REPORT_LIMIT || '50', 10);

function safeParse(json) {
  try { return JSON.parse(json); } catch { return null; }
}

if (!fs.existsSync(chainPath)) {
  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(outPath, JSON.stringify({ entries: [], warning: 'provenance chain not found' }, null, 2));
  console.warn(`[prov] No chain at ${chainPath}. Wrote empty report.`);
  process.exit(0);
}

const raw = fs.readFileSync(chainPath, 'utf8');
const data = safeParse(raw) || {};
const entries = Array.isArray(data.entries) ? data.entries : [];
const trimmed = entries.slice(-limit);

const summary = {
  total_entries: entries.length,
  included: trimmed.length,
  last_updated: data.lastUpdated || null,
  entries: trimmed,
};

fs.mkdirSync(outDir, { recursive: true });
fs.writeFileSync(outPath, JSON.stringify(summary, null, 2));
console.log(`[prov] Wrote report with ${trimmed.length}/${entries.length} entries to ${outPath}`);

