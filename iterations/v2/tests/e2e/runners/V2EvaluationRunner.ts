/**
 * Base E2E Evaluation Runner
 *
 * @author @darianrosebrook
 * @description Abstract base class for end-to-end agent evaluation with iterative feedback
 */

import type { ModelBasedJudge } from "@/evaluation/ModelBasedJudge";
import type { ArbiterMCPServer } from "@/mcp-server/ArbiterMCPServer";
import type { ModelRegistry } from "@/models/ModelRegistry";
import type { PerformanceTracker } from "@/rl/PerformanceTracker";
import type {
  AgentInteraction,
  EvaluationCriterion,
  EvaluationReport,
  GenerationContext,
  IterativeConfig,
  TestResult,
  TestStatistics,
} from "../types/evaluation";
import { DEFAULT_ITERATIVE_CONFIG } from "../types/evaluation";

/**
 * Base class for E2E evaluation runners
 *
 * Provides:
 * - Multi-turn iterative feedback loop
 * - Agent interaction tracking
 * - Satisficing logic (stops when "good enough")
 * - Consistent metric scoring
 * - Performance tracking integration
 */
export abstract class V2EvaluationRunner<TSpec = unknown, TOutput = unknown> {
  protected interactions: AgentInteraction[] = [];
  protected config: IterativeConfig;

  constructor(
    protected judge: ModelBasedJudge,
    protected mcpServer: ArbiterMCPServer,
    protected performanceTracker: PerformanceTracker,
    protected registry: ModelRegistry,
    config?: Partial<IterativeConfig>
  ) {
    this.config = {
      ...DEFAULT_ITERATIVE_CONFIG,
      ...config,
    };
  }

  /**
   * Run a test scenario
   *
   * Subclasses must implement this to define their specific test logic
   */
  abstract runScenario(spec: TSpec): Promise<TestResult>;

  /**
   * Generic iterative feedback loop
   *
   * This is the core of the E2E testing framework:
   * 1. Generate output from agent
   * 2. Evaluate output against criteria
   * 3. If passing, return success
   * 4. If not passing, generate feedback and iterate
   * 5. Repeat until max iterations or passing
   *
   * @param generateFn Function to generate agent output
   * @param evaluateFn Function to evaluate the output
   * @param config Optional configuration overrides
   * @returns Complete test result
   */
  protected async iterativeLoop(
    generateFn: (context: GenerationContext) => Promise<TOutput>,
    evaluateFn: (output: TOutput) => Promise<EvaluationReport>,
    config?: Partial<IterativeConfig>
  ): Promise<TestResult> {
    const loopConfig = { ...this.config, ...config };
    const startTime = Date.now();

    let iteration = 0;
    let lastOutput: TOutput | null = null;
    let lastReport: EvaluationReport | null = null;
    const feedbackHistory: string[] = [];
    const iterationScores: number[] = [];

    // Reset interactions for this test
    this.interactions = [];

    while (iteration < loopConfig.maxIterations) {
      iteration++;
      const iterationStartTime = Date.now();

      console.log(`\n🔄 Iteration ${iteration}/${loopConfig.maxIterations}...`);

      try {
        // 1. Generate output
        const genStartTime = Date.now();
        const output: TOutput = await this.withTimeout(
          generateFn({
            iteration,
            previousOutput: lastOutput,
            feedbackHistory,
          }),
          loopConfig.iterationTimeoutMs,
          `Generation timeout in iteration ${iteration}`
        );
        const genDuration = Date.now() - genStartTime;

        this.trackInteraction({
          type: "generation",
          timestamp: new Date(),
          details: {
            iteration,
            hasPreviousOutput: lastOutput !== null,
            feedbackCount: feedbackHistory.length,
          },
          duration: genDuration,
        });

        lastOutput = output;

        // 2. Evaluate output
        const evalStartTime = Date.now();
        const report = await this.withTimeout(
          evaluateFn(output),
          loopConfig.iterationTimeoutMs,
          `Evaluation timeout in iteration ${iteration}`
        );
        const evalDuration = Date.now() - evalStartTime;

        this.trackInteraction({
          type: "evaluation",
          timestamp: new Date(),
          details: {
            iteration,
            overallScore: report.overallScore,
            overallPassed: report.overallPassed,
            criteriaCount: report.criteria.length,
          },
          duration: evalDuration,
        });

        lastReport = report;
        iterationScores.push(report.overallScore);

        console.log(
          `📊 Iteration ${iteration} Score: ${(
            report.overallScore * 100
          ).toFixed(1)}%`
        );
        console.log(
          `   Passed: ${report.criteria.filter((c) => c.passed).length}/${
            report.criteria.length
          } criteria`
        );

        // 3. Check if we meet success criteria
        const meetsThreshold =
          report.overallScore >= loopConfig.passingThreshold;
        const allCriteriaPassed = report.overallPassed;

        const shouldStop = loopConfig.requireAllCriteriaPassed
          ? meetsThreshold && allCriteriaPassed
          : meetsThreshold;

        if (shouldStop) {
          console.log(
            `✅ Success on iteration ${iteration}! (Score: ${(
              report.overallScore * 100
            ).toFixed(1)}%)`
          );

          return {
            success: true,
            output,
            iterations: iteration,
            feedbackHistory,
            report,
            agentInteractions: this.interactions,
            totalExecutionTime: Date.now() - startTime,
          };
        }

        // 4. Generate feedback for next iteration
        if (iteration < loopConfig.maxIterations) {
          const feedback = this.generateFeedback(report, output);
          feedbackHistory.push(feedback);

          console.log(
            `📝 Feedback: ${feedback.substring(0, 100)}${
              feedback.length > 100 ? "..." : ""
            }`
          );

          // 5. Delay before next iteration
          if (loopConfig.delayBetweenIterationsMs > 0) {
            await new Promise((resolve) =>
              setTimeout(resolve, loopConfig.delayBetweenIterationsMs)
            );
          }
        }
      } catch (error) {
        console.error(`❌ Error in iteration ${iteration}:`, error);

        return {
          success: false,
          output: lastOutput,
          iterations: iteration,
          feedbackHistory,
          report: lastReport || this.createEmptyReport(),
          agentInteractions: this.interactions,
          totalExecutionTime: Date.now() - startTime,
          error:
            error instanceof Error
              ? error.message
              : "Unknown error in iteration",
        };
      }
    }

    // Max iterations reached without success
    console.log(
      `⚠️  Max iterations (${loopConfig.maxIterations}) reached without passing`
    );

    return {
      success: false,
      output: lastOutput,
      iterations: iteration,
      feedbackHistory,
      report: lastReport || this.createEmptyReport(),
      agentInteractions: this.interactions,
      totalExecutionTime: Date.now() - startTime,
      error: `Failed to pass after ${loopConfig.maxIterations} iterations`,
    };
  }

  /**
   * Generate actionable feedback from failed criteria
   *
   * Subclasses can override to customize feedback generation
   *
   * @param report Evaluation report with failed criteria
   * @param output The output that was evaluated
   * @returns Actionable feedback string
   */
  protected generateFeedback(
    report: EvaluationReport,
    output: TOutput
  ): string {
    const failedCriteria = report.criteria.filter((c) => !c.passed);

    if (failedCriteria.length === 0) {
      return "All criteria passed, but overall score is below threshold. Please improve quality.";
    }

    const feedbackParts: string[] = [
      `The output needs improvement in ${failedCriteria.length} area${
        failedCriteria.length > 1 ? "s" : ""
      }:`,
    ];

    failedCriteria.forEach((criterion, index) => {
      feedbackParts.push(
        `${index + 1}. ${criterion.name} (Score: ${(
          criterion.score * 100
        ).toFixed(1)}%, Required: ${(criterion.threshold * 100).toFixed(0)}%)`
      );
      feedbackParts.push(`   Issue: ${criterion.reasoning}`);

      // Add specific suggestions based on criterion type
      const suggestion = this.getSuggestionForCriterion(criterion, output);
      if (suggestion) {
        feedbackParts.push(`   Suggestion: ${suggestion}`);
      }
    });

    return feedbackParts.join("\n");
  }

  /**
   * Get specific suggestion for a failed criterion
   *
   * Subclasses can override to provide domain-specific suggestions
   */
  protected getSuggestionForCriterion(
    criterion: import("../types/evaluation").CriterionResult,
    output: TOutput
  ): string | null {
    // Base implementation - subclasses should override
    return null;
  }

  /**
   * Track an agent interaction
   */
  protected trackInteraction(interaction: AgentInteraction): void {
    this.interactions.push(interaction);
  }

  /**
   * Get all tracked interactions
   */
  public getInteractions(): AgentInteraction[] {
    return [...this.interactions];
  }

  /**
   * Calculate test statistics
   */
  public calculateStatistics(result: TestResult): TestStatistics {
    const generationTime = this.interactions
      .filter((i) => i.type === "generation")
      .reduce((sum, i) => sum + (i.duration || 0), 0);

    const evaluationTime = this.interactions
      .filter((i) => i.type === "evaluation")
      .reduce((sum, i) => sum + (i.duration || 0), 0);

    const toolCalls = this.interactions.filter(
      (i) => i.type === "tool_call"
    ).length;

    const evaluations = this.interactions.filter(
      (i) => i.type === "evaluation"
    ).length;

    // Calculate score improvement
    const evaluationScores = this.interactions
      .filter((i) => i.type === "evaluation")
      .map((i) => i.details.overallScore as number)
      .filter((s) => typeof s === "number");

    const firstScore = evaluationScores[0] || 0;
    const lastScore = evaluationScores[evaluationScores.length - 1] || 0;
    const averageScore =
      evaluationScores.length > 0
        ? evaluationScores.reduce((sum, s) => sum + s, 0) /
          evaluationScores.length
        : 0;

    return {
      totalIterations: result.iterations,
      totalGenerationTimeMs: generationTime,
      totalEvaluationTimeMs: evaluationTime,
      totalTestTimeMs: result.totalExecutionTime,
      totalToolCalls: toolCalls,
      totalEvaluations: evaluations,
      averageScore,
      scoreImprovement: lastScore - firstScore,
    };
  }

  /**
   * Helper to add timeout to any promise
   */
  private async withTimeout<T>(
    promise: Promise<T>,
    timeoutMs: number,
    errorMessage: string
  ): Promise<T> {
    return Promise.race([
      promise,
      new Promise<T>((_, reject) =>
        setTimeout(() => reject(new Error(errorMessage)), timeoutMs)
      ),
    ]);
  }

  /**
   * Create an empty evaluation report (for error cases)
   */
  private createEmptyReport(): EvaluationReport {
    return {
      overallScore: 0,
      overallPassed: false,
      criteria: [],
      executionTime: 0,
      metadata: {},
    };
  }

  /**
   * Helper to evaluate multiple criteria and aggregate results
   */
  protected async evaluateCriteria(
    output: TOutput,
    criteria: EvaluationCriterion[],
    context: Record<string, unknown> = {}
  ): Promise<EvaluationReport> {
    const startTime = Date.now();

    // Evaluate all criteria in parallel
    const results = await Promise.all(
      criteria.map((criterion) => criterion.evaluate(output, context))
    );

    // Calculate weighted average score
    const totalWeight = results.reduce(
      (sum, _, i) => sum + (criteria[i].weight || 1),
      0
    );
    const overallScore =
      results.reduce(
        (sum, result, i) => sum + result.score * (criteria[i].weight || 1),
        0
      ) / totalWeight;

    return {
      overallScore,
      overallPassed: results.every((r) => r.passed),
      criteria: results,
      executionTime: Date.now() - startTime,
      metadata: {
        criteriaCount: criteria.length,
        passedCount: results.filter((r) => r.passed).length,
      },
    };
  }

  /**
   * Log a formatted test result summary
   */
  public logTestSummary(result: TestResult): void {
    console.log("\n" + "=".repeat(60));
    console.log("📊 Test Summary");
    console.log("=".repeat(60));

    console.log(`\n✅ Success: ${result.success ? "PASSED" : "FAILED"}`);
    console.log(
      `📈 Final Score: ${(result.report.overallScore * 100).toFixed(1)}%`
    );
    console.log(`🔄 Iterations: ${result.iterations}`);
    console.log(
      `⏱️  Total Time: ${(result.totalExecutionTime / 1000).toFixed(2)}s`
    );
    console.log(`💬 Agent Interactions: ${result.agentInteractions.length}`);

    console.log("\n🔍 Criteria Results:");
    result.report.criteria.forEach((criterion) => {
      const status = criterion.passed ? "✅" : "❌";
      console.log(
        `${status} ${criterion.name}: ${(criterion.score * 100).toFixed(
          1
        )}% (Threshold: ${(criterion.threshold * 100).toFixed(0)}%)`
      );
      if (!criterion.passed) {
        console.log(`   💡 ${criterion.reasoning}`);
      }
    });

    if (result.feedbackHistory.length > 0) {
      console.log("\n📝 Feedback History:");
      result.feedbackHistory.forEach((feedback, index) => {
        console.log(`\nIteration ${index + 1}:`);
        console.log(
          feedback
            .split("\n")
            .map((line) => `   ${line}`)
            .join("\n")
        );
      });
    }

    const stats = this.calculateStatistics(result);
    console.log("\n📊 Statistics:");
    console.log(
      `   Generation Time: ${(stats.totalGenerationTimeMs / 1000).toFixed(2)}s`
    );
    console.log(
      `   Evaluation Time: ${(stats.totalEvaluationTimeMs / 1000).toFixed(2)}s`
    );
    console.log(`   Tool Calls: ${stats.totalToolCalls}`);
    console.log(`   Evaluations: ${stats.totalEvaluations}`);
    console.log(`   Average Score: ${(stats.averageScore * 100).toFixed(1)}%`);
    console.log(
      `   Score Improvement: ${(stats.scoreImprovement * 100).toFixed(1)}%`
    );

    console.log("\n" + "=".repeat(60));
  }
}
