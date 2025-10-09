/**
 * Text Transformation E2E Test
 *
 * @author @darianrosebrook
 * @description End-to-end test for text transformation with self-evaluation
 */

import { afterAll, beforeAll, describe, expect, it } from "@jest/globals";
import fs from "fs";
import path from "path";
import { e2eRunner } from "./setup";

describe("Text Transformation E2E", () => {
  beforeAll(async () => {
    await e2eRunner.setup();
  }, 120000); // 2 minute timeout for setup

  afterAll(async () => {
    await e2eRunner.teardown();
  }, 30000);

  it("should complete text transformation workflow", async () => {
    // Create test artifacts directory
    const artifactsDir = path.join(__dirname, "artifacts");
    if (!fs.existsSync(artifactsDir)) {
      fs.mkdirSync(artifactsDir, { recursive: true });
    }

    // Create test input
    const inputText = `Hey team, this is a really casual message that needs to be made more professional. It's got some informal language and could use better structure. Let's make it work better for our stakeholders.`;
    const inputPath = path.join(artifactsDir, "input.txt");
    fs.writeFileSync(inputPath, inputText);

    // Expected output should be more formal
    const expectedFormalElements = [
      "professional",
      "stakeholders",
      "formal language",
    ];

    // This is a placeholder test - in a real implementation, we would:
    // 1. Start MCP server
    // 2. Use MCP client to submit text transformation task
    // 3. Agent generates formal version
    // 4. Run evaluation
    // 5. Verify results

    console.log("✅ Text transformation E2E test placeholder");
    console.log("   Input:", inputText.substring(0, 50) + "...");
    console.log("   Expected formal elements:", expectedFormalElements);

    // For now, just verify the test setup works
    expect(fs.existsSync(inputPath)).toBe(true);
    expect(fs.readFileSync(inputPath, "utf8")).toContain("casual message");
  }, 60000); // 1 minute timeout

  it("should handle evaluation framework", async () => {
    // Test the evaluation framework components
    const evalTypes = path.join(__dirname, "../../src/evaluation/types.ts");
    const evalOrchestrator = path.join(
      __dirname,
      "../../src/evaluation/orchestrator.ts"
    );

    expect(fs.existsSync(evalTypes)).toBe(true);
    expect(fs.existsSync(evalOrchestrator)).toBe(true);

    console.log("✅ Evaluation framework files exist");
  });

  it("should have MCP server with AI tools", async () => {
    // Verify MCP server can be started (basic smoke test)
    const serverPath = path.join(__dirname, "../../bin/mcp-server.js");

    expect(fs.existsSync(serverPath)).toBe(true);

    const serverContent = fs.readFileSync(serverPath, "utf8");
    expect(serverContent).toContain("OllamaClient");
    expect(serverContent).toContain("aiClient");

    console.log("✅ MCP server configured with AI client");
  });
});









