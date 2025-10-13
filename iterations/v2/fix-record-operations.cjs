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

// Find all recordOperation calls and add missing fields
// We need to add timestamp, inputTokens, outputTokens after cpuUtilization

content = content.replace(
  /(cpuUtilization: \d+,)\s*\n(\s+tokensPerSecond:)/g,
  "$1\n          timestamp: new Date(),\n          inputTokens: 100,\n          outputTokens: 50,\n$2"
);

// Write the fixed content back
fs.writeFileSync(testFile, content, "utf8");

console.log(
  "✓ Added timestamp, inputTokens, outputTokens to all recordOperation calls"
);
