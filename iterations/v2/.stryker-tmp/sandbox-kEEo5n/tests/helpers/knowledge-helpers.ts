/**
 * @fileoverview Test helper functions for knowledge seeker tests
 *
 * Provides type-safe factory functions for creating knowledge test data
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { KnowledgeQuery, QueryType } from "@/types/knowledge";

/**
 * Create a test knowledge query with defaults
 */
export function createTestKnowledgeQuery(
  overrides: Partial<KnowledgeQuery> = {}
): KnowledgeQuery {
  return {
    id: `test-query-${Date.now()}`,
    query: "Test query for knowledge search",
    queryType: QueryType.FACTUAL,
    maxResults: 5,
    relevanceThreshold: 0.5,
    timeoutMs: 30000,
    metadata: {
      requesterId: "test-requester",
      priority: 5,
      createdAt: new Date(),
      tags: ["test"],
    },
    ...overrides,
  };
}

/**
 * Create multiple test queries with varying types
 */
export function createTestKnowledgeQueries(
  count: number,
  baseId = "query"
): KnowledgeQuery[] {
  const queryTypes = [
    QueryType.FACTUAL,
    QueryType.EXPLANATORY,
    QueryType.COMPARATIVE,
    QueryType.TREND,
    QueryType.TECHNICAL,
  ];

  return Array.from({ length: count }, (_, i) =>
    createTestKnowledgeQuery({
      id: `${baseId}-${i}`,
      query: `Test query ${i}`,
      queryType: queryTypes[i % queryTypes.length],
      metadata: {
        requesterId: "test-requester",
        priority: 3 + (i % 7),
        createdAt: new Date(),
        tags: [`tag-${i}`],
      },
    })
  );
}

/**
 * Create a high-priority factual query (triggers auto-verification)
 */
export function createHighPriorityFactualQuery(
  overrides: Partial<KnowledgeQuery> = {}
): KnowledgeQuery {
  return createTestKnowledgeQuery({
    queryType: QueryType.FACTUAL,
    metadata: {
      requesterId: "test-requester",
      priority: 8, // High priority to trigger verification
      createdAt: new Date(),
      tags: ["high-priority", "factual"],
    },
    ...overrides,
  });
}

/**
 * Create a low-priority query (should not trigger auto-verification)
 */
export function createLowPriorityQuery(
  overrides: Partial<KnowledgeQuery> = {}
): KnowledgeQuery {
  return createTestKnowledgeQuery({
    queryType: QueryType.EXPLANATORY,
    metadata: {
      requesterId: "test-requester",
      priority: 2, // Low priority
      createdAt: new Date(),
      tags: ["low-priority"],
    },
    ...overrides,
  });
}
