/**
 * Test Setup Configuration
 *
 * Configures test environment with database and service connections
 * for integration and E2E tests.
 */

import {
  DatabaseTestUtils,
  RedisTestUtils,
  TestEnvironment,
} from "./test-utils";

// Test database configuration
process.env.DB_HOST = process.env.DB_HOST || "localhost";
process.env.DB_PORT = process.env.DB_PORT || "5432";
process.env.DB_NAME = process.env.DB_NAME || "agent_agency_test";
process.env.DB_USER = process.env.DB_USER || "postgres";
process.env.DB_PASSWORD = process.env.DB_PASSWORD || "test123";

// Redis configuration for tests
process.env.REDIS_HOST = process.env.REDIS_HOST || "localhost";
process.env.REDIS_PORT = process.env.REDIS_PORT || "6379";

// Ollama configuration for AI tests
process.env.OLLAMA_HOST = process.env.OLLAMA_HOST || "http://localhost:11434";
process.env.OLLAMA_MODEL = process.env.OLLAMA_MODEL || "gemma3n:e2b";

// Set test environment
process.env.NODE_ENV = "test";

// Global test environment
let testEnv: TestEnvironment;

// Global test setup
beforeAll(async () => {
  // Setup test environment with mocks
  testEnv = TestEnvironment.setup();

  // Setup database mocks
  DatabaseTestUtils.setupMockDatabase();
  DatabaseTestUtils.mockSuccessfulOperations();

  // Setup Redis mocks
  RedisTestUtils.setupMockRedis();
});

afterAll(async () => {
  // Cleanup test environment
  if (testEnv) {
    testEnv.cleanup();
  }
});
