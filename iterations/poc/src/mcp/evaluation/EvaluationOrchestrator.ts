/**
 * Evaluation Orchestrator for Autonomous Reasoning
 *
 * @author @darianrosebrook
 * @description Orchestrates autonomous evaluation loops with satisficing logic
 */

import { TaskType } from "../../types/index.js";
import { Logger } from "../../utils/Logger.js";
import { CodeEvaluator } from "./evaluators/CodeEvaluator.js";
import { DesignEvaluator } from "./evaluators/DesignEvaluator.js";
import { TextEvaluator } from "./evaluators/TextEvaluator.js";

export interface EvaluationConfig {
  minScore: number;
  mandatoryGates: string[];
  iterationPolicy: {
    maxIterations: number;
    minDeltaToContinue: number;
    noChangeBudget: number;
  };
}

export interface EvaluationReport {
  taskId: string;
  artifactPaths: string[];
  status: "pass" | "iterate" | "fail";
  score: number;
  thresholdsMet: string[];
  thresholdsMissed: string[];
  criteria: EvalCriterion[];
  iterations: number;
  stopReason?:
    | "satisficed"
    | "max-iterations"
    | "quality-ceiling"
    | "failed-gates";
  nextActions?: string[];
  logs?: string[];
  timestamp: string;
}

export interface EvalCriterion {
  id: string;
  description: string;
  weight: number;
  passed: boolean;
  score: number;
  notes?: string;
}

export interface EvaluationParams {
  taskId: string;
  artifactPath: string;
  iterations: number;
  acceptance: {
    minScore: number;
    mandatoryGates: string[];
  };
  config?: any;
}

export interface EvaluationLoopResult {
  taskId: string;
  evaluations: EvaluationReport[];
  finalEvaluation: EvaluationReport;
  iterationsCompleted: number;
  successful: boolean;
  satisficed: boolean;
  message?: string;
}

export abstract class BaseEvaluator {
  abstract evaluate(params: EvaluationParams): Promise<EvaluationReport>;
}

export class EvaluationOrchestrator {
  private readonly config: EvaluationConfig;
  private readonly logger: Logger;
  private readonly evaluators: Map<TaskType | string, BaseEvaluator>;

  constructor(config: EvaluationConfig, logger: Logger) {
    this.config = config;
    this.logger = logger;

    // Initialize evaluators
    this.evaluators = new Map();
    this.evaluators.set("code", new CodeEvaluator(logger));
    this.evaluators.set("text", new TextEvaluator(logger));
    this.evaluators.set("design", new DesignEvaluator(logger));
  }

  /**
   * Evaluate a single task
   */
  async evaluateTask(
    taskId: string,
    taskType: TaskType | "code" | "text" | "design",
    artifactPath: string,
    iteration: number,
    evaluationConfig?: any
  ): Promise<EvaluationReport> {
    const evaluator = this.evaluators.get(taskType);

    if (!evaluator) {
      throw new Error(`No evaluator available for task type: ${taskType}`);
    }

    this.logger.debug(
      `Evaluating ${taskType} task ${taskId} (iteration ${iteration})`
    );

    const report = await evaluator.evaluate({
      taskId,
      artifactPath,
      iterations: iteration,
      acceptance: {
        minScore: this.config.minScore,
        mandatoryGates: this.config.mandatoryGates,
      },
      config: evaluationConfig,
    });

    // Apply satisficing logic
    return this.enhanceReportWithSatisficing(report, iteration);
  }

  /**
   * Run a complete evaluation loop with satisficing
   */
  async runEvaluationLoop(
    taskId: string,
    taskType: TaskType | "code" | "text" | "design",
    artifactPath: string,
    maxIterations?: number,
    evaluationConfig?: any
  ): Promise<EvaluationLoopResult> {
    const maxIters = maxIterations || this.config.iterationPolicy.maxIterations;
    const evaluations: EvaluationReport[] = [];

    this.logger.info(
      `Starting evaluation loop for task ${taskId} (${taskType})`
    );

    for (let i = 1; i <= maxIters; i++) {
      const evaluation = await this.evaluateTask(
        taskId,
        taskType,
        artifactPath,
        i,
        evaluationConfig
      );

      evaluations.push(evaluation);

      // Check if we should stop
      if (evaluation.status === "pass" || evaluation.stopReason) {
        this.logger.info(
          `Evaluation loop stopped at iteration ${i}: ${
            evaluation.stopReason || "passed"
          }`
        );
        break;
      }

      // In a real implementation, you might modify the artifact here
      // and continue the loop based on evaluation feedback
    }

    const finalEvaluation = evaluations[evaluations.length - 1];
    const successful = finalEvaluation.status === "pass";
    const satisficed =
      successful && finalEvaluation.stopReason === "satisficed";

    const result: EvaluationLoopResult = {
      taskId,
      evaluations,
      finalEvaluation,
      iterationsCompleted: evaluations.length,
      successful,
      satisficed,
      message: successful
        ? `Evaluation completed successfully after ${evaluations.length} iterations`
        : `Evaluation failed after ${evaluations.length} iterations`,
    };

    this.logger.info(`Evaluation loop completed: ${result.message}`);
    return result;
  }

  /**
   * Check if evaluation should continue based on satisficing logic
   */
  shouldContinueEvaluating(
    currentScore: number,
    iteration: number,
    mandatoryGatesPassed: boolean
  ): boolean {
    // Check iteration limit
    if (iteration >= this.config.iterationPolicy.maxIterations) {
      return false;
    }

    // Check minimum score achieved with all mandatory gates
    if (currentScore >= this.config.minScore && mandatoryGatesPassed) {
      return false;
    }

    return true;
  }

  /**
   * Apply satisficing logic to evaluation report
   */
  private enhanceReportWithSatisficing(
    report: EvaluationReport,
    iteration: number
  ): EvaluationReport {
    const { maxIterations } = this.config.iterationPolicy;

    // Check iteration limits
    if (iteration >= maxIterations) {
      report.stopReason = "max-iterations";
      report.status = "fail";
      return report;
    }

    // Check satisficing conditions
    const allMandatoryGatesMet = this.config.mandatoryGates.every((gate) =>
      report.thresholdsMet.includes(gate)
    );

    if (report.score >= this.config.minScore && allMandatoryGatesMet) {
      report.stopReason = "satisficed";
      report.status = "pass";
      return report;
    }

    // Continue iterating
    report.status = "iterate";
    report.nextActions = [
      "Review evaluation feedback and improve the artifact",
      "Address any failed mandatory gates",
      "Consider refactoring based on evaluation criteria",
    ];

    return report;
  }

  /**
   * Get available evaluators
   */
  getAvailableEvaluators(): string[] {
    return Array.from(this.evaluators.keys());
  }

  /**
   * Get evaluation configuration
   */
  getConfig(): EvaluationConfig {
    return { ...this.config };
  }
}
