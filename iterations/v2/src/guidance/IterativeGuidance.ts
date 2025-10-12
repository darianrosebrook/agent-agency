/**
 * Iterative Guidance System
 *
 * Provides intelligent progress tracking, gap analysis, and actionable guidance
 * for iterative software development using CAWS working specifications.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import * as path from "path";
import type {
  AcceptanceProgress,
  GapAnalysis,
  GuidanceAnalysis,
  GuidanceCapabilities,
  GuidanceConfig,
  GuidanceContext,
  GuidanceEvents,
  GuidanceRecommendation,
  NextStep,
  ProgressSummary,
  StepGuidance,
  WorkEstimate,
} from "./types/guidance-types.js";

/**
 * Iterative Guidance System
 *
 * Analyzes working spec progress, identifies gaps, and provides
 * actionable guidance for completing tasks efficiently.
 */
export class IterativeGuidance extends EventEmitter {
  private config: GuidanceConfig;
  private context: GuidanceContext;

  /** Default context for analysis */
  private static readonly DEFAULT_CONTEXT: GuidanceContext = {
    phase: "implementation",
    teamSize: 1,
    experienceLevel: "senior",
    timePressure: "medium",
    qualityRequirements: "high",
    technologyFamiliarity: "expert",
  };

  /** Effort multipliers for complexity levels */
  private static readonly COMPLEXITY_MULTIPLIERS = {
    simple: 1.0,
    moderate: 1.5,
    complex: 2.5,
    expert: 4.0,
  };

  /** Priority weightings for ordering */
  private static readonly PRIORITY_WEIGHTS = {
    critical: 4,
    high: 3,
    medium: 2,
    low: 1,
  };

  constructor(config: GuidanceConfig, context?: Partial<GuidanceContext>) {
    super();
    this.config = config;
    this.context = { ...IterativeGuidance.DEFAULT_CONTEXT, ...context };
  }

  /**
   * Get system capabilities
   */
  getCapabilities(): GuidanceCapabilities {
    return {
      analyzeAcceptanceCriteria: true,
      identifyGaps: true,
      generateNextSteps: true,
      estimateWork: true,
      provideStepGuidance: true,
      assessRisks: true,
      integrateExternal: true,
    };
  }

  /**
   * Analyze overall progress and generate guidance
   */
  async analyzeProgress(): Promise<GuidanceAnalysis> {
    const startTime = Date.now();

    try {
      this.emit("analysis:start");

      // Analyze acceptance criteria progress
      const acceptanceCriteria = await this.analyzeAcceptanceCriteria();

      // Identify gaps
      const gaps = await this.identifyGaps(acceptanceCriteria);

      // Generate next steps
      const nextSteps = await this.generateNextSteps(acceptanceCriteria, gaps);

      // Calculate work estimates
      const workEstimate = this.estimateWork(nextSteps, gaps);

      // Calculate overall progress
      const overallProgress = this.calculateOverallProgress(acceptanceCriteria);

      // Identify critical blockers
      const criticalBlockers = this.identifyCriticalBlockers(
        acceptanceCriteria,
        gaps
      );

      // Get recent achievements
      const recentAchievements = this.getRecentAchievements(acceptanceCriteria);

      // Assess risks
      const riskAssessment = this.assessRisks(
        acceptanceCriteria,
        gaps,
        workEstimate
      );

      const summary: ProgressSummary = {
        overallProgress,
        acceptanceCriteria,
        gaps,
        nextSteps,
        workEstimate,
        criticalBlockers,
        recentAchievements,
        riskAssessment,
        analysisConfidence:
          this.calculateAnalysisConfidence(acceptanceCriteria),
        generatedAt: new Date().toISOString(),
      };

      const result: GuidanceAnalysis = {
        success: true,
        summary,
        metadata: {
          duration: Date.now() - startTime,
          confidence: summary.analysisConfidence,
          analyzedAt: new Date().toISOString(),
        },
      };

      this.emit("analysis:complete", result);
      this.emit("progress:analyzed", summary);
      this.emit("gaps:identified", gaps);
      this.emit("steps:generated", nextSteps);
      this.emit("estimate:updated", workEstimate);

      return result;
    } catch (error) {
      const errorResult: GuidanceAnalysis = {
        success: false,
        error: {
          message: error instanceof Error ? error.message : "Unknown error",
          details: error,
        },
        metadata: {
          duration: Date.now() - startTime,
          confidence: "low",
          analyzedAt: new Date().toISOString(),
        },
      };

      this.emit("analysis:error", error as Error);
      this.emit("analysis:complete", errorResult);

      return errorResult;
    }
  }

  /**
   * Analyze progress on acceptance criteria
   */
  private async analyzeAcceptanceCriteria(): Promise<AcceptanceProgress[]> {
    const acceptance = this.config.spec.acceptance || [];
    const progress: AcceptanceProgress[] = [];

    for (const criterion of acceptance) {
      const criterionProgress = await this.analyzeSingleCriterion(criterion);
      progress.push(criterionProgress);
    }

    return progress;
  }

  /**
   * Analyze progress on a single acceptance criterion
   */
  private async analyzeSingleCriterion(
    criterion: any
  ): Promise<AcceptanceProgress> {
    const evidence = await this.findEvidenceForCriterion(criterion);
    const blockers = await this.identifyBlockers(criterion);

    // Estimate progress based on evidence found
    const progressPercent = this.calculateCriterionProgress(
      criterion,
      evidence,
      blockers
    );
    const status = this.determineCriterionStatus(progressPercent, blockers);

    // Estimate remaining work
    const estimatedHoursRemaining = this.estimateRemainingWork(
      criterion,
      progressPercent
    );

    return {
      id: criterion.id,
      status,
      progressPercent,
      given: criterion.given,
      when: criterion.when,
      then: criterion.then,
      evidence,
      blockers,
      estimatedHoursRemaining,
      confidence: this.assessProgressConfidence(evidence, blockers),
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Find evidence for a criterion being met
   */
  private async findEvidenceForCriterion(criterion: any): Promise<string[]> {
    const evidence: string[] = [];

    // Check for test files
    if (this.config.testFiles) {
      const relevantTests = this.config.testFiles.filter((file) =>
        this.isTestRelatedToCriterion(file, criterion)
      );
      evidence.push(...relevantTests.map((f) => `test:${f}`));
    }

    // Check for implementation files
    if (this.config.existingFiles) {
      const relevantFiles = this.config.existingFiles.filter((file) =>
        this.isImplementationRelatedToCriterion(file, criterion)
      );
      evidence.push(...relevantFiles.map((f) => `impl:${f}`));
    }

    // Check recent changes
    if (this.config.recentChanges) {
      const relevantChanges = this.config.recentChanges.filter((change) =>
        this.isChangeRelatedToCriterion(change, criterion)
      );
      evidence.push(...relevantChanges.map((c) => `change:${c.file}`));
    }

    return evidence;
  }

  /**
   * Identify blockers for a criterion
   */
  private async identifyBlockers(criterion: any): Promise<string[]> {
    const blockers: string[] = [];

    // Check for missing dependencies
    const dependencies = this.extractDependenciesFromCriterion(criterion);
    for (const dep of dependencies) {
      if (!(await this.isDependencyMet(dep))) {
        blockers.push(`Missing dependency: ${dep}`);
      }
    }

    // Check for external system requirements
    if (this.requiresExternalSystems(criterion)) {
      if (!this.areExternalSystemsAvailable(criterion)) {
        blockers.push("External system integration required");
      }
    }

    return blockers;
  }

  /**
   * Calculate progress percentage for a criterion
   */
  private calculateCriterionProgress(
    criterion: any,
    evidence: string[],
    blockers: string[]
  ): number {
    if (blockers.length > 0) {
      return Math.max(0, 90 - blockers.length * 10); // Blocked but some progress possible
    }

    if (evidence.length === 0) {
      return 0;
    }

    // Simple heuristic: more evidence = more progress
    const baseProgress = Math.min(80, evidence.length * 20);

    // Boost if we have both tests and implementation
    const hasTests = evidence.some((e) => e.startsWith("test:"));
    const hasImpl = evidence.some((e) => e.startsWith("impl:"));
    const hasBoth = hasTests && hasImpl;

    return hasBoth ? Math.min(95, baseProgress + 15) : baseProgress;
  }

  /**
   * Determine status based on progress and blockers
   */
  private determineCriterionStatus(
    progressPercent: number,
    blockers: string[]
  ): AcceptanceProgress["status"] {
    if (blockers.length > 0) {
      return "blocked";
    }

    if (progressPercent >= 95) {
      return "completed";
    }

    if (progressPercent > 10) {
      return "in_progress";
    }

    return "not_started";
  }

  /**
   * Estimate remaining work for a criterion
   */
  private estimateRemainingWork(
    criterion: any,
    progressPercent: number
  ): number {
    const baseHours = this.estimateCriterionComplexity(criterion);
    const remainingWork = (100 - progressPercent) / 100;

    return Math.max(0.5, baseHours * remainingWork);
  }

  /**
   * Estimate complexity of a criterion
   */
  private estimateCriterionComplexity(criterion: any): number {
    // Simple heuristic based on criterion description length and keywords
    const description = `${criterion.given} ${criterion.when} ${criterion.then}`;
    const complexity = description.length / 50; // Rough estimate

    // Adjust for complexity indicators
    if (
      description.includes("integration") ||
      description.includes("external")
    ) {
      return complexity * 1.5;
    }

    if (description.includes("security") || description.includes("auth")) {
      return complexity * 1.8;
    }

    return Math.max(1, Math.min(8, complexity));
  }

  /**
   * Assess confidence in progress assessment
   */
  private assessProgressConfidence(
    evidence: string[],
    blockers: string[]
  ): AcceptanceProgress["confidence"] {
    const evidenceStrength = evidence.length * 2; // More evidence = higher confidence
    const blockerPenalty = blockers.length * 5; // Blockers reduce confidence

    const confidenceScore = Math.max(0, evidenceStrength - blockerPenalty);

    if (confidenceScore >= 6) return "high";
    if (confidenceScore >= 3) return "medium";
    return "low";
  }

  /**
   * Identify implementation gaps
   */
  private async identifyGaps(
    acceptanceCriteria: AcceptanceProgress[]
  ): Promise<GapAnalysis[]> {
    const gaps: GapAnalysis[] = [];

    // Check for missing tests
    const criteriaWithoutTests = acceptanceCriteria.filter(
      (c) => !c.evidence.some((e) => e.startsWith("test:"))
    );

    if (criteriaWithoutTests.length > 0) {
      gaps.push({
        category: "testing",
        description: `${criteriaWithoutTests.length} acceptance criteria lack test coverage`,
        severity: criteriaWithoutTests.length > 2 ? "high" : "medium",
        affectedCriteria: criteriaWithoutTests.map((c) => c.id),
        estimatedEffort: {
          hours: criteriaWithoutTests.length * 2,
          complexity: "moderate",
        },
        remediationSteps: [
          "Create unit tests for each acceptance criterion",
          "Add integration tests for complex scenarios",
          "Ensure test coverage meets CAWS requirements",
        ],
        priority: "high",
      });
    }

    // Check for blocked criteria
    const blockedCriteria = acceptanceCriteria.filter(
      (c) => c.status === "blocked"
    );

    if (blockedCriteria.length > 0) {
      gaps.push({
        category: "implementation",
        description: `${blockedCriteria.length} acceptance criteria are blocked`,
        severity: "high",
        affectedCriteria: blockedCriteria.map((c) => c.id),
        estimatedEffort: {
          hours: blockedCriteria.reduce(
            (sum, c) => sum + c.estimatedHoursRemaining,
            0
          ),
          complexity: "complex",
        },
        remediationSteps: [
          "Identify and resolve blockers for each criterion",
          "Prioritize unblocking high-impact criteria first",
          "Consider alternative approaches if blockers are fundamental",
        ],
        priority: "critical",
      });
    }

    // Check budget usage
    if (this.config.budgetUsage) {
      const { filesPercentage, locPercentage } = this.config.budgetUsage;

      if (filesPercentage > 80 || locPercentage > 80) {
        gaps.push({
          category: "implementation",
          description: `Budget usage is high (${filesPercentage.toFixed(
            1
          )}% files, ${locPercentage.toFixed(1)}% LOC)`,
          severity:
            filesPercentage > 95 || locPercentage > 95 ? "critical" : "medium",
          affectedCriteria: acceptanceCriteria.map((c) => c.id),
          estimatedEffort: {
            hours: Math.max(4, (filesPercentage + locPercentage) / 20),
            complexity: "moderate",
          },
          remediationSteps: [
            "Review current implementation for optimization opportunities",
            "Consider refactoring to reduce code complexity",
            "Split remaining work into smaller, focused tasks",
          ],
          priority:
            filesPercentage > 95 || locPercentage > 95 ? "critical" : "medium",
        });
      }
    }

    return gaps;
  }

  /**
   * Generate prioritized next steps
   */
  private async generateNextSteps(
    acceptanceCriteria: AcceptanceProgress[],
    gaps: GapAnalysis[]
  ): Promise<NextStep[]> {
    const steps: NextStep[] = [];

    // Generate steps for incomplete criteria
    const incompleteCriteria = acceptanceCriteria.filter(
      (c) => c.status !== "completed"
    );

    for (const criterion of incompleteCriteria) {
      const criterionSteps = this.generateStepsForCriterion(criterion);
      steps.push(...criterionSteps);
    }

    // Generate steps for gaps
    for (const gap of gaps) {
      const gapSteps = this.generateStepsForGap(gap);
      steps.push(...gapSteps);
    }

    // Sort by priority and dependencies
    return this.prioritizeAndOrderSteps(steps);
  }

  /**
   * Generate steps for a single criterion
   */
  private generateStepsForCriterion(criterion: AcceptanceProgress): NextStep[] {
    const steps: NextStep[] = [];

    if (criterion.status === "blocked") {
      // Generate unblocking steps
      steps.push({
        id: `unblock-${criterion.id}`,
        title: `Resolve blockers for ${criterion.id}`,
        description: `Address the following blockers: ${criterion.blockers.join(
          ", "
        )}`,
        priority: "critical",
        category: "implementation",
        estimatedEffort: {
          hours: Math.max(1, criterion.estimatedHoursRemaining / 4),
          complexity: "moderate",
          confidence: "medium",
        },
        prerequisites: [],
        affectedFiles: [],
        expectedOutcomes: [`${criterion.id} is no longer blocked`],
        risk: criterion.blockers.length > 2 ? "high" : "medium",
        dependencies: [],
        parallelizable: false,
      });
    } else if (criterion.status === "not_started") {
      // Generate implementation steps
      steps.push({
        id: `implement-${criterion.id}`,
        title: `Implement ${criterion.id}`,
        description: `${criterion.given} ${criterion.when} ${criterion.then}`,
        priority: "high",
        category: "implementation",
        estimatedEffort: {
          hours: criterion.estimatedHoursRemaining,
          complexity:
            this.context.experienceLevel === "expert" ? "moderate" : "complex",
          confidence: criterion.confidence,
        },
        prerequisites: [],
        affectedFiles: this.predictAffectedFiles(criterion),
        expectedOutcomes: [`${criterion.id} meets acceptance criteria`],
        risk: "medium",
        dependencies: [],
        parallelizable: true,
      });
    } else if (criterion.status === "in_progress") {
      // Generate completion steps
      steps.push({
        id: `complete-${criterion.id}`,
        title: `Complete ${criterion.id}`,
        description: `Finish remaining work for: ${criterion.then}`,
        priority: "high",
        category: "implementation",
        estimatedEffort: {
          hours: criterion.estimatedHoursRemaining,
          complexity: "simple",
          confidence: "high",
        },
        prerequisites: [
          `Evidence exists: ${criterion.evidence.slice(0, 2).join(", ")}`,
        ],
        affectedFiles: this.predictAffectedFiles(criterion),
        expectedOutcomes: [`${criterion.id} is fully completed`],
        risk: "low",
        dependencies: [],
        parallelizable: true,
      });
    }

    return steps;
  }

  /**
   * Generate steps for addressing a gap
   */
  private generateStepsForGap(gap: GapAnalysis): NextStep[] {
    const steps: NextStep[] = [];

    for (let i = 0; i < gap.remediationSteps.length; i++) {
      steps.push({
        id: `gap-${gap.category}-${gap.affectedCriteria[0]}-${i}`,
        title: `Address ${gap.category} gap: ${gap.description}`,
        description: gap.remediationSteps[i],
        priority: gap.priority,
        category: gap.category as any,
        estimatedEffort: {
          hours: gap.estimatedEffort.hours / gap.remediationSteps.length,
          complexity: gap.estimatedEffort.complexity,
          confidence: "medium",
        },
        prerequisites:
          i > 0
            ? [`gap-${gap.category}-${gap.affectedCriteria[0]}-${i - 1}`]
            : [],
        affectedFiles: [],
        expectedOutcomes: [`Gap in ${gap.category} reduced`],
        risk: gap.severity === "critical" ? "high" : "medium",
        dependencies: [],
        parallelizable: gap.category === "testing",
      });
    }

    return steps;
  }

  /**
   * Estimate total work remaining
   */
  private estimateWork(
    nextSteps: NextStep[],
    gaps: GapAnalysis[]
  ): WorkEstimate {
    const totalHours =
      nextSteps.reduce((sum, step) => sum + step.estimatedEffort.hours, 0) +
      gaps.reduce((sum, gap) => sum + gap.estimatedEffort.hours, 0);

    const hoursByCategory = nextSteps.reduce((acc, step) => {
      acc[step.category] =
        (acc[step.category] || 0) + step.estimatedEffort.hours;
      return acc;
    }, {} as Record<string, number>);

    const hoursByPriority = nextSteps.reduce(
      (acc, step) => {
        acc[step.priority] =
          (acc[step.priority] || 0) + step.estimatedEffort.hours;
        return acc;
      },
      { critical: 0, high: 0, medium: 0, low: 0 }
    );

    // Calculate parallelization factor
    const parallelizableSteps = nextSteps.filter(
      (s) => s.parallelizable
    ).length;
    const parallelizationFactor = Math.min(
      1,
      parallelizableSteps / nextSteps.length + 0.3
    );

    // Calculate confidence intervals
    const complexityMultiplier =
      this.context.experienceLevel === "expert"
        ? 0.8
        : this.context.experienceLevel === "senior"
        ? 1.0
        : 1.2;
    const timePressureMultiplier =
      this.context.timePressure === "critical"
        ? 1.5
        : this.context.timePressure === "high"
        ? 1.2
        : 1.0;

    const adjustedHours =
      totalHours * complexityMultiplier * timePressureMultiplier;

    const confidenceIntervals = {
      pessimistic: adjustedHours * 1.5,
      optimistic: adjustedHours * 0.7,
      mostLikely: adjustedHours,
    };

    // Estimate completion dates (assuming 6 hours/day, 5 days/week)
    const workDays = confidenceIntervals.mostLikely / 6;
    const today = new Date();

    const completionEstimates = {
      earliest: new Date(
        today.getTime() + workDays * 0.7 * 24 * 60 * 60 * 1000
      ).toISOString(),
      mostLikely: new Date(
        today.getTime() + workDays * 24 * 60 * 60 * 1000
      ).toISOString(),
      latest: new Date(
        today.getTime() + workDays * 1.5 * 24 * 60 * 60 * 1000
      ).toISOString(),
    };

    // Calculate risk factors
    const riskFactors = {
      technicalDebt: this.config.budgetStats
        ? this.config.budgetStats.peakLocUsage / 100
        : 0.3,
      teamExperience:
        this.context.experienceLevel === "expert"
          ? 0.1
          : this.context.experienceLevel === "senior"
          ? 0.3
          : 0.6,
      requirementClarity: gaps.length > 3 ? 0.7 : 0.2,
      externalDependencies: this.hasExternalDependencies() ? 0.5 : 0.1,
    };

    return {
      totalHours: confidenceIntervals.mostLikely,
      hoursByCategory: {
        implementation: hoursByCategory.implementation || 0,
        testing: hoursByCategory.testing || 0,
        refactoring: hoursByCategory.refactoring || 0,
        documentation: hoursByCategory.documentation || 0,
        integration: hoursByCategory.integration || 0,
      },
      hoursByPriority,
      confidenceIntervals,
      parallelizationFactor,
      completionEstimates,
      riskFactors,
    };
  }

  /**
   * Calculate overall progress percentage
   */
  private calculateOverallProgress(
    acceptanceCriteria: AcceptanceProgress[]
  ): number {
    if (acceptanceCriteria.length === 0) return 0;

    const totalProgress = acceptanceCriteria.reduce(
      (sum, c) => sum + c.progressPercent,
      0
    );
    return totalProgress / acceptanceCriteria.length;
  }

  /**
   * Identify critical blockers
   */
  private identifyCriticalBlockers(
    acceptanceCriteria: AcceptanceProgress[],
    gaps: GapAnalysis[]
  ): string[] {
    const blockers: string[] = [];

    // Add blocked criteria
    const blockedCriteria = acceptanceCriteria.filter(
      (c) => c.status === "blocked"
    );
    blockers.push(
      ...blockedCriteria.map((c) => `${c.id}: ${c.blockers.join(", ")}`)
    );

    // Add high-severity gaps
    const criticalGaps = gaps.filter((g) => g.severity === "critical");
    blockers.push(...criticalGaps.map((g) => `Critical gap: ${g.description}`));

    // Add budget concerns
    if (this.config.budgetUsage) {
      const { filesPercentage, locPercentage } = this.config.budgetUsage;
      if (filesPercentage > 95) {
        blockers.push(
          `Budget exceeded: ${filesPercentage.toFixed(1)}% files used`
        );
      }
      if (locPercentage > 95) {
        blockers.push(`Budget exceeded: ${locPercentage.toFixed(1)}% LOC used`);
      }
    }

    return blockers;
  }

  /**
   * Get recent achievements
   */
  private getRecentAchievements(
    acceptanceCriteria: AcceptanceProgress[]
  ): string[] {
    const achievements: string[] = [];

    const completedCriteria = acceptanceCriteria.filter(
      (c) => c.status === "completed"
    );
    achievements.push(
      ...completedCriteria.map((c) => `Completed ${c.id}: ${c.then}`)
    );

    const inProgressCriteria = acceptanceCriteria.filter(
      (c) => c.status === "in_progress"
    );
    achievements.push(
      ...inProgressCriteria.map(
        (c) =>
          `In progress on ${c.id} (${c.progressPercent.toFixed(0)}% complete)`
      )
    );

    return achievements;
  }

  /**
   * Assess project risks
   */
  private assessRisks(
    acceptanceCriteria: AcceptanceProgress[],
    gaps: GapAnalysis[],
    workEstimate: WorkEstimate
  ): ProgressSummary["riskAssessment"] {
    let riskScore = 0;

    // Risk from blockers
    const blockedCount = acceptanceCriteria.filter(
      (c) => c.status === "blocked"
    ).length;
    riskScore += blockedCount * 2;

    // Risk from gaps
    const highSeverityGaps = gaps.filter(
      (g) => g.severity === "high" || g.severity === "critical"
    ).length;
    riskScore += highSeverityGaps * 1.5;

    // Risk from budget usage
    if (this.config.budgetUsage) {
      const { filesPercentage, locPercentage } = this.config.budgetUsage;
      riskScore += Math.max(0, (filesPercentage - 80) / 10);
      riskScore += Math.max(0, (locPercentage - 80) / 10);
    }

    // Risk from time pressure
    if (this.context.timePressure === "critical") riskScore += 2;
    if (this.context.timePressure === "high") riskScore += 1;

    // Determine overall risk level
    let overallRisk: "low" | "medium" | "high" | "critical";
    if (riskScore >= 8) overallRisk = "critical";
    else if (riskScore >= 5) overallRisk = "high";
    else if (riskScore >= 3) overallRisk = "medium";
    else overallRisk = "low";

    const riskFactors = [];
    if (blockedCount > 0)
      riskFactors.push(`${blockedCount} blocked acceptance criteria`);
    if (highSeverityGaps > 0)
      riskFactors.push(`${highSeverityGaps} high-severity gaps`);
    if (
      this.config.budgetUsage &&
      (this.config.budgetUsage.filesPercentage > 80 ||
        this.config.budgetUsage.locPercentage > 80)
    ) {
      riskFactors.push("High budget utilization");
    }
    if (
      this.context.timePressure === "high" ||
      this.context.timePressure === "critical"
    ) {
      riskFactors.push(`High time pressure (${this.context.timePressure})`);
    }

    const mitigationStrategies = [];
    if (blockedCount > 0)
      mitigationStrategies.push("Focus on unblocking critical paths first");
    if (highSeverityGaps > 0)
      mitigationStrategies.push("Address high-priority gaps immediately");
    if (
      this.config.budgetUsage &&
      (this.config.budgetUsage.filesPercentage > 80 ||
        this.config.budgetUsage.locPercentage > 80)
    ) {
      mitigationStrategies.push(
        "Optimize existing code or split remaining work"
      );
    }
    if (
      this.context.timePressure === "high" ||
      this.context.timePressure === "critical"
    ) {
      mitigationStrategies.push(
        "Prioritize critical path items and consider scope reduction"
      );
    }

    return {
      overallRisk,
      riskFactors,
      mitigationStrategies,
    };
  }

  /**
   * Calculate analysis confidence
   */
  private calculateAnalysisConfidence(
    acceptanceCriteria: AcceptanceProgress[]
  ): AcceptanceProgress["confidence"] {
    const confidences = acceptanceCriteria.map((c) => {
      switch (c.confidence) {
        case "high":
          return 3;
        case "medium":
          return 2;
        case "low":
          return 1;
      }
    });

    const averageConfidence =
      confidences.reduce((sum, c) => sum + c, 0) / confidences.length;

    if (averageConfidence >= 2.5) return "high";
    if (averageConfidence >= 1.8) return "medium";
    return "low";
  }

  /**
   * Provide step-by-step guidance for current work
   */
  async getStepGuidance(currentStepIndex: number = 0): Promise<StepGuidance> {
    const analysis = await this.analyzeProgress();

    if (!analysis.success || !analysis.summary) {
      throw new Error("Could not analyze progress for step guidance");
    }

    const steps = analysis.summary.nextSteps;
    const currentStep = steps[currentStepIndex];

    if (!currentStep) {
      throw new Error(`Step ${currentStepIndex} not found`);
    }

    const stepProgress = this.calculateStepProgress(currentStep);
    const timeSpent = 0; // Would need tracking implementation
    const estimatedTimeRemaining =
      currentStep.estimatedEffort.hours * (1 - stepProgress / 100);

    return {
      currentStep: currentStepIndex + 1,
      totalSteps: steps.length,
      step: currentStep,
      stepProgress,
      timeSpent,
      estimatedTimeRemaining,
      tips: this.generateTipsForStep(currentStep),
      pitfalls: this.generatePitfallsForStep(currentStep),
      qualityChecks: this.generateQualityChecksForStep(currentStep),
      nextStepsPreview: steps.slice(currentStepIndex + 1, currentStepIndex + 4),
    };
  }

  /**
   * Generate recommendations for improvement
   */
  getRecommendations(): GuidanceRecommendation[] {
    const recommendations: GuidanceRecommendation[] = [];

    // Check if testing coverage is adequate
    if (this.config.testFiles && this.config.existingFiles) {
      const testCoverage =
        this.config.testFiles.length / this.config.existingFiles.length;
      if (testCoverage < 0.5) {
        recommendations.push({
          type: "practice",
          title: "Increase Test Coverage",
          explanation: `Current test coverage is ${Math.round(
            testCoverage * 100
          )}%, which is below recommended levels`,
          benefit: "Higher confidence in code changes and fewer regressions",
          effort: "high",
          urgency: "medium",
          evidence: [
            "Industry standard: 70-80% test coverage",
            "CAWS requires comprehensive testing",
          ],
          prerequisites: ["Testing framework in place"],
        });
      }
    }

    // Check for parallel work opportunities
    if (this.context.teamSize > 1) {
      recommendations.push({
        type: "approach",
        title: "Leverage Team Parallelization",
        explanation:
          "With multiple team members, identify parallelizable tasks to accelerate delivery",
        benefit: "Faster time-to-completion through concurrent work",
        effort: "medium",
        urgency: "low",
        evidence: [
          "Team size allows parallel work",
          "Many tasks can be done independently",
        ],
        prerequisites: ["Clear task boundaries", "Good communication"],
      });
    }

    // Check for automation opportunities
    if (
      this.config.aiAttribution &&
      this.config.aiAttribution.aiAssistedCommits > 0
    ) {
      recommendations.push({
        type: "tool",
        title: "Optimize AI Tool Usage",
        explanation: `${this.config.aiAttribution.aiAssistedCommits} commits used AI assistance - consider standardizing AI workflows`,
        benefit: "Consistent quality and faster development",
        effort: "low",
        urgency: "low",
        evidence: [
          `${this.config.aiAttribution.aiAssistedCommits} AI-assisted commits`,
        ],
        prerequisites: ["AI tools available"],
      });
    }

    return recommendations;
  }

  // Helper methods for analysis

  private isTestRelatedToCriterion(file: string, criterion: any): boolean {
    const criterionText =
      `${criterion.given} ${criterion.when} ${criterion.then}`.toLowerCase();
    const fileName = path.basename(file, path.extname(file)).toLowerCase();

    // Check if test file name contains criterion keywords
    return criterionText
      .split(" ")
      .some((word) => word.length > 3 && fileName.includes(word));
  }

  private isImplementationRelatedToCriterion(
    file: string,
    criterion: any
  ): boolean {
    const criterionText =
      `${criterion.given} ${criterion.when} ${criterion.then}`.toLowerCase();
    const fileName = path.basename(file, path.extname(file)).toLowerCase();

    return criterionText
      .split(" ")
      .some((word) => word.length > 3 && fileName.includes(word));
  }

  private isChangeRelatedToCriterion(change: any, criterion: any): boolean {
    return this.isImplementationRelatedToCriterion(change.file, criterion);
  }

  private extractDependenciesFromCriterion(criterion: any): string[] {
    const text = `${criterion.given} ${criterion.when} ${criterion.then}`;
    const dependencies: string[] = [];

    // Look for common dependency indicators
    if (text.includes("database") || text.includes("db"))
      dependencies.push("database");
    if (text.includes("api") || text.includes("service"))
      dependencies.push("external-api");
    if (text.includes("auth") || text.includes("login"))
      dependencies.push("authentication");
    if (text.includes("file") || text.includes("upload"))
      dependencies.push("file-system");

    return dependencies;
  }

  private async isDependencyMet(dependency: string): Promise<boolean> {
    // Simple check - in real implementation, this would check actual system state
    switch (dependency) {
      case "database":
        return this.config.spec.blast_radius.data_migration !== undefined;
      case "external-api":
        return (this.config.spec.contracts?.length ?? 0) > 0;
      case "authentication":
        return this.config.spec.scope.in.some((p) => p.includes("auth"));
      case "file-system":
        return true; // Assume file system is always available
      default:
        return false;
    }
  }

  private requiresExternalSystems(criterion: any): boolean {
    const text =
      `${criterion.given} ${criterion.when} ${criterion.then}`.toLowerCase();
    return (
      text.includes("api") ||
      text.includes("service") ||
      text.includes("external")
    );
  }

  private areExternalSystemsAvailable(criterion: any): boolean {
    // Check if contracts are defined for external systems
    return (this.config.spec.contracts?.length ?? 0) > 0;
  }

  private predictAffectedFiles(criterion: AcceptanceProgress): string[] {
    const files: string[] = [];

    // Predict based on scope and criterion content
    const scopePaths = this.config.spec.scope.in;
    const criterionText =
      `${criterion.given} ${criterion.when} ${criterion.then}`.toLowerCase();

    for (const scopePath of scopePaths) {
      if (criterionText.includes("auth") && scopePath.includes("auth")) {
        files.push(`${scopePath}/authentication.ts`);
      }
      if (criterionText.includes("user") && scopePath.includes("user")) {
        files.push(`${scopePath}/user-service.ts`);
      }
      // Add more predictions based on common patterns
    }

    return files;
  }

  private prioritizeAndOrderSteps(steps: NextStep[]): NextStep[] {
    // Sort by priority weight, then by dependencies
    return steps.sort((a, b) => {
      const priorityDiff =
        IterativeGuidance.PRIORITY_WEIGHTS[b.priority] -
        IterativeGuidance.PRIORITY_WEIGHTS[a.priority];
      if (priorityDiff !== 0) return priorityDiff;

      // Sort by dependencies (steps with fewer dependencies first)
      return a.dependencies.length - b.dependencies.length;
    });
  }

  private calculateStepProgress(step: NextStep): number {
    // Simple heuristic based on prerequisites met
    const prereqsMet = step.prerequisites.length; // Assume all are met for now
    return Math.min(50, prereqsMet * 10); // Up to 50% based on prerequisites
  }

  private generateTipsForStep(step: NextStep): string[] {
    const tips: string[] = [];

    switch (step.category) {
      case "implementation":
        tips.push("Start with the simplest case first");
        tips.push("Write tests before implementation");
        tips.push("Consider edge cases and error handling");
        break;
      case "testing":
        tips.push("Test both happy path and error scenarios");
        tips.push("Use descriptive test names");
        tips.push("Mock external dependencies");
        break;
      case "refactoring":
        tips.push("Ensure all tests pass before and after");
        tips.push("Refactor in small, incremental changes");
        tips.push("Update any affected documentation");
        break;
    }

    return tips;
  }

  private generatePitfallsForStep(step: NextStep): string[] {
    const pitfalls: string[] = [];

    pitfalls.push("Not handling error cases properly");
    pitfalls.push("Making changes without running tests");
    pitfalls.push("Not updating documentation");

    if (step.category === "implementation") {
      pitfalls.push("Over-engineering the solution");
      pitfalls.push("Not considering the acceptance criteria fully");
    }

    return pitfalls;
  }

  private generateQualityChecksForStep(step: NextStep): string[] {
    const checks: string[] = [];

    checks.push("All tests pass");
    checks.push("Code follows project conventions");
    checks.push("Acceptance criteria are met");
    checks.push("No linting errors");

    if (step.category === "implementation") {
      checks.push("Edge cases are handled");
      checks.push("Error messages are user-friendly");
    }

    return checks;
  }

  private hasExternalDependencies(): boolean {
    return this.config.spec.contracts && this.config.spec.contracts.length > 0;
  }

  /**
   * Typed event emitter methods
   */
  on<K extends keyof GuidanceEvents>(
    event: K,
    listener: GuidanceEvents[K]
  ): this {
    return super.on(event, listener);
  }

  emit<K extends keyof GuidanceEvents>(
    event: K,
    ...args: Parameters<GuidanceEvents[K]>
  ): boolean {
    return super.emit(event, ...args);
  }
}
