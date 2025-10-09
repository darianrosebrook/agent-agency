/**
 * Jest Test Setup
 *
 * @author @darianrosebrook
 * @description Global test setup and configuration
 */

// Global test timeout
jest.setTimeout(10000);

// Mock MCP SDK to avoid ES module issues in tests
jest.mock("@modelcontextprotocol/sdk/server/index.js", () => ({
  Server: jest.fn().mockImplementation(() => ({
    setRequestHandler: jest.fn(),
    connect: jest.fn(),
    close: jest.fn(),
  })),
}));

jest.mock("@modelcontextprotocol/sdk/server/stdio.js", () => ({
  StdioServerTransport: jest.fn(),
}));

jest.mock("@modelcontextprotocol/sdk/types.js", () => ({
  CallToolRequestSchema: {},
  ListResourcesRequestSchema: {},
  ListToolsRequestSchema: {},
  ReadResourceRequestSchema: {},
}));

// Mock console methods to reduce noise in tests
const originalConsole = { ...console };

beforeEach(() => {
  // Suppress console output during tests unless explicitly needed
  console.log = jest.fn();
  console.warn = jest.fn();
  console.error = jest.fn();
});

afterEach(() => {
  // Restore original console methods
  console.log = originalConsole.log;
  console.warn = originalConsole.warn;
  console.error = originalConsole.error;
});

// Global test utilities
(global as any).testUtils = {
  // Generate test data
  createMockAgent: (overrides = {}) => ({
    id: "test-agent-1",
    name: "Test Agent",
    type: "worker",
    status: "idle",
    capabilities: ["process"],
    metadata: {},
    createdAt: new Date(),
    updatedAt: new Date(),
    ...overrides,
  }),

  createMockTask: (overrides = {}) => ({
    id: "test-task-1",
    agentId: "test-agent-1",
    type: "process",
    status: "pending",
    payload: { data: "test" },
    createdAt: new Date(),
    updatedAt: new Date(),
    ...overrides,
  }),
};
