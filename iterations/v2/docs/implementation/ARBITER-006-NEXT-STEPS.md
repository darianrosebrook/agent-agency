# ARBITER-006 Next Steps Guide

**Current Status**: Phase 1 & 3 Complete (75% Theory Aligned)  
**Last Updated**: October 12, 2025  
**Ready For**: Phase 2 or Phase 4 Implementation

---

## Quick Decision Matrix

**Choose Phase 2 if you need**:

- Real web search capabilities
- External API integration
- Production-ready research
- End-user value immediately

**Choose Phase 4 if you need**:

- Task automation features
- Orchestration integration
- Internal efficiency gains
- Complex workflow support

**Recommended**: Start with **Phase 2** for immediate user value

---

## Phase 2: Real Search Providers Implementation

### Overview

Replace MockSearchProvider with real search providers (Google, Bing, DuckDuckGo) to enable actual web research.

### Prerequisites

**API Keys Needed**:

```bash
# Google Custom Search
GOOGLE_SEARCH_API_KEY=your_key_here
GOOGLE_SEARCH_CX=your_custom_search_engine_id

# Bing Search
BING_SEARCH_API_KEY=your_key_here

# DuckDuckGo (no API key required)
# Uses HTML scraping or instant answer API
```

**How to Get API Keys**:

1. **Google Custom Search**:

   - Visit: https://developers.google.com/custom-search
   - Create a project in Google Cloud Console
   - Enable Custom Search API
   - Create credentials (API Key)
   - Create a Custom Search Engine: https://cse.google.com/cse/
   - Note your CX (Custom Search Engine ID)
   - Free tier: 100 queries/day

2. **Bing Search**:

   - Visit: https://www.microsoft.com/en-us/bing/apis/bing-web-search-api
   - Sign up for Azure account
   - Create Bing Search resource
   - Get API key from Azure portal
   - Free tier: 1000 queries/month

3. **DuckDuckGo**:
   - No API key required
   - Uses instant answer API: https://api.duckduckgo.com/
   - Rate limited by IP (be respectful)

### Implementation Tasks

#### 1. Create GoogleSearchProvider

**File**: `src/knowledge/providers/GoogleSearchProvider.ts`

```typescript
/**
 * @fileoverview Google Custom Search Provider
 */

import { BaseSearchProvider } from "../SearchProvider";
import {
  ISearchProvider,
  KnowledgeQuery,
  SearchProviderConfig,
  SearchResult,
  SourceType,
} from "../../types/knowledge";

export class GoogleSearchProvider extends BaseSearchProvider {
  private apiKey: string;
  private customSearchEngineId: string;

  constructor(config: SearchProviderConfig) {
    super(config);

    this.apiKey = process.env.GOOGLE_SEARCH_API_KEY || "";
    this.customSearchEngineId = process.env.GOOGLE_SEARCH_CX || "";

    if (!this.apiKey || !this.customSearchEngineId) {
      throw new Error(
        "Google Search requires GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_CX"
      );
    }
  }

  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    // Implement using Google Custom Search JSON API
    // https://developers.google.com/custom-search/v1/using_rest

    const url = new URL("https://www.googleapis.com/customsearch/v1");
    url.searchParams.set("key", this.apiKey);
    url.searchParams.set("cx", this.customSearchEngineId);
    url.searchParams.set("q", query.query);
    url.searchParams.set("num", query.maxResults.toString());

    const response = await fetch(url.toString());

    if (!response.ok) {
      throw new Error(`Google Search failed: ${response.statusText}`);
    }

    const data = await response.json();

    return this.parseGoogleResults(data, query.id);
  }

  private parseGoogleResults(data: any, queryId: string): SearchResult[] {
    // Parse Google Custom Search response format
    // Transform to SearchResult[]
  }
}
```

**Test**: `tests/integration/knowledge/google-search-provider.test.ts`

#### 2. Create BingSearchProvider

**File**: `src/knowledge/providers/BingSearchProvider.ts`

```typescript
/**
 * @fileoverview Bing Web Search Provider
 */

export class BingSearchProvider extends BaseSearchProvider {
  private apiKey: string;

  constructor(config: SearchProviderConfig) {
    super(config);

    this.apiKey = process.env.BING_SEARCH_API_KEY || "";

    if (!this.apiKey) {
      throw new Error("Bing Search requires BING_SEARCH_API_KEY");
    }
  }

  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    // Implement using Bing Web Search API v7
    // https://docs.microsoft.com/en-us/bing/search-apis/bing-web-search/

    const url = new URL("https://api.bing.microsoft.com/v7.0/search");
    url.searchParams.set("q", query.query);
    url.searchParams.set("count", query.maxResults.toString());

    const response = await fetch(url.toString(), {
      headers: {
        "Ocp-Apim-Subscription-Key": this.apiKey,
      },
    });

    if (!response.ok) {
      throw new Error(`Bing Search failed: ${response.statusText}`);
    }

    const data = await response.json();

    return this.parseBingResults(data, query.id);
  }

  private parseBingResults(data: any, queryId: string): SearchResult[] {
    // Parse Bing Web Search response format
  }
}
```

**Test**: `tests/integration/knowledge/bing-search-provider.test.ts`

#### 3. Create DuckDuckGoSearchProvider

**File**: `src/knowledge/providers/DuckDuckGoSearchProvider.ts`

```typescript
/**
 * @fileoverview DuckDuckGo Search Provider
 */

export class DuckDuckGoSearchProvider extends BaseSearchProvider {
  constructor(config: SearchProviderConfig) {
    super(config);
    // No API key required
  }

  async search(query: KnowledgeQuery): Promise<SearchResult[]> {
    // Use DuckDuckGo Instant Answer API
    // https://api.duckduckgo.com/api

    const url = new URL("https://api.duckduckgo.com/");
    url.searchParams.set("q", query.query);
    url.searchParams.set("format", "json");
    url.searchParams.set("no_html", "1");

    const response = await fetch(url.toString());

    if (!response.ok) {
      throw new Error(`DuckDuckGo Search failed: ${response.statusText}`);
    }

    const data = await response.json();

    return this.parseDuckDuckGoResults(data, query.id);
  }

  private parseDuckDuckGoResults(data: any, queryId: string): SearchResult[] {
    // Parse DuckDuckGo response format
    // Note: Instant Answer API has limited results
    // May need to implement HTML scraping for full results
  }
}
```

**Test**: `tests/integration/knowledge/duckduckgo-search-provider.test.ts`

#### 4. Update SearchProviderFactory

**File**: `src/knowledge/SearchProvider.ts`

```typescript
// Add imports
import { GoogleSearchProvider } from "./providers/GoogleSearchProvider";
import { BingSearchProvider } from "./providers/BingSearchProvider";
import { DuckDuckGoSearchProvider } from "./providers/DuckDuckGoSearchProvider";

export class SearchProviderFactory {
  static create(config: SearchProviderConfig): ISearchProvider {
    switch (config.type) {
      case SearchProviderType.WEB_SEARCH:
        // Determine which provider based on name
        switch (config.name.toLowerCase()) {
          case "google":
            return new GoogleSearchProvider(config);
          case "bing":
            return new BingSearchProvider(config);
          case "duckduckgo":
            return new DuckDuckGoSearchProvider(config);
          case "mock":
            return new MockSearchProvider(config);
          default:
            throw new Error(`Unknown web search provider: ${config.name}`);
        }

      case SearchProviderType.ACADEMIC_SEARCH:
        // Future: arXiv, PubMed, etc.
        throw new Error("Academic search not yet implemented");

      default:
        throw new Error(`Unknown search provider type: ${config.type}`);
    }
  }
}
```

#### 5. Update Configuration

**File**: Configuration example in docs

```typescript
const config: ArbiterOrchestratorConfig = {
  // ...
  knowledgeSeeker: {
    providers: [
      {
        name: "google",
        type: SearchProviderType.WEB_SEARCH,
        endpoint: "https://www.googleapis.com/customsearch/v1",
        priority: 1, // Primary provider
        rateLimit: {
          requestsPerMinute: 100, // Adjust based on your quota
          requestsPerHour: 6000,
        },
        timeout: 5000,
        enabled: true,
      },
      {
        name: "bing",
        type: SearchProviderType.WEB_SEARCH,
        endpoint: "https://api.bing.microsoft.com/v7.0/search",
        priority: 2, // Fallback provider
        rateLimit: {
          requestsPerMinute: 50,
          requestsPerHour: 1000,
        },
        timeout: 5000,
        enabled: true,
      },
      {
        name: "duckduckgo",
        type: SearchProviderType.WEB_SEARCH,
        endpoint: "https://api.duckduckgo.com/",
        priority: 3, // Second fallback
        rateLimit: {
          requestsPerMinute: 30, // Be conservative
          requestsPerHour: 500,
        },
        timeout: 5000,
        enabled: true,
      },
      {
        name: "mock",
        type: SearchProviderType.WEB_SEARCH,
        endpoint: "mock://",
        priority: 99, // Development only
        rateLimit: {
          requestsPerMinute: 1000,
          requestsPerHour: 10000,
        },
        timeout: 1000,
        enabled: process.env.NODE_ENV === "development",
      },
    ],
    // ... rest of config
  },
};
```

#### 6. Implement Provider Fallback Chain

**Enhancement to KnowledgeSeeker**:

```typescript
private async searchWithFallback(
  query: KnowledgeQuery,
  providers: ISearchProvider[]
): Promise<SearchResult[]> {
  const sortedProviders = providers.sort((a, b) =>
    (a.priority || 999) - (b.priority || 999)
  );

  for (const provider of sortedProviders) {
    try {
      const results = await provider.search(query);

      if (results.length > 0) {
        // Update provider health as successful
        await this.updateProviderHealth(provider.name, {
          available: true,
          lastChecked: new Date(),
          responseTimeMs: Date.now() - startTime,
          errorRate: 0,
        });

        return results;
      }
    } catch (error) {
      console.warn(`Provider ${provider.name} failed:`, error);

      // Update provider health as failed
      await this.updateProviderHealth(provider.name, {
        available: false,
        lastChecked: new Date(),
        errorMessage: error instanceof Error ? error.message : String(error),
      });

      // Continue to next provider
      continue;
    }
  }

  throw new Error("All search providers failed");
}
```

### Testing Strategy

#### Unit Tests

```bash
# Test each provider independently
npm test src/knowledge/providers/GoogleSearchProvider.test.ts
npm test src/knowledge/providers/BingSearchProvider.test.ts
npm test src/knowledge/providers/DuckDuckGoSearchProvider.test.ts
```

#### Integration Tests

```bash
# Test with real APIs (requires API keys)
GOOGLE_SEARCH_API_KEY=xxx GOOGLE_SEARCH_CX=xxx npm run test:integration
```

#### End-to-End Tests

```bash
# Test full flow: Worker → MCP → Orchestrator → Real Search
npm run test:e2e
```

### Acceptance Criteria

- [ ] GoogleSearchProvider works with real API
- [ ] BingSearchProvider works with real API
- [ ] DuckDuckGoSearchProvider works with real API
- [ ] Provider fallback chain works correctly
- [ ] Rate limiting enforced per provider
- [ ] Provider health tracking updates correctly
- [ ] All integration tests pass
- [ ] API errors handled gracefully
- [ ] Documentation updated with API key setup

### Estimated Completion Time

**2-3 days** (assuming API keys are available)

---

## Phase 4: Task-Driven Research Implementation

### Overview

Automatically detect when tasks require research and augment task context with knowledge findings.

### Implementation Tasks

#### 1. Research Detection Heuristics

**File**: `src/orchestrator/research/ResearchDetector.ts`

```typescript
/**
 * Detects when tasks require research
 */
export class ResearchDetector {
  detectResearchNeeds(task: Task): ResearchRequirement | null {
    const indicators = {
      hasQuestions: this.containsQuestions(task.description),
      hasUncertainty: this.containsUncertaintyKeywords(task.description),
      requiresFactChecking: this.requiresFactChecking(task.type),
      needsComparison: this.needsComparison(task.description),
      requiresTechnicalInfo: this.requiresTechnicalInfo(task.description),
    };

    const confidence = this.calculateResearchConfidence(indicators);

    if (confidence > 0.7) {
      return {
        required: true,
        confidence,
        queryType: this.inferQueryType(indicators),
        suggestedQueries: this.generateQueries(task),
      };
    }

    return null;
  }

  private containsQuestions(text: string): boolean {
    const questionPatterns = /\b(what|how|why|when|where|who|which)\b.*\?/gi;
    return questionPatterns.test(text);
  }

  private containsUncertaintyKeywords(text: string): boolean {
    const uncertaintyWords = [
      "not sure",
      "unclear",
      "unknown",
      "uncertain",
      "don't know",
      "need to find",
      "research",
      "investigate",
    ];
    return uncertaintyWords.some((word) => text.toLowerCase().includes(word));
  }
}
```

#### 2. Task Context Augmentation

**File**: `src/orchestrator/research/TaskResearchAugmenter.ts`

```typescript
/**
 * Augments tasks with research findings
 */
export class TaskResearchAugmenter {
  constructor(
    private knowledgeSeeker: IKnowledgeSeeker,
    private researchDetector: ResearchDetector
  ) {}

  async augmentTask(task: Task): Promise<AugmentedTask> {
    // Detect if research needed
    const researchReq = this.researchDetector.detectResearchNeeds(task);

    if (!researchReq || !researchReq.required) {
      return { ...task, researchProvided: false };
    }

    // Perform research
    const researchResults = await this.performResearch(
      researchReq.suggestedQueries,
      researchReq.queryType
    );

    // Augment task with research context
    return {
      ...task,
      researchProvided: true,
      researchContext: {
        queries: researchReq.suggestedQueries,
        findings: researchResults,
        confidence: researchReq.confidence,
        augmentedAt: new Date(),
      },
    };
  }

  private async performResearch(
    queries: string[],
    queryType: QueryType
  ): Promise<ResearchFindings[]> {
    const results = await Promise.all(
      queries.map((query) =>
        this.knowledgeSeeker.processQuery({
          id: `research-${Date.now()}-${Math.random()}`,
          query,
          queryType,
          maxResults: 3, // Limit for task augmentation
          relevanceThreshold: 0.8, // High bar for task context
          timeoutMs: 5000, // Fast timeout
          context: { purpose: "task-augmentation" },
          metadata: {
            requesterId: "task-research-augmenter",
            priority: 7, // Lower than direct requests
            createdAt: new Date(),
          },
        })
      )
    );

    return results.map((r) => ({
      query: r.query.query,
      summary: r.summary,
      confidence: r.confidence,
      keyFindings: r.results.slice(0, 3), // Top 3 results only
    }));
  }
}
```

#### 3. Integration with TaskRoutingManager

**File**: `src/orchestrator/TaskRoutingManager.ts` (enhancement)

```typescript
export class TaskRoutingManager {
  private taskResearchAugmenter: TaskResearchAugmenter;

  async routeTask(task: Task): Promise<RoutingDecision> {
    // Augment task with research if needed
    const augmentedTask = await this.taskResearchAugmenter.augmentTask(task);

    // Update task with research context
    if (augmentedTask.researchProvided) {
      console.log(`Task ${task.id} augmented with research:`, {
        queries: augmentedTask.researchContext.queries.length,
        confidence: augmentedTask.researchContext.confidence,
      });
    }

    // Continue with normal routing using augmented task
    return await this.selectAgent(augmentedTask);
  }
}
```

#### 4. Research Provenance Tracking

**File**: `src/orchestrator/research/ResearchProvenance.ts`

```typescript
/**
 * Tracks research provenance for audit trails
 */
export class ResearchProvenance {
  async recordResearch(
    taskId: string,
    research: ResearchContext
  ): Promise<void> {
    await this.dbClient.execute({
      query: `
        INSERT INTO task_research_provenance (
          task_id, queries, findings_count, 
          confidence, augmented_at
        ) VALUES ($1, $2, $3, $4, $5)
      `,
      params: [
        taskId,
        JSON.stringify(research.queries),
        research.findings.length,
        research.confidence,
        research.augmentedAt,
      ],
    });
  }

  async getTaskResearch(taskId: string): Promise<ResearchContext | null> {
    // Retrieve research context for a task
  }
}
```

### Performance Optimization

**Target**: <2s overhead for research augmentation

**Strategies**:

1. Parallel query execution
2. Aggressive caching (research by task type)
3. Limit results per query (3 max)
4. Short timeouts (5s max)
5. Skip research for simple tasks

### Acceptance Criteria

- [ ] Research detection accuracy >80%
- [ ] Task augmentation overhead <2s
- [ ] Research tracked in provenance
- [ ] Integration with routing works
- [ ] Performance targets met
- [ ] All tests pass

### Estimated Completion Time

**2-3 days**

---

## Recommended Implementation Order

1. **Week 1**: Phase 2 (Real Search Providers)
   - Days 1-2: Implement providers
   - Day 3: Testing and integration
2. **Week 2**: Phase 4 (Task-Driven Research)

   - Days 1-2: Implement detection and augmentation
   - Day 3: Integration and optimization

3. **Week 3**: Phase 5 (Documentation & Production)
   - Days 1-2: Write comprehensive tests
   - Day 3: Final docs and readiness check

---

## Success Metrics

### Phase 2 Success

- ✅ 3 real search providers working
- ✅ Fallback chain functional
- ✅ Rate limiting enforced
- ✅ >90% provider uptime

### Phase 4 Success

- ✅ >80% research detection accuracy
- ✅ <2s augmentation overhead
- ✅ Provenance tracked
- ✅ Routing integration works

### Overall Success (100% Theory Aligned)

- ✅ All 5 phases complete
- ✅ >85% test coverage
- ✅ >85% production readiness
- ✅ Zero critical issues

---

## Getting Help

### If Stuck on Phase 2

- Check API documentation links in this guide
- Test each provider independently
- Use curl/Postman to verify API keys work
- Check rate limits and quotas

### If Stuck on Phase 4

- Review existing task routing logic
- Start with simple heuristics
- Profile performance early
- Use feature flags for gradual rollout

### Resources

- [Google Custom Search Docs](https://developers.google.com/custom-search)
- [Bing Search API Docs](https://docs.microsoft.com/en-us/bing/search-apis/)
- [DuckDuckGo API Docs](https://duckduckgo.com/api)
- [Phase 1 Complete](./ARBITER-006-PHASE-1-COMPLETE.md)
- [Phase 3 Complete](./ARBITER-006-PHASE-3-COMPLETE.md)
- [Implementation Summary](./ARBITER-006-IMPLEMENTATION-SUMMARY.md)

---

**Last Updated**: October 12, 2025  
**Status**: Ready to proceed with Phase 2 or Phase 4  
**Estimated Total Time to 100%**: 5-7 days
