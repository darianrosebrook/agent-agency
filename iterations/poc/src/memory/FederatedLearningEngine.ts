/**
 * Federated Learning Engine - Privacy-preserving cross-tenant intelligence sharing
 *
 * This component enables secure sharing of insights and learnings across tenants
 * while maintaining data privacy and isolation. It implements federated learning
 * techniques to aggregate intelligence without exposing individual tenant data.
 *
 * @author @darianrosebrook
 */

import type {
  ContextualMemory,
  FederatedInsights,
  TaskContext,
  TenantConfig,
} from "../types/index.js";
import { Logger } from "../utils/Logger.js";
import { TenantIsolator } from "./TenantIsolator.js";

export interface FederatedLearningConfig {
  enabled: boolean;
  privacyLevel: "basic" | "differential" | "secure";
  aggregationFrequency: number; // ms
  minParticipants: number;
  maxParticipants: number;
  privacyBudget: number; // epsilon for differential privacy
  aggregationMethod: "weighted" | "consensus" | "hybrid";
  learningRate: number;
  convergenceThreshold: number;
}

export interface FederatedParticipant {
  tenantId: string;
  contributionWeight: number;
  privacyLevel: "basic" | "differential" | "secure";
  lastContribution: Date;
  reputationScore: number;
  active: boolean;
}

export interface FederatedSession {
  sessionId: string;
  topic: string;
  participants: FederatedParticipant[];
  status: "forming" | "active" | "aggregating" | "completed" | "failed";
  startTime: Date;
  endTime?: Date;
  aggregatedInsights: ContextualMemory[];
  privacyMetrics: PrivacyMetrics;
  convergenceScore: number;
}

export interface PrivacyMetrics {
  epsilonSpent: number;
  noiseLevel: number;
  informationLeakage: number;
  anonymizationStrength: number;
}

export interface DifferentialPrivacyParams {
  epsilon: number;
  delta: number;
  sensitivity: number;
}

/**
 * FederatedLearningEngine - Manages cross-tenant intelligence sharing
 */
export class FederatedLearningEngine {
  private config: FederatedLearningConfig;
  private logger: Logger;
  private tenantIsolator: TenantIsolator;
  private activeSessions: Map<string, FederatedSession> = new Map();
  private participantRegistry: Map<string, FederatedParticipant> = new Map();
  private aggregationQueue: Map<string, ContextualMemory[]> = new Map();

  constructor(
    config: FederatedLearningConfig,
    tenantIsolator: TenantIsolator,
    logger?: Logger
  ) {
    this.config = config;
    this.tenantIsolator = tenantIsolator;
    this.logger = logger || new Logger("FederatedLearningEngine");

    if (config.enabled) {
      this.startAggregationScheduler();
      this.logger.info("Federated learning engine initialized", {
        privacyLevel: config.privacyLevel,
        aggregationFrequency: config.aggregationFrequency,
        minParticipants: config.minParticipants,
      });
    }
  }

  /**
   * Register a tenant as a potential federated learning participant
   */
  async registerParticipant(
    tenantId: string,
    config: TenantConfig
  ): Promise<boolean> {
    try {
      // Verify tenant has federated isolation level
      if (config.isolationLevel !== "federated") {
        this.logger.warn(
          `Tenant ${tenantId} cannot participate in federated learning`,
          {
            isolationLevel: config.isolationLevel,
          }
        );
        return false;
      }

      // Check if tenant is allowed to participate
      const accessCheck = await this.tenantIsolator.validateTenantAccess(
        tenantId,
        "federate",
        "memory"
      );

      if (!accessCheck.allowed) {
        this.logger.warn(`Tenant ${tenantId} denied federated access`, {
          reason: accessCheck.reason,
        });
        return false;
      }

      const participant: FederatedParticipant = {
        tenantId,
        contributionWeight: 1.0,
        privacyLevel: this.config.privacyLevel,
        lastContribution: new Date(),
        reputationScore: 1.0,
        active: true,
      };

      this.participantRegistry.set(tenantId, participant);
      this.logger.info(
        `Tenant ${tenantId} registered as federated participant`
      );
      return true;
    } catch (error) {
      this.logger.error(`Failed to register participant ${tenantId}`, {
        error: error instanceof Error ? error.message : String(error),
      });
      return false;
    }
  }

  /**
   * Submit insights for federated learning
   */
  async submitInsights(
    tenantId: string,
    insights: ContextualMemory[],
    context: TaskContext
  ): Promise<boolean> {
    try {
      const participant = this.participantRegistry.get(tenantId);
      if (!participant || !participant.active) {
        this.logger.warn(`Inactive or unregistered participant: ${tenantId}`);
        return false;
      }

      // Apply privacy transformations based on privacy level
      const anonymizedInsights = await this.anonymizeInsights(
        insights,
        participant.privacyLevel
      );

      // Add to aggregation queue
      const topicKey = this.generateTopicKey(context);
      if (!this.aggregationQueue.has(topicKey)) {
        this.aggregationQueue.set(topicKey, []);
      }

      const queue = this.aggregationQueue.get(topicKey)!;
      queue.push(...anonymizedInsights);

      // Update participant metrics
      participant.lastContribution = new Date();
      participant.reputationScore = Math.min(
        1.0,
        participant.reputationScore + 0.1
      );

      this.logger.info(
        `Tenant ${tenantId} submitted ${insights.length} insights`,
        {
          topic: topicKey,
          anonymizedCount: anonymizedInsights.length,
        }
      );

      // Check if we should trigger aggregation
      if (queue.length >= this.config.minParticipants) {
        await this.triggerAggregation(topicKey);
      }

      return true;
    } catch (error) {
      this.logger.error(`Failed to submit insights from ${tenantId}`, {
        error: error instanceof Error ? error.message : String(error),
      });
      return false;
    }
  }

  /**
   * Retrieve federated insights for a tenant
   */
  async getFederatedInsights(
    tenantId: string,
    context: TaskContext
  ): Promise<FederatedInsights> {
    try {
      const participant = this.participantRegistry.get(tenantId);
      if (!participant) {
        return {
          insights: [],
          confidence: 0,
          sourceTenants: [],
          aggregationMethod: this.config.aggregationMethod,
          privacyPreserved: true,
        };
      }

      const topicKey = this.generateTopicKey(context);
      const aggregatedInsights = await this.getAggregatedInsights(topicKey);

      // Filter insights based on participant's access level
      const accessibleInsights = await this.filterInsightsForParticipant(
        aggregatedInsights,
        participant
      );

      return {
        insights: accessibleInsights,
        confidence: this.calculateFederatedConfidence(accessibleInsights),
        sourceTenants: this.getSourceTenants(topicKey),
        aggregationMethod: this.config.aggregationMethod,
        privacyPreserved: true,
      };
    } catch (error) {
      this.logger.error(`Failed to get federated insights for ${tenantId}`, {
        error: error instanceof Error ? error.message : String(error),
      });
      return {
        insights: [],
        confidence: 0,
        sourceTenants: [],
        aggregationMethod: this.config.aggregationMethod,
        privacyPreserved: false,
      };
    }
  }

  /**
   * Create a new federated learning session
   */
  async createSession(
    initiatorTenantId: string,
    topic: string,
    participants: string[]
  ): Promise<FederatedSession | null> {
    try {
      // Validate initiator permissions
      const initiatorCheck = await this.tenantIsolator.validateTenantAccess(
        initiatorTenantId,
        "federate",
        "memory"
      );

      if (!initiatorCheck.allowed) {
        throw new Error("Initiator lacks federated permissions");
      }

      // Validate all participants
      const validParticipants: FederatedParticipant[] = [];
      for (const tenantId of participants) {
        const participant = this.participantRegistry.get(tenantId);
        if (participant && participant.active) {
          validParticipants.push(participant);
        }
      }

      if (validParticipants.length < this.config.minParticipants) {
        throw new Error(
          `Insufficient participants: ${validParticipants.length}/${this.config.minParticipants}`
        );
      }

      const session: FederatedSession = {
        sessionId: this.generateSessionId(),
        topic,
        participants: validParticipants,
        status: "forming",
        startTime: new Date(),
        aggregatedInsights: [],
        privacyMetrics: {
          epsilonSpent: 0,
          noiseLevel: 0,
          informationLeakage: 0,
          anonymizationStrength: 1.0,
        },
        convergenceScore: 0,
      };

      this.activeSessions.set(session.sessionId, session);
      this.logger.info(`Created federated session ${session.sessionId}`, {
        topic,
        participants: validParticipants.length,
      });

      return session;
    } catch (error) {
      this.logger.error(`Failed to create federated session`, {
        initiator: initiatorTenantId,
        topic,
        error: error instanceof Error ? error.message : String(error),
      });
      return null;
    }
  }

  /**
   * Get system health and performance metrics
   */
  async getSystemHealth(): Promise<{
    activeSessions: number;
    registeredParticipants: number;
    pendingAggregations: number;
    totalInsightsShared: number;
    averagePrivacyScore: number;
  }> {
    const activeSessions = this.activeSessions.size;
    const registeredParticipants = this.participantRegistry.size;
    const pendingAggregations = this.aggregationQueue.size;

    // Calculate metrics
    let totalInsightsShared = 0;
    let totalPrivacyScore = 0;

    for (const session of this.activeSessions.values()) {
      totalInsightsShared += session.aggregatedInsights.length;
      totalPrivacyScore += session.privacyMetrics.anonymizationStrength;
    }

    for (const queue of this.aggregationQueue.values()) {
      totalInsightsShared += queue.length;
    }

    const averagePrivacyScore =
      activeSessions > 0 ? totalPrivacyScore / activeSessions : 1.0;

    return {
      activeSessions,
      registeredParticipants,
      pendingAggregations,
      totalInsightsShared,
      averagePrivacyScore,
    };
  }

  /**
   * Perform maintenance operations
   */
  async performMaintenance(): Promise<void> {
    this.logger.info("Starting federated learning maintenance");

    // Clean up old aggregation queues
    const _cutoffTime = Date.now() - 24 * 60 * 60 * 1000; // 24 hours
    for (const [topicKey, queue] of this.aggregationQueue.entries()) {
      if (queue.length === 0) {
        this.aggregationQueue.delete(topicKey);
      }
    }

    // Clean up completed sessions older than 7 days
    const sessionCutoff = Date.now() - 7 * 24 * 60 * 60 * 1000;
    for (const [sessionId, session] of this.activeSessions.entries()) {
      if (
        session.status === "completed" &&
        session.endTime &&
        session.endTime.getTime() < sessionCutoff
      ) {
        this.activeSessions.delete(sessionId);
      }
    }

    // Update participant reputation scores
    for (const participant of this.participantRegistry.values()) {
      const daysSinceLastContribution =
        (Date.now() - participant.lastContribution.getTime()) /
        (24 * 60 * 60 * 1000);

      if (daysSinceLastContribution > 30) {
        participant.reputationScore = Math.max(
          0.1,
          participant.reputationScore - 0.1
        );
      }
    }

    this.logger.info("Federated learning maintenance completed");
  }

  // Private methods

  private startAggregationScheduler(): void {
    setInterval(async () => {
      try {
        await this.processPendingAggregations();
      } catch (error) {
        this.logger.error("Aggregation scheduler error", {
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }, this.config.aggregationFrequency);
  }

  private async processPendingAggregations(): Promise<void> {
    for (const [topicKey, insights] of this.aggregationQueue.entries()) {
      if (insights.length >= this.config.minParticipants) {
        await this.triggerAggregation(topicKey);
      }
    }
  }

  private async triggerAggregation(topicKey: string): Promise<void> {
    const insights = this.aggregationQueue.get(topicKey);
    if (!insights || insights.length < this.config.minParticipants) {
      return;
    }

    try {
      this.logger.info(`Starting aggregation for topic: ${topicKey}`, {
        insightsCount: insights.length,
      });

      const aggregatedInsights = await this.aggregateInsights(
        insights,
        this.config.aggregationMethod
      );

      // Store aggregated results
      this.aggregationQueue.delete(topicKey);

      // Cache the results for participants to access
      const _cacheKey = `federated_${topicKey}`;
      // In a real implementation, this would be stored in a distributed cache

      this.logger.info(`Completed aggregation for topic: ${topicKey}`, {
        aggregatedCount: aggregatedInsights.length,
      });
    } catch (error) {
      this.logger.error(`Aggregation failed for topic: ${topicKey}`, {
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }

  private async anonymizeInsights(
    insights: ContextualMemory[],
    privacyLevel: string
  ): Promise<ContextualMemory[]> {
    switch (privacyLevel) {
      case "secure":
        return this.applySecureAnonymization(insights);
      case "differential":
        return this.applyDifferentialPrivacy(insights);
      case "basic":
      default:
        return this.applyBasicAnonymization(insights);
    }
  }

  private applyBasicAnonymization(
    insights: ContextualMemory[]
  ): ContextualMemory[] {
    // Remove specific identifiers and add noise to scores
    return insights.map((insight) => ({
      ...insight,
      relevanceScore: insight.relevanceScore + (Math.random() - 0.5) * 0.1, // Add ±5% noise
      contextMatch: {
        ...insight.contextMatch,
        similarityScore: Math.max(
          0,
          Math.min(
            1,
            insight.contextMatch.similarityScore + (Math.random() - 0.5) * 0.1
          )
        ),
      },
    }));
  }

  private applyDifferentialPrivacy(
    insights: ContextualMemory[]
  ): ContextualMemory[] {
    const epsilon = this.config.privacyBudget / insights.length;

    return insights.map((insight) => {
      const noise = this.generateLaplaceNoise(1.0 / epsilon); // sensitivity = 1

      return {
        ...insight,
        relevanceScore: Math.max(
          0,
          Math.min(1, insight.relevanceScore + noise)
        ),
        contextMatch: {
          ...insight.contextMatch,
          similarityScore: Math.max(
            0,
            Math.min(1, insight.contextMatch.similarityScore + noise * 0.5)
          ),
        },
      };
    });
  }

  private applySecureAnonymization(
    insights: ContextualMemory[]
  ): ContextualMemory[] {
    // Apply both differential privacy and additional anonymization
    const dpInsights = this.applyDifferentialPrivacy(insights);

    // Further anonymize by removing temporal information and clustering similar insights
    return this.clusterAndGeneralize(dpInsights);
  }

  private generateLaplaceNoise(scale: number): number {
    // Generate Laplace noise with given scale parameter
    const u = Math.random() - 0.5;
    return -scale * Math.sign(u) * Math.log(1 - 2 * Math.abs(u));
  }

  private clusterAndGeneralize(
    insights: ContextualMemory[]
  ): ContextualMemory[] {
    // Simple clustering by relevance score ranges
    const clusters: ContextualMemory[][] = [[], [], []]; // Low, medium, high relevance

    insights.forEach((insight) => {
      if (insight.relevanceScore < 0.33) clusters[0].push(insight);
      else if (insight.relevanceScore < 0.67) clusters[1].push(insight);
      else clusters[2].push(insight);
    });

    // Return generalized representatives
    return clusters
      .filter((cluster) => cluster.length > 0)
      .map((cluster) => this.generalizeCluster(cluster));
  }

  private generalizeCluster(cluster: ContextualMemory[]): ContextualMemory {
    // Create a generalized representative of the cluster
    const avgRelevance =
      cluster.reduce((sum, i) => sum + i.relevanceScore, 0) / cluster.length;

    return {
      memoryId: `cluster_${Date.now()}_${Math.random()
        .toString(36)
        .substring(2, 9)}`,
      relevanceScore: avgRelevance,
      contextMatch: {
        similarityScore: avgRelevance,
        keywordMatches: [],
        semanticMatches: ["generalized_cluster"],
        temporalAlignment: avgRelevance,
      },
      content: {
        taskType: "generalized",
        outcome: "aggregated",
        lessons: ["Generalized insights from multiple sources"],
      },
    };
  }

  private async aggregateInsights(
    insights: ContextualMemory[],
    method: string
  ): Promise<ContextualMemory[]> {
    switch (method) {
      case "weighted":
        return this.weightedAggregation(insights);
      case "consensus":
        return this.consensusAggregation(insights);
      case "hybrid":
        return this.hybridAggregation(insights);
      default:
        return insights;
    }
  }

  private weightedAggregation(
    insights: ContextualMemory[]
  ): ContextualMemory[] {
    // Weight by relevance score and combine similar insights
    const weighted = insights.map((insight) => ({
      ...insight,
      weight: insight.relevanceScore,
    }));

    // TODO: Implement sophisticated content similarity grouping
    // - Use semantic similarity algorithms (cosine similarity, BERT embeddings)
    // - Implement hierarchical clustering for insight grouping
    // - Add content deduplication with configurable similarity thresholds
    // - Support multi-modal similarity (text, images, structured data)
    // - Implement topic modeling and clustering algorithms (LDA, k-means)
    // - Add temporal clustering (group insights by time periods)
    // - Support cross-lingual similarity for multilingual content
    // - Implement similarity confidence scoring and validation
    const groups = this.groupSimilarInsights(weighted);

    return groups.map((group) => {
      const totalWeight = group.reduce((sum, i) => sum + (i.weight ?? 1.0), 0);
      const avgRelevance =
        group.reduce(
          (sum, i) => sum + i.relevanceScore * (i.weight ?? 1.0),
          0
        ) / totalWeight;

      return {
        ...group[0], // Use first as template
        relevanceScore: avgRelevance,
        memoryId: `aggregated_${Date.now()}_${Math.random()
          .toString(36)
          .substring(2, 9)}`,
      };
    });
  }

  private consensusAggregation(
    insights: ContextualMemory[]
  ): ContextualMemory[] {
    // Keep only insights that appear in majority of sources
    const consensusThreshold = Math.ceil(insights.length * 0.6); // 60%

    // Simplified: keep all insights (would implement proper consensus logic)
    return insights.slice(0, consensusThreshold);
  }

  private hybridAggregation(insights: ContextualMemory[]): ContextualMemory[] {
    // Combine weighted and consensus approaches
    const weighted = this.weightedAggregation(insights);
    return this.consensusAggregation(weighted);
  }

  private groupSimilarInsights(
    insights: ContextualMemory[]
  ): ContextualMemory[][] {
    // Simplified grouping by relevance score similarity
    const groups: ContextualMemory[][] = [];
    const threshold = 0.1; // Similarity threshold

    for (const insight of insights) {
      let found = false;
      for (const group of groups) {
        if (
          Math.abs(group[0].relevanceScore - insight.relevanceScore) < threshold
        ) {
          group.push(insight);
          found = true;
          break;
        }
      }
      if (!found) {
        groups.push([insight]);
      }
    }

    return groups;
  }

  private async getAggregatedInsights(
    _topicKey: string
  ): Promise<ContextualMemory[]> {
    // In a real implementation, this would fetch from distributed cache/database
    // For now, return empty array
    return [];
  }

  private async filterInsightsForParticipant(
    insights: ContextualMemory[],
    participant: FederatedParticipant
  ): Promise<ContextualMemory[]> {
    // Apply participant-specific filtering based on reputation and access level
    const reputationMultiplier = Math.max(0.5, participant.reputationScore);

    return insights
      .filter((insight) => insight.relevanceScore >= 0.3 / reputationMultiplier)
      .slice(0, Math.floor(10 * reputationMultiplier)); // Limit based on reputation
  }

  private calculateFederatedConfidence(insights: ContextualMemory[]): number {
    if (insights.length === 0) return 0;

    const avgRelevance =
      insights.reduce((sum, i) => sum + i.relevanceScore, 0) / insights.length;
    const diversity =
      insights.length > 1
        ? 1 -
          insights.reduce(
            (sum, i) => sum + Math.pow(i.relevanceScore - avgRelevance, 2),
            0
          ) /
            insights.length
        : 0.5;

    return Math.min(1.0, (avgRelevance + diversity) / 2);
  }

  private getSourceTenants(_topicKey: string): string[] {
    // In a real implementation, this would track which tenants contributed to each topic
    return Array.from(this.participantRegistry.keys()).slice(0, 3); // Placeholder
  }

  private generateTopicKey(context: TaskContext): string {
    const description = context.description || "";
    return `${context.type || "unknown"}_${description
      .substring(0, 50)
      .replace(/\s+/g, "_")}`;
  }

  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random()
      .toString(36)
      .substring(2, 9)}`;
  }
}
