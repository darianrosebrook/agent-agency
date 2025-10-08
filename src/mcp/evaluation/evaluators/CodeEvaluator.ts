/**
 * Code Evaluator for Quality Assessment
 *
 * @author @darianrosebrook
 * @description Evaluates code quality with automated testing and linting
 */

import { spawn } from "child_process";
import { Logger } from "../../../utils/Logger.js";
import {
  BaseEvaluator,
  EvalCriterion,
  EvaluationParams,
  EvaluationReport,
} from "../EvaluationOrchestrator.js";

export interface CodeEvaluationConfig {
  scripts?: {
    test?: string;
    lint?: string;
    typecheck?: string;
    coverage?: string;
  };
  coverageThreshold?: number;
  failOnAnyError?: boolean;
}

export class CodeEvaluator extends BaseEvaluator {
  constructor(private logger: Logger) {
    super();
  }

  async evaluate(params: EvaluationParams): Promise<EvaluationReport> {
    const { taskId, artifactPath, iterations, acceptance, config } = params;
    const codeConfig = config as CodeEvaluationConfig | undefined;

    this.logger.debug(`Evaluating code for task ${taskId} at ${artifactPath}`);

    const criteria: EvalCriterion[] = [];
    const results: Record<string, { code: number; out: string }> = {};

    // Run tests
    const testScript = codeConfig?.scripts?.test || "npm run test --silent";
    results.tests = await this.runScript("tests", testScript, artifactPath);

    // Run linting
    const lintScript = codeConfig?.scripts?.lint || "npm run lint --silent";
    results.lint = await this.runScript("lint", lintScript, artifactPath);

    // Run type checking
    const typeScript =
      codeConfig?.scripts?.typecheck || "npm run typecheck --silent";
    results.typecheck = await this.runScript(
      "typecheck",
      typeScript,
      artifactPath
    );

    // Optional: Run coverage if configured
    if (codeConfig?.scripts?.coverage) {
      results.coverage = await this.runScript(
        "coverage",
        codeConfig.scripts.coverage,
        artifactPath
      );
    }

    // Create evaluation criteria
    criteria.push(
      this.createGateCriterion(
        "tests-pass",
        "Unit/integration tests pass",
        0.4,
        results.tests
      )
    );

    criteria.push(
      this.createGateCriterion(
        "lint-clean",
        "Linting passes with no errors",
        0.25,
        results.lint
      )
    );

    criteria.push(
      this.createGateCriterion(
        "types-ok",
        "Type checking passes",
        0.25,
        results.typecheck
      )
    );

    // Coverage criterion (optional)
    if (results.coverage) {
      const coverageThreshold = codeConfig?.coverageThreshold || 80;
      const coveragePassed = results.coverage.code === 0;
      criteria.push({
        id: "coverage-sufficient",
        description: `Code coverage meets ${coverageThreshold}% threshold`,
        weight: 0.1,
        passed: coveragePassed,
        score: coveragePassed ? 1 : 0,
        notes: coveragePassed ? undefined : "Coverage requirements not met",
      });
    }

    const score = criteria.reduce((s, c) => s + c.score * c.weight, 0);
    const thresholdsMissed: string[] = [];
    const thresholdsMet: string[] = [];

    for (const gate of acceptance.mandatoryGates) {
      const crit = criteria.find((c) => c.id === gate);
      if (crit) {
        (crit.passed ? thresholdsMet : thresholdsMissed).push(gate);
      }
    }

    const passCore =
      score >= acceptance.minScore && thresholdsMissed.length === 0;

    const report: EvaluationReport = {
      taskId,
      artifactPaths: [artifactPath],
      status: passCore ? "pass" : "iterate",
      score: Number(score.toFixed(3)),
      thresholdsMet,
      thresholdsMissed,
      criteria,
      iterations,
      stopReason: passCore ? "satisficed" : undefined,
      nextActions: passCore ? [] : this.generateImprovementActions(criteria),
      logs: Object.entries(results).map(
        ([name, result]) => `${name}: ${result.out.slice(0, 500)}`
      ),
      timestamp: new Date().toISOString(),
    };

    this.logger.debug(
      `Code evaluation completed for ${taskId}: score=${report.score}, status=${report.status}`
    );
    return report;
  }

  private async runScript(
    name: string,
    command: string,
    cwd: string
  ): Promise<{ code: number; out: string }> {
    return new Promise((resolve) => {
      const [cmd, ...args] = command.split(" ");
      const child = spawn(cmd, args, {
        cwd,
        stdio: ["ignore", "pipe", "pipe"],
        shell: true,
      });

      let stdout = "";
      let stderr = "";

      child.stdout?.on("data", (data) => {
        stdout += data.toString();
      });
      child.stderr?.on("data", (data) => {
        stderr += data.toString();
      });

      child.on("close", (code) => {
        resolve({
          code: code ?? 1,
          out: stdout + stderr,
        });
      });

      child.on("error", (error) => {
        resolve({
          code: 1,
          out: `Error: ${error.message}`,
        });
      });
    });
  }

  private createGateCriterion(
    id: string,
    description: string,
    weight: number,
    result: { code: number; out: string }
  ): EvalCriterion {
    const passed = result.code === 0;
    return {
      id,
      description,
      weight,
      passed,
      score: passed ? 1 : 0,
      notes: passed ? undefined : this.truncateOutput(result.out),
    };
  }

  private truncateOutput(output: string, maxLength: number = 1200): string {
    return output.length > maxLength
      ? output.slice(0, maxLength) + " …[truncated]"
      : output;
  }

  private generateImprovementActions(criteria: EvalCriterion[]): string[] {
    const actions: string[] = [];

    const failedCriteria = criteria.filter((c) => !c.passed);

    for (const criterion of failedCriteria) {
      switch (criterion.id) {
        case "tests-pass":
          actions.push("Fix failing tests and ensure all test suites pass");
          break;
        case "lint-clean":
          actions.push("Fix linting errors and warnings");
          break;
        case "types-ok":
          actions.push("Fix TypeScript compilation errors");
          break;
        case "coverage-sufficient":
          actions.push("Improve code coverage by adding missing tests");
          break;
        default:
          actions.push(`Address ${criterion.description.toLowerCase()}`);
      }
    }

    if (actions.length === 0) {
      actions.push(
        "Review evaluation criteria and improve overall code quality"
      );
    }

    return actions;
  }
}
