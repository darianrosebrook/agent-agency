const fs = require("fs");
const path = require("path");

const testFile = path.join(
  __dirname,
  "tests/unit/models/LocalModelSelector.test.ts"
);
let content = fs.readFileSync(testFile, "utf8");

// Fix 1: Remove timestamp property from metrics (not in the interface)
content = content.replace(/timestamp: new Date\(\),\s*\n/g, "");

// Fix 2: Replace await selector.updatePerformanceHistory with correct signature
// Pattern: await selector.updatePerformanceHistory("model-name", { ... })
// Should be: selector.updatePerformanceHistory("model-name", "task-type", { ... })
content = content.replace(
  /await selector\.updatePerformanceHistory\("([^"]+)", \{/g,
  'selector.updatePerformanceHistory("$1", "text-generation", {'
);

// Fix 3: Remove unnecessary await from updatePerformanceHistory (it's void, not async)
content = content.replace(
  /selector\.updatePerformanceHistory\("([^"]+)", "([^"]+)", \{\s*\n/g,
  'selector.updatePerformanceHistory("$1", "$2", {\n'
);

// Fix 4: Replace result.model with result.primary
content = content.replace(/result\.model\b/g, "result.primary");

// Fix 5: Fix ModelSelectionCriteria objects - add taskType, maxMemoryMB, fix availableHardware
// This is trickier - we need to find criteria objects and fix them

// Pattern to find criteria objects and fix them
const criteriaRegex = /const criteria: ModelSelectionCriteria = \{[^}]+\}/gs;
const matches = content.match(criteriaRegex);

if (matches) {
  matches.forEach((match) => {
    let fixed = match;

    // Add taskType if missing
    if (!fixed.includes("taskType:")) {
      fixed = fixed.replace(
        /const criteria: ModelSelectionCriteria = \{/,
        'const criteria: ModelSelectionCriteria = {\n        taskType: "text-generation",'
      );
    }

    // Add maxMemoryMB if missing
    if (!fixed.includes("maxMemoryMB:")) {
      fixed = fixed.replace(
        /maxLatencyMs: \d+,/,
        "$&\n        maxMemoryMB: 4096,"
      );
    }

    // Remove minQuality (doesn't exist in interface)
    fixed = fixed.replace(/minQuality: [0-9.]+,\s*\n\s*/g, "");

    // Fix preferredHardware to availableHardware
    fixed = fixed.replace(
      /preferredHardware: \["([^"]+)"\],/g,
      'availableHardware: {\n          cpu: true,\n          gpu: $1 === "gpu",\n        },'
    );

    // Fix availableHardware if it's still in array format
    fixed = fixed.replace(
      /availableHardware: \[([^\]]+)\],/g,
      (match, hardware) => {
        const hardwareList = hardware.match(/"([^"]+)"/g) || [];
        const hasCpu = hardwareList.some((h) => h.includes("cpu"));
        const hasGpu = hardwareList.some((h) => h.includes("gpu"));
        return `availableHardware: {\n          cpu: ${hasCpu},\n          gpu: ${hasGpu},\n        },`;
      }
    );

    // Replace in content
    content = content.replace(match, fixed);
  });
}

// Write the fixed content back
fs.writeFileSync(testFile, content, "utf8");

console.log("Fixed LocalModelSelector.test.ts");
console.log("Summary of fixes:");
console.log("- Removed timestamp properties");
console.log("- Fixed updatePerformanceHistory signatures");
console.log("- Replaced result.model with result.primary");
console.log("- Fixed ModelSelectionCriteria objects");
