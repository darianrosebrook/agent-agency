import { v4 as uuidv4 } from "uuid";
import { ConfigManager } from "../config/ConfigManager";
import { Logger } from "../observability/Logger";
import {
  FeedbackAnalysis,
  FeedbackAnalysisConfig,
  FeedbackEvent,
  FeedbackInsight,
  FeedbackRecommendation,
  FeedbackSource,
} from "../types/feedback-loop";

export class FeedbackAnalyzer {
  private config: FeedbackAnalysisConfig;
  private logger: Logger;

  // Analysis data storage (in production, this would be a database)
  private feedbackHistory: Map<string, FeedbackEvent[]> = new Map(); // entityId -> events
  private analysisCache: Map<string, FeedbackAnalysis> = new Map(); // entityId -> latest analysis

  // Statistical accumulators
  private timeSeriesData: Map<string, { timestamp: number; value: number }[]> =
    new Map();

  constructor(configManager: ConfigManager) {
    this.config = configManager.get("feedbackLoop.analysis");
    this.logger = new Logger("FeedbackAnalyzer");
  }

  public analyzeEntityFeedback(
    entityId: string,
    entityType: string,
    timeWindowHours: number = this.config.trendWindowHours
  ): FeedbackAnalysis {
    const now = new Date();
    const windowStart = new Date(
      now.getTime() - timeWindowHours * 60 * 60 * 1000
    );

    // Get feedback events for this entity
    const events = this.getEntityFeedback(entityId, windowStart, now);

    if (events.length < this.config.minDataPoints) {
      this.logger.debug(
        `Insufficient data for ${entityType} ${entityId}: ${events.length} events`
      );
      return this.createEmptyAnalysis(entityId, entityType, windowStart, now);
    }

    // Perform analysis
    const metrics = this.calculateMetrics(events);
    const insights = this.generateInsights(events, metrics);
    const recommendations = this.generateRecommendations(
      entityId,
      entityType,
      insights,
      metrics
    );

    const analysis: FeedbackAnalysis = {
      id: uuidv4(),
      entityId,
      entityType,
      timeWindow: {
        start: windowStart.toISOString(),
        end: now.toISOString(),
      },
      metrics,
      insights,
      recommendations,
      confidence: this.calculateConfidence(events, insights),
      generatedAt: now.toISOString(),
    };

    // Cache analysis
    this.analysisCache.set(entityId, analysis);

    this.logger.debug(`Completed analysis for ${entityType} ${entityId}`, {
      eventsAnalyzed: events.length,
      insightsFound: insights.length,
      recommendations: recommendations.length,
      confidence: analysis.confidence,
    });

    return analysis;
  }

  public analyzeAllEntities(entityType?: string): FeedbackAnalysis[] {
    const analyses: FeedbackAnalysis[] = [];
    const entities = entityType
      ? Array.from(this.feedbackHistory.keys()).filter((id) => {
          const events = this.feedbackHistory.get(id) || [];
          return events.length > 0 && events[0].entityType === entityType;
        })
      : Array.from(this.feedbackHistory.keys());

    for (const entityId of entities) {
      try {
        const events = this.feedbackHistory.get(entityId) || [];
        const actualEntityType =
          events.length > 0
            ? events[0].entityType
            : this.getEntityType(entityId);
        const analysis = this.analyzeEntityFeedback(entityId, actualEntityType);
        analyses.push(analysis);
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        this.logger.error(`Failed to analyze ${entityId}`, {
          error: errorMessage,
        });
      }
    }

    return analyses;
  }

  public detectAnomalies(
    entityId: string,
    recentEvents: FeedbackEvent[]
  ): FeedbackInsight[] {
    const anomalies: FeedbackInsight[] = [];
    const baselineEvents = this.getEntityFeedback(
      entityId,
      new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
      new Date()
    ); // Last 7 days

    if (baselineEvents.length < this.config.minDataPoints) {
      return anomalies;
    }

    // Calculate baseline statistics
    const baselineStats = this.calculateBaselineStats(baselineEvents);

    // Check recent events against baseline
    for (const event of recentEvents) {
      const anomaly = this.checkForAnomaly(event, baselineStats);
      if (anomaly) {
        anomalies.push(anomaly);
      }
    }

    return anomalies;
  }

  public findCorrelations(entityId: string): FeedbackInsight[] {
    const events = this.feedbackHistory.get(entityId) || [];
    if (events.length < this.config.minDataPoints * 2) {
      return [];
    }

    const correlations: FeedbackInsight[] = [];

    // Analyze correlation between different metrics
    const metricCorrelations = this.analyzeMetricCorrelations(events);
    for (const correlation of metricCorrelations) {
      if (
        Math.abs(correlation.coefficient) > this.config.correlationThreshold
      ) {
        correlations.push({
          type: "correlation",
          description: `Strong ${
            correlation.coefficient > 0 ? "positive" : "negative"
          } correlation between ${correlation.metric1} and ${
            correlation.metric2
          }`,
          severity: Math.abs(correlation.coefficient) > 0.8 ? "high" : "medium",
          evidence: {
            metric: `${correlation.metric1}_vs_${correlation.metric2}`,
            value: correlation.coefficient,
            baseline: 0,
            changePercent: Math.abs(correlation.coefficient) * 100,
          },
          impact: {
            affectedEntities: [entityId],
            estimatedImpact:
              correlation.coefficient > 0 ? "positive" : "negative",
            confidence: Math.abs(correlation.coefficient),
          },
        });
      }
    }

    return correlations;
  }

  public predictTrends(
    entityId: string,
    hoursAhead: number = this.config.predictionHorizonHours
  ): FeedbackInsight[] {
    const predictions: FeedbackInsight[] = [];
    const events = this.feedbackHistory.get(entityId) || [];

    if (events.length < this.config.minDataPoints * 3) {
      return predictions;
    }

    // Simple linear regression for trend prediction
    const timeSeries = this.buildTimeSeries(events);
    const trend = this.calculateLinearTrend(timeSeries);

    if (Math.abs(trend.slope) > 0.01) {
      // Significant trend
      const futureValue =
        trend.intercept + trend.slope * (timeSeries.length + hoursAhead);
      const direction = trend.slope > 0 ? "increasing" : "decreasing";

      predictions.push({
        type: "prediction",
        description: `Performance trend ${direction} over next ${hoursAhead} hours`,
        severity: Math.abs(trend.slope) > 0.05 ? "high" : "medium",
        evidence: {
          metric: "predicted_value",
          value: futureValue,
          baseline: timeSeries[timeSeries.length - 1]?.value || 0,
          changePercent: trend.slope * 100,
        },
        impact: {
          affectedEntities: [entityId],
          estimatedImpact: trend.slope > 0 ? "positive" : "negative",
          confidence: trend.rSquared,
        },
      });
    }

    return predictions;
  }

  // Event handling
  public addFeedbackEvent(event: FeedbackEvent): void {
    const entityEvents = this.feedbackHistory.get(event.entityId) || [];
    entityEvents.push(event);

    // Keep only recent events based on retention policy
    const retentionMs = this.config.trendWindowHours * 60 * 60 * 1000;
    const cutoffTime = Date.now() - retentionMs;
    const filteredEvents = entityEvents.filter(
      (e) => new Date(e.timestamp).getTime() > cutoffTime
    );

    this.feedbackHistory.set(event.entityId, filteredEvents);

    // Update time series data
    this.updateTimeSeries(event);
  }

  public getEntityFeedback(
    entityId: string,
    startTime: Date,
    endTime: Date
  ): FeedbackEvent[] {
    const events = this.feedbackHistory.get(entityId) || [];
    return events.filter((event) => {
      const eventTime = new Date(event.timestamp);
      return eventTime >= startTime && eventTime <= endTime;
    });
  }

  private createEmptyAnalysis(
    entityId: string,
    entityType: string,
    startTime: Date,
    endTime: Date
  ): FeedbackAnalysis {
    return {
      id: uuidv4(),
      entityId,
      entityType,
      timeWindow: {
        start: startTime.toISOString(),
        end: endTime.toISOString(),
      },
      metrics: {
        totalFeedbackEvents: 0,
        performanceTrend: "stable",
        anomalyCount: 0,
        correlationStrength: 0,
      },
      insights: [],
      recommendations: [],
      confidence: 0,
      generatedAt: new Date().toISOString(),
    };
  }

  private calculateMetrics(
    events: FeedbackEvent[]
  ): FeedbackAnalysis["metrics"] {
    const totalEvents = events.length;
    let averageRating = undefined;
    let performanceTrend: "improving" | "stable" | "declining" = "stable";
    const anomalyCount = 0; // Would be calculated by anomaly detection
    const correlationStrength = 0; // Would be calculated by correlation analysis

    // Calculate average rating from user feedback
    const ratings = events
      .filter((e) => e.source === FeedbackSource.USER_RATINGS)
      .map((e) => (e.value as any).rating)
      .filter((r) => typeof r === "number");

    if (ratings.length > 0) {
      averageRating = ratings.reduce((sum, r) => sum + r, 0) / ratings.length;
    }

    // Simple trend analysis
    const timeSeries = this.buildTimeSeries(events);
    if (timeSeries.length >= 3) {
      const recent = timeSeries.slice(-3);
      const older = timeSeries.slice(0, -3);
      if (older.length > 0) {
        const recentAvg =
          recent.reduce((sum, p) => sum + p.value, 0) / recent.length;
        const olderAvg =
          older.reduce((sum, p) => sum + p.value, 0) / older.length;
        const change = (recentAvg - olderAvg) / olderAvg;

        if (change > 0.05) performanceTrend = "improving";
        else if (change < -0.05) performanceTrend = "declining";
      }
    }

    return {
      totalFeedbackEvents: totalEvents,
      averageRating,
      performanceTrend,
      anomalyCount,
      correlationStrength,
    };
  }

  private generateInsights(
    events: FeedbackEvent[],
    metrics: FeedbackAnalysis["metrics"]
  ): FeedbackInsight[] {
    const insights: FeedbackInsight[] = [];

    // Trend insights
    if (metrics.performanceTrend !== "stable") {
      insights.push({
        type: "trend",
        description: `Performance is ${metrics.performanceTrend}`,
        severity: metrics.performanceTrend === "declining" ? "high" : "medium",
        evidence: {
          metric: "performance_trend",
          value: metrics.performanceTrend,
        },
        impact: {
          affectedEntities: [],
          estimatedImpact:
            metrics.performanceTrend === "improving" ? "positive" : "negative",
          confidence: 0.8,
        },
      });
    }

    // Rating insights
    if (metrics.averageRating !== undefined) {
      if (metrics.averageRating < 3.0) {
        insights.push({
          type: "trend",
          description: "Low user satisfaction ratings detected",
          severity: "high",
          evidence: {
            metric: "average_rating",
            value: metrics.averageRating,
            baseline: 4.0,
          },
          impact: {
            affectedEntities: [],
            estimatedImpact: "negative",
            confidence: 0.9,
          },
        });
      }
    }

    // Error rate insights
    const errorEvents = events.filter(
      (e) =>
        e.source === FeedbackSource.TASK_OUTCOMES &&
        !(e.value as any).outcome?.success
    );

    if (errorEvents.length > events.length * 0.1) {
      // >10% error rate
      insights.push({
        type: "anomaly",
        description: "High error rate detected",
        severity: "high",
        evidence: {
          metric: "error_rate",
          value: errorEvents.length / events.length,
          baseline: 0.1,
        },
        impact: {
          affectedEntities: [],
          estimatedImpact: "negative",
          confidence: 0.85,
        },
      });
    }

    return insights;
  }

  private generateRecommendations(
    entityId: string,
    entityType: string,
    insights: FeedbackInsight[],
    metrics: FeedbackAnalysis["metrics"]
  ): FeedbackRecommendation[] {
    const recommendations: FeedbackRecommendation[] = [];

    for (const insight of insights) {
      switch (insight.type) {
        case "trend":
          if (insight.description.includes("declining")) {
            recommendations.push({
              id: uuidv4(),
              type: "agent_update",
              priority: "high",
              description: `Performance declining for ${entityType} ${entityId}`,
              action: {
                targetEntity: entityId,
                operation: "update_performance_profile",
                parameters: { investigation_required: true },
              },
              expectedImpact: {
                metric: "performance_score",
                improvementPercent: 15,
                timeToEffect: "1-2 days",
              },
              riskAssessment: {
                riskLevel: "medium",
                rollbackPlan: "Revert to previous performance weights",
                monitoringRequired: true,
              },
              implementationStatus: "pending",
            });
          }
          break;

        case "anomaly":
          if (insight.description.includes("error rate")) {
            recommendations.push({
              id: uuidv4(),
              type: "routing_adjustment",
              priority: "high",
              description: `High error rate for ${entityType} ${entityId} - reduce load`,
              action: {
                targetEntity: entityId,
                operation: "reduce_routing_weight",
                parameters: { reduction_factor: 0.5 },
              },
              expectedImpact: {
                metric: "error_rate",
                improvementPercent: -50, // Negative = improvement
                timeToEffect: "immediate",
              },
              riskAssessment: {
                riskLevel: "low",
                rollbackPlan: "Restore original routing weights",
                monitoringRequired: true,
              },
              implementationStatus: "pending",
            });
          }
          break;
      }
    }

    return recommendations;
  }

  private calculateConfidence(
    events: FeedbackEvent[],
    insights: FeedbackInsight[]
  ): number {
    const baseConfidence = Math.min(
      events.length / this.config.minDataPoints,
      1.0
    );
    const insightConfidence = insights.length > 0 ? 0.8 : 0.6;
    return Math.min(baseConfidence * insightConfidence, 1.0);
  }

  private getEntityType(entityId: string): string {
    // In a real system, this would query the registry
    // For now, make a best guess based on ID patterns
    if (entityId.startsWith("agent-")) return "agent";
    if (entityId.startsWith("task-")) return "task";
    if (entityId.startsWith("component-")) return "component";
    return "unknown";
  }

  private calculateBaselineStats(events: FeedbackEvent[]): any {
    // Calculate mean and standard deviation for anomaly detection
    const values = events
      .map((e) => this.extractNumericValue(e))
      .filter((v) => v !== null) as number[];
    if (values.length === 0) return { mean: 0, stdDev: 0 };

    const mean = values.reduce((sum, v) => sum + v, 0) / values.length;
    const variance =
      values.reduce((sum, v) => sum + Math.pow(v - mean, 2), 0) / values.length;
    const stdDev = Math.sqrt(variance);

    return { mean, stdDev };
  }

  private checkForAnomaly(
    event: FeedbackEvent,
    baseline: { mean: number; stdDev: number }
  ): FeedbackInsight | null {
    const value = this.extractNumericValue(event);
    if (value === null) return null;

    const zScore = Math.abs((value - baseline.mean) / baseline.stdDev);
    if (zScore > this.config.anomalyThreshold) {
      return {
        type: "anomaly",
        description: `Anomalous ${event.source} value detected`,
        severity: zScore > 3 ? "high" : "medium",
        evidence: {
          metric: event.source,
          value,
          baseline: baseline.mean,
        },
        impact: {
          affectedEntities: [event.entityId],
          estimatedImpact: "negative",
          confidence: Math.min(zScore / 5, 1.0),
        },
      };
    }

    return null;
  }

  private analyzeMetricCorrelations(
    events: FeedbackEvent[]
  ): Array<{ metric1: string; metric2: string; coefficient: number }> {
    // Simplified correlation analysis
    const correlations: Array<{
      metric1: string;
      metric2: string;
      coefficient: number;
    }> = [];

    // Example: correlate performance metrics with user ratings
    const perfEvents = events.filter(
      (e) => e.source === FeedbackSource.PERFORMANCE_METRICS
    );
    const ratingEvents = events.filter(
      (e) => e.source === FeedbackSource.USER_RATINGS
    );

    if (perfEvents.length >= 5 && ratingEvents.length >= 5) {
      const perfValues = perfEvents
        .map((e) => this.extractNumericValue(e))
        .filter((v) => v !== null) as number[];
      const ratingValues = ratingEvents
        .map((e) => (e.value as any).rating)
        .filter((v) => typeof v === "number");

      const minLength = Math.min(perfValues.length, ratingValues.length);
      if (minLength >= 5) {
        const coefficient = this.calculatePearsonCorrelation(
          perfValues.slice(0, minLength),
          ratingValues.slice(0, minLength)
        );

        correlations.push({
          metric1: "performance_score",
          metric2: "user_rating",
          coefficient,
        });
      }
    }

    return correlations;
  }

  private calculatePearsonCorrelation(x: number[], y: number[]): number {
    const n = x.length;
    const sumX = x.reduce((sum, val) => sum + val, 0);
    const sumY = y.reduce((sum, val) => sum + val, 0);
    const sumXY = x.reduce((sum, val, i) => sum + val * y[i], 0);
    const sumX2 = x.reduce((sum, val) => sum + val * val, 0);
    const sumY2 = y.reduce((sum, val) => sum + val * val, 0);

    const numerator = n * sumXY - sumX * sumY;
    const denominator = Math.sqrt(
      (n * sumX2 - sumX * sumX) * (n * sumY2 - sumY * sumY)
    );

    return denominator === 0 ? 0 : numerator / denominator;
  }

  private buildTimeSeries(
    events: FeedbackEvent[]
  ): Array<{ timestamp: number; value: number }> {
    return events
      .map((event) => ({
        timestamp: new Date(event.timestamp).getTime(),
        value: this.extractNumericValue(event) || 0,
      }))
      .sort((a, b) => a.timestamp - b.timestamp);
  }

  private calculateLinearTrend(
    data: Array<{ timestamp: number; value: number }>
  ): {
    slope: number;
    intercept: number;
    rSquared: number;
  } {
    if (data.length < 2) return { slope: 0, intercept: 0, rSquared: 0 };

    const n = data.length;
    const sumX = data.reduce((sum, p) => sum + p.timestamp, 0);
    const sumY = data.reduce((sum, p) => sum + p.value, 0);
    const sumXY = data.reduce((sum, p) => sum + p.timestamp * p.value, 0);
    const sumX2 = data.reduce((sum, p) => sum + p.timestamp * p.timestamp, 0);

    const slope = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
    const intercept = (sumY - slope * sumX) / n;

    // Calculate R-squared
    const yMean = sumY / n;
    const totalSumSquares = data.reduce(
      (sum, p) => sum + Math.pow(p.value - yMean, 2),
      0
    );
    const residualSumSquares = data.reduce((sum, p) => {
      const predicted = slope * p.timestamp + intercept;
      return sum + Math.pow(p.value - predicted, 2);
    }, 0);
    const rSquared =
      totalSumSquares === 0 ? 1 : 1 - residualSumSquares / totalSumSquares;

    return { slope, intercept, rSquared };
  }

  private extractNumericValue(event: FeedbackEvent): number | null {
    switch (event.source) {
      case FeedbackSource.PERFORMANCE_METRICS:
        return (
          (event.value as any).latencyMs ||
          (event.value as any).qualityScore ||
          null
        );
      case FeedbackSource.USER_RATINGS:
        return (event.value as any).rating || null;
      case FeedbackSource.TASK_OUTCOMES:
        return (event.value as any).executionTimeMs || null;
      default:
        return typeof event.value === "number" ? event.value : null;
    }
  }

  private updateTimeSeries(event: FeedbackEvent): void {
    const value = this.extractNumericValue(event);
    if (value !== null) {
      const series = this.timeSeriesData.get(event.entityId) || [];
      series.push({
        timestamp: new Date(event.timestamp).getTime(),
        value,
      });

      // Keep only recent data
      const cutoffTime =
        Date.now() - this.config.trendWindowHours * 60 * 60 * 1000;
      const filteredSeries = series.filter((p) => p.timestamp > cutoffTime);

      this.timeSeriesData.set(event.entityId, filteredSeries);
    }
  }

  public clearAnalysisCache(): void {
    this.analysisCache.clear();
  }

  public getAnalysisCache(): Map<string, FeedbackAnalysis> {
    return new Map(this.analysisCache);
  }
}
