#!/usr/bin/env node
// Simple schema validator for v3 contract examples using AJV
const fs = require('fs');
const path = require('path');
const Ajv = require('ajv').default;

const root = __dirname;
const schemas = {
  worker: path.join(root, 'worker-output.schema.json'),
  judge: path.join(root, 'judge-verdict.schema.json'),
  final: path.join(root, 'final-verdict.schema.json'),
  router: path.join(root, 'router-decision.schema.json')
};
const examples = {
  worker: path.join(root, 'examples', 'worker-output.json'),
  judge: path.join(root, 'examples', 'judge-verdict.json'),
  final: path.join(root, 'examples', 'final-verdict.json'),
  router: path.join(root, 'examples', 'router-decision.json')
};

function load(p) { return JSON.parse(fs.readFileSync(p, 'utf8')); }

const ajv = new Ajv({ allErrors: true, strict: true });

const report = [];
for (const key of Object.keys(schemas)) {
  const schema = load(schemas[key]);
  const validate = ajv.compile(schema);
  const data = load(examples[key]);
  const ok = validate(data);
  report.push({ key, ok, errors: validate.errors });
}

let success = true;
for (const r of report) {
  if (!r.ok) success = false;
  console.log(`Schema ${r.key}: ${r.ok ? 'OK' : 'FAIL'}`);
  if (!r.ok) console.log(JSON.stringify(r.errors, null, 2));
}

process.exit(success ? 0 : 1);

