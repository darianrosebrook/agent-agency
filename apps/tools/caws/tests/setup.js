/**
 * @fileoverview Jest Setup for CAWS Tools Tests
 * @author @darianrosebrook
 */

// Global test setup for CAWS tools tests
// This file is run before each test suite

// Set test timeout for integration tests
jest.setTimeout(30000);

// Mock console methods to reduce noise in tests
const originalConsoleLog = console.log;
const originalConsoleError = console.error;

beforeAll(() => {
  // Suppress console output during tests unless explicitly needed
  console.log = jest.fn();
  console.error = jest.fn();
});

afterAll(() => {
  // Restore console methods after tests
  console.log = originalConsoleLog;
  console.error = originalConsoleError;
});

// Global test utilities
global.testUtils = {
  /**
   * Create a temporary directory for testing
   */
  createTempDir: () => {
    const os = require("os");
    const path = require("path");
    const fs = require("fs");

    return fs.mkdtempSync(path.join(os.tmpdir(), "caws-test-"));
  },

  /**
   * Clean up a temporary directory
   */
  cleanupTempDir: (dirPath) => {
    const fs = require("fs");
    if (fs.existsSync(dirPath)) {
      fs.rmSync(dirPath, { recursive: true, force: true });
    }
  },

  /**
   * Create a basic working spec for testing
   */
  createBasicWorkingSpec: (overrides = {}) => {
    return {
      id: "TEST-001",
      title: "Test specification",
      risk_tier: 2,
      mode: "feature",
      scope: {
        in: ["src/test.ts"],
        out: ["node_modules/"],
      },
      acceptance: [
        {
          id: "A1",
          given: "Valid input",
          when: "Processing occurs",
          then: "Expected output",
        },
      ],
      ...overrides,
    };
  },

  /**
   * Write a working spec to a file
   */
  writeWorkingSpec: (dirPath, spec) => {
    const fs = require("fs");
    const path = require("path");
    const yaml = require("js-yaml");

    const specPath = path.join(dirPath, ".caws", "working-spec.yaml");
    fs.mkdirSync(path.dirname(specPath), { recursive: true });
    fs.writeFileSync(specPath, yaml.dump(spec));
  },
};
