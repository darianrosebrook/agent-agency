/**
 * Jest Configuration
 *
 * @author @darianrosebrook
 *
 * Jest configuration to prevent process accumulation and system crashes.
 * This file takes precedence over package.json jest configuration.
 */

module.exports = {
  // Test environment
  testEnvironment: "node",

  // Test file patterns
  roots: ["<rootDir>/src", "<rootDir>/tests"],
  testMatch: ["**/__tests__/**/*.ts", "**/?(*.)+(spec|test).ts"],

  // CRITICAL: Limit parallel workers to prevent system overload
  maxWorkers: 2,

  // Timeout settings
  testTimeout: 30000,

  // Process management
  forceExit: true,
  detectOpenHandles: false,

  // Transform configuration
  transform: {
    "^.+\\.ts$": [
      "ts-jest",
      {
        tsconfig: {
          ...require("./tsconfig.json").compilerOptions,
          module: "ES2022",
          target: "ES2022",
        },
        useESM: true,
      },
    ],
    "^.+\\.js$": "ts-jest",
  },

  // Coverage settings
  collectCoverageFrom: [
    "src/**/*.{ts,js}",
    "!src/**/*.d.ts",
    "!src/**/index.ts",
  ],

  // Coverage thresholds
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80,
    },
  },

  // Module resolution
  moduleNameMapper: {
    "^@/(.*)$": "<rootDir>/src/$1",
    "^(\\.\\.?\\/.*)\\.js$": "$1",
  },

  moduleFileExtensions: ["ts", "js"],
  extensionsToTreatAsEsm: [".ts"],

  // Transform ignore patterns
  transformIgnorePatterns: [
    "node_modules/(?!(supertest|axios|@modelcontextprotocol)/)",
  ],

  // Setup files
  setupFilesAfterEnv: ["<rootDir>/tests/setup.ts"],

  // Additional safety settings
  clearMocks: true,
  restoreMocks: true,

  // Verbose output for debugging
  verbose: false,

  // Bail on first failure to prevent cascading issues
  bail: false,

  // Worker idle memory limit (in MB)
  workerIdleMemoryLimit: "512MB",
};
