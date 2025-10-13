const fs = require("fs");
const path = require("path");

const testFile = path.join(
  __dirname,
  "tests/integration/models/ModelRegistryIntegration.test.ts"
);

if (!fs.existsSync(testFile)) {
  console.log("Integration test file not found");
  process.exit(1);
}

let content = fs.readFileSync(testFile, "utf8");

// Fix all remaining updatePerformanceHistory calls that are missing taskType
// Pattern: await selector.updatePerformanceHistory(modelName, {
// Should be: selector.updatePerformanceHistory(modelName, "text-generation", {

content = content.replace(
  /await selector\.updatePerformanceHistory\(([^,]+), \{/g,
  'selector.updatePerformanceHistory($1, "text-generation", {'
);

// Write the fixed content back
fs.writeFileSync(testFile, content, "utf8");

console.log("✓ Fixed remaining updatePerformanceHistory calls");
