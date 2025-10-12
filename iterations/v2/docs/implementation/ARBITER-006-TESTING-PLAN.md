# ARBITER-006: Comprehensive Testing Plan

**Date**: October 12, 2025  
**Author**: @darianrosebrook  
**Status**: 🚧 In Progress  
**Target Coverage**: 85%+ line, 90%+ branch

---

## Testing Strategy

### Pyramid Approach

```
        /\
       /  \
      / E2E \          5% - End-to-end scenarios
     /--------\
    /          \
   / Integration \     25% - Component integration
  /--------------\
 /                \
/    Unit Tests    \   70% - Individual functions
--------------------
```

### Coverage Targets by Component

| Component | Lines | Branches | Priority |
|-----------|-------|----------|----------|
| ResearchDetector | 90%+ | 95%+ | Critical |
| TaskResearchAugmenter | 85%+ | 90%+ | Critical |
| ResearchProvenance | 85%+ | 90%+ | High |
| KnowledgeDatabaseClient | 85%+ | 90%+ | High |
| Search Providers | 80%+ | 85%+ | Medium |

---

## Phase 1: Unit Tests (Priority: Critical)

### 1.1 ResearchDetector Tests

**File**: `tests/unit/orchestrator/research/ResearchDetector.test.ts`  
**Lines**: ~500  
**Coverage Target**: 90%+ lines, 95%+ branches

#### Test Suites

##### Question Detection
```typescript
describe("ResearchDetector - Question Detection", () => {
  test("should detect explicit questions", () => {
    // "How do I...?", "What is...?", "Why does...?"
  });

  test("should not false-positive on statements", () => {
    // "I know how to...", "This is what..."
  });

  test("should handle multiple questions", () => {
    // "How do I X? What about Y?"
  });

  test("should respect enableQuestionDetection config", () => {
    // disabled: should not detect
  });
});
```

##### Uncertainty Detection
```typescript
describe("ResearchDetector - Uncertainty Detection", () => {
  test("should detect uncertainty keywords", () => {
    // "not sure", "unclear", "need to find"
  });

  test("should calculate weighted confidence", () => {
    // multiple indicators → higher confidence
  });

  test("should respect minConfidence threshold", () => {
    // below threshold → null
  });
});
```

##### Technical Detection
```typescript
describe("ResearchDetector - Technical Detection", () => {
  test("should detect technical keywords", () => {
    // "API", "implementation", "documentation"
  });

  test("should infer task type technical needs", () => {
    // "implementation" task → likely technical
  });

  test("should respect enableTechnicalDetection config", () => {});
});
```

##### Query Generation
```typescript
describe("ResearchDetector - Query Generation", () => {
  test("should generate relevant queries", () => {
    // task description → 3 queries
  });

  test("should respect maxQueries config", () => {
    // maxQueries: 2 → only 2 queries
  });

  test("should infer correct query type", () => {
    // questions → EXPLANATORY
    // comparison → COMPARATIVE
    // technical → TECHNICAL
  });
});
```

##### Edge Cases
```typescript
describe("ResearchDetector - Edge Cases", () => {
  test("should handle empty task description", () => {});
  test("should handle very long descriptions", () => {});
  test("should handle special characters", () => {});
  test("should handle non-English text", () => {});
  test("should handle null/undefined metadata", () => {});
});
```

**Total**: ~40 test cases

---

### 1.2 TaskResearchAugmenter Tests

**File**: `tests/unit/orchestrator/research/TaskResearchAugmenter.test.ts`  
**Lines**: ~600  
**Coverage Target**: 85%+ lines, 90%+ branches

#### Test Suites

##### Task Augmentation Flow
```typescript
describe("TaskResearchAugmenter - Augmentation Flow", () => {
  test("should augment task when research required", async () => {
    // detector returns requirement → augment
  });

  test("should skip augmentation when not required", async () => {
    // detector returns null → skip
  });

  test("should add researchContext to task", async () => {
    // augmented task has researchContext field
  });

  test("should set researchProvided flag", async () => {});
});
```

##### Query Execution
```typescript
describe("TaskResearchAugmenter - Query Execution", () => {
  test("should execute queries in parallel", async () => {
    // 3 queries → parallel execution
  });

  test("should respect maxQueries config", async () => {
    // maxQueries: 2 → only 2 executed
  });

  test("should handle query failures gracefully", async () => {
    // 1 fails → continue with others
  });

  test("should respect timeoutMs config", async () => {
    // timeout → partial results
  });
});
```

##### Findings Transformation
```typescript
describe("TaskResearchAugmenter - Findings", () => {
  test("should transform results to findings", async () => {
    // KnowledgeResponse → ResearchFindings[]
  });

  test("should calculate overall confidence", async () => {
    // average of individual confidences
  });

  test("should respect relevanceThreshold", async () => {
    // low relevance → filtered out
  });

  test("should limit results per query", async () => {
    // maxResultsPerQuery: 3 → only 3
  });
});
```

##### Helper Methods
```typescript
describe("TaskResearchAugmenter - Helpers", () => {
  test("getResearchSummary should format findings", () => {});
  test("getResearchSources should extract citations", () => {});
  test("hasResearch should detect augmented tasks", () => {});
});
```

##### Error Handling
```typescript
describe("TaskResearchAugmenter - Error Handling", () => {
  test("should handle KnowledgeSeeker errors", async () => {});
  test("should handle detector errors", async () => {});
  test("should return original task on failure", async () => {});
  test("should log errors appropriately", async () => {});
});
```

**Total**: ~35 test cases

---

### 1.3 ResearchProvenance Tests

**File**: `tests/unit/orchestrator/research/ResearchProvenance.test.ts`  
**Lines**: ~400  
**Coverage Target**: 85%+ lines, 90%+ branches

#### Test Suites

##### Recording Operations
```typescript
describe("ResearchProvenance - Recording", () => {
  test("should record successful research", async () => {
    // recordResearch → database insert
  });

  test("should record failed research", async () => {
    // recordFailure → database insert with error
  });

  test("should handle database unavailable", async () => {
    // dbClient not connected → graceful skip
  });

  test("should log database errors", async () => {});
});
```

##### Retrieval Operations
```typescript
describe("ResearchProvenance - Retrieval", () => {
  test("getTaskResearch should return all records", async () => {});
  test("getStatistics should calculate correctly", async () => {
    // totalResearch, successRate, avgConfidence, avgDuration
  });
  test("should handle empty database", async () => {});
  test("should handle null database client", async () => {});
});
```

##### Data Validation
```typescript
describe("ResearchProvenance - Validation", () => {
  test("should validate ResearchContext structure", async () => {});
  test("should handle malformed data", async () => {});
  test("should sanitize error messages", async () => {});
});
```

**Total**: ~25 test cases

---

### 1.4 KnowledgeDatabaseClient Tests

**File**: `tests/unit/database/KnowledgeDatabaseClient.test.ts`  
**Lines**: ~500  
**Coverage Target**: 85%+ lines, 90%+ branches

#### Test Suites

##### Cache Operations
```typescript
describe("KnowledgeDatabaseClient - Cache", () => {
  test("checkQueryCache should return cached results", async () => {});
  test("checkQueryCache should return null for cache miss", async () => {});
  test("cacheQueryResponse should store in database", async () => {});
  test("should respect cache TTL", async () => {
    // expired cache → cache miss
  });
  test("should handle database errors gracefully", async () => {});
});
```

##### Query Operations
```typescript
describe("KnowledgeDatabaseClient - Queries", () => {
  test("storeQuery should insert query", async () => {});
  test("updateQueryStatus should update status", async () => {});
  test("getQuery should retrieve by ID", async () => {});
  test("should handle duplicate queries", async () => {});
});
```

##### Result Operations
```typescript
describe("KnowledgeDatabaseClient - Results", () => {
  test("storeResult should insert result", async () => {});
  test("should deduplicate by contentHash", async () => {
    // same hash → update, not insert
  });
  test("getQueryResults should retrieve all results", async () => {});
});
```

##### Response Operations
```typescript
describe("KnowledgeDatabaseClient - Responses", () => {
  test("storeResponse should insert response", async () => {});
  test("getQueryResponse should retrieve response", async () => {});
  test("should link response to query", async () => {});
});
```

##### Provider Health
```typescript
describe("KnowledgeDatabaseClient - Provider Health", () => {
  test("recordProviderHealth should store metrics", async () => {});
  test("getProviderHealth should retrieve history", async () => {});
  test("should calculate health statistics", async () => {});
});
```

**Total**: ~30 test cases

---

### 1.5 Search Provider Tests

**File**: `tests/unit/knowledge/providers/SearchProvider.test.ts`  
**Lines**: ~600  
**Coverage Target**: 80%+ lines, 85%+ branches

#### Test Suites per Provider

##### GoogleSearchProvider
```typescript
describe("GoogleSearchProvider", () => {
  test("should execute search with API", async () => {});
  test("should parse Google response correctly", async () => {});
  test("should apply credibility scoring", async () => {});
  test("should apply relevance scoring", async () => {});
  test("should handle API errors", async () => {});
  test("should handle rate limits", async () => {});
  test("should respect config options", async () => {});
});
```

##### BingSearchProvider
```typescript
describe("BingSearchProvider", () => {
  test("should execute search with API", async () => {});
  test("should parse Bing response correctly", async () => {});
  test("should apply scoring", async () => {});
  test("should handle errors", async () => {});
});
```

##### DuckDuckGoSearchProvider
```typescript
describe("DuckDuckGoSearchProvider", () => {
  test("should execute search without API key", async () => {});
  test("should parse DDG response correctly", async () => {});
  test("should handle instant answer format", async () => {});
  test("should handle no results", async () => {});
});
```

##### Base Provider
```typescript
describe("BaseSearchProvider", () => {
  test("extractDomain should parse URLs correctly", () => {});
  test("inferSourceType should classify sources", () => {});
  test("generateContentHash should be consistent", () => {});
});
```

**Total**: ~25 test cases

---

## Phase 2: Integration Tests (Priority: High)

### 2.1 End-to-End Research Flow

**File**: `tests/integration/orchestrator/research-flow.test.ts`  
**Lines**: ~400  
**Coverage Target**: Critical paths only

#### Test Scenarios

```typescript
describe("Research Flow - End-to-End", () => {
  test("should detect → augment → record complete flow", async () => {
    // 1. Submit task with research need
    // 2. Detector identifies need
    // 3. Augmenter executes queries
    // 4. Provenance records activity
    // 5. Task is augmented
  });

  test("should handle no research needed", async () => {
    // 1. Submit task without research need
    // 2. Detector returns null
    // 3. Task passes through unchanged
  });

  test("should handle research failure gracefully", async () => {
    // 1. Detector identifies need
    // 2. All queries fail
    // 3. Provenance records failure
    // 4. Original task continues
  });

  test("should use cached results when available", async () => {
    // 1. First request → cache miss → fetch
    // 2. Second request → cache hit → instant
  });
});
```

**Total**: ~10 test cases

---

### 2.2 Database Integration

**File**: `tests/integration/database/knowledge-database.test.ts`  
**Lines**: ~350  
**Coverage Target**: Database operations

#### Test Scenarios

```typescript
describe("Knowledge Database Integration", () => {
  beforeAll(async () => {
    // Set up test database
    // Run migrations
  });

  afterAll(async () => {
    // Clean up test database
  });

  test("should persist query → results → response", async () => {
    // Full database round-trip
  });

  test("should handle concurrent writes", async () => {
    // Parallel inserts → no conflicts
  });

  test("should maintain referential integrity", async () => {
    // results reference queries, etc.
  });

  test("should clean up old cache entries", async () => {
    // TTL expiration
  });
});
```

**Total**: ~8 test cases

---

### 2.3 MCP Tool Integration

**File**: `tests/integration/mcp/knowledge-tools.test.ts`  
**Lines**: ~300  
**Coverage Target**: MCP tool invocation

#### Test Scenarios

```typescript
describe("MCP Knowledge Tools Integration", () => {
  test("knowledge_search should execute real search", async () => {
    // MCP call → search → results
  });

  test("knowledge_status should return health", async () => {
    // MCP call → provider status
  });

  test("should validate input parameters", async () => {
    // invalid params → error
  });

  test("should format responses correctly", async () => {
    // structured JSON output
  });
});
```

**Total**: ~6 test cases

---

### 2.4 Orchestrator Integration

**File**: `tests/integration/orchestrator/research-integration.test.ts`  
**Lines**: ~400  
**Coverage Target**: Orchestrator pipeline

#### Test Scenarios

```typescript
describe("Orchestrator Research Integration", () => {
  test("submitTask should trigger research when enabled", async () => {
    // research.enabled: true → augmentation runs
  });

  test("submitTask should skip research when disabled", async () => {
    // research.enabled: false → no augmentation
  });

  test("should integrate with prompting engine", async () => {
    // research → augment → prompting
  });

  test("should handle research timeout", async () => {
    // slow research → timeout → continue
  });

  test("should track research in provenance", async () => {
    // successful → recorded
    // failed → recorded with error
  });
});
```

**Total**: ~8 test cases

---

## Phase 3: Performance Tests (Priority: Medium)

### 3.1 Benchmark Tests

**File**: `tests/performance/research-benchmarks.test.ts`  
**Lines**: ~200

#### Test Scenarios

```typescript
describe("Research Performance Benchmarks", () => {
  test("detection should complete in <10ms", async () => {
    // 100 iterations → average < 10ms
  });

  test("augmentation should complete in <2000ms", async () => {
    // 3 queries → average < 2000ms
  });

  test("database operations should be <50ms", async () => {
    // cache check, store → < 50ms
  });

  test("should handle concurrent augmentations", async () => {
    // 10 parallel → no degradation
  });
});
```

**Total**: ~6 test cases

---

## Test Infrastructure

### Mock Setup

```typescript
// tests/mocks/knowledge-mocks.ts

export const mockSearchResult = (overrides = {}) => ({
  id: "result-1",
  title: "Test Result",
  url: "https://example.com",
  content: "Test content",
  source: "mock-provider",
  sourceType: SourceType.OFFICIAL_DOCS,
  relevanceScore: 0.9,
  credibilityScore: 0.85,
  retrievedAt: new Date(),
  contentHash: "hash123",
  ...overrides,
});

export const mockKnowledgeResponse = (overrides = {}) => ({
  query: mockKnowledgeQuery(),
  results: [mockSearchResult()],
  summary: "Test summary",
  confidence: 0.88,
  metadata: {},
  ...overrides,
});

export const mockResearchContext = (overrides = {}) => ({
  queries: ["test query"],
  findings: [
    {
      query: "test query",
      summary: "Test finding",
      confidence: 0.9,
      keyFindings: [],
    },
  ],
  confidence: 0.9,
  augmentedAt: new Date(),
  metadata: {},
  ...overrides,
});
```

### Test Utilities

```typescript
// tests/utils/test-database.ts

export async function setupTestDatabase() {
  // Create test database
  // Run migrations
  // Return client
}

export async function cleanupTestDatabase(client) {
  // Drop test tables
  // Close connection
}

export async function seedTestData(client) {
  // Insert test records
}
```

---

## Test Execution Plan

### Order of Implementation

1. **Week 1**: Unit Tests
   - Day 1-2: ResearchDetector tests
   - Day 3: TaskResearchAugmenter tests
   - Day 4: ResearchProvenance tests
   - Day 5: KnowledgeDatabaseClient tests

2. **Week 2**: Integration Tests
   - Day 1: Database integration tests
   - Day 2: Research flow integration tests
   - Day 3: MCP tool integration tests
   - Day 4: Orchestrator integration tests
   - Day 5: Buffer/fixes

3. **Week 3**: Polish
   - Day 1-2: Search provider tests
   - Day 3: Performance tests
   - Day 4: Coverage gaps
   - Day 5: Documentation

### Running Tests

```bash
# All tests
npm test

# Unit tests only
npm run test:unit

# Integration tests only
npm run test:integration

# Coverage report
npm run test:coverage

# Watch mode
npm run test:watch

# Specific file
npm test -- ResearchDetector.test.ts
```

---

## Coverage Goals

### Current Coverage: 0%

```
Component                    | Lines | Branches | Status
-----------------------------|-------|----------|--------
ResearchDetector             |   0%  |    0%    | 🔴 TODO
TaskResearchAugmenter        |   0%  |    0%    | 🔴 TODO
ResearchProvenance           |   0%  |    0%    | 🔴 TODO
KnowledgeDatabaseClient      |   0%  |    0%    | 🔴 TODO
GoogleSearchProvider         |   0%  |    0%    | 🔴 TODO
BingSearchProvider           |   0%  |    0%    | 🔴 TODO
DuckDuckGoSearchProvider     |   0%  |    0%    | 🔴 TODO
-----------------------------|-------|----------|--------
TOTAL                        |   0%  |    0%    | 🔴 TODO
```

### Target Coverage: 85%+

```
Component                    | Lines | Branches | Status
-----------------------------|-------|----------|--------
ResearchDetector             |  90%+ |   95%+   | 🎯 Target
TaskResearchAugmenter        |  85%+ |   90%+   | 🎯 Target
ResearchProvenance           |  85%+ |   90%+   | 🎯 Target
KnowledgeDatabaseClient      |  85%+ |   90%+   | 🎯 Target
GoogleSearchProvider         |  80%+ |   85%+   | 🎯 Target
BingSearchProvider           |  80%+ |   85%+   | 🎯 Target
DuckDuckGoSearchProvider     |  80%+ |   85%+   | 🎯 Target
-----------------------------|-------|----------|--------
TOTAL                        |  85%+ |   90%+   | 🎯 Target
```

---

## Success Criteria

✅ **Code Coverage**:
- Overall: 85%+ lines, 90%+ branches
- Critical components: 90%+ lines, 95%+ branches

✅ **Test Quality**:
- All tests pass consistently
- No flaky tests
- Fast execution (<30s for unit, <2min for integration)

✅ **Documentation**:
- All test files have descriptive headers
- Complex tests have inline comments
- Test utilities documented

✅ **Maintainability**:
- Clear test names
- Isolated test cases
- Reusable mocks and utilities

---

## Appendix: Test File Structure

```
tests/
├── unit/
│   ├── orchestrator/
│   │   └── research/
│   │       ├── ResearchDetector.test.ts
│   │       ├── TaskResearchAugmenter.test.ts
│   │       └── ResearchProvenance.test.ts
│   ├── database/
│   │   └── KnowledgeDatabaseClient.test.ts
│   └── knowledge/
│       └── providers/
│           ├── GoogleSearchProvider.test.ts
│           ├── BingSearchProvider.test.ts
│           └── DuckDuckGoSearchProvider.test.ts
├── integration/
│   ├── orchestrator/
│   │   ├── research-flow.test.ts
│   │   └── research-integration.test.ts
│   ├── database/
│   │   └── knowledge-database.test.ts
│   └── mcp/
│       └── knowledge-tools.test.ts
├── performance/
│   └── research-benchmarks.test.ts
├── mocks/
│   ├── knowledge-mocks.ts
│   ├── database-mocks.ts
│   └── provider-mocks.ts
└── utils/
    ├── test-database.ts
    └── test-helpers.ts
```

---

**Status**: 🚧 Ready to implement  
**Estimated Time**: 3 weeks  
**Target Completion**: November 2, 2025  
**Risk**: Low - Clear plan, existing code works

