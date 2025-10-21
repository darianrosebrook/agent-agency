/**
 * RL Data Pipeline for Training Data Management
 *
 * @author @darianrosebrook
 * @module rl-data-pipeline
 *
 * Manages the pipeline for converting performance data into RL training samples
 * with data quality validation, batching, and delivery to training systems.
 */

import { createHash } from "crypto";
import { EventEmitter } from "events";
import { Timestamp } from "../types/agent-registry";
import {
  AgentPerformanceProfile,
  PerformanceEvent,
  RLTrainingBatch,
  RLTrainingSample,
} from "../types/performance-tracking";

/**
 * RL pipeline configuration.
 */
export interface RLDataPipelineConfig {
  /**
   * Data quality thresholds.
   */
  qualityThresholds: {
    minSampleDiversity: number;
    maxTemporalGapMinutes: number;
    minRewardVariance: number;
    maxDuplicateRatio: number;
  };

  /**
   * Batch configuration.
   */
  batching: {
    maxBatchSize: number;
    maxBatchAgeMinutes: number;
    minBatchSize: number;
  };

  /**
   * Training data retention and cleanup.
   */
  retention: {
    maxSamplesInMemory: number;
    maxBatchesInMemory: number;
    cleanupIntervalMinutes: number;
  };

  /**
   * State representation configuration.
   */
  stateRepresentation: {
    includeHistoricalMetrics: boolean;
    includeAgentLoad: boolean;
    includeTaskContext: boolean;
    temporalWindowSize: number;
  };

  /**
   * Reward function configuration.
   */
  rewardFunction: {
    latencyWeight: number;
    accuracyWeight: number;
    costWeight: number;
    complianceWeight: number;
    temporalDecayFactor: number;
  };
}

/**
 * Default RL pipeline configuration.
 */
const DEFAULT_CONFIG: RLDataPipelineConfig = {
  qualityThresholds: {
    minSampleDiversity: 0.7,
    maxTemporalGapMinutes: 30,
    minRewardVariance: 0.1,
    maxDuplicateRatio: 0.2,
  },
  batching: {
    maxBatchSize: 1000,
    maxBatchAgeMinutes: 15,
    minBatchSize: 100,
  },
  retention: {
    maxSamplesInMemory: 50000,
    maxBatchesInMemory: 100,
    cleanupIntervalMinutes: 30,
  },
  stateRepresentation: {
    includeHistoricalMetrics: true,
    includeAgentLoad: true,
    includeTaskContext: true,
    temporalWindowSize: 10,
  },
  rewardFunction: {
    latencyWeight: 0.3,
    accuracyWeight: 0.4,
    costWeight: 0.2,
    complianceWeight: 0.1,
    temporalDecayFactor: 0.95,
  },
};

/**
 * Internal pipeline state for tracking processing.
 */
interface PipelineState {
  agentId: string;
  recentSamples: RLTrainingSample[];
  pendingBatch: RLTrainingSample[];
  batchStartTime: Timestamp;
  lastProcessedEvent: Timestamp;
  performanceHistory: AgentPerformanceProfile[];
}

/**
 * Data quality metrics for monitoring pipeline health.
 */
interface DataQualityMetrics {
  sampleDiversity: number;
  temporalGapMinutes: number;
  rewardVariance: number;
  duplicateRatio: number;
  sampleCount: number;
  batchCount: number;
}

/**
 * RL Data Pipeline for managing training data flow.
 *
 * This component transforms raw performance events into RL training samples
 * with quality validation, batching, and delivery to training systems.
 */
export class RLDataPipeline extends EventEmitter {
  private config: RLDataPipelineConfig;
  private pipelineStates: Map<string, PipelineState> = new Map();
  private completedBatches: RLTrainingBatch[] = [];
  private isProcessing = false;
  private cleanupTimer?: ReturnType<typeof setTimeout>;
  private qualityMetrics: Map<string, DataQualityMetrics> = new Map();

  /**
   * Creates a new RL Data Pipeline instance.
   *
   * @param config - Pipeline configuration. Uses defaults if not provided.
   */
  constructor(config: Partial<RLDataPipelineConfig> = {}) {
    super();
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.setupEventHandlers();
    this.startCleanupTimer();
  }

  /**
   * Starts the data pipeline processing.
   */
  startProcessing(): void {
    this.isProcessing = true;
    this.emit("pipeline_started");
  }

  /**
   * Stops the data pipeline processing.
   */
  stopProcessing(): void {
    this.isProcessing = false;
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = undefined;
    }
    this.emit("pipeline_stopped");
  }

  /**
   * Processes performance events into RL training samples.
   *
   * @param events - Performance events to process
   * @param agentProfiles - Current agent performance profiles for context
   * @returns Processing statistics
   */
  async processEvents(
    events: PerformanceEvent[],
    agentProfiles: AgentPerformanceProfile[]
  ): Promise<{
    samplesGenerated: number;
    batchesCompleted: number;
    qualityIssues: string[];
  }> {
    if (!this.isProcessing) {
      return { samplesGenerated: 0, batchesCompleted: 0, qualityIssues: [] };
    }

    const startTime = performance.now();
    let samplesGenerated = 0;
    let batchesCompleted = 0;
    const qualityIssues: string[] = [];

    try {
      // Group events by agent
      const eventsByAgent = this.groupEventsByAgent(events);

      for (const [agentId, agentEvents] of Array.from(eventsByAgent)) {
        const agentProfile = agentProfiles.find((p) => p.agentId === agentId);
        if (!agentProfile) continue;

        const result = await this.processAgentEvents(
          agentId,
          agentEvents,
          agentProfile
        );
        samplesGenerated += result.samplesGenerated;
        batchesCompleted += result.batchesCompleted;
        qualityIssues.push(...result.qualityIssues);
      }

      // Update quality metrics
      this.updateQualityMetrics();

      const processingTime = performance.now() - startTime;
      this.emit("events_processed", {
        eventCount: events.length,
        samplesGenerated,
        batchesCompleted,
        processingTimeMs: processingTime,
        qualityIssues,
      });
    } catch (error) {
      this.emit("processing_error", error);
      qualityIssues.push(`Processing error: ${error}`);
    }

    return { samplesGenerated, batchesCompleted, qualityIssues };
  }

  /**
   * Retrieves ready training batches for consumption.
   *
   * @param agentId - Specific agent to get batches for (optional)
   * @param maxBatches - Maximum number of batches to retrieve
   * @returns Array of training batches
   */
  getTrainingBatches(
    agentId?: string,
    maxBatches: number = 10
  ): RLTrainingBatch[] {
    let batches = this.completedBatches;

    if (agentId) {
      batches = batches.filter((batch) => batch.agentId === agentId);
    }

    // Sort by creation time (newest first)
    batches.sort(
      (a, b) =>
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
    );

    const result = batches.slice(0, maxBatches);

    // Remove retrieved batches from completed queue
    this.completedBatches = this.completedBatches.filter(
      (batch) => !result.some((retrieved) => retrieved.id === batch.id)
    );

    return result;
  }

  /**
   * Gets current pipeline statistics and health metrics.
   */
  getPipelineStats() {
    const activeAgents = Array.from(this.pipelineStates.keys());
    const totalSamples = activeAgents.reduce((sum, agentId) => {
      const state = this.pipelineStates.get(agentId);
      return (
        sum +
        (state?.recentSamples.length || 0) +
        (state?.pendingBatch.length || 0)
      );
    }, 0);

    const totalBatches = this.completedBatches.length;
    const pendingBatches = activeAgents
      .map((agentId) => this.pipelineStates.get(agentId)?.pendingBatch || [])
      .filter((batch) => batch.length > 0);

    return {
      isProcessing: this.isProcessing,
      activeAgents: activeAgents.length,
      totalSamples,
      totalBatches,
      completedBatches: this.completedBatches,
      pendingBatches,
      qualityMetrics: Object.fromEntries(this.qualityMetrics),
      config: this.config,
    };
  }

  /**
   * Updates pipeline configuration.
   *
   * @param config - New configuration to apply
   */
  updateConfig(config: Partial<RLDataPipelineConfig>): void {
    this.config = { ...this.config, ...config };
    this.emit("config_updated", this.config);
  }

  /**
   * Clears all pipeline data and resets state.
   */
  clearData(): void {
    this.pipelineStates.clear();
    this.completedBatches = [];
    this.qualityMetrics.clear();
    this.emit("data_cleared");
  }

  /**
   * Groups events by agent for processing.
   */
  private groupEventsByAgent(
    events: PerformanceEvent[]
  ): Map<string, PerformanceEvent[]> {
    const grouped = new Map<string, PerformanceEvent[]>();

    for (const event of events) {
      const agentId = event.agentId || "unknown";

      if (!grouped.has(agentId)) {
        grouped.set(agentId, []);
      }

      grouped.get(agentId)!.push(event);
    }

    return grouped;
  }

  /**
   * Processes events for a specific agent.
   */
  private async processAgentEvents(
    agentId: string,
    events: PerformanceEvent[],
    agentProfile: AgentPerformanceProfile
  ): Promise<{
    samplesGenerated: number;
    batchesCompleted: number;
    qualityIssues: string[];
  }> {
    let samplesGenerated = 0;
    let batchesCompleted = 0;
    const qualityIssues: string[] = [];

    // Get or create pipeline state for this agent
    let state = this.pipelineStates.get(agentId);
    if (!state) {
      state = this.createPipelineState(agentId);
      this.pipelineStates.set(agentId, state);
    }

    // Update performance history
    state.performanceHistory.push(agentProfile);
    if (
      state.performanceHistory.length >
      this.config.stateRepresentation.temporalWindowSize
    ) {
      state.performanceHistory.shift(); // Keep only recent history
    }

    // Process each event
    for (const event of events) {
      try {
        const sample = await this.createTrainingSample(
          event,
          state,
          agentProfile
        );
        if (sample) {
          state.pendingBatch.push(sample);
          samplesGenerated++;

          // Check if batch should be completed
          const shouldComplete = this.shouldCompleteBatch(state);
          if (shouldComplete) {
            const batch = await this.createTrainingBatch(state);
            this.completedBatches.push(batch);
            batchesCompleted++;

            // Add samples to recent samples for tracking
            state.recentSamples.push(...state.pendingBatch);

            // Reset batch state
            state.pendingBatch = [];
            state.batchStartTime = new Date().toISOString();
          }
        }
      } catch (error) {
        qualityIssues.push(`Failed to process event ${event.id}: ${error}`);
      }
    }

    // Update last processed time
    state.lastProcessedEvent = new Date().toISOString();

    // Check for quality issues
    const agentQualityIssues = this.checkDataQuality(agentId, state);
    qualityIssues.push(...agentQualityIssues);

    return { samplesGenerated, batchesCompleted, qualityIssues };
  }

  /**
   * Creates a training sample from a performance event.
   */
  private async createTrainingSample(
    event: PerformanceEvent,
    state: PipelineState,
    agentProfile: AgentPerformanceProfile
  ): Promise<RLTrainingSample | null> {
    // TODO: Implement comprehensive event-driven RL training data collection
    // - Support multiple event types (task_start, task_progress, task_failure, agent_feedback)
    // - Implement temporal event sequences and state transitions
    // - Add event context enrichment and metadata correlation
    // - Support event filtering and sampling strategies
    // - Implement event replay and simulation capabilities
    // - Add event quality assessment and noise filtering
    // - Support multi-agent event correlation and coordination
    // - Implement event-driven reward signal generation
    if (event.type !== "task_execution_complete") {
      return null;
    }

    // Create state representation
    const stateRepresentation = this.createStateRepresentation(
      event,
      state,
      agentProfile
    );

    // TODO: Implement comprehensive action representation for RL training
    // - Create rich action representations capturing decision context
    // - Include action confidence scores and uncertainty measures
    // - Add alternative action exploration and evaluation
    // - Support action sequence modeling and temporal dependencies
    // - Implement action space discretization and encoding
    // - Add action cost and resource consumption tracking
    // - Support multi-agent action coordination and representation
    // - Implement action quality assessment and feedback integration
    const action = {
      selectedAgent: event.agentId,
      confidence: 0.8, // Would come from routing decision context
      alternativesCount: 1, // Would come from routing decision context
    };

    // Calculate reward
    const reward = this.calculateReward(event, agentProfile);

    // TODO: Implement comprehensive state representation for RL training
    // - Create detailed state representations capturing system status
    // - Include temporal state evolution and transition modeling
    // - Support partial observability and state estimation
    // - Add state abstraction and dimensionality reduction
    // - Implement state space exploration and coverage analysis
    // - Support multi-agent state coordination and representation
    // - Add state quality assessment and validation
    // - Implement state compression and efficient storage
    const nextState = { ...stateRepresentation };
    // Would update based on action outcome

    return {
      id: `sample_${event.id}_${Date.now()}`,
      agentId: event.agentId!,
      taskType: this.extractTaskType(event),
      state: stateRepresentation,
      action,
      reward,
      nextState,
      done: true, // Task completion marks episode end
      timestamp: event.timestamp,
      integrityHash: this.calculateSampleHash(
        event,
        stateRepresentation,
        action,
        reward
      ),
    };
  }

  /**
   * Creates state representation for RL training.
   */
  private createStateRepresentation(
    event: PerformanceEvent,
    state: PipelineState,
    agentProfile: AgentPerformanceProfile
  ): Record<string, unknown> {
    const stateRep: Record<string, unknown> = {
      taskId: event.taskId,
      taskType: this.extractTaskType(event),
      agentId: event.agentId,
      timestamp: event.timestamp,
    };

    if (this.config.stateRepresentation.includeHistoricalMetrics) {
      stateRep.historicalPerformance = {
        successRate: agentProfile.metrics.accuracy.successRate,
        averageLatency: agentProfile.metrics.latency.averageMs,
        trendDirection: agentProfile.trend.direction,
        sampleSize: agentProfile.sampleSize,
      };
    }

    if (this.config.stateRepresentation.includeAgentLoad) {
      // Would include current load metrics from agent profile
      stateRep.agentLoad = {
        utilizationPercent: 50, // Placeholder - would come from real load data
        queueDepth: 5, // Placeholder
      };
    }

    if (this.config.stateRepresentation.includeTaskContext) {
      stateRep.taskContext = {
        complexity: "medium", // Would be extracted from task metadata
        priority: "normal", // Would be extracted from task metadata
        estimatedDuration: 1000, // Would be extracted from task metadata
      };
    }

    return stateRep;
  }

  /**
   * Calculates reward for a performance event.
   */
  private calculateReward(
    event: PerformanceEvent,
    _agentProfile: AgentPerformanceProfile
  ): number {
    const { latencyWeight, accuracyWeight, costWeight, complianceWeight } =
      this.config.rewardFunction;

    // Extract metrics from event
    const eventLatency = event.metrics.latency?.averageMs || 0;
    const eventAccuracy = event.metrics.accuracy?.successRate || 0;
    const eventCost = event.metrics.cost?.costPerTask || 0;
    const eventCompliance = event.metrics.compliance?.validationPassRate || 0;

    // Normalize latency (lower is better, so invert)
    const latencyScore = Math.max(0, 1 - eventLatency / 10000); // Assume 10s max latency

    // Accuracy score (higher is better)
    const accuracyScore = eventAccuracy;

    // Cost score (lower is better, so invert)
    const costScore = Math.max(0, 1 - eventCost);

    // Compliance score (higher is better)
    const complianceScore = eventCompliance;

    // Calculate weighted reward
    let reward =
      latencyWeight * latencyScore +
      accuracyWeight * accuracyScore +
      costWeight * costScore +
      complianceWeight * complianceScore;

    // Apply temporal decay for delayed rewards
    const eventTime = new Date(event.timestamp).getTime();
    const now = Date.now();
    const ageMinutes = (now - eventTime) / (1000 * 60);
    const decayFactor = Math.pow(
      this.config.rewardFunction.temporalDecayFactor,
      ageMinutes
    );
    reward *= decayFactor;

    return reward;
  }

  /**
   * Creates a training batch from pending samples.
   */
  private async createTrainingBatch(
    state: PipelineState
  ): Promise<RLTrainingBatch> {
    const batchId = `batch_${state.agentId}_${Date.now()}`;
    const qualityScore = this.calculateBatchQuality(state.pendingBatch);

    return {
      id: batchId,
      agentId: state.agentId,
      samples: [...state.pendingBatch],
      createdAt: new Date().toISOString(),
      qualityScore,
      anonymizationLevel: "differential", // Configurable
    };
  }

  /**
   * Determines if a batch should be completed.
   */
  private shouldCompleteBatch(state: PipelineState): boolean {
    const batchSize = state.pendingBatch.length;
    const batchAge =
      (Date.now() - new Date(state.batchStartTime).getTime()) / (1000 * 60); // minutes

    return (
      batchSize >= this.config.batching.maxBatchSize ||
      batchAge >= this.config.batching.maxBatchAgeMinutes ||
      (batchSize >= this.config.batching.minBatchSize && batchAge >= 5) // Allow completion after 5 minutes with min size
    );
  }

  /**
   * Calculates quality score for a batch of samples.
   */
  private calculateBatchQuality(samples: RLTrainingSample[]): number {
    if (samples.length === 0) return 0;

    // Calculate reward variance (higher variance = more informative)
    const rewards = samples.map((s) => s.reward);
    const meanReward = rewards.reduce((sum, r) => sum + r, 0) / rewards.length;
    const variance =
      rewards.reduce((sum, r) => sum + Math.pow(r - meanReward, 2), 0) /
      rewards.length;
    const rewardVarianceScore = Math.min(
      1,
      variance / this.config.qualityThresholds.minRewardVariance
    );

    // Calculate sample diversity (unique states/actions)
    const uniqueStates = new Set(samples.map((s) => JSON.stringify(s.state)));
    const stateDiversityScore = uniqueStates.size / samples.length;

    // Calculate temporal distribution (should be relatively even)
    const timestamps = samples
      .map((s) => new Date(s.timestamp).getTime())
      .sort((a, b) => a - b);
    const timeSpan = timestamps[timestamps.length - 1] - timestamps[0];
    const avgInterval = timeSpan / (samples.length - 1);
    const temporalScore =
      avgInterval > 0 ? Math.min(1, (5 * 60 * 1000) / avgInterval) : 0; // Prefer ~5min intervals

    // Weighted quality score
    return (
      0.4 * rewardVarianceScore +
      0.4 * stateDiversityScore +
      0.2 * temporalScore
    );
  }

  /**
   * Extracts task type from event.
   */
  private extractTaskType(event: PerformanceEvent): string {
    if (event.context?.taskType) return event.context.taskType as string;
    if (event.taskId) {
      const match = event.taskId.match(/^([^-]+)-/);
      if (match) return match[1];
    }
    return "unknown";
  }

  /**
   * Calculates integrity hash for training sample.
   */
  private calculateSampleHash(
    event: PerformanceEvent,
    state: Record<string, unknown>,
    action: Record<string, unknown>,
    reward: number
  ): string {
    // Simplified hash calculation
    const data = JSON.stringify({ event: event.id, state, action, reward });
    return createHash("sha256").update(data).digest("hex");
  }

  /**
   * Creates initial pipeline state for an agent.
   */
  private createPipelineState(agentId: string): PipelineState {
    return {
      agentId,
      recentSamples: [],
      pendingBatch: [],
      batchStartTime: new Date().toISOString(),
      lastProcessedEvent: new Date().toISOString(),
      performanceHistory: [],
    };
  }

  /**
   * Checks data quality for an agent.
   */
  private checkDataQuality(agentId: string, state: PipelineState): string[] {
    const issues: string[] = [];

    // Check temporal gaps
    if (state.recentSamples.length > 1) {
      const timestamps = state.recentSamples
        .map((s) => new Date(s.timestamp).getTime())
        .sort((a, b) => a - b);
      const maxGap = Math.max(
        ...timestamps.slice(1).map((t, i) => t - timestamps[i])
      );
      const gapMinutes = maxGap / (1000 * 60);

      if (gapMinutes > this.config.qualityThresholds.maxTemporalGapMinutes) {
        issues.push(
          `Large temporal gap detected: ${gapMinutes.toFixed(
            1
          )} minutes for agent ${agentId}`
        );
      }
    }

    // Check sample diversity
    if (state.pendingBatch.length > 10) {
      const uniqueRewards = new Set(
        state.pendingBatch.map((s) => s.reward.toFixed(2))
      );
      const diversity = uniqueRewards.size / state.pendingBatch.length;

      if (diversity < this.config.qualityThresholds.minSampleDiversity) {
        issues.push(
          `Low sample diversity: ${diversity.toFixed(2)} for agent ${agentId}`
        );
      }
    }

    // Check for duplicates
    if (state.pendingBatch.length > 0) {
      const hashes = state.pendingBatch.map((s) => s.integrityHash);
      const uniqueHashes = new Set(hashes);
      const duplicateRatio = 1 - uniqueHashes.size / hashes.length;

      if (duplicateRatio > this.config.qualityThresholds.maxDuplicateRatio) {
        issues.push(
          `High duplicate ratio: ${(duplicateRatio * 100).toFixed(
            1
          )}% for agent ${agentId}`
        );
      }
    }

    return issues;
  }

  /**
   * Updates global quality metrics.
   */
  private updateQualityMetrics(): void {
    for (const [agentId, state] of Array.from(this.pipelineStates)) {
      const samples = [...state.recentSamples, ...state.pendingBatch];
      if (samples.length === 0) continue;

      const rewards = samples.map((s) => s.reward);
      const meanReward =
        rewards.reduce((sum, r) => sum + r, 0) / rewards.length;
      const rewardVariance =
        rewards.reduce((sum, r) => sum + Math.pow(r - meanReward, 2), 0) /
        rewards.length;

      const uniqueStates = new Set(samples.map((s) => JSON.stringify(s.state)));
      const diversity = uniqueStates.size / samples.length;

      const timestamps = samples
        .map((s) => new Date(s.timestamp).getTime())
        .sort((a, b) => a - b);
      let maxGap = 0;
      for (let i = 1; i < timestamps.length; i++) {
        maxGap = Math.max(maxGap, timestamps[i] - timestamps[i - 1]);
      }
      const temporalGapMinutes = maxGap / (1000 * 60);

      const hashes = samples.map((s) => s.integrityHash);
      const uniqueHashes = new Set(hashes);
      const duplicateRatio = 1 - uniqueHashes.size / hashes.length;

      this.qualityMetrics.set(agentId, {
        sampleDiversity: diversity,
        temporalGapMinutes,
        rewardVariance,
        duplicateRatio,
        sampleCount: samples.length,
        batchCount: this.completedBatches.filter((b) => b.agentId === agentId)
          .length,
      });
    }
  }

  /**
   * Starts periodic cleanup timer.
   */
  private startCleanupTimer(): void {
    this.cleanupTimer = setInterval(() => {
      this.performCleanup();
    }, this.config.retention.cleanupIntervalMinutes * 60 * 1000);
  }

  /**
   * Performs periodic cleanup of old data.
   */
  private performCleanup(): void {
    // Clean up old samples from pipeline states
    for (const [_agentId, state] of this.pipelineStates) {
      const maxAge = 24 * 60 * 60 * 1000; // 24 hours
      const cutoffTime = Date.now() - maxAge;

      state.recentSamples = state.recentSamples.filter(
        (sample) => new Date(sample.timestamp).getTime() > cutoffTime
      );

      // Clean up old performance history
      if (
        state.performanceHistory.length >
        this.config.stateRepresentation.temporalWindowSize
      ) {
        state.performanceHistory = state.performanceHistory.slice(
          -this.config.stateRepresentation.temporalWindowSize
        );
      }
    }

    // Clean up old completed batches
    const maxBatchAge = 7 * 24 * 60 * 60 * 1000; // 7 days
    const batchCutoffTime = Date.now() - maxBatchAge;
    this.completedBatches = this.completedBatches.filter(
      (batch) => new Date(batch.createdAt).getTime() > batchCutoffTime
    );

    // Enforce memory limits
    let totalSamples = 0;
    for (const state of Array.from(this.pipelineStates.values())) {
      totalSamples += state.recentSamples.length + state.pendingBatch.length;
    }

    if (totalSamples > this.config.retention.maxSamplesInMemory) {
      // Remove oldest samples across all agents
      this.trimSamplesToLimit();
    }

    if (
      this.completedBatches.length > this.config.retention.maxBatchesInMemory
    ) {
      // Keep only newest batches
      this.completedBatches.sort(
        (a, b) =>
          new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
      );
      this.completedBatches = this.completedBatches.slice(
        0,
        this.config.retention.maxBatchesInMemory
      );
    }

    this.emit("cleanup_completed");
  }

  /**
   * Trims samples to stay within memory limits.
   */
  private trimSamplesToLimit(): void {
    const allSamples: Array<{
      agentId: string;
      sample: RLTrainingSample;
      timestamp: number;
    }> = [];

    // Collect all samples with metadata
    for (const [agentId, state] of Array.from(this.pipelineStates)) {
      for (const sample of state.recentSamples) {
        allSamples.push({
          agentId,
          sample,
          timestamp: new Date(sample.timestamp).getTime(),
        });
      }
    }

    // Sort by timestamp (newest first)
    allSamples.sort((a, b) => b.timestamp - a.timestamp);

    // Keep only the newest samples up to the limit
    const samplesToKeep = allSamples.slice(
      0,
      this.config.retention.maxSamplesInMemory
    );

    // Update pipeline states
    for (const [agentId, state] of Array.from(this.pipelineStates)) {
      const agentSamples = samplesToKeep
        .filter((s) => s.agentId === agentId)
        .map((s) => s.sample);

      state.recentSamples = agentSamples;
    }
  }

  /**
   * Sets up event handlers for internal events.
   */
  private setupEventHandlers(): void {
    this.on("events_processed", (_stats) => {
      // Could trigger training system notifications
    });

    this.on("processing_error", (_error) => {
      // Could trigger alerting
    });
  }
}
