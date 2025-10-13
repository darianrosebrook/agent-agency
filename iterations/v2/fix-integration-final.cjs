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

// Fix: Add taskType to all criteria objects that don't have it
// Find all ModelSelectionCriteria blocks
const lines = content.split("\n");
const fixedLines = [];
let i = 0;

while (i < lines.length) {
  const line = lines[i];

  // Check if this is a criteria declaration
  if (line.includes("const criteria: ModelSelectionCriteria = {")) {
    fixedLines.push(line);
    i++;

    // Check if next line has taskType
    if (i < lines.length && !lines[i].includes("taskType:")) {
      // Add taskType after the opening brace
      fixedLines.push('        taskType: "text-generation",');
    }
  } else {
    fixedLines.push(line);
    i++;
  }
}

content = fixedLines.join("\n");

// Write the fixed content back
fs.writeFileSync(testFile, content, "utf8");

console.log("✓ Added taskType to all ModelSelectionCriteria objects");
