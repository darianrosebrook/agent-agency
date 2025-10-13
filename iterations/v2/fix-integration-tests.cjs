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

// Fix 1: Convert vitest to jest
content = content.replace(
  /import \{ ([^}]+) \} from "vitest";/g,
  'import { $1 } from "@jest/globals";'
);

// Fix 2: Fix updatePerformanceHistory calls (add taskType)
content = content.replace(
  /selector\.updatePerformanceHistory\("([^"]+)", \{/g,
  'selector.updatePerformanceHistory("$1", "text-generation", {'
);

// Fix 3: Fix getPerformanceHistory calls (add taskType)
content = content.replace(
  /selector\.getPerformanceHistory\("([^"]+)"\)/g,
  'selector.getPerformanceHistory("$1", "text-generation")'
);

// Fix 4: Remove timestamp property
content = content.replace(/timestamp: new Date\(\),?\s*\n/g, "");

// Fix 5: Fix deprecateModel calls (remove second parameter if present)
content = content.replace(
  /registry\.deprecateModel\(([^,]+),\s*"[^"]*"\)/g,
  "registry.deprecateModel($1)"
);

// Fix 6: Remove minQuality from criteria
content = content.replace(/minQuality: [0-9.]+,\s*\n\s*/g, "");

// Fix 7: Replace result.model with result.primary
content = content.replace(/\.model\b/g, ".primary");

// Fix 8: Add missing taskType to ModelSelectionCriteria
const criteriaRegex = /const [a-zA-Z]+ = \{\s*\n\s*requiredCapabilities:/g;
content = content.replace(criteriaRegex, (match) => {
  return match.replace(
    /const ([a-zA-Z]+) = \{/,
    'const $1 = {\n        taskType: "text-generation",'
  );
});

// Fix 9: Add missing maxMemoryMB and availableHardware
content = content.replace(
  /maxLatencyMs: (\d+),/g,
  "maxLatencyMs: $1,\n        maxMemoryMB: 4096,"
);

content = content.replace(
  /qualityThreshold: ([0-9.]+),/g,
  "qualityThreshold: $1,\n        availableHardware: { cpu: true, gpu: false },"
);

// Write the fixed content back
fs.writeFileSync(testFile, content, "utf8");

console.log("✓ Fixed ModelRegistryIntegration.test.ts");
console.log("  - Converted vitest to jest");
console.log("  - Fixed updatePerformanceHistory signatures");
console.log("  - Fixed getPerformanceHistory signatures");
console.log("  - Fixed deprecateModel signatures");
console.log("  - Fixed ModelSelectionCriteria objects");
console.log("  - Replaced .model with .primary");
