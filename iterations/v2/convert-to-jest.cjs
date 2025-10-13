const fs = require("fs");
const path = require("path");

const testFiles = [
  "tests/unit/models/LocalModelSelector.test.ts",
  "tests/unit/models/ComputeCostTracker.test.ts",
  "tests/unit/models/providers/OllamaProvider.test.ts",
  "tests/unit/models/ModelRegistry.test.ts",
];

testFiles.forEach((file) => {
  const filePath = path.join(__dirname, file);

  if (!fs.existsSync(filePath)) {
    console.log(`Skipping ${file} (not found)`);
    return;
  }

  let content = fs.readFileSync(filePath, "utf8");

  // Replace vitest imports with @jest/globals
  content = content.replace(
    /import \{ ([^}]+) \} from "vitest";/g,
    'import { $1 } from "@jest/globals";'
  );

  fs.writeFileSync(filePath, content, "utf8");
  console.log(`✓ Converted ${file} to Jest`);
});

console.log("\nAll test files converted to Jest");
