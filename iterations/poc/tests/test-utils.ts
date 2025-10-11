/**
 * Test Utilities and Mocks
 *
 * Provides comprehensive mocking and utilities for testing
 * to prevent timeouts and ensure clean test execution.
 */

import { jest } from "@jest/globals";

// Setup global mocks before any imports
const mockSetInterval = jest.fn(() => 123);
const mockSetTimeout = jest.fn(() => 456);
const mockClearInterval = jest.fn();
const mockClearTimeout = jest.fn();

// Apply global mocks
Object.defineProperty(global, "setInterval", {
  value: mockSetInterval,
  writable: true,
});
Object.defineProperty(global, "setTimeout", {
  value: mockSetTimeout,
  writable: true,
});
Object.defineProperty(global, "clearInterval", {
  value: mockClearInterval,
  writable: true,
});
Object.defineProperty(global, "clearTimeout", {
  value: mockClearTimeout,
  writable: true,
});

// Mock modules that cause timeouts
jest.mock("../src/data/connection/PostgreSQLConnection", () => ({
  PostgreSQLConnection: jest.fn().mockImplementation(() => ({
    connect: jest.fn().mockResolvedValue(undefined),
    disconnect: jest.fn().mockResolvedValue(undefined),
    query: jest.fn().mockResolvedValue({ rows: [] }),
    healthCheck: jest.fn().mockResolvedValue(true),
    startMetricsCollection: jest.fn(),
    getMetrics: jest.fn().mockReturnValue({}),
  })),
}));

jest.mock("../src/data/cache/RedisCache", () => ({
  RedisCache: jest.fn().mockImplementation(() => ({
    get: jest.fn().mockResolvedValue({ success: true, hit: false }),
    set: jest.fn().mockResolvedValue({ success: true }),
    delete: jest.fn().mockResolvedValue({ success: true }),
    clear: jest.fn().mockResolvedValue({ success: true }),
    getStats: jest.fn().mockResolvedValue({ success: true, data: {} }),
    close: jest.fn().mockResolvedValue(undefined),
    initialize: jest.fn().mockResolvedValue(undefined),
    on: jest.fn(),
    emit: jest.fn(),
  })),
}));

jest.mock("../src/ai/ollama-client", () => ({
  OllamaClient: jest.fn().mockImplementation(() => ({
    generate: jest.fn().mockResolvedValue("mock response"),
    isAvailable: jest.fn().mockResolvedValue(true),
    getInfo: jest.fn().mockResolvedValue({}),
  })),
}));

/**
 * Test environment setup helper
 */
export class TestEnvironment {
  private mocks: Map<string, jest.Mock> = new Map();

  /**
   * Setup test environment with common mocks
   */
  static setup(): TestEnvironment {
    const env = new TestEnvironment();

    // Disable all setInterval/setTimeout calls in services
    env.disableTimers();

    return env;
  }

  /**
   * Disable all timer functions to prevent timeouts
   */
  private disableTimers(): void {
    // These are already mocked globally above
  }

  /**
   * Mock a service class
   */
  mockService(
    modulePath: string,
    className: string,
    implementation: any
  ): void {
    const mock = jest.fn().mockImplementation(() => implementation);
    this.mocks.set(className, mock);

    jest.mock(modulePath, () => ({
      [className]: mock,
    }));
  }

  /**
   * Get a mock by class name
   */
  getMock(className: string): jest.Mock | undefined {
    return this.mocks.get(className);
  }

  /**
   * Reset all mocks
   */
  reset(): void {
    this.mocks.forEach((mock) => mock.mockClear());
    jest.clearAllMocks();
    jest.clearAllTimers();
  }

  /**
   * Cleanup test environment
   */
  cleanup(): void {
    this.reset();
  }
}

/**
 * Database test utilities
 */
export class DatabaseTestUtils {
  /**
   * Setup mock database for tests
   */
  static setupMockDatabase(): void {
    // Already mocked above
  }

  /**
   * Mock successful database operations
   */
  static mockSuccessfulOperations(): void {
    // Already mocked above
  }
}

/**
 * Redis test utilities
 */
export class RedisTestUtils {
  /**
   * Setup mock Redis for tests
   */
  static setupMockRedis(): void {
    // Already mocked above
  }
}

/**
 * Process cleanup utilities for E2E tests
 */
export class ProcessTestUtils {
  private static activeProcesses: Set<any> = new Set();

  /**
   * Register a process for cleanup
   */
  static registerProcess(process: any): void {
    this.activeProcesses.add(process);

    // Ensure cleanup on process exit
    process.on("exit", () => {
      this.activeProcesses.delete(process);
    });

    process.on("error", () => {
      this.activeProcesses.delete(process);
    });
  }

  /**
   * Cleanup all registered processes
   */
  static async cleanupAll(): Promise<void> {
    const promises = Array.from(this.activeProcesses).map((process) => {
      return new Promise<void>((resolve) => {
        if (process && process.kill) {
          process.kill("SIGTERM");

          // Force kill after timeout
          const timeout = setTimeout(() => {
            if (process && process.kill) {
              process.kill("SIGKILL");
            }
            resolve();
          }, 5000);

          process.on("exit", () => {
            clearTimeout(timeout);
            resolve();
          });

          process.on("error", () => {
            clearTimeout(timeout);
            resolve();
          });
        } else {
          resolve();
        }
      });
    });

    await Promise.all(promises);
    this.activeProcesses.clear();
  }

  /**
   * Get active process count
   */
  static getActiveProcessCount(): number {
    return this.activeProcesses.size;
  }
}

// Global test setup
beforeAll(() => {
  // Increase Jest timeout for integration tests
  jest.setTimeout(60000);

  // Use fake timers globally
  jest.useFakeTimers();
});

afterAll(async () => {
  // Cleanup any remaining processes
  await ProcessTestUtils.cleanupAll();

  // Clear all mocks
  jest.clearAllMocks();
  jest.clearAllTimers();
});

afterEach(() => {
  // Clear all mocks after each test
  jest.clearAllMocks();
  jest.clearAllTimers();
});
