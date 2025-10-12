/**
 * @fileoverview Task Research Augmenter for ARBITER-006 Phase 4
 *
 * Augments tasks with research findings from Knowledge Seeker.
 * Integrates with ResearchDetector to automatically perform research when needed.
 *
 * @author @darianrosebrook
 */

import { Task } from "../../types/arbiter-orchestration";
import { IKnowledgeSeeker, KnowledgeResponse, SearchResult } from "../../types/knowledge";
import { ResearchDetector, ResearchRequirement } from "./ResearchDetector";

/**
 * Research findings summary
 */
export interface ResearchFindings {
  /** Query that was executed */
  query: string;

  /** Summary of findings */
  summary: string;

  /** Confidence in findings */
  confidence: number;

  /** Key findings (top results) */
  keyFindings: Array<{
    title: string;
    url: string;
    snippet: string;
    relevance: number;
  }>;
}

/**
 * Research context added to tasks
 */
export interface ResearchContext {
  /** Queries that were executed */
  queries: string[];

  /** Research findings */
  findings: ResearchFindings[];

  /** Overall confidence in research */
  confidence: number;

  /** When research was performed */
  augmentedAt: Date;

  /** Research requirement that triggered this */
  requirement: ResearchRequirement;
}

/**
 * Augmented task with research context
 */
export interface AugmentedTask extends Task {
  /** Whether research was provided */
  researchProvided: boolean;

  /** Research context if provided */
  researchContext?: ResearchContext;
}

/**
 * Task Research Augmenter Configuration
 */
export interface TaskResearchAugmenterConfig {
  /** Maximum results per query */
  maxResultsPerQuery?: number;

  /** Relevance threshold for results */
  relevanceThreshold?: number;

  /** Timeout per query (ms) */
  timeoutMs?: number;

  /** Maximum queries to execute */
  maxQueries?: number;

  /** Enable research caching */
  enableCaching?: boolean;
}

/**
 * Task Research Augmenter
 *
 * Automatically detects when tasks need research and augments them
 * with findings from the Knowledge Seeker.
 */
export class TaskResearchAugmenter {
  private knowledgeSeeker: IKnowledgeSeeker;
  private researchDetector: ResearchDetector;
  private config: Required<TaskResearchAugmenterConfig>;

  constructor(
    knowledgeSeeker: IKnowledgeSeeker,
    researchDetector: ResearchDetector,
    config: TaskResearchAugmenterConfig = {}
  ) {
    this.knowledgeSeeker = knowledgeSeeker;
    this.researchDetector = researchDetector;
    this.config = {
      maxResultsPerQuery: config.maxResultsPerQuery ?? 3,
      relevanceThreshold: config.relevanceThreshold ?? 0.8,
      timeoutMs: config.timeoutMs ?? 5000,
      maxQueries: config.maxQueries ?? 3,
      enableCaching: config.enableCaching ?? true,
    };
  }

  /**
   * Augment task with research if needed
   */
  async augmentTask(task: Task): Promise<AugmentedTask> {
    // Detect if research is needed
    const researchReq = this.researchDetector.detectResearchNeeds(task);

    if (!researchReq || !researchReq.required) {
      return {
        ...task,
        researchProvided: false,
      };
    }

    console.log(
      `Research detected for task ${task.id}: ${researchReq.reason} (confidence: ${researchReq.confidence.toFixed(2)})`
    );

    try {
      // Perform research
      const findings = await this.performResearch(
        researchReq.suggestedQueries,
        researchReq.queryType
      );

      // Calculate overall confidence
      const overallConfidence = this.calculateOverallConfidence(findings);

      // Create research context
      const researchContext: ResearchContext = {
        queries: researchReq.suggestedQueries,
        findings,
        confidence: overallConfidence,
        augmentedAt: new Date(),
        requirement: researchReq,
      };

      console.log(
        `Research completed for task ${task.id}: ${findings.length} findings (confidence: ${overallConfidence.toFixed(2)})`
      );

      return {
        ...task,
        researchProvided: true,
        researchContext,
      };
    } catch (error) {
      console.warn(
        `Research failed for task ${task.id}:`,
        error instanceof Error ? error.message : error
      );

      // Return task without research on failure
      return {
        ...task,
        researchProvided: false,
      };
    }
  }

  /**
   * Perform research for given queries
   */
  private async performResearch(
    queries: string[],
    queryType: string
  ): Promise<ResearchFindings[]> {
    const startTime = Date.now();

    // Limit queries
    const limitedQueries = queries.slice(0, this.config.maxQueries);

    // Execute queries in parallel
    const responses = await Promise.all(
      limitedQueries.map((query) =>
        this.executeQuery(query, queryType).catch((error) => {
          console.warn(`Query failed: ${query}`, error);
          return null;
        })
      )
    );

    // Filter out failed queries
    const successfulResponses = responses.filter(
      (r): r is KnowledgeResponse => r !== null
    );

    // Transform to research findings
    const findings = successfulResponses.map((response) =>
      this.transformToFindings(response)
    );

    const duration = Date.now() - startTime;
    console.log(
      `Research completed in ${duration}ms: ${findings.length}/${limitedQueries.length} queries successful`
    );

    return findings;
  }

  /**
   * Execute a single research query
   */
  private async executeQuery(
    query: string,
    queryType: string
  ): Promise<KnowledgeResponse> {
    return await this.knowledgeSeeker.processQuery({
      id: `task-research-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      query,
      queryType: queryType as any,
      maxResults: this.config.maxResultsPerQuery,
      relevanceThreshold: this.config.relevanceThreshold,
      timeoutMs: this.config.timeoutMs,
      context: { purpose: "task-augmentation" },
      metadata: {
        requesterId: "task-research-augmenter",
        priority: 7, // Lower priority than direct user requests
        createdAt: new Date(),
        tags: ["task-augmentation", "automatic-research"],
      },
    });
  }

  /**
   * Transform knowledge response to research findings
   */
  private transformToFindings(response: KnowledgeResponse): ResearchFindings {
    return {
      query: response.query.query,
      summary: response.summary,
      confidence: response.confidence,
      keyFindings: response.results.slice(0, 3).map((result: SearchResult) => ({
        title: result.title,
        url: result.url,
        snippet: result.content.substring(0, 200),
        relevance: result.relevanceScore,
      })),
    };
  }

  /**
   * Calculate overall confidence from findings
   */
  private calculateOverallConfidence(findings: ResearchFindings[]): number {
    if (findings.length === 0) {
      return 0;
    }

    // Average confidence weighted by number of findings
    const totalConfidence = findings.reduce(
      (sum, finding) => sum + finding.confidence,
      0
    );
    return totalConfidence / findings.length;
  }

  /**
   * Get research summary for a task (for display)
   */
  getResearchSummary(augmentedTask: AugmentedTask): string | null {
    if (!augmentedTask.researchProvided || !augmentedTask.researchContext) {
      return null;
    }

    const ctx = augmentedTask.researchContext;
    const summaries = ctx.findings.map((f) => f.summary).filter((s) => s);

    if (summaries.length === 0) {
      return null;
    }

    return `Research findings (confidence: ${(ctx.confidence * 100).toFixed(0)}%):\n${summaries.join("\n\n")}`;
  }

  /**
   * Get research sources for a task (for citations)
   */
  getResearchSources(augmentedTask: AugmentedTask): Array<{
    title: string;
    url: string;
  }> {
    if (!augmentedTask.researchProvided || !augmentedTask.researchContext) {
      return [];
    }

    const sources: Array<{ title: string; url: string }> = [];

    for (const finding of augmentedTask.researchContext.findings) {
      for (const keyFinding of finding.keyFindings) {
        sources.push({
          title: keyFinding.title,
          url: keyFinding.url,
        });
      }
    }

    // Deduplicate by URL
    const uniqueSources = sources.filter(
      (source, index, self) =>
        index === self.findIndex((s) => s.url === source.url)
    );

    return uniqueSources;
  }
}

