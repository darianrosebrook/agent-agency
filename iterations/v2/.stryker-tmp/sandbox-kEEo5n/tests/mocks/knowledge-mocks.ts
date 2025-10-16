/**
 * @fileoverview Test Mocks for Knowledge Seeker Components (ARBITER-006)
 *
 * @author @darianrosebrook
 *
 * Provides mock factories for knowledge-related types including:
 * - Search results
 * - Knowledge queries and responses
 * - Research context and findings
 * - Search providers
 */
// @ts-nocheck


import { ResearchRequirement } from "../../src/orchestrator/research/ResearchDetector";
import {
  AugmentedTask,
  ResearchFindings,
} from "../../src/orchestrator/research/TaskResearchAugmenter";
import { Task } from "../../src/types/arbiter-orchestration";
import {
  IKnowledgeSeeker,
  KnowledgeQuery,
  KnowledgeResponse,
  QueryType,
  ResultQuality,
  SearchResult,
} from "../../src/types/knowledge";
import { VerificationPriority } from "../../src/types/verification";

// Re-export for convenience
export { VerificationPriority };

/**
 * Creates a mock SearchResult with customizable fields
 */
export const mockSearchResult = (
  overrides: Partial<SearchResult> = {}
): SearchResult => ({
  id: `result-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
  queryId: overrides.queryId || `query-${Date.now()}`,
  title: "Test Search Result",
  url: "https://example.com/test-result",
  content:
    "This is test content for a search result. It contains relevant information.",
  domain: "example.com",
  sourceType: "documentation",
  relevanceScore: 0.9,
  credibilityScore: 0.85,
  quality: ResultQuality.HIGH,
  provider: "mock-provider",
  providerMetadata: {},
  processedAt: new Date(),
  retrievedAt: new Date(),
  contentHash: `hash-${Date.now()}`,
  ...overrides,
});

/**
 * Creates a mock KnowledgeQuery with customizable fields
 */
export const mockKnowledgeQuery = (
  overrides: Partial<KnowledgeQuery> = {}
): KnowledgeQuery => ({
  id: `query-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
  query: "How do I implement OAuth2 in Express.js?",
  queryType: QueryType.TECHNICAL,
  maxResults: 3,
  relevanceThreshold: 0.8,
  timeoutMs: 5000,
  context: {
    taskId: "task-123",
    purpose: "task-augmentation",
  },
  metadata: {
    requesterId: "test-requester",
    priority: 1,
    createdAt: new Date(),
    tags: ["test", "arbiter-006"],
  },
  ...overrides,
});

/**
 * Creates a mock KnowledgeResponse with customizable fields
 */
export const mockKnowledgeResponse = (
  overrides: Partial<KnowledgeResponse> = {}
): KnowledgeResponse => {
  const query = mockKnowledgeQuery(overrides.query);
  const results = overrides.results || [
    mockSearchResult(),
    mockSearchResult({
      id: `result-2-${Date.now()}`,
      title: "Test Result 2",
      relevanceScore: 0.88,
    }),
    mockSearchResult({
      id: `result-3-${Date.now()}`,
      title: "Test Result 3",
      relevanceScore: 0.86,
    }),
  ];

  return {
    query,
    results,
    summary: "Found 3 high-quality results for your query.",
    confidence: 0.88,
    sourcesUsed: ["mock-provider"],
    metadata: {
      totalResultsFound: results.length,
      resultsFiltered: 0,
      processingTimeMs: 345,
      cacheUsed: false,
      providersQueried: ["mock-provider"],
    },
    respondedAt: new Date(),
    ...overrides,
  };
};

/**
 * Creates a mock ResearchFindings with customizable fields
 */
export const mockResearchFindings = (
  overrides: Partial<ResearchFindings> = {}
): ResearchFindings => ({
  query: "How do I implement OAuth2 in Express.js?",
  summary:
    "OAuth2 can be implemented in Express.js using passport-oauth2 library.",
  confidence: 0.92,
  keyFindings: [
    {
      title: "Express OAuth2 Server Guide",
      url: "https://oauth2.expressjs.com/guide",
      snippet: "Step-by-step guide to implementing OAuth2 in Express.js...",
      relevance: 0.96,
    },
    {
      title: "Passport OAuth2 Strategy",
      url: "https://passportjs.org/packages/passport-oauth2",
      snippet: "OAuth 2.0 authentication strategy for Passport and Node.js...",
      relevance: 0.93,
    },
  ],
  ...overrides,
});

/**
 * Creates a mock ResearchContext
 */
export const mockResearchContext = (overrides: any = {}) => ({
  queries: ["query 1", "query 2", "query 3"],
  findings: [
    mockResearchFindings(),
    mockResearchFindings({ query: "query 2" }),
  ],
  confidence: 0.9,
  augmentedAt: new Date(),
  metadata: {
    durationMs: 487,
    detectorConfidence: 0.95,
    queryType: QueryType.TECHNICAL,
  },
  ...overrides,
});

/**
 * Creates a mock Task with customizable fields
 */
export const mockTask = (overrides: Partial<Task> = {}): Task => ({
  id: `task-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
  type: "code-editing",
  description: "Implement OAuth2 authentication in the Express.js API",
  requiredCapabilities: {},
  priority: 1,
  timeoutMs: 300000,
  budget: {
    maxFiles: 25,
    maxLoc: 1000,
  },
  createdAt: new Date(),
  metadata: {
    prompt: "How do I implement OAuth2 in Express.js?",
    requester: "test-user",
  },
  attempts: 0,
  maxAttempts: 3,
  ...overrides,
});

/**
 * Creates a mock AugmentedTask with research context
 */
export const mockAugmentedTask = (
  overrides: Partial<AugmentedTask> = {}
): AugmentedTask => ({
  ...mockTask(overrides),
  researchProvided: true,
  researchContext: mockResearchContext(),
  ...overrides,
});

/**
 * Creates a mock ResearchRequirement
 */
export const mockResearchRequirement = (
  overrides: Partial<ResearchRequirement> = {}
): ResearchRequirement => ({
  required: true,
  confidence: 0.95,
  suggestedQueries: [
    "How do I implement OAuth2 in Express.js?",
    "OAuth2 Express.js implementation",
    "OAuth2 Express.js documentation",
  ],
  queryType: QueryType.TECHNICAL,
  indicators: {
    hasQuestions: true,
    hasUncertainty: true,
    requiresFactChecking: false,
    needsComparison: false,
    requiresTechnicalInfo: true,
  },
  reason:
    "Task contains questions (0.3) + technical keywords (0.15) + uncertainty (0.3) + requires technical info (0.2) = 0.95 confidence",
  ...overrides,
});

/**
 * Creates a mock IKnowledgeSeeker implementation
 */
export class MockKnowledgeSeeker implements IKnowledgeSeeker {
  private shouldFail: boolean;
  private responseDelay: number;

  constructor(config?: { shouldFail?: boolean; responseDelay?: number }) {
    this.shouldFail = config?.shouldFail ?? false;
    this.responseDelay = config?.responseDelay ?? 100;
  }

  async processQuery(query: KnowledgeQuery): Promise<KnowledgeResponse> {
    await new Promise((resolve) => setTimeout(resolve, this.responseDelay));

    if (this.shouldFail) {
      throw new Error("Mock knowledge seeker failure");
    }

    return mockKnowledgeResponse({ query });
  }

  async getStatus(): Promise<any> {
    return {
      enabled: true,
      providers: [
        {
          name: "mock-provider",
          available: true,
          health: {
            available: true,
            responseTimeMs: 100,
            successRate: 1.0,
          },
        },
      ],
    };
  }

  async clearCaches(): Promise<void> {
    // Mock implementation
  }
}

/**
 * Creates a mock database client for testing
 */
export class MockDatabaseClient {
  private data: Map<string, any[]>;

  constructor() {
    this.data = new Map();
  }

  async query(sql: string, params: any[]): Promise<{ rows: any[] }> {
    // Simple mock implementation - just store data
    const tableName = this.extractTableName(sql);
    if (sql.includes("INSERT")) {
      const rows = this.data.get(tableName) || [];
      rows.push(params);
      this.data.set(tableName, rows);
      return { rows: [{ id: rows.length }] };
    } else if (sql.includes("SELECT")) {
      const rows = this.data.get(tableName) || [];
      return { rows };
    }
    return { rows: [] };
  }

  isConnected(): boolean {
    return true;
  }

  async end(): Promise<void> {
    this.data.clear();
  }

  private extractTableName(sql: string): string {
    const match = sql.match(/(?:FROM|INTO|UPDATE)\s+(\w+)/i);
    return match ? match[1] : "default";
  }
}
