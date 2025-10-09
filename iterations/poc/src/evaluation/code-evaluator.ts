/**
 * Code Evaluator
 *
 * @author @darianrosebrook
 * @description Evaluates code quality by running linting, testing, and type checking
 */

import { spawnSync } from "node:child_process";
import path from "node:path";
import { EvalCriterion, EvalReport } from "./types";

function run(cmd: string, args: string[], cwd: string) {
  const res = spawnSync(cmd, args, { cwd, encoding: "utf8" });
  return {
    code: res.status ?? 1,
    out: (res.stdout ?? "") + (res.stderr ?? ""),
  };
}

export async function evaluateCode(params: {
  taskId: string;
  projectDir: string;
  iterations: number;
  acceptance: { minScore: number; mandatoryGates: string[] };
  scripts?: { test?: string; lint?: string; typecheck?: string; a11y?: string };
}): Promise<EvalReport> {
  const { taskId, projectDir, iterations, acceptance, scripts } = params;

  const results: Record<string, { code: number; out: string }> = {};
  const criteria: EvalCriterion[] = [];

  const scriptMap = {
    tests: scripts?.test ?? "npm run test --silent",
    lint: scripts?.lint ?? "npm run lint --silent",
    typecheck: scripts?.typecheck ?? "npm run typecheck --silent",
    a11y: scripts?.a11y, // optional (axe, eslint-plugin-jsx-a11y, etc.)
  };

  function runScript(label: string, cmdLine?: string) {
    if (!cmdLine) return { code: 0, out: `${label}:skipped` };
    const [cmd, ...args] = cmdLine.split(/\s+/);
    return run(cmd, args, projectDir);
  }

  results.tests = runScript("tests", scriptMap.tests);
  results.lint = runScript("lint", scriptMap.lint);
  results.typecheck = runScript("typecheck", scriptMap.typecheck);
  if (scriptMap.a11y) results.a11y = runScript("a11y", scriptMap.a11y);

  function gateCriterion(
    id: string,
    description: string,
    weight: number,
    r?: { code: number; out: string }
  ) {
    if (!r) {
      return <EvalCriterion>{
        id,
        description: `${description} (skipped)`,
        weight,
        passed: true,
        score: 1,
      };
    }
    const passed = r.code === 0;
    return <EvalCriterion>{
      id,
      description,
      weight,
      passed,
      score: passed ? 1 : 0,
      notes: passed ? undefined : truncate(r.out),
    };
  }

  function truncate(s: string, n = 1200) {
    return s.length > n ? s.slice(0, n) + " …[truncated]" : s;
  }

  criteria.push(
    gateCriterion(
      "tests-pass",
      "Unit/integration tests pass",
      0.4,
      results.tests
    )
  );
  criteria.push(
    gateCriterion(
      "lint-clean",
      "Lint passes with no errors",
      0.25,
      results.lint
    )
  );
  criteria.push(
    gateCriterion("types-ok", "Typecheck passes", 0.25, results.typecheck)
  );
  if (results.a11y)
    criteria.push(
      gateCriterion("a11y-ok", "A11y checks pass", 0.1, results.a11y)
    );

  const score = criteria.reduce((s, c) => s + c.score * c.weight, 0);
  const thresholdsMissed: string[] = [];
  const thresholdsMet: string[] = [];

  for (const g of acceptance.mandatoryGates) {
    const crit = criteria.find((c) => c.id === g);
    if (!crit) continue;
    (crit.passed ? thresholdsMet : thresholdsMissed).push(g);
  }

  const passCore =
    score >= acceptance.minScore && thresholdsMissed.length === 0;

  const report = <EvalReport>{
    taskId,
    artifactPaths: [path.resolve(projectDir)],
    status: passCore ? "pass" : "iterate",
    score: Number(score.toFixed(3)),
    thresholdsMet,
    thresholdsMissed,
    criteria,
    iterations,
    stopReason: passCore ? "satisficed" : undefined,
    nextActions: passCore ? [] : ["Fix failing gates (see notes) and re-run."],
    logs: Object.values(results)
      .map((r) => r.out)
      .slice(0, 3),
    timestamp: new Date().toISOString(),
  };

  return report;
}
