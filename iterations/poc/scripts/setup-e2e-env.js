#!/usr/bin/env node

/**
 * E2E Environment Setup Script
 *
 * @author @darianrosebrook
 * @description Sets up the environment for end-to-end testing
 */

import { execSync } from "child_process";
import fs from "fs";
import path from "path";

const __dirname = path.dirname(new URL(import.meta.url).pathname);

console.log("🚀 Setting up E2E environment...");

// Check if Docker is available
try {
  execSync("docker --version", { stdio: "pipe" });
  console.log("✅ Docker available");
} catch (error) {
  console.error(
    "❌ Docker not available. Please install Docker for E2E tests."
  );
  process.exit(1);
}

// Check if Ollama is available (optional)
try {
  execSync("ollama --version", { stdio: "pipe" });
  console.log("✅ Ollama available");
} catch (error) {
  console.log("⚠️  Ollama not available - AI tests will be skipped");
}

// Create artifacts directory
const artifactsDir = path.join(__dirname, "..", "tests", "e2e", "artifacts");
if (!fs.existsSync(artifactsDir)) {
  fs.mkdirSync(artifactsDir, { recursive: true });
  console.log("✅ Created artifacts directory");
}

// Build the project
try {
  console.log("🔨 Building project...");
  execSync("npm run build", { stdio: "inherit" });
  console.log("✅ Project built successfully");
} catch (error) {
  console.error("❌ Failed to build project");
  process.exit(1);
}

console.log("🎉 E2E environment setup complete!");
console.log("");
console.log("To run E2E tests:");
console.log("  npm run test:e2e");
console.log("");
console.log("To run specific test:");
console.log("  npx jest tests/e2e/text-transformation.test.ts");
