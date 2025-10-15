/**
 * @fileoverview
 * Type definitions for the local-first Model Registry/Pool Manager.
 * Focused on bring-your-own-model (BYOM) philosophy with hot-swappable local models.
 *
 * @author @darianrosebrook
 */

/**
 * Local model deployment type
 */
export type LocalModelType = "ollama" | "custom" | "hardware-optimized";

/**
 * Hardware accelerator type
 */
export type AcceleratorType = "cpu" | "gpu" | "ane" | "hybrid";

/**
 * Model quantization level
 */
export type QuantizationType = "4bit" | "8bit" | "16bit" | "full";

/**
 * Task category for model selection
 */
export type TaskCategory =
  | "fast"
  | "primary"
  | "quality"
  | "alternative"
  | "specialized";

/**
 * Hardware type for optimization
 */
export type HardwareType =
  | "apple-silicon"
  | "nvidia-gpu"
  | "amd-gpu"
  | "custom-server"
  | "cpu-only";

/**
 * Model optimization framework
 */
export type OptimizationFramework =
  | "coreml"
  | "metal"
  | "onnx"
  | "ggml"
  | "pytorch"
  | "tensorrt";

/**
 * Base local model metadata
 */
export interface LocalModelMetadata {
  /** Unique model identifier */
  id: string;

  /** Human-readable name */
  name: string;

  /** Model deployment type */
  type: LocalModelType;

  /** Model version */
  version: string;

  /** Task category */
  category: TaskCategory;

  /** Model capabilities */
  capabilities: string[];

  /** Context window size (tokens) */
  contextWindow: number;

  /** Supports streaming responses */
  supportsStreaming: boolean;

  /** Supports batch inference */
  supportsBatching: boolean;

  /** Creation timestamp */
  createdAt: Date;

  /** Last updated timestamp */
  updatedAt: Date;

  /** Model status */
  status: "active" | "deprecated" | "testing";

  /** Optional deprecation date */
  deprecatedAt?: Date;

  /** Optional description */
  description?: string;

  /** Optional tags for categorization */
  tags?: string[];

  /** Hardware requirements (optional for Ollama models) */
  hardwareRequirements?: HardwareSpec;

  /** Performance profile (optional, populated after measurement) */
  performanceProfile?: PerformanceCharacteristics;
}

/**
 * Ollama model configuration
 */
export interface OllamaModelConfig extends LocalModelMetadata {
  type: "ollama";

  /** Ollama model name (e.g., "gemma3n:e2b") */
  ollamaName: string;

  /** Quantization level */
  quantization: QuantizationType;

  /** Measured tokens per second */
  tokensPerSec?: number;

  /** Memory usage in MB */
  memoryUsageMB?: number;

  /** Ollama API endpoint */
  endpoint?: string; // Default: http://localhost:11434
}

/**
 * Custom trained model configuration
 */
export interface CustomModelConfig extends LocalModelMetadata {
  type: "custom";

  /** Path to model files */
  modelPath: string;

  /** Framework used */
  framework: OptimizationFramework;

  /** Training data provenance */
  trainingProvenance?: string;

  /** Model specialization */
  specialization?: string;

  /** Hardware requirements */
  hardwareRequirements: HardwareSpec;
}

/**
 * Hardware-optimized model configuration
 */
export interface HardwareOptimizedModelConfig extends LocalModelMetadata {
  type: "hardware-optimized";

  /** Target hardware */
  targetHardware: HardwareType;

  /** Optimization framework */
  optimization: OptimizationFramework;

  /** Accelerator to use */
  accelerator: AcceleratorType;

  /** Path to optimized model */
  modelPath: string;

  /** Expected performance characteristics */
  expectedPerformance?: PerformanceCharacteristics;
}

/**
 * Union type for all local model configurations
 */
export type LocalModelConfig =
  | OllamaModelConfig
  | CustomModelConfig
  | HardwareOptimizedModelConfig;

/**
 * Hardware specification
 */
export interface HardwareSpec {
  /** Minimum CPU cores */
  minCpuCores?: number;

  /** Minimum RAM in GB */
  minRamGB?: number;

  /** Minimum memory in MB (alternative to minRamGB) */
  minMemoryMB?: number;

  /** GPU required */
  requiresGpu?: boolean;

  /** Minimum VRAM in GB */
  minVramGB?: number;

  /** Preferred hardware type */
  preferredHardware?: HardwareType;

  /** Specific hardware type */
  hardwareType?: HardwareType;
}

/**
 * Available hardware capabilities
 */
export interface AvailableHardware {
  /** CPU available */
  cpu: boolean;

  /** GPU available */
  gpu: boolean;

  /** Apple Neural Engine available */
  ane?: boolean;

  /** Hardware details */
  details?: {
    cpuCores: number;
    ramGB: number;
    gpuModel?: string;
    vramGB?: number;
  };
}

/**
 * Performance characteristics of a model
 */
export interface PerformanceCharacteristics {
  /** Average latency in milliseconds */
  avgLatencyMs: number;

  /** P95 latency in milliseconds */
  p95LatencyMs: number;

  /** Tokens per second */
  tokensPerSec: number;

  /** Memory usage in MB */
  memoryUsageMB: number;

  /** CPU utilization percentage */
  cpuUtilization: number;

  /** GPU utilization percentage */
  gpuUtilization?: number;

  /** ANE utilization percentage (Apple only) */
  aneUtilization?: number;
}

/**
 * Local compute cost tracking
 */
export interface LocalComputeCost {
  /** Model ID */
  modelId: string;

  /** Operation ID for tracing */
  operationId: string;

  /** Timestamp */
  timestamp: Date;

  /** Wall clock time in milliseconds */
  wallClockMs: number;

  /** CPU time in milliseconds */
  cpuTimeMs: number;

  /** GPU time in milliseconds */
  gpuTimeMs?: number;

  /** Peak memory usage in MB */
  peakMemoryMB: number;

  /** Average memory usage in MB */
  avgMemoryMB: number;

  /** Estimated energy in milliwatt-hours */
  estimatedEnergyMWh?: number;

  /** CPU utilization 0-100 */
  cpuUtilization: number;

  /** GPU utilization 0-100 */
  gpuUtilization?: number;

  /** ANE utilization 0-100 (Apple only) */
  aneUtilization?: number;

  /** Input tokens */
  inputTokens: number;

  /** Output tokens */
  outputTokens: number;

  /** Tokens per second */
  tokensPerSecond: number;
}

/**
 * Performance history for a model on a task type
 */
export interface PerformanceHistory {
  /** Model ID */
  modelId: string;

  /** Task type */
  taskType: string;

  /** Number of samples */
  samples: number;

  /** Average quality score (0-1) */
  avgQuality: number;

  /** Average latency in milliseconds */
  avgLatencyMs: number;

  /** P95 latency in milliseconds */
  p95LatencyMs: number;

  /** Average memory usage in MB */
  avgMemoryMB: number;

  /** Success rate (0-1) */
  successRate: number;

  /** Last updated */
  lastUpdated: Date;
}

/**
 * Performance profile of a model across all tasks
 */
export interface PerformanceProfile {
  /** Model ID */
  modelId: string;

  /** Task-specific performance */
  taskCategories: {
    taskType: string;
    successRate: number;
    avgLatency: number;
    qualityScore: number;
  }[];

  /** Model capabilities */
  capabilities: {
    maxContextWindow: number;
    streamingSupport: boolean;
    batchingSupport: boolean;
  };

  /** Resource usage */
  resourceUsage: {
    avgMemoryMB: number;
    avgCPUPercent: number;
    avgGPUPercent?: number;
    energyPerToken?: number; // mWh per token
  };

  /** Profile timestamp */
  capturedAt: Date;
}

/**
 * Model selection criteria
 */
export interface ModelSelectionCriteria {
  /** Task type */
  taskType: string;

  /** Required capabilities */
  requiredCapabilities: string[];

  /** Quality threshold (0-1) */
  qualityThreshold: number;

  /** Maximum latency in milliseconds */
  maxLatencyMs: number;

  /** Maximum memory in MB */
  maxMemoryMB: number;

  /** Available hardware */
  availableHardware: AvailableHardware;

  /** Prefer local models */
  preferLocal?: boolean;

  /** Optional model preferences */
  preferences?: {
    preferFast?: boolean;
    preferQuality?: boolean;
    preferLowMemory?: boolean;
  };
}

/**
 * Selected model result
 */
export interface SelectedModel {
  /** Primary model to use */
  primary: LocalModelConfig;

  /** Fallback model (optional) */
  fallback?: LocalModelConfig;

  /** Selection reasoning */
  reasoning: string[];

  /** Confidence score (0-1) */
  confidence: number;

  /** Expected performance */
  expectedPerformance: PerformanceCharacteristics;
}

/**
 * Model compatibility assessment
 */
export interface CompatibilityResult {
  /** Can new model replace old model */
  canReplace: boolean;

  /** Reason if cannot replace */
  reason?: string;

  /** Warnings about the replacement */
  warnings: string[];

  /** Compatibility score (0-1) */
  compatibilityScore: number;

  /** Capability gaps */
  capabilityGaps?: string[];

  /** Performance comparison */
  performanceComparison?: {
    latencyChange: number; // Percentage change
    qualityChange: number; // Percentage change
    memoryChange: number; // Percentage change
  };
}

/**
 * Model swap configuration
 */
export interface SwapConfig {
  /** Old model ID */
  oldModelId: string;

  /** New model ID */
  newModelId: string;

  /** Gradual rollout settings */
  rollout: {
    startPercent: number;
    rampUpRate: number; // Percent per hour
    rollbackThreshold: number; // Rollback if performance < threshold
  };

  /** Keep old model warm for rollback */
  keepOldModelWarm: boolean;

  /** Duration to keep old model (e.g., "7d") */
  deprecationDuration?: string;

  /** Validation set for testing */
  validationSet?: string[];
}

/**
 * Hot-swap configuration
 */
export interface HotSwapConfig {
  /** Enable automatic swapping based on performance */
  enableAutoSwap: boolean;

  /** Minimum time between swaps (milliseconds) */
  swapCooldownMs: number;

  /** Minimum samples before considering swap */
  minSamplesBeforeSwap: number;

  /** Performance threshold (0-1) - swap if below */
  performanceThreshold: number;

  /** Strict compatibility checking */
  compatibilityCheckStrict: boolean;
}

/**
 * Swap event for tracking
 */
export interface SwapEvent {
  /** Event timestamp */
  timestamp: Date;

  /** Previous model ID */
  fromModelId: string;

  /** New model ID */
  toModelId: string;

  /** Task type context */
  taskType: string;

  /** Swap reason */
  reason: string;

  /** Success status */
  success: boolean;

  /** Swap duration in milliseconds */
  durationMs: number;

  /** Compatibility warnings */
  compatibilityWarnings?: string[];

  /** Error message if failed */
  error?: string;
}

/**
 * Model swap result
 */
export interface SwapResult {
  /** Success status */
  success: boolean;

  /** Rollout ID for tracking */
  rolloutId?: string;

  /** Reason if failed */
  reason?: string;

  /** Compatibility result */
  compatibility?: CompatibilityResult;

  /** Rollout status */
  rolloutStatus?: {
    currentPercent: number;
    startedAt: Date;
    estimatedCompletionAt?: Date;
  };
}

/**
 * Cost profile for model selection
 */
export interface CostProfile {
  /** Model ID */
  modelId: string;

  /** Average wall clock time in milliseconds */
  avgWallClockMs: number;

  /** Average energy in milliwatt-hours */
  avgEnergyMWh?: number;

  /** Average tokens per second */
  avgTokensPerSec: number;

  /** P95 wall clock time in milliseconds */
  p95WallClockMs: number;

  /** Total operations tracked */
  totalOperations: number;

  /** Last updated */
  lastUpdated: Date;
}

/**
 * Model registry query options
 */
export interface RegistryQueryOptions {
  /** Filter by status */
  status?: "active" | "deprecated" | "testing";

  /** Filter by type */
  type?: LocalModelType;

  /** Filter by capabilities */
  capabilities?: string[];

  /** Filter by category */
  category?: TaskCategory;

  /** Filter by tags */
  tags?: string[];

  /** Sort field */
  sortBy?: "name" | "createdAt" | "updatedAt" | "performance";

  /** Sort order */
  sortOrder?: "asc" | "desc";

  /** Pagination */
  limit?: number;
  offset?: number;
}

/**
 * Model registration request
 */
export interface ModelRegistrationRequest {
  /** Model configuration */
  config: Omit<LocalModelConfig, "id" | "createdAt" | "updatedAt" | "status">;

  /** Validate on registration */
  validate?: boolean;

  /** Run performance profiling */
  profile?: boolean;
}

/**
 * Model update request
 */
export interface ModelUpdateRequest {
  /** Model ID */
  modelId: string;

  /** Fields to update */
  updates: Partial<Omit<LocalModelConfig, "id" | "type" | "createdAt">>;
}

/**
 * Warm instance status
 */
export interface WarmInstanceStatus {
  /** Model ID */
  modelId: string;

  /** Instance ID */
  instanceId: string;

  /** Status */
  status: "warming" | "ready" | "busy" | "cooling";

  /** Last used timestamp */
  lastUsedAt: Date;

  /** Request count */
  requestCount: number;

  /** Memory usage in MB */
  memoryUsageMB: number;

  /** Last health check */
  lastHealthCheck: Date;

  /** Health status */
  healthy: boolean;
}

/**
 * Load balancing strategy
 */
export type LoadBalancingStrategy =
  | "round-robin"
  | "least-busy"
  | "performance-based"
  | "random";

/**
 * Pool configuration
 */
export interface PoolConfig {
  /** Maximum warm instances per model */
  maxWarmInstances: number;

  /** Minimum warm instances per model */
  minWarmInstances: number;

  /** Idle timeout before cooling (milliseconds) */
  idleTimeoutMs: number;

  /** Load balancing strategy */
  loadBalancingStrategy: LoadBalancingStrategy;

  /** Enable health monitoring */
  enableHealthMonitoring: boolean;

  /** Health check interval (milliseconds) */
  healthCheckIntervalMs: number;
}

// Re-export generation types from LocalModelProvider for convenience
export type {
  ModelGenerationRequest as GenerationRequest,
  ModelGenerationResponse as GenerationResponse,
  ModelHealthStatus as ModelHealth,
} from "../models/providers/LocalModelProvider";
