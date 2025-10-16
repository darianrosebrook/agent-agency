/**
 * @fileoverview ARBITER-006 Knowledge Seeker - Performance Benchmarks
 *
 * Performance testing and benchmarking covering:
 * - Cache hit performance (<50ms P95)
 * - Search query performance (<500ms P95)
 * - Information processing performance (<200ms P95)
 * - Concurrent operations (50 max)
 * - Load testing with realistic patterns
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { KnowledgeSeeker } from "../../src/knowledge/KnowledgeSeeker";
import { SearchProviderFactory } from "../../src/knowledge/SearchProvider";
import {
  KnowledgeQuery,
  KnowledgeSeekerConfig,
  QueryType,
  SearchProviderType,
} from "../../src/types/knowledge";

describe("ARBITER-006 Knowledge Seeker - Performance Benchmarks", () => {
  const createPerfConfig = (): KnowledgeSeekerConfig => ({
    enabled: true,
    providers: [
      {
        name: "mock",
        type: SearchProviderType.WEB_SEARCH,
        endpoint: "mock://",
        rateLimit: { requestsPerMinute: 1000, requestsPerHour: 10000 },
        limits: { maxResultsPerQuery: 100, maxConcurrentQueries: 100 },
        options: {},
      },
    ],
    processor: {
      minRelevanceScore: 0.5,
      minCredibilityScore: 0.5,
      maxResultsToProcess: 100,
      diversity: { minSources: 2, minSourceTypes: 1, maxResultsPerDomain: 5 },
      quality: {
        enableCredibilityScoring: true,
        enableRelevanceFiltering: true,
        enableDuplicateDetection: true,
      },
      caching: {
        enableResultCaching: true,
        cacheTtlMs: 300000,
        maxCacheSize: 1000,
      },
    },
    queryProcessing: {
      maxConcurrentQueries: 100,
      defaultTimeoutMs: 30000,
      retryAttempts: 3,
    },
    caching: {
      enableQueryCaching: true,
      enableResultCaching: true,
      cacheTtlMs: 300000,
    },
    observability: {
      enableMetrics: true,
      enableTracing: false, // Disable for performance testing
      logLevel: "error", // Reduce logging overhead
    },
  });

  const createPerfQuery = (id: string): KnowledgeQuery => ({
    id,
    query: `Performance test query ${id}`,
    queryType: QueryType.EXPLANATORY,
    maxResults: 10,
    relevanceThreshold: 0.7,
    timeoutMs: 5000,
    metadata: {
      requesterId: "perf-test",
      priority: 5,
      createdAt: new Date(),
      tags: ["performance"],
    },
  });

  // Helper to calculate percentiles
  const calculatePercentile = (
    values: number[],
    percentile: number
  ): number => {
    const sorted = [...values].sort((a, b) => a - b);
    const index = Math.ceil((percentile / 100) * sorted.length) - 1;
    return sorted[index];
  };

  // Helper to calculate statistics
  const calculateStats = (values: number[]) => {
    const sorted = [...values].sort((a, b) => a - b);
    const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
    const median = sorted[Math.floor(sorted.length / 2)];
    const p50 = calculatePercentile(values, 50);
    const p95 = calculatePercentile(values, 95);
    const p99 = calculatePercentile(values, 99);
    const min = Math.min(...values);
    const max = Math.max(...values);

    return { mean, median, p50, p95, p99, min, max };
  };

  // ============================================================================
  // Cache Performance Benchmarks (<50ms P95)
  // ============================================================================

  describe("Cache Performance (<50ms P95)", () => {
    it("should achieve P95 cache hit latency <50ms", async () => {
      const config = createPerfConfig();
      config.caching.enableQueryCaching = true;
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("cache-perf");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("cache-perf", provider);

      const query = createPerfQuery("cache-perf-test");

      // Warm up cache
      await seeker.processQuery(query);

      // Measure cache hit performance
      const timings: number[] = [];
      const iterations = 100;

      for (let i = 0; i < iterations; i++) {
        const start = Date.now();
        await seeker.processQuery(query);
        const duration = Date.now() - start;
        timings.push(duration);
      }

      const stats = calculateStats(timings);

      console.log("Cache Performance Benchmark:");
      console.log(`  Iterations: ${iterations}`);
      console.log(`  Mean: ${stats.mean.toFixed(2)}ms`);
      console.log(`  Median: ${stats.median}ms`);
      console.log(`  P50: ${stats.p50}ms`);
      console.log(`  P95: ${stats.p95}ms`);
      console.log(`  P99: ${stats.p99}ms`);
      console.log(`  Min: ${stats.min}ms`);
      console.log(`  Max: ${stats.max}ms`);

      // Validate SLA
      expect(stats.p95).toBeLessThan(50);
    });

    it("should maintain cache performance under load", async () => {
      const config = createPerfConfig();
      config.caching.enableQueryCaching = true;
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("cache-load");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("cache-load", provider);

      // Create multiple queries and cache them
      const queries = Array.from({ length: 10 }, (_, i) =>
        createPerfQuery(`cache-load-${i}`)
      );

      // Warm up cache
      for (const query of queries) {
        await seeker.processQuery(query);
      }

      // Measure cache hit performance with load
      const timings: number[] = [];
      const iterations = 50;

      for (let i = 0; i < iterations; i++) {
        const randomQuery = queries[Math.floor(Math.random() * queries.length)];
        const start = Date.now();
        await seeker.processQuery(randomQuery);
        const duration = Date.now() - start;
        timings.push(duration);
      }

      const stats = calculateStats(timings);

      console.log("\nCache Performance Under Load:");
      console.log(`  P95: ${stats.p95}ms (target: <50ms)`);

      expect(stats.p95).toBeLessThan(50);
    });
  });

  // ============================================================================
  // Search Query Performance (<500ms P95)
  // ============================================================================

  describe("Search Query Performance (<500ms P95)", () => {
    it("should achieve P95 search latency <500ms", async () => {
      const config = createPerfConfig();
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("search-perf");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("search-perf", provider);

      const timings: number[] = [];
      const iterations = 100;

      for (let i = 0; i < iterations; i++) {
        const query = createPerfQuery(`search-perf-${i}`);
        const start = Date.now();
        await seeker.processQuery(query);
        const duration = Date.now() - start;
        timings.push(duration);
      }

      const stats = calculateStats(timings);

      console.log("\nSearch Query Performance Benchmark:");
      console.log(`  Iterations: ${iterations}`);
      console.log(`  Mean: ${stats.mean.toFixed(2)}ms`);
      console.log(`  Median: ${stats.median}ms`);
      console.log(`  P50: ${stats.p50}ms`);
      console.log(`  P95: ${stats.p95}ms`);
      console.log(`  P99: ${stats.p99}ms`);

      // Validate SLA
      expect(stats.p95).toBeLessThan(500);
    });

    it("should handle burst traffic with consistent performance", async () => {
      const config = createPerfConfig();
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("burst-perf");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("burst-perf", provider);

      // Simulate burst of 20 concurrent queries
      const burstSize = 20;
      const queries = Array.from({ length: burstSize }, (_, i) =>
        createPerfQuery(`burst-${i}`)
      );

      const startTime = Date.now();
      const responses = await Promise.all(
        queries.map((q) => seeker.processQuery(q))
      );
      const totalDuration = Date.now() - startTime;

      console.log("\nBurst Traffic Performance:");
      console.log(`  Burst size: ${burstSize} concurrent queries`);
      console.log(`  Total duration: ${totalDuration}ms`);
      console.log(
        `  Average per query: ${(totalDuration / burstSize).toFixed(2)}ms`
      );

      // All queries should succeed
      expect(responses.length).toBe(burstSize);
      responses.forEach((response) => {
        expect(response.results.length).toBeGreaterThan(0);
      });

      // Total duration should be reasonable for parallel processing
      expect(totalDuration).toBeLessThan(3000); // 3 seconds for 20 concurrent
    });
  });

  // ============================================================================
  // Concurrent Operations (50 max)
  // ============================================================================

  describe("Concurrent Operations (50 max)", () => {
    it("should handle 50 concurrent searches efficiently", async () => {
      const config = createPerfConfig();
      config.queryProcessing.maxConcurrentQueries = 50;
      const seeker = new KnowledgeSeeker(config);

      const provider =
        SearchProviderFactory.createMockProvider("concurrent-50");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("concurrent-50", provider);

      const concurrentCount = 50;
      const queries = Array.from({ length: concurrentCount }, (_, i) =>
        createPerfQuery(`concurrent-50-${i}`)
      );

      const startTime = Date.now();
      const responses = await Promise.all(
        queries.map((q) => seeker.processQuery(q))
      );
      const totalDuration = Date.now() - startTime;

      console.log("\n50 Concurrent Operations Benchmark:");
      console.log(`  Concurrent queries: ${concurrentCount}`);
      console.log(`  Total duration: ${totalDuration}ms`);
      console.log(
        `  Average per query: ${(totalDuration / concurrentCount).toFixed(2)}ms`
      );
      console.log(
        `  Throughput: ${(concurrentCount / (totalDuration / 1000)).toFixed(
          2
        )} queries/sec`
      );

      // All queries should succeed
      expect(responses.length).toBe(concurrentCount);
      responses.forEach((response) => {
        expect(response.results.length).toBeGreaterThan(0);
      });

      // Should complete in reasonable time
      expect(totalDuration).toBeLessThan(5000); // 5 seconds
    });

    it("should maintain performance with sustained load", async () => {
      const config = createPerfConfig();
      const seeker = new KnowledgeSeeker(config);

      const provider =
        SearchProviderFactory.createMockProvider("sustained-load");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("sustained-load", provider);

      // Simulate sustained load: 3 waves of 20 concurrent queries
      const waveDurations: number[] = [];

      for (let wave = 0; wave < 3; wave++) {
        const queries = Array.from({ length: 20 }, (_, i) =>
          createPerfQuery(`sustained-wave${wave}-${i}`)
        );

        const waveStart = Date.now();
        await Promise.all(queries.map((q) => seeker.processQuery(q)));
        const waveDuration = Date.now() - waveStart;
        waveDurations.push(waveDuration);

        // Small delay between waves
        await new Promise((resolve) => setTimeout(resolve, 100));
      }

      console.log("\nSustained Load Performance:");
      waveDurations.forEach((duration, i) => {
        console.log(`  Wave ${i + 1}: ${duration}ms`);
      });

      // Performance should remain consistent across waves
      const maxVariation =
        Math.max(...waveDurations) - Math.min(...waveDurations);
      console.log(`  Variation: ${maxVariation}ms`);

      // All waves should complete in reasonable time
      waveDurations.forEach((duration) => {
        expect(duration).toBeLessThan(3000);
      });
    });
  });

  // ============================================================================
  // Load Testing with Realistic Patterns
  // ============================================================================

  describe("Realistic Load Patterns", () => {
    it("should handle mixed workload (cached + new queries)", async () => {
      const config = createPerfConfig();
      config.caching.enableQueryCaching = true;
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("mixed-load");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("mixed-load", provider);

      // Create base queries and cache them
      const cachedQueries = Array.from({ length: 5 }, (_, i) =>
        createPerfQuery(`cached-${i}`)
      );

      for (const query of cachedQueries) {
        await seeker.processQuery(query);
      }

      // Mix of cached (70%) and new queries (30%)
      const mixedQueries: KnowledgeQuery[] = [];
      for (let i = 0; i < 100; i++) {
        if (Math.random() < 0.7) {
          // 70% cached
          mixedQueries.push(
            cachedQueries[Math.floor(Math.random() * cachedQueries.length)]
          );
        } else {
          // 30% new
          mixedQueries.push(createPerfQuery(`new-${i}`));
        }
      }

      const timings: number[] = [];

      for (const query of mixedQueries) {
        const start = Date.now();
        await seeker.processQuery(query);
        const duration = Date.now() - start;
        timings.push(duration);
      }

      const stats = calculateStats(timings);

      console.log("\nMixed Workload Performance:");
      console.log(`  Total queries: ${mixedQueries.length}`);
      console.log(`  Mean: ${stats.mean.toFixed(2)}ms`);
      console.log(`  P95: ${stats.p95}ms`);
      console.log(`  P99: ${stats.p99}ms`);

      // P95 should benefit from cache hits
      expect(stats.p95).toBeLessThan(300); // Better than non-cached P95
    });

    it("should handle query complexity variations", async () => {
      const config = createPerfConfig();
      const seeker = new KnowledgeSeeker(config);

      const provider =
        SearchProviderFactory.createMockProvider("complexity-test");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("complexity-test", provider);

      const complexityLevels = [
        { maxResults: 5, label: "Simple" },
        { maxResults: 10, label: "Medium" },
        { maxResults: 20, label: "Complex" },
      ];

      const results: Record<string, { mean: number; p95: number }> = {};

      for (const level of complexityLevels) {
        const timings: number[] = [];

        for (let i = 0; i < 30; i++) {
          const query: KnowledgeQuery = {
            ...createPerfQuery(`complexity-${level.label}-${i}`),
            maxResults: level.maxResults,
          };

          const start = Date.now();
          await seeker.processQuery(query);
          const duration = Date.now() - start;
          timings.push(duration);
        }

        const stats = calculateStats(timings);
        results[level.label] = { mean: stats.mean, p95: stats.p95 };
      }

      console.log("\nQuery Complexity Performance:");
      Object.entries(results).forEach(([label, stats]) => {
        console.log(
          `  ${label}: Mean=${stats.mean.toFixed(2)}ms, P95=${stats.p95}ms`
        );
      });

      // All complexity levels should meet SLA
      Object.values(results).forEach((stats) => {
        expect(stats.p95).toBeLessThan(500);
      });
    });

    it("should demonstrate performance scalability", async () => {
      const config = createPerfConfig();
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("scalability");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("scalability", provider);

      const loadLevels = [10, 25, 50];
      const throughputs: number[] = [];

      for (const load of loadLevels) {
        const queries = Array.from({ length: load }, (_, i) =>
          createPerfQuery(`scale-${load}-${i}`)
        );

        const startTime = Date.now();
        await Promise.all(queries.map((q) => seeker.processQuery(q)));
        const duration = Date.now() - startTime;

        const throughput = load / (duration / 1000); // queries per second
        throughputs.push(throughput);

        console.log(`\nLoad Level: ${load} concurrent queries`);
        console.log(`  Duration: ${duration}ms`);
        console.log(`  Throughput: ${throughput.toFixed(2)} queries/sec`);
      }

      // Throughput should scale reasonably with load
      expect(throughputs[2]).toBeGreaterThan(throughputs[0] * 0.8); // At least 80% scaling efficiency
    });
  });

  // ============================================================================
  // Performance Regression Detection
  // ============================================================================

  describe("Performance Regression Detection", () => {
    it("should establish baseline performance metrics", async () => {
      const config = createPerfConfig();
      const seeker = new KnowledgeSeeker(config);

      const provider = SearchProviderFactory.createMockProvider("baseline");
      (seeker as any).providers.clear();
      (seeker as any).providers.set("baseline", provider);

      const iterations = 50;
      const timings: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const query = createPerfQuery(`baseline-${i}`);
        const start = Date.now();
        await seeker.processQuery(query);
        const duration = Date.now() - start;
        timings.push(duration);
      }

      const stats = calculateStats(timings);

      console.log("\nBaseline Performance Metrics:");
      console.log(`  Iterations: ${iterations}`);
      console.log(`  Mean: ${stats.mean.toFixed(2)}ms`);
      console.log(`  Median: ${stats.median}ms`);
      console.log(`  P50: ${stats.p50}ms`);
      console.log(`  P95: ${stats.p95}ms`);
      console.log(`  P99: ${stats.p99}ms`);
      console.log(`  Min: ${stats.min}ms`);
      console.log(`  Max: ${stats.max}ms`);

      // Store baseline for regression detection
      const baseline = {
        mean: stats.mean,
        p95: stats.p95,
        p99: stats.p99,
      };

      // Verify baseline meets SLAs
      expect(baseline.p95).toBeLessThan(500);

      // Export baseline for CI/CD monitoring
      console.log("\nBaseline for CI/CD:");
      console.log(JSON.stringify(baseline, null, 2));
    });
  });
});
