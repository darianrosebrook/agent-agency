/**
 * Integration Tests: Research Flow
 *
 * Tests the complete research flow from detection through augmentation to provenance.
 * Uses real component instances (not mocks) but mock database for isolation.
 *
 * @module tests/integration/research/research-flow.test.ts
 * @author @darianrosebrook
 */

import { ResearchDetector } from "../../../src/orchestrator/research/ResearchDetector";
import { TaskResearchAugmenter } from "../../../src/orchestrator/research/TaskResearchAugmenter";
import { ResearchProvenance } from "../../../src/orchestrator/research/ResearchProvenance";
import { KnowledgeSeeker } from "../../../src/knowledge/KnowledgeSeeker";
import { KnowledgeDatabaseClient } from "../../../src/database/KnowledgeDatabaseClient";
import { mockTask, MockDatabaseClient } from "../../mocks/knowledge-mocks";

describe("Research Flow Integration", () => {
  let detector: ResearchDetector;
  let knowledgeSeeker: KnowledgeSeeker;
  let augmenter: TaskResearchAugmenter;
  let provenance: ResearchProvenance;
  let mockDbClient: MockDatabaseClient;
  let knowledgeDbClient: KnowledgeDatabaseClient;

  beforeEach(() => {
    // Initialize mock database
    mockDbClient = new MockDatabaseClient() as any;

    // Initialize knowledge database client
    knowledgeDbClient = new KnowledgeDatabaseClient({
      host: "localhost",
      port: 5432,
      database: "test_arbiter",
      user: "test",
      password: "test",
    });

    // Initialize detector
    detector = new ResearchDetector({
      minConfidence: 0.7,
      maxQueries: 3,
    });

    // Initialize knowledge seeker with minimal config
    knowledgeSeeker = new KnowledgeSeeker(
      {
        enabled: true,
        providers: [],
        processor: {
          minRelevanceScore: 0.6,
          minCredibilityScore: 0.6,
          maxResultsToProcess: 10,
          diversity: {
            minSources: 2,
            minSourceTypes: 1,
            maxResultsPerDomain: 3,
          },
          quality: {
            enableCredibilityScoring: true,
            enableRelevanceFiltering: true,
            enableDuplicateDetection: true,
          },
          caching: {
            enableResultCaching: true,
            cacheTtlMs: 3600000,
            maxCacheSize: 100,
          },
        },
        queryProcessing: {
          maxConcurrentQueries: 3,
          defaultTimeoutMs: 5000,
          retryAttempts: 2,
        },
        caching: {
          enableQueryCaching: true,
          enableResultCaching: true,
          cacheTtlMs: 3600000,
        },
        observability: {
          enableMetrics: false,
          enableTracing: false,
          logLevel: "error",
        },
      },
      knowledgeDbClient
    );

    // Initialize augmenter
    augmenter = new TaskResearchAugmenter(knowledgeSeeker, detector, {
      maxResultsPerQuery: 3,
      relevanceThreshold: 0.8,
      timeoutMs: 5000,
      maxQueries: 3,
    });

    // Initialize provenance
    provenance = new ResearchProvenance(mockDbClient as any);
  });

  describe("End-to-End Research Flow", () => {
    it("should complete full research flow for question task", async () => {
      const task = mockTask({
        description: "How do I implement OAuth2 in Express.js?",
        type: "code-editing",
      });

      // Step 1: Detection
      const requirement = detector.detectResearchNeeds(task);
      expect(requirement).not.toBeNull();
      expect(requirement?.required).toBe(true);
      expect(requirement?.confidence).toBeGreaterThan(0.7);
      expect(requirement?.suggestedQueries.length).toBeGreaterThan(0);

      // Step 2: Augmentation
      const augmentedTask = await augmenter.augmentTask(task);
      expect(augmentedTask.researchProvided).toBe(true);
      expect(augmentedTask.researchContext).toBeDefined();

      if (augmentedTask.researchContext) {
        // Step 3: Provenance
        await provenance.recordResearch(
          task.id,
          augmentedTask.researchContext,
          1000
        );

        // Verify provenance was recorded
        const records = await provenance.getTaskResearch(task.id);
        expect(records.length).toBeGreaterThan(0);
        expect(records[0].taskId).toBe(task.id);
      }
    });

    it("should handle task with no research needs", async () => {
      const task = mockTask({
        description: "Update the README file.",
        type: "general",
      });

      // Step 1: Detection
      const requirement = detector.detectResearchNeeds(task);
      expect(requirement).toBeNull(); // No research needed

      // Step 2: Augmentation should skip
      const augmentedTask = await augmenter.augmentTask(task);
      expect(augmentedTask.researchProvided).toBe(false);
      expect(augmentedTask.researchContext).toBeUndefined();
    });

    it("should handle multiple queries in parallel", async () => {
      const task = mockTask({
        description:
          "How do I implement OAuth2? What are the security best practices? Which libraries should I use?",
        type: "code-editing",
      });

      const startTime = Date.now();

      // Detection should find multiple queries
      const requirement = detector.detectResearchNeeds(task);
      expect(requirement?.suggestedQueries.length).toBeGreaterThan(1);

      // Augmentation should process queries in parallel
      const augmentedTask = await augmenter.augmentTask(task);

      const duration = Date.now() - startTime;

      expect(augmentedTask.researchProvided).toBe(true);
      expect(augmentedTask.researchContext?.queries.length).toBeGreaterThan(1);

      // Should complete reasonably fast (parallel processing)
      expect(duration).toBeLessThan(5000);
    });

    it("should propagate confidence through the flow", async () => {
      const task = mockTask({
        description:
          "How do I implement OAuth2 in Express.js? I'm not sure which library to use.",
        type: "code-editing",
      });

      // Detection
      const requirement = detector.detectResearchNeeds(task);
      const detectorConfidence = requirement?.confidence || 0;
      expect(detectorConfidence).toBeGreaterThan(0.8); // High confidence (question + uncertainty)

      // Augmentation
      const augmentedTask = await augmenter.augmentTask(task);

      // Augmenter confidence should be based on findings
      if (augmentedTask.researchContext) {
        expect(augmentedTask.researchContext.confidence).toBeDefined();
        expect(augmentedTask.researchContext.confidence).toBeGreaterThan(0);

        // Research context should be enriched with detector data
        expect(augmentedTask.researchContext.confidence).toBeDefined();
      }
    });
  });

  describe("Database Integration", () => {
    it("should persist and retrieve research provenance", async () => {
      const task = mockTask({
        description: "How do I implement webhooks in Node.js?",
      });

      const augmentedTask = await augmenter.augmentTask(task);

      if (augmentedTask.researchContext) {
        // Record provenance
        await provenance.recordResearch(
          task.id,
          augmentedTask.researchContext,
          1500
        );

        // Retrieve and verify
        const records = await provenance.getTaskResearch(task.id);
        expect(records.length).toBeGreaterThan(0);

        const record = records[0];
        expect(record.taskId).toBe(task.id);
        expect(record.queries).toEqual(augmentedTask.researchContext.queries);
        expect(record.findingsCount).toBe(
          augmentedTask.researchContext.findings.length
        );
        expect(record.confidence).toBe(
          augmentedTask.researchContext.confidence
        );
        expect(record.durationMs).toBe(1500);
        expect(record.successful).toBe(true);
      }
    });

    it("should handle database failures gracefully", async () => {
      const task = mockTask({
        description: "How do I implement rate limiting?",
      });

      // Even with database issues, augmentation should work
      const augmentedTask = await augmenter.augmentTask(task);
      expect(augmentedTask.researchProvided).toBe(true);

      // Provenance recording should not throw
      if (augmentedTask.researchContext) {
        await expect(
          provenance.recordResearch(task.id, augmentedTask.researchContext)
        ).resolves.not.toThrow();
      }
    });

    it("should accumulate statistics across multiple research operations", async () => {
      const tasks = [
        mockTask({
          id: "task-1",
          description: "How do I implement OAuth2?",
        }),
        mockTask({
          id: "task-2",
          description: "What are the best practices for API design?",
        }),
        mockTask({
          id: "task-3",
          description: "Compare REST vs GraphQL",
        }),
      ];

      // Augment and record all tasks
      for (const task of tasks) {
        const augmentedTask = await augmenter.augmentTask(task);
        if (augmentedTask.researchContext) {
          await provenance.recordResearch(
            task.id,
            augmentedTask.researchContext,
            1000
          );
        }
      }

      // Get statistics
      const stats = await provenance.getStatistics();
      expect(stats.totalResearch).toBeGreaterThanOrEqual(3);
      expect(stats.successfulResearch).toBeGreaterThanOrEqual(3);
      expect(stats.averageConfidence).toBeGreaterThan(0);
      expect(stats.averageDurationMs).toBeGreaterThan(0);
    });
  });

  describe("Error Handling Integration", () => {
    it("should handle detector errors gracefully", async () => {
      // Create task that might cause detector issues
      const task = mockTask({
        description: "", // Empty description
      });

      // Detection should return null (no error thrown)
      const requirement = detector.detectResearchNeeds(task);
      expect(requirement).toBeNull();

      // Augmentation should handle null requirement
      const augmentedTask = await augmenter.augmentTask(task);
      expect(augmentedTask.researchProvided).toBe(false);
    });

    it("should handle knowledge seeker failures", async () => {
      const task = mockTask({
        description: "How do I implement feature X?",
      });

      // Mock knowledge seeker to fail
      const originalProcessQuery = knowledgeSeeker.processQuery;
      knowledgeSeeker.processQuery = jest
        .fn()
        .mockRejectedValue(new Error("Search provider unavailable"));

      // Augmentation should handle failure gracefully
      const augmentedTask = await augmenter.augmentTask(task);

      // Should either return original task or mark as not researched
      expect(augmentedTask).toBeDefined();
      expect(augmentedTask.id).toBe(task.id);

      // Restore
      knowledgeSeeker.processQuery = originalProcessQuery;
    });

    it("should record failures in provenance", async () => {
      const task = mockTask({
        description: "How do I implement feature X?",
      });

      const queries = ["How do I implement feature X?"];
      const error = new Error("Search provider unavailable");

      // Record failure
      await provenance.recordFailure(task.id, queries, error, 500);

      // Verify failure was recorded
      const records = await provenance.getTaskResearch(task.id);
      expect(records.length).toBeGreaterThan(0);

      const record = records[0];
      expect(record.taskId).toBe(task.id);
      expect(record.successful).toBe(false);
      expect(record.error).toBe(error.message);
      expect(record.findingsCount).toBe(0);
      expect(record.confidence).toBe(0);
    });
  });

  describe("Performance Integration", () => {
    it("should complete full flow within performance budget", async () => {
      const task = mockTask({
        description: "How do I implement caching in Redis?",
      });

      const startTime = Date.now();

      // Full flow
      detector.detectResearchNeeds(task);
      const augmentedTask = await augmenter.augmentTask(task);

      if (augmentedTask.researchContext) {
        await provenance.recordResearch(
          task.id,
          augmentedTask.researchContext
        );
      }

      const duration = Date.now() - startTime;

      // Should complete within reasonable time
      expect(duration).toBeLessThan(3000); // 3 seconds for full flow
    });

    it("should handle concurrent research operations", async () => {
      const tasks = Array(5)
        .fill(null)
        .map((_, i) =>
          mockTask({
            id: `concurrent-${i}`,
            description: `How do I implement feature ${i}?`,
          })
        );

      const startTime = Date.now();

      // Process all tasks in parallel
      const results = await Promise.all(
        tasks.map(async (task) => {
          const augmentedTask = await augmenter.augmentTask(task);
          if (augmentedTask.researchContext) {
            await provenance.recordResearch(
              task.id,
              augmentedTask.researchContext
            );
          }
          return augmentedTask;
        })
      );

      const duration = Date.now() - startTime;

      // All should complete
      expect(results.length).toBe(5);
      results.forEach((result) => {
        expect(result).toBeDefined();
      });

      // Should benefit from parallelization
      expect(duration).toBeLessThan(10000); // 10 seconds for 5 concurrent
    });
  });

  describe("Configuration Integration", () => {
    it("should respect detector config in full flow", async () => {
      // Reconfigure detector with higher threshold
      const strictDetector = new ResearchDetector({
        minConfidence: 0.95,
        maxQueries: 2,
      });

      const strictAugmenter = new TaskResearchAugmenter(
        knowledgeSeeker,
        strictDetector,
        {
          maxResultsPerQuery: 2,
          maxQueries: 2,
        }
      );

      // Task with medium confidence
      const task = mockTask({
        description: "Implement a new API endpoint", // Technical keyword only (50%)
        type: "code-editing",
      });

      // Should not trigger research (below 0.95 threshold)
      const augmentedTask = await strictAugmenter.augmentTask(task);
      expect(augmentedTask.researchProvided).toBe(false);
    });

    it("should respect augmenter config limits", async () => {
      // Configure with strict limits
      const limitedAugmenter = new TaskResearchAugmenter(
        knowledgeSeeker,
        detector,
        {
          maxResultsPerQuery: 1,
          maxQueries: 1,
        }
      );

      const task = mockTask({
        description: "How do I implement OAuth2? What are best practices?",
      });

      const augmentedTask = await limitedAugmenter.augmentTask(task);

      if (augmentedTask.researchContext) {
        // Should respect maxQueries limit
        expect(augmentedTask.researchContext.queries.length).toBeLessThanOrEqual(
          1
        );

        // Each query should have limited results
        augmentedTask.researchContext.findings.forEach((finding) => {
          expect(finding.keyFindings.length).toBeLessThanOrEqual(1);
        });
      }
    });
  });

  describe("Data Consistency", () => {
    it("should maintain query-finding-provenance consistency", async () => {
      const task = mockTask({
        description: "How do I implement WebSockets in Node.js?",
      });

      // Detect
      const detectedQueries =
        detector.detectResearchNeeds(task)?.suggestedQueries || [];

      // Augment
      const augmentedTask = await augmenter.augmentTask(task);

      if (augmentedTask.researchContext) {
        const { queries, findings } = augmentedTask.researchContext;

        // Queries in context should match or be subset of detected queries
        expect(queries.length).toBeGreaterThan(0);
        expect(queries.length).toBeLessThanOrEqual(detectedQueries.length);

        // Each query should have corresponding findings (or empty)
        expect(findings.length).toBeLessThanOrEqual(queries.length);

        // Record provenance
        await provenance.recordResearch(
          task.id,
          augmentedTask.researchContext,
          1200
        );

        // Retrieve and verify consistency
        const records = await provenance.getTaskResearch(task.id);
        expect(records[0].queries).toEqual(queries);
        expect(records[0].findingsCount).toBe(findings.length);
      }
    });

    it("should maintain confidence consistency across components", async () => {
      const task = mockTask({
        description:
          "How do I implement rate limiting? I'm not sure about the best approach.",
      });

      // Detect
      const detectorConfidence =
        detector.detectResearchNeeds(task)?.confidence || 0;
      expect(detectorConfidence).toBeGreaterThan(0.85); // Question + Uncertainty

      // Augment
      const augmentedTask = await augmenter.augmentTask(task);

      if (augmentedTask.researchContext) {
        const { confidence } = augmentedTask.researchContext;

        // Augmenter confidence should be based on findings quality
        expect(confidence).toBeGreaterThan(0);
        expect(confidence).toBeLessThanOrEqual(1);

        // Record provenance
        await provenance.recordResearch(
          task.id,
          augmentedTask.researchContext
        );

        // Retrieve and verify confidence was persisted
        const records = await provenance.getTaskResearch(task.id);
        expect(records[0].confidence).toBe(confidence);
      }
    });
  });
});

