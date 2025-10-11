/**
 * Test Utilities for V2
 *
 * Shared utilities for testing across unit, integration, and E2E tests.
 *
 * @author @darianrosebrook
 */

import { jest } from '@jest/globals';

// Database test utilities
export class DatabaseTestUtils {
  /**
   * Setup mock database for testing
   */
  static setupMockDatabase(): void {
    // Mock pg Pool and PoolClient
    jest.mock('pg', () => ({
      Pool: jest.fn().mockImplementation(() => ({
        connect: jest.fn(),
        end: jest.fn(),
        on: jest.fn(),
        removeListener: jest.fn(),
      })),
      PoolClient: jest.fn(),
    }));
  }

  /**
   * Mock successful database operations
   */
  static mockSuccessfulOperations(): void {
    // This will be expanded as needed for specific tests
  }

  /**
   * Mock database connection failure
   */
  static mockConnectionFailure(): void {
    const mockPool = require('pg').Pool;
    mockPool.mockImplementation(() => ({
      connect: jest.fn().mockRejectedValue(new Error('Connection failed')),
      end: jest.fn(),
      on: jest.fn(),
      removeListener: jest.fn(),
    }));
  }

  /**
   * Mock database query failure
   */
  static mockQueryFailure(): void {
    const mockPool = require('pg').Pool;
    const mockClient = {
      query: jest.fn().mockRejectedValue(new Error('Query failed')),
      release: jest.fn(),
    };

    mockPool.mockImplementation(() => ({
      connect: jest.fn().mockResolvedValue(mockClient),
      end: jest.fn(),
      on: jest.fn(),
      removeListener: jest.fn(),
    }));
  }
}

// Redis test utilities
export class RedisTestUtils {
  /**
   * Setup mock Redis for testing
   */
  static setupMockRedis(): void {
    // Mock redis client
    jest.mock('redis', () => ({
      createClient: jest.fn().mockReturnValue({
        connect: jest.fn(),
        disconnect: jest.fn(),
        get: jest.fn(),
        set: jest.fn(),
        del: jest.fn(),
        expire: jest.fn(),
        on: jest.fn(),
      }),
    }));
  }
}

// General test environment utilities
export class TestEnvironment {
  private static originalEnv: NodeJS.ProcessEnv;

  /**
   * Setup test environment
   */
  static setup(): TestEnvironment {
    // Save original environment
    this.originalEnv = { ...process.env };

    // Set test environment variables
    process.env.NODE_ENV = 'test';
    process.env.DB_HOST = 'localhost';
    process.env.DB_PORT = '5432';
    process.env.DB_NAME = 'agent_agency_test';
    process.env.DB_USER = 'postgres';
    process.env.DB_PASSWORD = 'test123';
    process.env.REDIS_HOST = 'localhost';
    process.env.REDIS_PORT = '6379';

    return new TestEnvironment();
  }

  /**
   * Cleanup test environment
   */
  cleanup(): void {
    // Restore original environment
    process.env = TestEnvironment.originalEnv;
  }
}

// Mock factory utilities
export class MockFactory {
  /**
   * Create mock agent profile
   */
  static createMockAgent(overrides: Partial<any> = {}): any {
    return {
      id: 'mock-agent-id',
      name: 'Mock Agent',
      modelFamily: 'gpt-4',
      capabilities: [
        { name: 'code-editing', score: 0.9 },
        { name: 'debugging', score: 0.8 },
      ],
      performanceHistory: [],
      registeredAt: new Date().toISOString(),
      lastActiveAt: new Date().toISOString(),
      activeTasks: 0,
      queuedTasks: 0,
      utilizationPercent: 0,
      createdAt: new Date(),
      updatedAt: new Date(),
      ...overrides,
    };
  }

  /**
   * Create mock performance metrics
   */
  static createMockPerformanceMetrics(overrides: Partial<any> = {}): any {
    return {
      success: true,
      latency: 150,
      quality: 0.85,
      confidence: 0.9,
      taskType: 'code-editing',
      taskId: 'task-123',
      ...overrides,
    };
  }

  /**
   * Create mock database result
   */
  static createMockDatabaseResult(overrides: Partial<any> = {}): any {
    return {
      agents: [],
      total: 0,
      hasMore: false,
      query: { limit: 50, offset: 0 },
      ...overrides,
    };
  }
}

// Assertion helpers
export class AssertionHelpers {
  /**
   * Assert agent profile matches expected structure
   */
  static assertValidAgentProfile(profile: any): void {
    expect(profile).toHaveProperty('id');
    expect(profile).toHaveProperty('name');
    expect(profile).toHaveProperty('modelFamily');
    expect(profile).toHaveProperty('capabilities');
    expect(profile).toHaveProperty('performanceHistory');
    expect(profile).toHaveProperty('registeredAt');
    expect(profile).toHaveProperty('lastActiveAt');
    expect(profile).toHaveProperty('activeTasks');
    expect(profile).toHaveProperty('queuedTasks');
    expect(profile).toHaveProperty('utilizationPercent');
    expect(profile).toHaveProperty('createdAt');
    expect(profile).toHaveProperty('updatedAt');

    expect(Array.isArray(profile.capabilities)).toBe(true);
    expect(Array.isArray(profile.performanceHistory)).toBe(true);
    expect(typeof profile.activeTasks).toBe('number');
    expect(typeof profile.queuedTasks).toBe('number');
    expect(typeof profile.utilizationPercent).toBe('number');
  }

  /**
   * Assert performance metrics are valid
   */
  static assertValidPerformanceMetrics(metrics: any): void {
    expect(metrics).toHaveProperty('success');
    expect(metrics).toHaveProperty('latency');
    expect(metrics).toHaveProperty('quality');
    expect(metrics).toHaveProperty('confidence');

    expect(typeof metrics.success).toBe('boolean');
    expect(typeof metrics.latency).toBe('number');
    expect(typeof metrics.quality).toBe('number');
    expect(typeof metrics.confidence).toBe('number');

    if (metrics.taskType) {
      expect(typeof metrics.taskType).toBe('string');
    }
    if (metrics.taskId) {
      expect(typeof metrics.taskId).toBe('string');
    }
  }
}
