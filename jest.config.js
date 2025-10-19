const config = {
  preset: "ts-jest",
  testEnvironment: "node",
  extensionsToTreatAsEsm: [".ts"],
  globals: {
    "ts-jest": {
      useESM: true,
      tsconfig: "iterations/v2/tsconfig.json",
    },
  },
  moduleNameMapper: {
    "^@/(.*)$": "<rootDir>/iterations/v2/src/$1",
    "^(\\.\\.?\\/.*)\\.js$": "$1",
  },
  testMatch: ["**/__tests__/**/*.ts", "**/?(*.)+(spec|test).ts"],
  collectCoverageFrom: [
    "src/**/*.{ts,js}",
    "!src/**/*.d.ts",
    "!src/**/index.ts",
  ],
  coverageReporters: ["text", "lcov", "html"],
  setupFilesAfterEnv: ["<rootDir>/iterations/v2/tests/setup.ts"],
  testTimeout: 10000,
  detectOpenHandles: true,
  forceExit: true,
};

export default config;
