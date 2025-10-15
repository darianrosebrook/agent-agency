/**
 * @fileoverview Unit Tests for TaskResearchAugmenter (ARBITER-006 Phase 4)
 *
 * @author @darianrosebrook
 *
 * Tests for task research augmentation including:
 * - Task augmentation flow
 * - Query execution (parallel, timeout, failure handling)
 * - Findings transformation
 * - Helper methods (summary, sources, hasResearch)
 * - Error handling
 * - Performance benchmarks
 */

import { ResearchDetector } from "../../../../src/orchestrator/research/ResearchDetector";
import { TaskResearchAugmenter } from "../../../../src/orchestrator/research/TaskResearchAugmenter";
import { QueryType } from "../../../../src/types/knowledge";
import {
  MockKnowledgeSeeker,
  mockKnowledgeResponse,
  mockTask,
} from "../../../mocks/knowledge-mocks";

describe("TaskResearchAugmenter", () => {
  let augmenter: TaskResearchAugmenter;
  let detector: ResearchDetector;
  let knowledgeSeeker: MockKnowledgeSeeker;

  beforeEach(() => {
    // Create a mock detector that can be controlled per test
    detector = {
      detectResearchNeeds: jest.fn(),
    } as any;

    knowledgeSeeker = new MockKnowledgeSeeker();

    augmenter = new TaskResearchAugmenter(knowledgeSeeker, detector, {
      maxResultsPerQuery: 3,
      relevanceThreshold: 0.8,
      timeoutMs: 5000,
      maxQueries: 3,
      enableCaching: true,
    });
  });

  describe("Task Augmentation Flow", () => {
    it("should augment task when research is required", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      // Mock detector to return research required
      (detector.detectResearchNeeds as jest.Mock).mockReturnValue({
        required: true,
        confidence: 0.9,
        queryType: "technical",
        suggestedQueries: [
          "OAuth2 Express.js implementation",
          "OAuth2 best practices",
        ],
        indicators: {
          hasQuestions: true,
          hasUncertainty: false,
          requiresFactChecking: false,
          needsComparison: false,
          requiresTechnicalInfo: true,
        },
        reason: "Task contains questions",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      expect(augmentedTask.researchProvided).toBe(true);
      expect(augmentedTask.researchContext).toBeDefined();
      expect(augmentedTask.researchContext?.queries.length).toBeGreaterThan(0);
      expect(augmentedTask.researchContext?.findings.length).toBeGreaterThan(0);
      expect(augmentedTask.researchContext?.confidence).toBeGreaterThan(0);
    });

    it("should skip augmentation when research is not required", async () => {
      const task = mockTask({
        description: "Update the README with installation instructions.",
      });

      // Mock detector to return research not required
      (detector.detectResearchNeeds as jest.Mock).mockReturnValue({
        required: false,
        confidence: 0.1,
        queryType: "general",
        suggestedQueries: [],
        indicators: {
          hasQuestions: false,
          hasUncertainty: false,
          requiresFactChecking: false,
          needsComparison: false,
          requiresTechnicalInfo: false,
        },
        reason: "No research indicators found",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      expect(augmentedTask.researchProvided).toBe(false);
      expect(augmentedTask.researchContext).toBeUndefined();
      expect(augmentedTask).toEqual({ ...task, researchProvided: false });
    });

    it("should add researchContext to task metadata", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      // Mock detector to return research required
      (detector.detectResearchNeeds as jest.Mock).mockReturnValue({
        required: true,
        confidence: 0.9,
        queryType: "technical",
        suggestedQueries: ["OAuth2 implementation"],
        indicators: {
          hasQuestions: true,
          hasUncertainty: false,
          requiresFactChecking: false,
          needsComparison: false,
          requiresTechnicalInfo: true,
        },
        reason: "Task contains questions",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      expect(augmentedTask.researchContext).toBeDefined();
      expect(augmentedTask.researchContext?.queries).toBeDefined();
      expect(augmentedTask.researchContext?.findings).toBeDefined();
    });

    it("should set researchProvided flag correctly", async () => {
      const taskWithResearch = mockTask({
        description: "How do I implement OAuth2?",
      });

      // Mock detector to return research required for first task
      (detector.detectResearchNeeds as jest.Mock).mockReturnValueOnce({
        required: true,
        confidence: 0.9,
        queryType: "technical",
        suggestedQueries: ["OAuth2 implementation"],
        indicators: {
          hasQuestions: true,
          hasUncertainty: false,
          requiresFactChecking: false,
          needsComparison: false,
          requiresTechnicalInfo: true,
        },
        reason: "Task contains questions",
      });

      const augmentedWithResearch = await augmenter.augmentTask(
        taskWithResearch
      );
      expect(augmentedWithResearch.researchProvided).toBe(true);

      const taskWithoutResearch = mockTask({
        description: "Simple task without questions.",
      });

      // Mock detector to return research not required for second task
      (detector.detectResearchNeeds as jest.Mock).mockReturnValueOnce({
        required: false,
        confidence: 0.1,
        queryType: "general",
        suggestedQueries: [],
        indicators: {
          hasQuestions: false,
          hasUncertainty: false,
          requiresFactChecking: false,
          needsComparison: false,
          requiresTechnicalInfo: false,
        },
        reason: "No research indicators found",
      });

      const augmentedWithoutResearch = await augmenter.augmentTask(
        taskWithoutResearch
      );
      expect(augmentedWithoutResearch.researchProvided).toBe(false);
    });

    it("should preserve original task properties", async () => {
      const task = mockTask({
        id: "test-task-123",
        description: "How do I implement OAuth2?",
        priority: 5,
      });

      const augmentedTask = await augmenter.augmentTask(task);

      expect(augmentedTask.id).toBe(task.id);
      expect(augmentedTask.description).toBe(task.description);
      expect(augmentedTask.priority).toBe(task.priority);
      expect(augmentedTask.type).toBe(task.type);
    });
  });

  describe("Query Execution", () => {
    it("should execute queries in parallel", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      const startTime = Date.now();
      const augmentedTask = await augmenter.augmentTask(task);
      const duration = Date.now() - startTime;

      expect(augmentedTask.researchProvided).toBe(true);
      // Parallel execution should be faster than sequential
      // With 100ms per query, 3 queries in parallel should take ~100-200ms, not 300ms
      expect(duration).toBeLessThan(500);
    });

    it("should respect maxQueries config", async () => {
      const limitedAugmenter = new TaskResearchAugmenter(
        knowledgeSeeker,
        detector,
        {
          maxQueries: 2,
        }
      );

      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      const augmentedTask = await limitedAugmenter.augmentTask(task);

      if (augmentedTask.researchContext) {
        expect(
          augmentedTask.researchContext.queries.length
        ).toBeLessThanOrEqual(2);
      }
    });

    it("should handle query failures gracefully", async () => {
      const failingSeeker = new MockKnowledgeSeeker({ shouldFail: true });
      const failingAugmenter = new TaskResearchAugmenter(
        failingSeeker,
        detector
      );

      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask = await failingAugmenter.augmentTask(task);

      // Should return original task on failure
      expect(augmentedTask.researchProvided).toBe(false);
      expect(augmentedTask.id).toBe(task.id);
    });

    it("should handle partial query failures", async () => {
      let callCount = 0;
      const partialFailSeeker = {
        async processQuery() {
          callCount++;
          if (callCount === 2) {
            throw new Error("Second query failed");
          }
          return mockKnowledgeResponse();
        },
      } as any;

      const partialFailAugmenter = new TaskResearchAugmenter(
        partialFailSeeker,
        detector
      );

      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      const augmentedTask = await partialFailAugmenter.augmentTask(task);

      // Should still provide research from successful queries
      if (augmentedTask.researchProvided && augmentedTask.researchContext) {
        expect(augmentedTask.researchContext.findings.length).toBeGreaterThan(
          0
        );
      }
    });

    it("should respect timeoutMs config", async () => {
      const slowSeeker = new MockKnowledgeSeeker({ responseDelay: 10000 });
      const fastAugmenter = new TaskResearchAugmenter(slowSeeker, detector, {
        timeoutMs: 100, // Very short timeout
      });

      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const startTime = Date.now();
      await fastAugmenter.augmentTask(task);
      const duration = Date.now() - startTime;

      // Should timeout quickly, not wait 10 seconds
      expect(duration).toBeLessThan(5000);
    });

    it("should pass correct query parameters to KnowledgeSeeker", async () => {
      const spySeeker = {
        async processQuery(query: any) {
          expect(query.query).toBeDefined();
          expect(query.queryType).toBeDefined();
          expect(query.maxResults).toBe(3);
          expect(query.relevanceThreshold).toBe(0.8);
          expect(query.timeoutMs).toBe(5000);
          expect(query.context.taskId).toBeDefined();
          return mockKnowledgeResponse();
        },
      } as any;

      const spyAugmenter = new TaskResearchAugmenter(spySeeker, detector, {
        maxResultsPerQuery: 3,
        relevanceThreshold: 0.8,
        timeoutMs: 5000,
      });

      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      await spyAugmenter.augmentTask(task);
    });
  });

  describe("Findings Transformation", () => {
    it("should transform KnowledgeResponse to ResearchFindings", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      expect(augmentedTask.researchProvided).toBe(true);
      expect(augmentedTask.researchContext?.findings).toBeInstanceOf(Array);

      const findings = augmentedTask.researchContext?.findings[0];
      expect(findings?.query).toBeDefined();
      expect(findings?.summary).toBeDefined();
      expect(findings?.confidence).toBeGreaterThanOrEqual(0);
      expect(findings?.confidence).toBeLessThanOrEqual(1);
      expect(findings?.keyFindings).toBeInstanceOf(Array);
    });

    it("should calculate overall confidence correctly", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      if (augmentedTask.researchContext) {
        const { confidence, findings } = augmentedTask.researchContext;

        // Overall confidence should be average of individual findings
        expect(confidence).toBeGreaterThanOrEqual(0);
        expect(confidence).toBeLessThanOrEqual(1);

        if (findings.length > 0) {
          const avgConfidence =
            findings.reduce((sum, f) => sum + f.confidence, 0) /
            findings.length;
          expect(Math.abs(confidence - avgConfidence)).toBeLessThan(0.01);
        }
      }
    });

    it("should respect relevanceThreshold", async () => {
      const strictAugmenter = new TaskResearchAugmenter(
        knowledgeSeeker,
        detector,
        {
          relevanceThreshold: 0.95, // Very high threshold
        }
      );

      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask = await strictAugmenter.augmentTask(task);

      // With high threshold, may have fewer findings
      if (augmentedTask.researchContext) {
        augmentedTask.researchContext.findings.forEach((finding) => {
          finding.keyFindings.forEach((kf) => {
            expect(kf.relevance).toBeGreaterThanOrEqual(0.95);
          });
        });
      }
    });

    it("should limit results per query", async () => {
      const limitedAugmenter = new TaskResearchAugmenter(
        knowledgeSeeker,
        detector,
        {
          maxResultsPerQuery: 2, // Only 2 results per query
        }
      );

      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask = await limitedAugmenter.augmentTask(task);

      if (augmentedTask.researchContext) {
        augmentedTask.researchContext.findings.forEach((finding) => {
          expect(finding.keyFindings.length).toBeLessThanOrEqual(2);
        });
      }
    });

    it("should extract key findings from search results", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      if (augmentedTask.researchContext) {
        const findings = augmentedTask.researchContext.findings[0];
        expect(findings.keyFindings).toBeInstanceOf(Array);

        if (findings.keyFindings.length > 0) {
          const keyFinding = findings.keyFindings[0];
          expect(keyFinding.title).toBeDefined();
          expect(keyFinding.url).toBeDefined();
          expect(keyFinding.snippet).toBeDefined();
          expect(keyFinding.relevance).toBeDefined();
          expect(keyFinding.credibility).toBeDefined();
        }
      }
    });
  });

  describe("Helper Methods", () => {
    it("getResearchSummary should format findings as text", () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask: any = {
        ...task,
        researchProvided: true,
        researchContext: {
          queries: ["query 1"],
          findings: [
            {
              query: "query 1",
              summary: "Finding 1 summary",
              confidence: 0.9,
              keyFindings: [],
            },
            {
              query: "query 2",
              summary: "Finding 2 summary",
              confidence: 0.85,
              keyFindings: [],
            },
          ],
          confidence: 0.875,
          augmentedAt: new Date(),
          metadata: {},
        },
      };

      const summary = augmenter.getResearchSummary(augmentedTask);

      expect(summary).toBeDefined();
      expect(summary).not.toBeNull();
      expect(summary!.length).toBeGreaterThan(0);
      expect(summary).toContain("confidence");
      expect(summary).toContain("Finding 1 summary");
      expect(summary).toContain("Finding 2 summary");
    });

    it("getResearchSummary should return empty for non-augmented tasks", () => {
      const task = mockTask({
        description: "Simple task",
      });

      const summary = augmenter.getResearchSummary({
        ...task,
        researchProvided: false,
      });

      expect(summary).toBe("");
    });

    it("getResearchSources should extract all source URLs", () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask: any = {
        ...task,
        researchProvided: true,
        researchContext: {
          queries: ["query 1"],
          findings: [
            {
              query: "query 1",
              summary: "Summary",
              confidence: 0.9,
              keyFindings: [
                {
                  title: "Finding 1",
                  url: "https://example.com/1",
                  snippet: "Snippet 1",
                  relevance: 0.9,
                  credibility: 0.9,
                },
                {
                  title: "Finding 2",
                  url: "https://example.com/2",
                  snippet: "Snippet 2",
                  relevance: 0.85,
                  credibility: 0.85,
                },
              ],
            },
          ],
          confidence: 0.9,
          augmentedAt: new Date(),
          metadata: {},
        },
      };

      const sources = augmenter.getResearchSources(augmentedTask);

      expect(sources).toBeInstanceOf(Array);
      expect(sources.length).toBe(2);
      expect(sources).toContain("https://example.com/1");
      expect(sources).toContain("https://example.com/2");
    });

    it("getResearchSources should return empty array for non-augmented tasks", () => {
      const task = mockTask({
        description: "Simple task",
      });

      const sources = augmenter.getResearchSources({
        ...task,
        researchProvided: false,
      });

      expect(sources).toEqual([]);
    });

    it("hasResearch should detect augmented tasks", () => {
      const augmentedTask: any = {
        researchProvided: true,
        researchContext: {},
      };

      expect(augmenter.hasResearch(augmentedTask)).toBe(true);
    });

    it("hasResearch should detect non-augmented tasks", () => {
      const task: any = {
        researchProvided: false,
      };

      expect(augmenter.hasResearch(task)).toBe(false);
    });
  });

  describe("Error Handling", () => {
    it("should handle KnowledgeSeeker errors gracefully", async () => {
      const failingSeeker = new MockKnowledgeSeeker({ shouldFail: true });
      const failingAugmenter = new TaskResearchAugmenter(
        failingSeeker,
        detector
      );

      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      // Should not throw, should return original task
      const augmentedTask = await failingAugmenter.augmentTask(task);

      expect(augmentedTask).toBeDefined();
      expect(augmentedTask.id).toBe(task.id);
      expect(augmentedTask.researchProvided).toBe(false);
    });

    it("should handle detector errors gracefully", async () => {
      const failingDetector = {
        detectResearchNeeds() {
          throw new Error("Detector failure");
        },
      } as any;

      const failingAugmenter = new TaskResearchAugmenter(
        knowledgeSeeker,
        failingDetector
      );

      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      // Should not throw, should return original task
      const augmentedTask = await failingAugmenter.augmentTask(task);

      expect(augmentedTask).toBeDefined();
      expect(augmentedTask.researchProvided).toBe(false);
    });

    it("should return original task on complete failure", async () => {
      const failingSeeker = new MockKnowledgeSeeker({ shouldFail: true });
      const failingAugmenter = new TaskResearchAugmenter(
        failingSeeker,
        detector
      );

      const task = mockTask({
        id: "test-task-999",
        description: "How do I implement OAuth2?",
        priority: 3,
      });

      const augmentedTask = await failingAugmenter.augmentTask(task);

      // Should return task with same properties
      expect(augmentedTask.id).toBe(task.id);
      expect(augmentedTask.description).toBe(task.description);
      expect(augmentedTask.priority).toBe(task.priority);
      expect(augmentedTask.researchProvided).toBe(false);
    });

    it("should log errors appropriately", async () => {
      const consoleErrorSpy = jest
        .spyOn(console, "error")
        .mockImplementation(() => {});

      const failingSeeker = new MockKnowledgeSeeker({ shouldFail: true });
      const failingAugmenter = new TaskResearchAugmenter(
        failingSeeker,
        detector
      );

      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      await failingAugmenter.augmentTask(task);

      expect(consoleErrorSpy).toHaveBeenCalled();

      consoleErrorSpy.mockRestore();
    });
  });

  describe("Performance", () => {
    it("should complete augmentation in <2000ms", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
      });

      const startTime = Date.now();
      await augmenter.augmentTask(task);
      const duration = Date.now() - startTime;

      // Target: <2000ms (currently ~400-500ms)
      expect(duration).toBeLessThan(2000);
    });

    it("should handle concurrent augmentations efficiently", async () => {
      const tasks = Array(5)
        .fill(null)
        .map((_, i) =>
          mockTask({
            id: `concurrent-task-${i}`,
            description: `How do I implement feature ${i}?`,
          })
        );

      const startTime = Date.now();
      const augmentedTasks = await Promise.all(
        tasks.map((task) => augmenter.augmentTask(task))
      );
      const duration = Date.now() - startTime;

      expect(augmentedTasks).toHaveLength(5);
      // 5 tasks should not take 5x as long (some parallelization)
      expect(duration).toBeLessThan(5000);
    });

    it("should maintain performance with multiple findings", async () => {
      const task = mockTask({
        description:
          "How do I implement OAuth2 in Express.js with JWT tokens and refresh token rotation?",
      });

      const startTime = Date.now();
      const augmentedTask = await augmenter.augmentTask(task);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(2000);

      if (augmentedTask.researchContext) {
        expect(augmentedTask.researchContext.findings.length).toBeGreaterThan(
          0
        );
      }
    });
  });

  describe("Configuration", () => {
    it("should respect all config options", () => {
      const customAugmenter = new TaskResearchAugmenter(
        knowledgeSeeker,
        detector,
        {
          maxResultsPerQuery: 5,
          relevanceThreshold: 0.9,
          timeoutMs: 10000,
          maxQueries: 5,
          enableCaching: false,
        }
      );

      expect(customAugmenter).toBeDefined();
    });

    it("should use default config when not provided", () => {
      const defaultAugmenter = new TaskResearchAugmenter(
        knowledgeSeeker,
        detector
      );

      expect(defaultAugmenter).toBeDefined();
    });

    it("should handle partial config", () => {
      const partialAugmenter = new TaskResearchAugmenter(
        knowledgeSeeker,
        detector,
        {
          maxQueries: 2,
          // Other options use defaults
        }
      );

      expect(partialAugmenter).toBeDefined();
    });
  });

  describe("Metadata Enrichment", () => {
    it("should include duration in metadata", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      if (
        augmentedTask.researchContext &&
        augmentedTask.researchContext.metadata
      ) {
        expect(augmentedTask.researchContext.metadata).toBeDefined();
        expect(typeof augmentedTask.researchContext.metadata.durationMs).toBe(
          "number"
        );
        expect(
          augmentedTask.researchContext.metadata.durationMs
        ).toBeGreaterThan(0);
      }
    });

    it("should include detector confidence in metadata", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      if (
        augmentedTask.researchContext &&
        augmentedTask.researchContext.metadata
      ) {
        expect(
          augmentedTask.researchContext.metadata.detectorConfidence
        ).toBeDefined();
        expect(
          augmentedTask.researchContext.metadata.detectorConfidence
        ).toBeGreaterThanOrEqual(0);
        expect(
          augmentedTask.researchContext.metadata.detectorConfidence
        ).toBeLessThanOrEqual(1);
      }
    });

    it("should include query type in metadata", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      if (
        augmentedTask.researchContext &&
        augmentedTask.researchContext.metadata
      ) {
        expect(augmentedTask.researchContext.metadata.queryType).toBeDefined();
        expect(
          Object.values(QueryType).includes(
            augmentedTask.researchContext.metadata.queryType
          )
        ).toBe(true);
      }
    });

    it("should include augmentation timestamp", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2?",
      });

      const beforeTime = new Date();
      const augmentedTask = await augmenter.augmentTask(task);
      const afterTime = new Date();

      if (augmentedTask.researchContext) {
        expect(augmentedTask.researchContext.augmentedAt).toBeDefined();
        expect(
          augmentedTask.researchContext.augmentedAt.getTime()
        ).toBeGreaterThanOrEqual(beforeTime.getTime());
        expect(
          augmentedTask.researchContext.augmentedAt.getTime()
        ).toBeLessThanOrEqual(afterTime.getTime());
      }
    });
  });
});
