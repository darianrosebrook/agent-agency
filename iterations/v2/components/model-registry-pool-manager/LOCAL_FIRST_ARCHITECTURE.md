# ARBITER-017: Local-First Model Registry Architecture

**Date**: 2025-10-13  
**Author**: @darianrosebrook  
**Philosophy**: Bring Your Own Model (BYOM) - Local First, Hot-Swappable, Learning-Preserving

---

## Core Philosophy

ARBITER-017 is designed around **local-first AI** with the ability to hot-swap models as technology improves, without losing system learnings. This aligns with the kokoro project vision of hardware-optimized, privacy-first AI orchestration.

### Key Principles

1. **Local First**: Prefer local models (Ollama, custom trained, self-hosted)
2. **Model Agnostic**: System learns independently of specific models
3. **Hot-Swappable**: Upgrade models without retraining the system
4. **Learning Preservation**: Knowledge lives in the orchestration layer, not the models
5. **Hardware Optimized**: Leverage Apple Silicon, custom servers, GPU acceleration
6. **Zero Vendor Lock-in**: No dependency on cloud APIs

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                 Model-Agnostic Learning Layer                │
│  (Task routing, debate history, performance metrics)        │
└────────────────────┬────────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │  Model Registry Core   │
         │  (Hot-swap orchestrator)│
         └───────────┬───────────┘
                     │
    ┌────────────────┼────────────────┐
    │                │                │
┌───▼────┐      ┌───▼────┐      ┌───▼────┐
│ Ollama │      │ Custom │      │Hardware│
│ Models │      │ Trained│      │Optimized│
└────────┘      └────────┘      └────────┘
  Local           Local           Local
```

### Separation of Concerns

**What Lives in Models** (Hot-Swappable):

- Raw inference capability
- Token generation
- Context window handling
- Model-specific parameters

**What Lives in System** (Preserved Across Swaps):

- Task routing preferences
- Historical performance data
- Debate patterns and outcomes
- Quality metrics and thresholds
- Agent capability profiles
- Constitutional rules and waivers

---

## Local Model Provider Types

### 1. Ollama Models (Primary Target)

**Already Integrated**: RL-011 has 4 Gemma models running

```typescript
interface OllamaModel {
  name: string; // "gemma3n:e2b", "gemma3:1b", etc.
  taskCategory: "fast" | "primary" | "quality" | "alternative";
  tokensPerSec: number; // Measured performance
  memoryUsage: number; // MB
  quantization: string; // "4bit", "8bit", "full"
}
```

**Advantages**:

- $0/month operational costs
- Full privacy (no data leaves device)
- Unlimited experimentation
- 4 models already validated and working

### 2. Custom Trained Models

**Use Case**: Domain-specific models, fine-tuned judges

```typescript
interface CustomModel {
  name: string;
  framework: "pytorch" | "onnx" | "coreml" | "ggml";
  trainingData: string; // Provenance tracking
  specialization: string; // "code-review", "security-audit", etc.
  hardwareRequirements: HardwareSpec;
}
```

**Integration**:

- Load from local filesystem
- Version control with git-lfs
- Training provenance in CAWS ledger

### 3. Hardware-Optimized Models

**Use Case**: Apple Silicon (ANE), custom servers, GPU clusters

```typescript
interface HardwareOptimizedModel {
  name: string;
  targetHardware: "apple-silicon" | "nvidia-gpu" | "custom-server";
  optimization: "coreml" | "metal" | "cuda" | "tensorrt";
  accelerator: "ane" | "gpu" | "cpu" | "hybrid";
}
```

**Kokoro Project Integration**:

- Core ML models for Apple Neural Engine
- Metal-optimized inference
- Distributed inference across server farm

---

## Hot-Swap Mechanism

### Model Swap Without Retraining

The key insight: **Keep system knowledge separate from model knowledge**

```typescript
class ModelRegistryPoolManager {
  // System knowledge (preserved)
  private performanceHistory: Map<TaskType, ModelPerformance[]>;
  private routingRules: Map<TaskType, SelectionCriteria>;
  private qualityThresholds: Map<string, number>;

  /**
   * Hot-swap a model without losing system learnings
   */
  async hotSwap(oldModelId: string, newModelId: string): Promise<SwapResult> {
    // 1. Capture old model's performance profile
    const oldProfile = await this.capturePerformanceProfile(oldModelId);

    // 2. Warm up new model with validation set
    const newProfile = await this.warmupAndProfile(newModelId);

    // 3. Compare capabilities
    const compatibility = this.assessCompatibility(oldProfile, newProfile);

    if (!compatibility.canReplace) {
      return { success: false, reason: compatibility.reason };
    }

    // 4. Transfer routing rules (task → model mappings)
    await this.transferRoutingRules(oldModelId, newModelId, compatibility);

    // 5. Gradual rollout with A/B testing
    await this.gradualRollout(newModelId, {
      startPercent: 10,
      rampUpRate: 10, // 10% per hour
      rollbackThreshold: 0.95, // Rollback if performance < 95% of old model
    });

    // 6. Mark old model as deprecated (keep for rollback)
    await this.deprecateModel(oldModelId, { keepWarm: true, duration: "7d" });

    return { success: true, rolloutId: generateId() };
  }
}
```

### Performance Profile Transfer

```typescript
interface PerformanceProfile {
  taskCategories: {
    taskType: string;
    successRate: number;
    avgLatency: number;
    qualityScore: number;
  }[];

  capabilities: {
    maxContextWindow: number;
    streamingSupport: boolean;
    batchingSupport: boolean;
  };

  resourceUsage: {
    avgMemoryMB: number;
    avgCPUPercent: number;
    avgGPUPercent: number;
    energyPerToken: number; // mWh per token (local cost)
  };
}

/**
 * Assess if new model can replace old model
 */
function assessCompatibility(
  oldProfile: PerformanceProfile,
  newProfile: PerformanceProfile
): CompatibilityResult {
  const issues: string[] = [];

  // Check context window
  if (
    newProfile.capabilities.maxContextWindow <
    oldProfile.capabilities.maxContextWindow
  ) {
    issues.push("Context window reduced");
  }

  // Check performance on each task category
  for (const oldTask of oldProfile.taskCategories) {
    const newTask = newProfile.taskCategories.find(
      (t) => t.taskType === oldTask.taskType
    );

    if (!newTask) {
      issues.push(`Missing capability: ${oldTask.taskType}`);
      continue;
    }

    // New model should be at least 90% as good
    if (newTask.successRate < oldTask.successRate * 0.9) {
      issues.push(`Degraded performance on ${oldTask.taskType}`);
    }
  }

  return {
    canReplace: issues.length === 0,
    reason: issues.join(", "),
    warnings: issues,
  };
}
```

---

## Compute Cost Tracking (Local Models)

**Not API costs** - Track actual compute resources:

```typescript
interface LocalComputeCost {
  modelId: string;
  operationId: string;

  // Time costs
  wallClockMs: number;
  cpuTimeMs: number;
  gpuTimeMs: number;

  // Memory costs
  peakMemoryMB: number;
  avgMemoryMB: number;

  // Energy costs (for local optimization)
  estimatedEnergyMWh: number; // milliwatt-hours

  // Hardware utilization
  cpuUtilization: number; // 0-100%
  gpuUtilization: number; // 0-100%
  aneUtilization?: number; // 0-100% (Apple only)

  // Tokens
  inputTokens: number;
  outputTokens: number;
  tokensPerSecond: number;
}

/**
 * Track local compute costs for optimization
 */
class LocalComputeTracker {
  /**
   * Record usage for a local model operation
   */
  async recordOperation(cost: LocalComputeCost): Promise<void> {
    // Store in performance tracker
    await this.db.insertComputeCost(cost);

    // Update model performance profile
    await this.updateModelProfile(cost.modelId, {
      avgLatency: cost.wallClockMs,
      avgMemory: cost.avgMemoryMB,
      tokensPerSec: cost.tokensPerSecond,
    });

    // Check for optimization opportunities
    if (cost.cpuUtilization < 50 && cost.gpuUtilization < 30) {
      this.logger.warn("Underutilized hardware", {
        model: cost.modelId,
        suggestion: "Consider larger batch size or parallel inference",
      });
    }
  }

  /**
   * Calculate cost per task type (for model selection)
   */
  async getCostProfile(modelId: string): Promise<CostProfile> {
    const operations = await this.db.getRecentOperations(modelId, 1000);

    return {
      avgWallClockMs: mean(operations.map((o) => o.wallClockMs)),
      avgEnergyMWh: mean(operations.map((o) => o.estimatedEnergyMWh)),
      avgTokensPerSec: mean(operations.map((o) => o.tokensPerSecond)),
      p95WallClockMs: percentile(
        operations.map((o) => o.wallClockMs),
        95
      ),
    };
  }
}
```

---

## Model Selection for Local Models

**Philosophy**: Choose based on **task requirements + hardware availability**

```typescript
interface ModelSelectionCriteria {
  // Task requirements
  taskType: string;
  requiredCapabilities: string[];
  qualityThreshold: number;

  // Performance requirements
  maxLatencyMs: number;
  maxMemoryMB: number;

  // Hardware availability
  availableHardware: {
    cpu: boolean;
    gpu: boolean;
    ane?: boolean; // Apple Neural Engine
  };
}

class LocalModelSelector {
  /**
   * Select best local model for task
   */
  async selectModel(criteria: ModelSelectionCriteria): Promise<SelectedModel> {
    // 1. Filter by capabilities
    const capable = await this.registry.findByCapabilities(
      criteria.requiredCapabilities
    );

    // 2. Filter by hardware compatibility
    const available = capable.filter((model) =>
      this.isHardwareCompatible(model, criteria.availableHardware)
    );

    if (available.length === 0) {
      throw new Error("No capable local models available");
    }

    // 3. Score based on historical performance
    const scored = await Promise.all(
      available.map(async (model) => ({
        model,
        score: await this.scoreModel(model, criteria),
      }))
    );

    // 4. Sort by score (higher = better)
    scored.sort((a, b) => b.score - a.score);

    // 5. Return best model (with fallback)
    return {
      primary: scored[0].model,
      fallback: scored[1]?.model, // Optional backup
      reasoning: this.explainSelection(scored[0]),
    };
  }

  /**
   * Score model based on historical performance + requirements
   */
  private async scoreModel(
    model: LocalModel,
    criteria: ModelSelectionCriteria
  ): Promise<number> {
    const history = await this.performanceTracker.getHistory(
      model.id,
      criteria.taskType
    );

    if (!history || history.samples < 10) {
      // New model or insufficient data - use conservative score
      return 0.5;
    }

    let score = 0;

    // Quality score (0-1)
    score += history.avgQuality * 0.4;

    // Latency score (0-1, inverted)
    const latencyScore = Math.max(
      0,
      1 - history.avgLatencyMs / criteria.maxLatencyMs
    );
    score += latencyScore * 0.3;

    // Resource efficiency score (0-1)
    const memoryScore = Math.max(
      0,
      1 - history.avgMemoryMB / criteria.maxMemoryMB
    );
    score += memoryScore * 0.2;

    // Reliability score (0-1)
    score += history.successRate * 0.1;

    return score; // 0-1 range
  }
}
```

---

## Hardware Optimization Integration

### Apple Silicon Optimization

```typescript
interface AppleSiliconOptimization {
  // Core ML integration
  coreMLModel?: string; // Path to .mlmodel or .mlpackage
  useANE: boolean; // Apple Neural Engine
  useMetal: boolean; // GPU acceleration

  // Memory optimization
  unifiedMemory: boolean; // Use unified memory architecture

  // Performance
  targetTokensPerSec: number;
}

class AppleSiliconProvider extends LocalModelProvider {
  /**
   * Load model optimized for Apple Silicon
   */
  async loadModel(config: AppleSiliconOptimization): Promise<Model> {
    if (config.coreMLModel && config.useANE) {
      // Load Core ML model optimized for ANE
      return await this.loadCoreMLModel(config.coreMLModel, {
        computeUnits: "neuralEngine", // ANE priority
        preferredDevice: "ane",
      });
    }

    if (config.useMetal) {
      // Use Metal-optimized inference (GPU)
      return await this.loadMetalOptimizedModel(config);
    }

    // Fallback to CPU with Accelerate framework
    return await this.loadCPUOptimizedModel(config);
  }
}
```

### Custom Server Optimization

```typescript
interface CustomServerOptimization {
  // Hardware specs
  gpuType: "nvidia" | "amd" | "intel" | "custom";
  gpuCount: number;
  vramGB: number;

  // Optimization strategy
  strategy: "throughput" | "latency" | "balanced";

  // Distributed inference
  multiNode: boolean;
  nodeAddresses?: string[];
}

class CustomServerProvider extends LocalModelProvider {
  /**
   * Distributed inference across multiple nodes
   */
  async loadDistributedModel(
    config: CustomServerOptimization
  ): Promise<DistributedModel> {
    if (config.multiNode && config.nodeAddresses) {
      // Split model across nodes (pipeline parallelism)
      return await this.loadPipelineParallelModel(config);
    }

    // Single node, multiple GPUs (tensor parallelism)
    return await this.loadTensorParallelModel(config);
  }
}
```

---

## Integration with Existing Components

### 1. DSPy Integration (Already Built)

**Reuse**: `python-services/dspy-integration/ollama_lm.py`

```typescript
class DSPyBridge {
  /**
   * Use DSPy's Ollama integration for model calls
   */
  async callDSPyModel(modelId: string, prompt: string): Promise<string> {
    // Forward to Python DSPy service
    return await this.httpClient.post("http://localhost:8001/dspy/generate", {
      model: modelId,
      prompt,
    });
  }
}
```

### 2. ModelBasedJudge (RL-003)

**Upgrade**: Replace MockLLMProvider with LocalModelProvider

```typescript
class ModelBasedJudge {
  constructor(
    private modelRegistry: ModelRegistryPoolManager, // NEW: Use registry
    private provider: LLMProvider // Keep interface
  ) {}

  async evaluate(
    input: JudgmentInput,
    criterion: EvaluationCriterion
  ): Promise<LLMResponse> {
    // Select best local model for judgment
    const model = await this.modelRegistry.selectBestModel({
      taskType: "judgment",
      requiredCapabilities: ["reasoning", "scoring"],
      maxLatencyMs: 2000,
    });

    // Use selected model
    return await this.provider.evaluate(input, criterion, model);
  }
}
```

### 3. Performance Tracker (ARBITER-004)

**Integration**: Feed model performance back to registry

```typescript
class PerformanceTracker {
  async recordModelPerformance(
    modelId: string,
    taskType: string,
    metrics: PerformanceMetrics
  ): Promise<void> {
    // Store in performance tracker
    await this.db.insertMetrics(metrics);

    // Update model registry's performance profile
    await this.modelRegistry.updatePerformanceProfile(modelId, {
      taskType,
      latency: metrics.latency,
      quality: metrics.quality,
      successRate: metrics.successRate,
    });
  }
}
```

---

## File Structure

```
src/models/
├── ModelRegistryPoolManager.ts      # Main orchestrator
├── ModelRegistry.ts                 # Registration & versioning
├── LocalModelSelector.ts            # Selection algorithms
├── ComputeCostTracker.ts            # Local compute tracking
├── HotSwapManager.ts                # Model hot-swapping
├── PerformanceProfiler.ts           # Profile capture & transfer
│
├── providers/
│   ├── LocalModelProvider.ts       # Abstract base
│   ├── OllamaProvider.ts           # Ollama integration
│   ├── CustomModelProvider.ts      # Custom trained models
│   ├── AppleSiliconProvider.ts     # Core ML / Metal
│   ├── CustomServerProvider.ts     # Hardware-optimized servers
│   └── DSPyBridge.ts               # Bridge to existing DSPy
│
├── pool/
│   ├── WarmInstanceManager.ts      # Warm model cache
│   ├── LoadBalancer.ts             # Request distribution
│   └── HealthMonitor.ts            # Model health checks
│
└── types/
    ├── model-registry.ts           # Type definitions
    ├── compute-cost.ts             # Cost tracking types
    └── hardware-optimization.ts    # Hardware-specific types
```

---

## Implementation Roadmap

### Week 1-2: Core Registry (Local-First)

**Day 1-3: Type System & Registry Core**

- [ ] Define local model types
- [ ] Implement ModelRegistry with versioning
- [ ] Add Ollama model registration
- [ ] Write 20+ unit tests

**Day 4-7: Local Model Providers**

- [ ] Create LocalModelProvider abstraction
- [ ] Implement OllamaProvider (reuse RL-011)
- [ ] Add CustomModelProvider
- [ ] Write 25+ integration tests with real Ollama models

**Day 8-10: Compute Cost Tracking**

- [ ] Implement ComputeCostTracker for local resources
- [ ] Add performance profiling
- [ ] Create model selection based on compute costs
- [ ] Write 15+ unit tests

### Week 3-4: Hot-Swap & Hardware Optimization

**Day 1-5: Hot-Swap Mechanism**

- [ ] Implement performance profile capture
- [ ] Create compatibility assessment
- [ ] Build gradual rollout with A/B testing
- [ ] Write 20+ integration tests

**Day 6-8: Hardware Optimization**

- [ ] Add Apple Silicon provider (Core ML/Metal)
- [ ] Create custom server provider
- [ ] Implement distributed inference support
- [ ] Write 15+ hardware-specific tests

**Day 9-12: Integration & Hardening**

- [ ] Integrate with RL-003 (ModelBasedJudge)
- [ ] Integrate with ARBITER-004 (Performance Tracker)
- [ ] Integrate with DSPy service
- [ ] End-to-end tests (30+ tests)
- [ ] Mutation testing to 50%+
- [ ] Documentation complete

---

## Success Criteria

### Technical Metrics

- 80%+ branch coverage (Tier 2)
- 50%+ mutation score (Tier 2)
- Model selection <50ms P95
- Hot-swap with <5% performance degradation
- Learning preservation across swaps (95%+ routing rule retention)

### Functional Validation

- All 4 Ollama models (RL-011) integrated
- Custom model loading working
- Hot-swap from gemma3:1b → gemma3n:e2b without retraining
- Compute cost tracking accurate (within 5%)
- Hardware optimization verified (Apple Silicon ANE)

### Philosophy Validation

- Zero API dependencies
- Local-first operation confirmed
- Model-agnostic learning verified
- Hot-swap preserves system knowledge
- Hardware optimization measurable

---

**This architecture embodies the "bring your own model" philosophy, ensuring Agent Agency V2 can leverage the best local models as they evolve, without vendor lock-in or loss of system learnings.**

---

**Author**: @darianrosebrook  
**Vision**: Local-first AI with hot-swappable models and preserved learnings  
**Status**: Ready for implementation
