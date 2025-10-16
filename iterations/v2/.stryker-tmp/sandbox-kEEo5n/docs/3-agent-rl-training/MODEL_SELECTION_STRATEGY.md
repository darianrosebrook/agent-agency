# Local-First Model Selection Strategy

**Status**: ✅ Definitive Strategy Approved  
**Last Updated**: October 13, 2025  
**Priority**: Local/Open-Source First, Paid/Closed-Source Fallback Only

## Executive Summary

Based on comprehensive benchmarking in iterations/poc and alignment with the parent Kokoro project's optimization work, this document defines the **definitive model selection strategy** for Agent Agency V2. The strategy prioritizes **local inference through Ollama** with carefully selected open-source models, falling back to paid APIs only when absolutely necessary.

## Core Principles

1. **Local-First**: Prioritize models that run efficiently on local hardware
2. **Open-Source Preference**: Use open-source models to avoid vendor lock-in and costs
3. **Performance-Optimized**: Leverage Kokoro project optimizations for inference speed
4. **Task-Specific Selection**: Route different tasks to optimal models
5. **Cost-Conscious**: Minimize reliance on paid API services
6. **Privacy-Preserving**: Keep sensitive data local when possible

## Benchmark-Validated Model Selection

### Based on POC Results (`iterations/poc/results/LLM_BENCHMARK_E2E_RESULTS.md`)

| Model           | Size  | Tokens/Sec | Quality | Use Case                         | Priority |
| --------------- | ----- | ---------- | ------- | -------------------------------- | -------- |
| **gemma3n:e2b** | 5.6GB | 36.02      | 8.5/10  | **Primary** - General purpose    | 1st      |
| gemma3:4b       | 3.3GB | 38.02      | 8.8/10  | Alternative - Higher quality     | 2nd      |
| gemma3:1b       | 815MB | 72.18      | 6.2/10  | Fast path - Speed-critical tasks | 3rd      |
| gemma3n:e4b     | 7.5GB | 23.83      | 9.1/10  | Quality path - Complex reasoning | 4th      |

### Selection Rationale (POC-Validated)

**Primary: `gemma3n:e2b`**

- **Why**: Optimal balance of speed (36 tok/sec) and quality (8.5/10)
- **Size**: 5.6GB fits on most developer machines
- **Benchmarked**: Proven in real-world agent workflows
- **Cost**: $0 (free local model)
- **Use Cases**: 80% of agent tasks

**Alternative: `gemma3:4b`**

- **Why**: Slightly faster, slightly better quality
- **When**: Tasks requiring higher quality but similar speed
- **Trade-off**: Smaller than e2b but still capable

**Fast Path: `gemma3:1b`**

- **Why**: 2.2s responses (vs 9.4s for e2b)
- **When**: Quick classification, simple transformations
- **Trade-off**: Lower quality (6.2/10) but 3.2x faster
- **Use Cases**: Initial task classification, quick feedback

**Quality Path: `gemma3n:e4b`**

- **Why**: Highest quality (9.1/10)
- **When**: Complex reasoning, critical evaluations
- **Trade-off**: Slower (23.83 tok/sec) but best output
- **Use Cases**: DSPy judge evaluation, complex rubric optimization

## Task-Specific Model Routing

### 1. DSPy Rubric Optimization

**Primary Model**: `gemma3n:e4b` (7.5GB)

- **Rationale**: Highest quality (9.1/10) for systematic prompt optimization
- **Expected Performance**: 15-20% improvement in rubric effectiveness
- **Configuration**:
  ```python
  dspy.settings.configure(
      lm=OllamaClient(
          model="gemma3n:e4b",
          host="http://localhost:11434",
          max_tokens=2048,
          temperature=0.7
      )
  )
  ```

**Fallback Model**: `gemma3:4b` (3.3GB)

- **When**: Resource constraints or faster iteration needed
- **Trade-off**: Slightly lower quality (8.8 vs 9.1) but faster

### 2. DSPy Model Judges

**Primary Models by Judge Type**:

| Judge Type   | Model         | Rationale                                  |
| ------------ | ------------- | ------------------------------------------ |
| Relevance    | `gemma3n:e2b` | Balance of speed/quality                   |
| Faithfulness | `gemma3n:e4b` | Requires deep understanding                |
| Minimality   | `gemma3:1b`   | Fast classification task                   |
| Safety       | `gemma3n:e4b` | Critical evaluation, needs highest quality |

**Configuration Example**:

```python
from dspy import OllamaClient

# Relevance Judge - Balanced
relevance_lm = OllamaClient(
    model="gemma3n:e2b",
    host="http://localhost:11434",
    max_tokens=512
)

# Faithfulness Judge - High Quality
faithfulness_lm = OllamaClient(
    model="gemma3n:e4b",
    host="http://localhost:11434",
    max_tokens=1024
)

# Minimality Judge - Fast
minimality_lm = OllamaClient(
    model="gemma3:1b",
    host="http://localhost:11434",
    max_tokens=256
)
```

### 3. Agent Task Execution

**Task Complexity Routing**:

```typescript
interface TaskComplexityProfile {
  simple: "gemma3:1b"; // Quick responses, classification
  medium: "gemma3n:e2b"; // General purpose, most tasks
  complex: "gemma3:4b"; // Detailed analysis, code generation
  critical: "gemma3n:e4b"; // Mission-critical, high-stakes
}
```

**Routing Logic**:

```typescript
function selectModelForTask(task: Task): string {
  // Fast path for simple tasks
  if (task.estimatedTokens < 100 && task.priority < 5) {
    return "gemma3:1b";
  }

  // Quality path for critical tasks
  if (task.priority >= 9 || task.requiresHighQuality) {
    return "gemma3n:e4b";
  }

  // Alternative for code generation
  if (task.type === "code_generation") {
    return "gemma3:4b";
  }

  // Default to primary
  return "gemma3n:e2b";
}
```

### 4. Thinking Budget Allocation

**Model Selection Based on Budget**:

```typescript
interface ThinkingBudgetModelStrategy {
  budget: number; // tokens
  model: string;
  expectedQuality: number;
}

const budgetStrategies: ThinkingBudgetModelStrategy[] = [
  { budget: 500, model: "gemma3:1b", expectedQuality: 6.2 },
  { budget: 1000, model: "gemma3n:e2b", expectedQuality: 8.5 },
  { budget: 2000, model: "gemma3:4b", expectedQuality: 8.8 },
  { budget: 4000, model: "gemma3n:e4b", expectedQuality: 9.1 },
];
```

## Kokoro-Inspired Optimizations

### From Parent Project (`../Kokoro`)

**Reference**: The parent Kokoro project has optimized local inference speed through:

1. **Model Quantization**: INT8 quantization for faster inference
2. **Context Caching**: Aggressive KV cache reuse
3. **Batched Inference**: Process multiple requests simultaneously
4. **Metal Performance Shaders**: Direct GPU acceleration on macOS
5. **Prompt Compilation**: Pre-compute embeddings for common prompts

**Application to V2**:

```typescript
// Future optimization integration
interface KokoroOptimizations {
  quantization: {
    enabled: true;
    precision: "int8" | "fp16";
  };
  caching: {
    kvCache: boolean;
    promptCache: boolean;
    maxCacheSize: number; // MB
  };
  batching: {
    enabled: boolean;
    maxBatchSize: number;
    timeout: number; // ms
  };
  hardware: {
    useMetalShaders: boolean; // macOS only
    useANE: boolean; // Apple Neural Engine
  };
}
```

**Performance Gains** (from Kokoro benchmarks):

- **+40%** inference speed with INT8 quantization
- **+60%** throughput with batching
- **+25%** speed with Metal shaders (macOS M-series)

**Integration Path**:

1. **Phase 2** (Current): Standard Ollama integration
2. **Phase 3** (Next 2 weeks): Add basic caching
3. **Phase 4** (Weeks 5-6): Integrate Kokoro optimizations
4. **Phase 5** (Weeks 7-8): Hardware-specific tuning

## Paid API Fallback Strategy

### When to Use Paid APIs

**Only use OpenAI/Anthropic when**:

1. Local models consistently fail for a specific task type
2. Task explicitly requires capabilities not available locally (e.g., DALL-E images)
3. Emergency fallback during local service outage
4. A/B testing new model capabilities

### Cost-Conscious Configuration

```typescript
interface FallbackConfig {
  enablePaidFallback: boolean;
  maxMonthlySpend: number; // USD
  costPerTask: {
    simple: 0.001; // $0.001 target
    medium: 0.005; // $0.005 target
    complex: 0.02; // $0.02 target
  };
  fallbackTriggers: {
    localFailureThreshold: 3; // failures before fallback
    timeoutThreshold: 30000; // ms
    qualityThreshold: 0.7; // minimum acceptable
  };
}
```

### Paid Model Selection (When Required)

| Provider  | Model             | Use Case                 | Cost     | Priority |
| --------- | ----------------- | ------------------------ | -------- | -------- |
| OpenAI    | `gpt-4o-mini`     | Fast, cost-effective     | $0.15/1M | 1st      |
| OpenAI    | `gpt-4-turbo`     | Complex reasoning        | $10/1M   | 2nd      |
| Anthropic | `claude-3-haiku`  | Alternative fast option  | $0.25/1M | 3rd      |
| Anthropic | `claude-3-sonnet` | Alternative quality path | $3/1M    | 4th      |

**Cost Comparison**:

```
Local (Ollama): $0/month
Paid Fallback: ~$5-20/month (with 95% local routing)
Full Paid: ~$200-500/month (100% paid APIs)
```

## Implementation Configuration

### Environment Variables

```bash
# Local Ollama Configuration
OLLAMA_HOST=http://localhost:11434
OLLAMA_PRIMARY_MODEL=gemma3n:e2b
OLLAMA_FAST_MODEL=gemma3:1b
OLLAMA_QUALITY_MODEL=gemma3n:e4b
OLLAMA_ALTERNATIVE_MODEL=gemma3:4b

# Model Routing Strategy
MODEL_ROUTING_STRATEGY=local-first
MODEL_FALLBACK_ENABLED=false
MODEL_SELECTION_LOGGING=true

# Cost Controls (for paid fallback if enabled)
PAID_API_ENABLED=false
PAID_API_MAX_MONTHLY_SPEND=20
PAID_API_FAILURE_THRESHOLD=3

# Performance Optimization
ENABLE_KV_CACHE=true
ENABLE_PROMPT_CACHE=true
ENABLE_BATCHED_INFERENCE=false
MAX_CACHE_SIZE_MB=2048
```

### DSPy Configuration

```python
# python-services/dspy-integration/config.py

import os
from dspy import OllamaClient, settings

# Model selection based on environment
OLLAMA_HOST = os.getenv("OLLAMA_HOST", "http://localhost:11434")
PRIMARY_MODEL = os.getenv("OLLAMA_PRIMARY_MODEL", "gemma3n:e2b")
QUALITY_MODEL = os.getenv("OLLAMA_QUALITY_MODEL", "gemma3n:e4b")
FAST_MODEL = os.getenv("OLLAMA_FAST_MODEL", "gemma3:1b")

# DSPy Clients
primary_lm = OllamaClient(
    model=PRIMARY_MODEL,
    host=OLLAMA_HOST,
    max_tokens=2048,
    temperature=0.7,
)

quality_lm = OllamaClient(
    model=QUALITY_MODEL,
    host=OLLAMA_HOST,
    max_tokens=2048,
    temperature=0.7,
)

fast_lm = OllamaClient(
    model=FAST_MODEL,
    host=OLLAMA_HOST,
    max_tokens=512,
    temperature=0.7,
)

# Configure default
settings.configure(lm=primary_lm)

# Export for use in signatures
__all__ = ["primary_lm", "quality_lm", "fast_lm", "PRIMARY_MODEL", "QUALITY_MODEL", "FAST_MODEL"]
```

### TypeScript Configuration

```typescript
// src/config/model-selection.ts

export interface ModelSelectionConfig {
  provider: "ollama" | "openai" | "anthropic";
  primary: string;
  fast: string;
  quality: string;
  alternative: string;
  fallbackEnabled: boolean;
  localFirst: boolean;
}

export const defaultModelConfig: ModelSelectionConfig = {
  provider: "ollama",
  primary: process.env.OLLAMA_PRIMARY_MODEL ?? "gemma3n:e2b",
  fast: process.env.OLLAMA_FAST_MODEL ?? "gemma3:1b",
  quality: process.env.OLLAMA_QUALITY_MODEL ?? "gemma3n:e4b",
  alternative: process.env.OLLAMA_ALTERNATIVE_MODEL ?? "gemma3:4b",
  fallbackEnabled: process.env.PAID_API_ENABLED === "true",
  localFirst: true,
};
```

## Performance Expectations

### Local Inference Performance (Ollama + Gemma)

| Model       | Cold Start | Warm Latency | Throughput | Quality |
| ----------- | ---------- | ------------ | ---------- | ------- |
| gemma3:1b   | <1s        | 2.2s         | 72 tok/s   | 6.2/10  |
| gemma3n:e2b | <2s        | 9.4s         | 36 tok/s   | 8.5/10  |
| gemma3:4b   | <2s        | 5.0s         | 38 tok/s   | 8.8/10  |
| gemma3n:e4b | <3s        | 5.3s         | 24 tok/s   | 9.1/10  |

### With Kokoro Optimizations (Projected)

| Optimization             | Speed Improvement | Quality Impact |
| ------------------------ | ----------------- | -------------- |
| INT8 Quantization        | +40%              | -5%            |
| KV Cache                 | +20%              | None           |
| Batched Inference        | +60%              | None           |
| Metal Shaders (M-series) | +25%              | None           |
| **Combined**             | **+90%**          | **-5%**        |

**Example: gemma3n:e2b with optimizations**

- Current: 36 tok/s
- With optimizations: ~68 tok/s
- Quality: 8.5/10 → 8.0/10 (acceptable trade-off)

## Migration Path from POC to V2

### Step 1: Port Ollama Integration (Complete)

```typescript
// Already done in POC: iterations/poc/src/ai/ollama-client.ts
// Copy to V2: src/ai/OllamaClient.ts
```

### Step 2: Multi-Model Orchestrator

```typescript
// Copy from: iterations/poc/src/ai/multi-model-orchestrator.ts
// To: src/ai/MultiModelOrchestrator.ts
// Update for V2 type system
```

### Step 3: DSPy Integration

```python
# Already implemented: python-services/dspy-integration/
# Update main.py to use Ollama by default
```

### Step 4: Configuration Update

```bash
# Update .env files across V2
cp iterations/poc/.env.example iterations/v2/.env.example
# Add OLLAMA_* variables
```

## Testing & Validation

### Model Performance Tests

```typescript
// tests/integration/model-performance/ollama-performance.test.ts

describe("Ollama Model Performance", () => {
  it("should meet latency requirements for gemma3:1b", async () => {
    const start = Date.now();
    const response = await ollamaClient.generate({
      model: "gemma3:1b",
      prompt: "Simple test",
      max_tokens: 50,
    });
    const duration = Date.now() - start;

    expect(duration).toBeLessThan(3000); // 3s target
    expect(response.text).toBeDefined();
  });

  it("should meet quality requirements for gemma3n:e4b", async () => {
    const response = await ollamaClient.generate({
      model: "gemma3n:e4b",
      prompt: "Complex reasoning task",
      max_tokens: 200,
    });

    const quality = await evaluateQuality(response.text);
    expect(quality).toBeGreaterThan(8.5);
  });
});
```

### Cost Tracking Tests

```typescript
describe("Cost Tracking", () => {
  it("should prefer local models over paid APIs", async () => {
    const costTracker = new CostTracker();

    await modelOrchestrator.generate({
      prompt: "Test prompt",
      maxCost: 0.01,
    });

    const costs = costTracker.getTotalCost();
    expect(costs.local).toBeGreaterThan(costs.paid * 10);
  });
});
```

## Deployment Checklist

- [ ] Ollama server running (`docker-compose up ollama` or local install)
- [ ] Gemma models downloaded (`ollama pull gemma3n:e2b gemma3:1b gemma3:4b gemma3n:e4b`)
- [ ] Environment variables configured
- [ ] DSPy configured with Ollama clients
- [ ] Model routing logic implemented
- [ ] Performance benchmarks passing
- [ ] Cost tracking enabled
- [ ] Monitoring and logging active

## Future Enhancements

### Short Term (Phase 2-3, Weeks 3-6)

1. **Implement KV cache** for faster repeated prompts
2. **Add prompt caching** for common evaluations
3. **Monitor model performance** across different tasks
4. **Tune model selection** based on real-world metrics

### Medium Term (Phase 4, Weeks 7-12)

1. **Integrate Kokoro optimizations**:
   - INT8 quantization
   - Batched inference
   - Metal shader acceleration (macOS)
2. **Fine-tune models** for specific agent tasks
3. **Implement model distillation** (gemma3n:e4b → gemma3n:e2b knowledge transfer)

### Long Term (Phase 5+, Months 4-6)

1. **Custom model training** on agent interaction data
2. **Hybrid local/cloud** routing with predictive fallback
3. **Multi-modal support** (vision, audio) when models mature
4. **Hardware-specific optimization** (ANE, CUDA, ROCm)

## References

1. **POC Benchmark Results**: `iterations/poc/results/LLM_BENCHMARK_E2E_RESULTS.md`
2. **Ollama Client Implementation**: `iterations/poc/src/ai/ollama-client.ts`
3. **Multi-Model Orchestrator**: `iterations/poc/src/ai/multi-model-orchestrator.ts`
4. **Theory Document**: `iterations/v2/docs/1-core-orchestration/theory.md`
5. **Kokoro Project**: `../Kokoro` (parent folder optimizations)
6. **Docker Compose**: `iterations/poc/docker-compose.yml` (Ollama setup)

## Conclusion

This strategy ensures Agent Agency V2 prioritizes **local, open-source models** while maintaining flexibility for future enhancements. The `gemma3n:e2b` model provides the optimal balance for most use cases, with task-specific routing to faster or higher-quality models as needed.

**Cost Savings**: ~$200-500/month (vs full paid API usage)  
**Performance**: Competitive with paid APIs for most tasks  
**Privacy**: All sensitive data stays local  
**Flexibility**: Easy to add new models or fall back to paid APIs

**Status**: ✅ **Ready for Implementation**

---

**Author**: @darianrosebrook  
**Last Updated**: October 13, 2025  
**Implementation Priority**: Phase 2 (Weeks 3-4)
