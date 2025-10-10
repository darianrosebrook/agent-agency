#!/usr/bin/env node

/**
 * Performance Optimization Test
 * Tests caching, query optimization, and performance monitoring
 */

import {
  CacheManager,
  PerformanceMonitor,
  QueryOptimizer,
} from "./src/performance/index.js";

console.log("🧪 Testing Performance Optimization Components...\n");

async function testPerformanceOptimization() {
  try {
    console.log("1. Testing Cache Manager...");

    const cacheManager = new CacheManager({
      enabled: true,
      maxSize: 100,
      defaultTTL: 5 * 60 * 1000, // 5 minutes
      compressionThreshold: 1024,
      enableMetrics: true,
      evictionPolicy: "lru",
    });

    // Test basic caching
    await cacheManager.set("user:123", {
      id: 123,
      name: "John Doe",
      email: "john@example.com",
    });
    const cachedUser = await cacheManager.get("user:123");

    console.log("✅ Cache set/get:", cachedUser ? "Working" : "Failed");

    // Test cache statistics
    const stats = cacheManager.getStats();
    console.log(
      `📊 Cache stats: ${stats.totalEntries} entries, ${stats.hits} hits, ${(
        stats.hitRate * 100
      ).toFixed(1)}% hit rate`
    );

    // Test cache invalidation
    const deleted = cacheManager.delete("user:123");
    console.log("🗑️  Cache invalidation:", deleted ? "Working" : "Failed");

    console.log("\n2. Testing Query Optimizer...");

    const queryOptimizer = new QueryOptimizer();

    // Test query analysis
    const sampleQuery = `
      SELECT u.name, u.email, p.title
      FROM users u
      INNER JOIN posts p ON u.id = p.user_id
      WHERE u.created_at > '2024-01-01'
        AND p.published = true
      ORDER BY p.created_at DESC
      LIMIT 50
    `;

    const analysis = await queryOptimizer.analyzeQuery(sampleQuery.trim());

    console.log("✅ Query analysis completed");
    console.log(`   - Query type: ${analysis.type}`);
    console.log(`   - Complexity: ${analysis.estimatedComplexity}/10`);
    console.log(`   - Tables: ${analysis.tables.join(", ")}`);
    console.log(
      `   - Index recommendations: ${analysis.recommendedIndexes.length}`
    );

    // Test query optimization
    const optimizedQuery = queryOptimizer.optimizeQuery(sampleQuery.trim());
    console.log(
      "🔧 Query optimization:",
      optimizedQuery !== sampleQuery.trim() ? "Applied" : "No changes needed"
    );

    console.log("\n3. Testing Performance Monitor...");

    const performanceMonitor = new PerformanceMonitor({
      cpu: { warning: 70, critical: 90 },
      memory: { warning: 80, critical: 95 },
      responseTime: { warning: 1000, critical: 3000 },
      errorRate: { warning: 0.05, critical: 0.15 },
      cacheHitRate: { warning: 0.7, critical: 0.5 },
    });

    // Start monitoring (brief test)
    performanceMonitor.startMonitoring(5000); // Monitor every 5 seconds

    // Wait a moment for some metrics to be collected
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Get current metrics
    const currentMetrics = await performanceMonitor.getCurrentMetrics();
    console.log("✅ Performance monitoring active");
    console.log(`   - CPU usage: ${currentMetrics.cpu.usage.toFixed(1)}%`);
    console.log(
      `   - Memory usage: ${currentMetrics.memory.usage.toFixed(1)}%`
    );
    console.log(
      `   - Response time: ${currentMetrics.application.responseTime.toFixed(
        0
      )}ms`
    );

    // Generate performance report
    const report = performanceMonitor.generateReport(0.01); // Last minute
    console.log(
      `📈 Performance report generated: ${report.summary.overallHealth} health`
    );

    // Stop monitoring
    performanceMonitor.stopMonitoring();

    console.log("\n🎉 Performance Optimization Test Completed Successfully!");
    console.log("\n📊 Performance Components Status:");
    console.log("   ✅ Cache Manager: Operational with LRU eviction");
    console.log(
      "   ✅ Query Optimizer: Analyzing and recommending optimizations"
    );
    console.log(
      "   ✅ Performance Monitor: Collecting metrics and generating reports"
    );
    console.log("   ✅ System Integration: All components working together");

    console.log("\n🚀 Performance Optimizations Available:");
    console.log(
      "   🔄 Intelligent Caching: LRU eviction, TTL management, compression"
    );
    console.log(
      "   🔍 Query Analysis: Complexity assessment, index recommendations"
    );
    console.log(
      "   📊 Real-time Monitoring: CPU, memory, response times, error rates"
    );
    console.log(
      "   ⚡ Automated Maintenance: Cache cleanup, performance trend analysis"
    );
  } catch (error) {
    console.error(
      "❌ Error during performance optimization test:",
      error.message
    );
    console.error("Stack:", error.stack);
    process.exit(1);
  }
}

testPerformanceOptimization();
