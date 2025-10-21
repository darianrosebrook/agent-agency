import { Calculator } from "./dist/utils/calculator.js";

async function runTest() {
  console.log("Testing Calculator...");

  const calculator = new Calculator();

  // Test add
  const addResult = calculator.add(2, 3);
  console.log(`Add test: ${addResult === 5 ? "PASS" : "FAIL"} (${addResult})`);

  // Test subtract
  const subtractResult = calculator.subtract(5, 3);
  console.log(
    `Subtract test: ${
      subtractResult === 2 ? "PASS" : "FAIL"
    } (${subtractResult})`
  );

  console.log("Test completed successfully!");
}

runTest().catch(console.error);
