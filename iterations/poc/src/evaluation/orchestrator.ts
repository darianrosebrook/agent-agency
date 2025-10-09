/**
 * Evaluation Orchestrator
 *
 * @author @darianrosebrook
 * @description Coordinates evaluation runs and implements satisficing logic
 */

import fs from "node:fs";
import path from "node:path";
import { evaluateCode } from "./code-evaluator";
import { evaluateText } from "./text-evaluator";
import { evaluateTokens } from "./token-evaluator";
import { EvalReport } from "./types";

interface Acceptance {
  minScore: number;
  mandatoryGates: string[];
  iterationPolicy: {
    maxIterations: number;
    minDeltaToContinue: number;
    noChangeBudget: number;
  };
}

interface History {
  reports: EvalReport[];
  noChangeStreak: number;
}

function loadAcceptance(): Acceptance {
  return JSON.parse(
    fs.readFileSync(path.resolve("src/evaluation/acceptance.json"), "utf8")
  );
}

function delta(prev: EvalReport | undefined, curr: EvalReport) {
  if (!prev) return 1; // first run
  return curr.score - prev.score;
}

function decide(history: History, current: EvalReport, a: Acceptance) {
  const prev = history.reports.at(-1);
  const d = delta(prev, current);
  const iterations = current.iterations;

  let noChangeStreak = history.noChangeStreak;
  if (d < a.minDeltaToContinue) noChangeStreak += 1;
  else noChangeStreak = 0;

  if (current.status === "pass") {
    current.stopReason = "satisficed";
    return { decision: "stop", noChangeStreak };
  }

  if (iterations >= a.iterationPolicy.maxIterations) {
    current.stopReason = "max-iterations";
    return { decision: "stop", noChangeStreak };
  }

  if (noChangeStreak > a.iterationPolicy.noChangeBudget) {
    current.stopReason = "quality-ceiling";
    return { decision: "stop", noChangeStreak };
  }

  return { decision: "iterate", noChangeStreak };
}

export async function runEvaluation(
  task: string,
  iterations: number
): Promise<{ report: EvalReport; decision: string }> {
  const acceptance = loadAcceptance();
  const historyPath = path.resolve("src/evaluation/history.json");
  const hist: History = fs.existsSync(historyPath)
    ? JSON.parse(fs.readFileSync(historyPath, "utf8"))
    : { reports: [], noChangeStreak: 0 };

  let report: EvalReport;

  if (task === "text") {
    report = await evaluateText({
      taskId: "rewrite-para",
      artifactPath: "artifacts/out.txt",
      iterations,
      config: {
        style: "formal",
        maxChars: 800,
        bannedPhrases: ["very", "really"],
        requiredPhrases: ["acceptance criteria"],
      },
      acceptance,
    });
  } else if (task === "code") {
    report = await evaluateCode({
      taskId: "ui-button",
      projectDir: "./project",
      iterations,
      acceptance,
      scripts: {
        test: "npm run test",
        lint: "npm run lint",
        typecheck: "npm run typecheck",
      },
    });
  } else if (task === "tokens") {
    report = await evaluateTokens({
      taskId: "apply-design-tokens",
      artifactPath: "artifacts/Button.tsx",
      tokenJsonPath: "design-tokens.json",
      iterations,
      acceptance,
    });
  } else {
    throw new Error(`Unknown task '${task}'`);
  }

  // Decide next step
  const decision = decide(hist, report, acceptance);
  const newHist: History = {
    reports: [...hist.reports, report],
    noChangeStreak: decision.noChangeStreak,
  };
  fs.writeFileSync(historyPath, JSON.stringify(newHist, null, 2));

  return { report, decision: decision.decision };
}

// CLI interface
async function main() {
  const task = process.argv[2]; // e.g., "text", "code", "tokens"
  const iterations =
    (JSON.parse(
      fs.readFileSync(path.resolve("src/evaluation/history.json"), "utf8")
    ).reports.at(-1)?.iterations ?? 0) + 1;

  const result = await runEvaluation(task, iterations);

  // Emit final machine-readable result to stdout for the agent
  process.stdout.write(JSON.stringify(result, null, 2));
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
