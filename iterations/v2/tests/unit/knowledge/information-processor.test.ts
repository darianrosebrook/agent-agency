/**
 * @fileoverview Tests for Information Processor Component (ARBITER-006)
 *
 * @author @darianrosebrook
 */

import { InformationProcessor } from "../../../src/knowledge/InformationProcessor";
import { events } from "../../../src/orchestrator/EventEmitter";
import {
  VerificationPriority,
  InformationProcessorConfig,
  KnowledgeQuery,
  QueryType,
  ResultQuality,
  SearchResult,
  SourceType,
} from "../../../src/types/knowledge";

describe("InformationProcessor", () => {
  let processor: InformationProcessor;
  let testQuery: KnowledgeQuery;
  let mockResults: SearchResult[];

  const defaultConfig: InformationProcessorConfig = {
    minRelevanceScore: 0.5,
    minCredibilityScore: 0.5,
    maxResultsToProcess: 10,
    diversity: {
      minSources: 1,
      minSourceTypes: 1,
      maxResultsPerDomain: 3,
    },
    quality: {
      enableCredibilityScoring: true,
      enableRelevanceFiltering: true,
      enableDuplicateDetection: true,
    },
    caching: {
      enableResultCaching: false,
      cacheTtlMs: 3600000,
      maxCacheSize: 1000,
    },
  };

  beforeEach(() => {
    processor = new InformationProcessor(defaultConfig);
    testQuery = {
      id: "test-query",
      query: "What is TypeScript?",
      queryType: QueryType.FACTUAL,
      maxResults: 5,
      relevanceThreshold: 0.6,
      timeoutMs: 10000,
      metadata: {
        requesterId: "test-user",
        priority: 1,
        createdAt: new Date(),
      },
    };

    // Create mock search results with varying quality
    mockResults = [
      {
        id: "result-1",
        queryId: testQuery.id,
        title: "TypeScript Official Documentation",
        content:
          "TypeScript is a programming language developed by Microsoft. It is a strict syntactical superset of JavaScript.",
        url: "https://www.typescriptlang.org/docs/",
        domain: "typescriptlang.org",
        sourceType: "documentation" as SourceType,
        relevanceScore: 0.9,
        credibilityScore: 0.95,
        quality: "high" as ResultQuality,
        provider: "mock",
        providerMetadata: {},
        processedAt: new Date(),
      },
      {
        id: "result-2",
        queryId: testQuery.id,
        title: "What is TypeScript?",
        content:
          "TypeScript is a typed superset of JavaScript that compiles to plain JavaScript.",
        url: "https://example.com/typescript",
        domain: "example.com",
        sourceType: "web" as SourceType,
        relevanceScore: 0.8,
        credibilityScore: 0.7,
        quality: "medium" as ResultQuality,
        provider: "mock",
        providerMetadata: {},
        processedAt: new Date(),
      },
      {
        id: "result-3",
        queryId: testQuery.id,
        title: "TypeScript Tutorial",
        content: "Learn TypeScript basics and advanced features.",
        url: "https://tutorial.example.com/typescript",
        domain: "tutorial.example.com",
        sourceType: "web" as SourceType,
        relevanceScore: 0.6,
        credibilityScore: 0.6,
        quality: "medium" as ResultQuality,
        provider: "mock",
        providerMetadata: {},
        processedAt: new Date(),
      },
      {
        id: "result-4",
        queryId: testQuery.id,
        title: "Old TypeScript Info",
        content: "Outdated information about TypeScript from 2015.",
        url: "https://old.example.com/typescript",
        domain: "old.example.com",
        sourceType: "web" as SourceType,
        relevanceScore: 0.4,
        credibilityScore: 0.3,
        quality: "low" as ResultQuality,
        publishedAt: new Date("2015-01-01"),
        provider: "mock",
        providerMetadata: {},
        processedAt: new Date(),
      },
      // Duplicate result
      {
        id: "result-5",
        queryId: testQuery.id,
        title: "TypeScript Official Documentation",
        content:
          "TypeScript is a programming language developed by Microsoft. It is a strict syntactical superset of JavaScript.",
        url: "https://www.typescriptlang.org/docs/",
        domain: "typescriptlang.org",
        sourceType: "documentation" as SourceType,
        relevanceScore: 0.9,
        credibilityScore: 0.95,
        quality: "high" as ResultQuality,
        provider: "mock",
        providerMetadata: {},
        processedAt: new Date(),
      },
    ];
  });

  describe("Result Processing", () => {
    it("should process results and filter by quality thresholds", async () => {
      const processed = await processor.processResults(testQuery, mockResults);

      expect(processed.length).toBeGreaterThan(0);
      expect(processed.length).toBeLessThanOrEqual(mockResults.length);

      // All results should meet minimum thresholds
      processed.forEach((result) => {
        expect(result.relevanceScore).toBeGreaterThanOrEqual(
          defaultConfig.minRelevanceScore
        );
        expect(result.credibilityScore).toBeGreaterThanOrEqual(
          defaultConfig.minCredibilityScore
        );
      });
    });

    it("should remove duplicate results", async () => {
      const processed = await processor.processResults(testQuery, mockResults);

      // Should have results filtered by quality thresholds AND duplicates removed
      // mockResults has 5 items, 1 duplicate, 1 low quality -> expect 3
      expect(processed.length).toBe(3);

      // Should not have duplicate domains
      const domains = processed.map((r) => r.domain);
      const uniqueDomains = new Set(domains);
      expect(uniqueDomains.size).toBe(domains.length);
    });

    it("should respect max results limit", async () => {
      const limitedConfig = { ...defaultConfig, maxResultsToProcess: 2 };
      const limitedProcessor = new InformationProcessor(limitedConfig);

      const processed = await limitedProcessor.processResults(
        testQuery,
        mockResults
      );
      expect(processed.length).toBeLessThanOrEqual(2);
    });

    it("should sort results by combined score", async () => {
      const processed = await processor.processResults(testQuery, mockResults);

      // Results should be sorted in descending order of combined score
      for (let i = 1; i < processed.length; i++) {
        const currentScore =
          processed[i].relevanceScore + processed[i].credibilityScore;
        const previousScore =
          processed[i - 1].relevanceScore + processed[i - 1].credibilityScore;
        expect(previousScore).toBeGreaterThanOrEqual(currentScore);
      }
    });
  });

  describe("Relevance Scoring", () => {
    it("should score relevance based on query match", () => {
      const result1 = mockResults[0]; // High relevance
      const result4 = mockResults[3]; // Low relevance

      const score1 = processor.scoreRelevance(testQuery, result1);
      const score4 = processor.scoreRelevance(testQuery, result4);

      expect(score1).toBeGreaterThan(score4);
      expect(score1).toBeGreaterThan(0.5); // Should be above base score due to word matches
      expect(score4).toBeGreaterThan(0.5); // Should be above base score due to "typescript" word match
    });

    it("should adjust scores based on query type", () => {
      const academicQuery = { ...testQuery, queryType: QueryType.EXPLANATORY };
      const technicalQuery = { ...testQuery, queryType: QueryType.TECHNICAL };

      const docResult = mockResults[0]; // Documentation source
      const webResult = mockResults[1]; // Web source

      const academicScore = processor.scoreRelevance(academicQuery, docResult);
      const technicalScore = processor.scoreRelevance(
        technicalQuery,
        docResult
      );

      // Documentation should score higher for technical queries
      expect(technicalScore).toBeGreaterThan(academicScore);
    });

    it("should boost scores for recent content", () => {
      const recentResult = { ...mockResults[0], publishedAt: new Date() };
      const oldResult = {
        ...mockResults[3],
        publishedAt: new Date("2010-01-01"),
      };

      const recentScore = processor.scoreRelevance(testQuery, recentResult);
      const oldScore = processor.scoreRelevance(testQuery, oldResult);

      expect(recentScore).toBeGreaterThan(oldScore);
    });
  });

  describe("Credibility Assessment", () => {
    it("should assess credibility based on source type", () => {
      const docResult = mockResults[0]; // Documentation
      const webResult = mockResults[1]; // Web

      const docCredibility = processor.assessCredibility(docResult);
      const webCredibility = processor.assessCredibility(webResult);

      expect(docCredibility).toBeGreaterThan(webCredibility);
    });

    it("should boost credibility for trusted domains", () => {
      const trustedResult = {
        ...mockResults[1],
        domain: "edu",
        url: "https://university.edu/typescript",
      };
      const regularResult = mockResults[1];

      const trustedCredibility = processor.assessCredibility(trustedResult);
      const regularCredibility = processor.assessCredibility(regularResult);

      expect(trustedCredibility).toBeGreaterThan(regularCredibility);
    });

    it("should penalize suspicious domains", () => {
      const suspiciousResult = {
        ...mockResults[1],
        domain: "blogspot.com",
        url: "https://random.blogspot.com/typescript",
      };
      const regularResult = mockResults[1];

      const suspiciousCredibility =
        processor.assessCredibility(suspiciousResult);
      const regularCredibility = processor.assessCredibility(regularResult);

      expect(suspiciousCredibility).toBeLessThan(regularCredibility);
    });
  });

  describe("Duplicate Detection", () => {
    it("should detect exact duplicates", () => {
      const duplicates = processor.detectDuplicates(mockResults);
      expect(duplicates.length).toBe(mockResults.length - 1); // One duplicate removed
    });

    it("should preserve unique results", () => {
      const uniqueResults = [mockResults[0], mockResults[1], mockResults[2]];

      const deduplicated = processor.detectDuplicates(uniqueResults);
      expect(deduplicated.length).toBe(uniqueResults.length);
    });
  });

  describe("Summary Generation", () => {
    it("should generate meaningful summaries", () => {
      const summary = processor.generateSummary(testQuery, mockResults);

      expect(summary).toBeDefined();
      expect(summary.length).toBeGreaterThan(20);
      expect(summary.toLowerCase()).toContain("found");
      expect(summary.toLowerCase()).toContain("result");
    });

    it("should handle empty results", () => {
      const summary = processor.generateSummary(testQuery, []);

      expect(summary).toContain("No relevant information found");
      expect(summary).toContain(testQuery.query);
    });

    it("should include quality metrics in summary", () => {
      const summary = processor.generateSummary(testQuery, mockResults);

      // Should mention number of results and domains
      expect(summary).toMatch(/\d+/); // Contains numbers
    });
  });

  describe("Diversity Constraints", () => {
    it("should respect domain limits", async () => {
      const manySameDomain = Array(5)
        .fill(null)
        .map((_, i) => ({
          ...mockResults[0],
          id: `duplicate-${i}`,
          url: `https://typescriptlang.org/page${i}`,
        }));

      const processed = await processor.processResults(
        testQuery,
        manySameDomain
      );

      // Should limit to maxResultsPerDomain
      const domainCount = processed.filter(
        (r) => r.domain === "typescriptlang.org"
      ).length;
      expect(domainCount).toBeLessThanOrEqual(
        defaultConfig.diversity.maxResultsPerDomain
      );
    });

    it("should ensure source type diversity when possible", async () => {
      const diverseResults = [
        { ...mockResults[0], sourceType: "documentation" as SourceType },
        { ...mockResults[1], sourceType: "web" as SourceType },
        { ...mockResults[2], sourceType: "news" as SourceType },
      ];

      const processed = await processor.processResults(
        testQuery,
        diverseResults
      );

      const sourceTypes = new Set(processed.map((r) => r.sourceType));
      expect(sourceTypes.size).toBeGreaterThanOrEqual(
        defaultConfig.diversity.minSourceTypes
      );
    });
  });

  describe("Configuration", () => {
    it("should respect disabled features", async () => {
      const disabledConfig = {
        ...defaultConfig,
        quality: {
          ...defaultConfig.quality,
          enableRelevanceFiltering: false,
          enableDuplicateDetection: false,
        },
      };

      const disabledProcessor = new InformationProcessor(disabledConfig);
      const processed = await disabledProcessor.processResults(
        testQuery,
        mockResults
      );

      // Should include results that pass credibility threshold (always applied)
      // mockResults has 5 items, 1 with low credibility -> expect 4
      expect(processed.length).toBe(4);
    });

    it("should handle edge case configurations", async () => {
      const edgeConfig = {
        ...defaultConfig,
        minRelevanceScore: 0.0,
        minCredibilityScore: 0.0,
        maxResultsToProcess: 1,
      };

      const edgeProcessor = new InformationProcessor(edgeConfig);
      const processed = await edgeProcessor.processResults(
        testQuery,
        mockResults
      );

      expect(processed.length).toBe(1); // Limited to maxResultsToProcess
    });
  });

  afterAll(() => {
    // Clean up global event emitter to prevent Jest from hanging
    events.shutdown();
  });
});
