/**
 * Jest Test Setup
 *
 * @author @darianrosebrook
 *
 * Global test configuration and setup for Jest tests.
 */

// Set NODE_ENV for tests
process.env.NODE_ENV = "test";

import { ConnectionPoolManager } from "@/database/ConnectionPoolManager";

// Set test timeout
// eslint-disable-next-line no-undef
jest.setTimeout(30000);

// Setup timer mocks to prevent hanging tests
// Use fake timers but allow real timers for database operations
jest.useFakeTimers({
  advanceTimers: false,
  doNotFake: ["setImmediate", "setInterval", "setTimeout"],
});

// Global test setup
// eslint-disable-next-line no-undef
beforeAll(async () => {
  // Initialize centralized database connection pool for all tests
  try {
    const manager = ConnectionPoolManager.getInstance();
    if (!manager.isInitialized()) {
      // Initialize with test database configuration
      manager.initialize({
        host: process.env.DB_HOST || "localhost",
        port: parseInt(process.env.DB_PORT || "5432", 10),
        database: process.env.DB_NAME || "agent_agency_v2_test",
        user: process.env.DB_USER || "postgres",
        password: process.env.DB_PASSWORD || "",
        min: 2,
        max: 10, // Lower max for test environment
        idleTimeoutMs: 10000, // Shorter timeout for tests
        connectionTimeoutMs: 5000,
        statementTimeoutMs: 30000,
        applicationName: "v2-arbiter-test",
      });

      // Verify database is accessible
      const isHealthy = await manager.healthCheck();
      if (!isHealthy) {
        console.warn(
          "Database health check failed. Some tests may fail if they require database access."
        );
      } else {
        const stats = manager.getStats();
        console.log(
          `Database pool initialized for tests (${stats.totalCount} connections, status: ${stats.healthCheckStatus})`
        );
      }
    }
  } catch (error) {
    console.warn(
      "Failed to initialize database connection pool:",
      error instanceof Error ? error.message : error
    );
    console.warn(
      "Tests requiring database will be skipped or use mocks. Set DB_* environment variables if real database is available."
    );
  }
});

// Global test teardown
// eslint-disable-next-line no-undef
afterAll(async () => {
  // Cleanup database connection pool first
  try {
    const manager = ConnectionPoolManager.getInstance();
    if (manager.isInitialized()) {
      await manager.shutdown();
      console.log("Database pool closed after tests");
    }
  } catch (error) {
    console.warn(
      "Error closing database pool:",
      error instanceof Error ? error.message : error
    );
  }

  // Reset singleton for clean state between test suites
  ConnectionPoolManager.resetForTesting();

  // Restore real timers for cleanup
  jest.useRealTimers();

  // Give a moment for any remaining async operations to complete
  await new Promise((resolve) => setTimeout(resolve, 100));
});
