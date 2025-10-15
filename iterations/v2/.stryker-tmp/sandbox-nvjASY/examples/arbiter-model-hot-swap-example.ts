/**
 * @fileoverview
 * Complete example: Arbiter using hot-swap for performance-based model selection
 *
 * This demonstrates:
 * 1. Setting up the model registry with local models
 * 2. Configuring hot-swap with learning preservation
 * 3. Executing tasks with automatic model selection
 * 4. Automatic performance-based model swapping
 * 5. Manual swaps and rollbacks
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { ArbiterModelManager } from "@/models/ArbiterModelManager";
import { ComputeCostTracker } from "@/models/ComputeCostTracker";
import { LocalModelSelector } from "@/models/LocalModelSelector";
import { ModelHotSwapManager } from "@/models/ModelHotSwap";
import { ModelRegistry } from "@/models/ModelRegistry";
import { OllamaProvider } from "@/models/providers/OllamaProvider";
import type {
  GenerationRequest,
  HotSwapConfig,
  ModelSelectionCriteria,
  OllamaModelConfig,
} from "@/types/model-registry";

/**
 * Example: Complete arbiter model management with hot-swap
 */
async function arbiterModelManagementExample() {
  console.log("=== Arbiter Model Management with Hot-Swap ===\n");

  // ========================================
  // 1. Setup: Initialize all components
  // ========================================

  console.log("1. Initializing components...");

  const registry = new ModelRegistry();
  const costTracker = new ComputeCostTracker();
  const selector = new LocalModelSelector(registry, costTracker);

  // Configure hot-swap behavior
  const hotSwapConfig: HotSwapConfig = {
    enableAutoSwap: true,
    swapCooldownMs: 60000, // 1 minute cooldown
    minSamplesBeforeSwap: 5, // Need 5 samples before considering swap
    performanceThreshold: 0.75, // Swap if success rate < 75%
    compatibilityCheckStrict: false, // Allow minor compatibility issues
  };

  const hotSwap = new ModelHotSwapManager(
    registry,
    selector,
    costTracker,
    hotSwapConfig
  );

  const arbiter = new ArbiterModelManager(
    registry,
    selector,
    costTracker,
    hotSwap
  );

  // ========================================
  // 2. Register Models: Local Ollama models
  // ========================================

  console.log("2. Registering local models...");

  // Fast model (2B parameters)
  const fastModel = await registry.registerOllamaModel(
    "gemma-2b",
    "gemma3:1b",
    "1.0.0",
    "fast"
  );

  // Balanced model (7B parameters)
  const balancedModel = await registry.registerOllamaModel(
    "gemma-7b",
    "gemma3n:e2b",
    "1.0.0",
    "primary"
  );

  // Quality model (14B parameters)
  const qualityModel = await registry.registerOllamaModel(
    "gemma-14b",
    "gemma3n:e4b",
    "1.0.0",
    "quality"
  );

  // Activate all models
  await registry.activateModel(fastModel.id);
  await registry.activateModel(balancedModel.id);
  await registry.activateModel(qualityModel.id);

  console.log(`   ✅ Registered ${registry.getAllModels().length} models`);

  // ========================================
  // 3. Create Providers: Connect to Ollama
  // ========================================

  console.log("3. Creating model providers...");

  const fastProvider = new OllamaProvider({
    capabilities: ["text-generation", "chat"],
    ollamaModelName: "gemma3:1b",
    ollamaEndpoint: "http://localhost:11434",
    hardwareRequirements: {
      preferredHardware: ["cpu"],
      minMemoryMB: 2048,
    },
  } as OllamaModelConfig);

  const balancedProvider = new OllamaProvider({
    capabilities: ["text-generation", "chat"],
    ollamaModelName: "gemma3n:e2b",
    ollamaEndpoint: "http://localhost:11434",
    hardwareRequirements: {
      preferredHardware: ["cpu", "gpu"],
      minMemoryMB: 8192,
    },
  } as OllamaModelConfig);

  const qualityProvider = new OllamaProvider({
    capabilities: ["text-generation", "chat"],
    ollamaModelName: "gemma3n:e4b",
    ollamaEndpoint: "http://localhost:11434",
    hardwareRequirements: {
      preferredHardware: ["gpu"],
      minMemoryMB: 16384,
    },
  } as OllamaModelConfig);

  // Register providers with hot-swap manager
  hotSwap.registerProvider(fastModel.id, fastProvider);
  hotSwap.registerProvider(balancedModel.id, balancedProvider);
  hotSwap.registerProvider(qualityModel.id, qualityProvider);

  console.log("   ✅ Providers registered and ready");

  // ========================================
  // 4. Execute Tasks: Automatic selection
  // ========================================

  console.log("\n4. Executing tasks with automatic model selection...\n");

  // Define different task types
  const taskTypes = {
    // Fast tasks: Simple queries, quick responses
    simple: {
      taskType: "simple-query",
      requiredCapabilities: ["text-generation"],
      qualityThreshold: 0.7,
      maxLatencyMs: 1000,
      maxMemoryMB: 4096,
      availableHardware: { cpu: true, gpu: false },
      preferences: {
        preferFast: true,
        preferQuality: false,
        preferLowMemory: true,
      },
    } as ModelSelectionCriteria,

    // Complex tasks: Detailed analysis, high quality
    complex: {
      taskType: "complex-analysis",
      requiredCapabilities: ["text-generation", "chat"],
      qualityThreshold: 0.9,
      maxLatencyMs: 10000,
      maxMemoryMB: 20480,
      availableHardware: { cpu: true, gpu: true },
      preferences: {
        preferFast: false,
        preferQuality: true,
        preferLowMemory: false,
      },
    } as ModelSelectionCriteria,

    // Balanced tasks: Mix of speed and quality
    balanced: {
      taskType: "balanced-task",
      requiredCapabilities: ["text-generation"],
      qualityThreshold: 0.8,
      maxLatencyMs: 3000,
      maxMemoryMB: 10240,
      availableHardware: { cpu: true, gpu: true },
      preferences: {
        preferFast: false,
        preferQuality: false,
        preferLowMemory: false,
      },
    } as ModelSelectionCriteria,
  };

  // Execute simple task
  console.log("   📝 Executing simple query...");
  const simpleRequest: GenerationRequest = {
    prompt: "What is 2+2?",
    maxTokens: 50,
    temperature: 0.7,
  };

  try {
    const simpleResult = await arbiter.executeTask(
      simpleRequest,
      taskTypes.simple
    );

    console.log(`   ✅ Model used: ${simpleResult.modelId}`);
    console.log(`   ⏱️  Latency: ${simpleResult.performance.latencyMs}ms`);
    console.log(
      `   📊 Quality: ${(simpleResult.performance.quality * 100).toFixed(1)}%`
    );

    if (simpleResult.swapped) {
      console.log(`   🔄 Auto-swapped: ${simpleResult.swapDetails?.reason}`);
    }
  } catch (error) {
    console.log(`   ⚠️  Simulated execution (Ollama not running)`);
  }

  // Execute complex task
  console.log("\n   📝 Executing complex analysis...");
  const complexRequest: GenerationRequest = {
    prompt: "Explain quantum computing in detail with examples.",
    maxTokens: 500,
    temperature: 0.7,
  };

  try {
    const complexResult = await arbiter.executeTask(
      complexRequest,
      taskTypes.complex
    );

    console.log(`   ✅ Model used: ${complexResult.modelId}`);
    console.log(`   ⏱️  Latency: ${complexResult.performance.latencyMs}ms`);
    console.log(
      `   📊 Quality: ${(complexResult.performance.quality * 100).toFixed(1)}%`
    );
  } catch (error) {
    console.log(`   ⚠️  Simulated execution (Ollama not running)`);
  }

  // ========================================
  // 5. Demonstrate Hot-Swap
  // ========================================

  console.log("\n5. Demonstrating hot-swap mechanism...\n");

  // Simulate multiple task executions to trigger auto-swap
  console.log("   🔁 Executing multiple tasks to build performance history...");

  // Simulate 10 task executions
  for (let i = 0; i < 10; i++) {
    const learningLayer = hotSwap.getLearningLayer();

    // Simulate varying performance
    learningLayer.recordTaskPerformance("balanced-task", {
      latencyMs: 250 + Math.random() * 100,
      quality: 0.7 + Math.random() * 0.1,
      memoryMB: 8000 + Math.random() * 2000,
      success: Math.random() > 0.3, // 70% success rate
    });
  }

  console.log("   ✅ Performance history built (10 samples)");

  // Try auto-swap
  console.log("\n   🔄 Checking if auto-swap is triggered...");

  try {
    const currentModel =
      arbiter.getCurrentModel("balanced-task") || balancedModel.id;
    const swapResult = await hotSwap.autoSwap(currentModel, taskTypes.balanced);

    if (swapResult?.swapped) {
      console.log(`   ✅ Auto-swap triggered!`);
      console.log(`   ├─ From: ${currentModel}`);
      console.log(`   ├─ To: ${swapResult.newModelId}`);
      console.log(`   └─ Reason: ${swapResult.reason}`);
    } else {
      console.log(`   ℹ️  No swap needed - current model performing well`);
    }
  } catch (error) {
    console.log(`   ⚠️  Auto-swap conditions not met`);
  }

  // ========================================
  // 6. Manual Swap
  // ========================================

  console.log("\n6. Demonstrating manual swap...\n");

  try {
    console.log(`   🔧 Forcing swap to quality model for complex tasks...`);

    await arbiter.forceSwap("complex-analysis", qualityModel.id);

    console.log(`   ✅ Manual swap successful`);
    console.log(`   └─ Task 'complex-analysis' now uses: ${qualityModel.name}`);
  } catch (error) {
    console.log(`   ⚠️  Manual swap requires existing task execution`);
  }

  // ========================================
  // 7. Rollback
  // ========================================

  console.log("\n7. Demonstrating rollback...\n");

  try {
    console.log(`   ↩️  Rolling back complex-analysis task...`);

    const rollbackResult = await arbiter.rollback("complex-analysis");

    if (rollbackResult.success) {
      console.log(`   ✅ Rollback successful`);
      console.log(`   └─ Restored to: ${rollbackResult.previousModelId}`);
    }
  } catch (error: any) {
    console.log(`   ⚠️  ${error.message}`);
  }

  // ========================================
  // 8. Statistics & Analytics
  // ========================================

  console.log("\n8. Performance analytics...\n");

  // Get overall statistics
  const stats = arbiter.getStatistics();

  console.log(`   📊 Total task types managed: ${stats.totalTasks}`);
  console.log(`   🔄 Total swaps: ${stats.swapStats.totalSwaps}`);
  console.log(`   ✅ Successful swaps: ${stats.swapStats.successfulSwaps}`);
  console.log(`   ❌ Failed swaps: ${stats.swapStats.failedSwaps}`);

  if (stats.swapStats.avgSwapDurationMs > 0) {
    console.log(
      `   ⏱️  Avg swap duration: ${stats.swapStats.avgSwapDurationMs.toFixed(
        0
      )}ms`
    );
  }

  console.log("\n   🏆 Top models by task count:");

  for (const [idx, model] of stats.topModels.slice(0, 3).entries()) {
    console.log(
      `   ${idx + 1}. ${model.modelId}: ${model.taskTypes.length} tasks`
    );
  }

  // Get task-specific performance
  console.log("\n   📈 Task-specific performance:");

  for (const taskType of [
    "simple-query",
    "complex-analysis",
    "balanced-task",
  ]) {
    const summary = arbiter.getTaskPerformanceSummary(taskType);

    if (summary.learnings) {
      console.log(`\n   📋 ${taskType}:`);
      console.log(`      ├─ Current model: ${summary.currentModel || "none"}`);
      console.log(`      ├─ Samples: ${summary.learnings.samples}`);
      console.log(
        `      ├─ Avg latency: ${summary.learnings.avgLatencyMs.toFixed(0)}ms`
      );
      console.log(
        `      ├─ Avg quality: ${(summary.learnings.avgQuality * 100).toFixed(
          1
        )}%`
      );
      console.log(
        `      └─ Success rate: ${(summary.learnings.successRate * 100).toFixed(
          1
        )}%`
      );
    }
  }

  // ========================================
  // 9. Learning Preservation Demonstration
  // ========================================

  console.log("\n9. Learning preservation across swaps...\n");

  const learningLayer = hotSwap.getLearningLayer();

  // Show task characteristics learned
  console.log("   🧠 Learned task characteristics:");

  learningLayer.learnTaskCharacteristics("simple-query", {
    preferFast: true,
    preferQuality: false,
    complexity: "low",
  });

  learningLayer.learnTaskCharacteristics("complex-analysis", {
    preferFast: false,
    preferQuality: true,
    complexity: "high",
  });

  const simpleChars = learningLayer.getTaskCharacteristics("simple-query");
  const complexChars = learningLayer.getTaskCharacteristics("complex-analysis");

  if (simpleChars) {
    console.log("\n   ├─ simple-query:");
    console.log(`   │  ├─ Prefer fast: ${simpleChars.preferFast}`);
    console.log(`   │  ├─ Prefer quality: ${simpleChars.preferQuality}`);
    console.log(`   │  └─ Complexity: ${simpleChars.complexity}`);
  }

  if (complexChars) {
    console.log("\n   └─ complex-analysis:");
    console.log(`      ├─ Prefer fast: ${complexChars.preferFast}`);
    console.log(`      ├─ Prefer quality: ${complexChars.preferQuality}`);
    console.log(`      └─ Complexity: ${complexChars.complexity}`);
  }

  console.log("\n   💡 These learnings are preserved across model swaps!");
  console.log("   💡 System learns about TASKS, not specific models");

  // ========================================
  // 10. Summary
  // ========================================

  console.log("\n" + "=".repeat(60));
  console.log("✅ DEMONSTRATION COMPLETE");
  console.log("=".repeat(60));
  console.log("\nKey Takeaways:");
  console.log("1. ✅ Zero-downtime model swapping");
  console.log("2. ✅ Automatic performance-based selection");
  console.log("3. ✅ Learning preservation across swaps");
  console.log("4. ✅ Manual control when needed");
  console.log("5. ✅ Comprehensive analytics");
  console.log("\nThe arbiter can now pick and choose the best performing");
  console.log("LLMs based on internal benchmarking, with zero retraining!");
}

// Run if executed directly
if (require.main === module) {
  arbiterModelManagementExample().catch(console.error);
}

export { arbiterModelManagementExample };
