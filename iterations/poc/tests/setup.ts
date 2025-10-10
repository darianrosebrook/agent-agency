/**
 * Test Setup Configuration
 *
 * Configures test environment with database and service connections
 * for integration and E2E tests.
 */

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

// Global test setup
beforeAll(async () => {
  // Increase Jest timeout for integration tests
  jest.setTimeout(30000);
});

afterAll(async () => {
  // Cleanup will be handled by individual test suites
});
