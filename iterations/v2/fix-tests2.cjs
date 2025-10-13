const fs = require("fs");
const path = require("path");

const testFile = path.join(
  __dirname,
  "tests/unit/models/LocalModelSelector.test.ts"
);
let content = fs.readFileSync(testFile, "utf8");

// Fix 1: Fix the broken availableHardware objects (fix the cpu variable issue)
content = content.replace(/gpu: cpu === "gpu",/g, "gpu: false,");
content = content.replace(/gpu: gpu === "gpu",/g, "gpu: true,");

// Fix 2: Remove weights property (doesn't exist in interface)
content = content.replace(/,\s*weights:\s*\{[^}]+\}/gs, "");

// Fix 3: Fix PerformanceMetrics usage
content = content.replace(
  /const metrics: PerformanceMetrics = \{/g,
  "const metrics = {"
);

// Fix 4: Fix getPerformanceHistory calls to include taskType
content = content.replace(
  /selector\.getPerformanceHistory\("([^"]+)"\)/g,
  'selector.getPerformanceHistory("$1", "text-generation")'
);

// Fix 5: Fix clearHistory calls (remove parameter)
content = content.replace(
  /selector\.clearHistory\("([^"]+)"\)/g,
  "selector.clearHistory()"
);

// Fix 6: Fix the await on updatePerformanceHistory with only 2 params
content = content.replace(
  /await selector\.updatePerformanceHistory\("([^"]+)", metrics\);/g,
  'selector.updatePerformanceHistory("$1", "text-generation", metrics);'
);

// Fix 7: Remove timestamp property if it still exists
content = content.replace(/,?\s*timestamp: [^,\n}]+/g, "");

// Fix 8: Fix hardwareRequirements.preferredHardware access
content = content.replace(
  /result\.primary\.hardwareRequirements\.preferredHardware/g,
  "result.primary.id"
);

// Fix 9: Fix registerCustomModel to registerModel
content = content.replace(
  /registry\.registerCustomModel\(/g,
  "registry.registerModel("
);

// Write the fixed content back
fs.writeFileSync(testFile, content, "utf8");

console.log("Applied additional fixes to LocalModelSelector.test.ts");
