/**
 * Design Token Application Evaluator
 *
 * @author @darianrosebrook
 * @description Evaluates design token usage in UI code
 */

import fs from "node:fs";
import path from "node:path";
import { EvalCriterion, EvalReport } from "./types";

interface TokenRegistry {
  colors: Record<string, string>; // e.g., { "bg.default": "{palette.gray.50}" }
  space: Record<string, string>;
  radius: Record<string, string>;
  // ...
}

export async function evaluateTokens(params: {
  taskId: string;
  artifactPath: string; // css/tsx/html to scan
  tokenJsonPath: string; // registry (design-tokens.json)
  iterations: number;
  acceptance: { minScore: number; mandatoryGates: string[] };
}): Promise<EvalReport> {
  const { taskId, artifactPath, tokenJsonPath, iterations, acceptance } =
    params;
  const text = fs.readFileSync(artifactPath, "utf8");
  const reg: TokenRegistry = JSON.parse(fs.readFileSync(tokenJsonPath, "utf8"));

  const criteria: EvalCriterion[] = [];

  // C1: No hard-coded hex
  const hexHits = text.match(/#[0-9a-f]{3,8}\b/gi) ?? [];
  const c1Pass = hexHits.length === 0;
  criteria.push({
    id: "no-hardcoded-hex",
    description: "No raw hex colors (use tokens/vars)",
    weight: 0.35,
    passed: c1Pass,
    score: c1Pass ? 1 : 0,
    notes: c1Pass
      ? undefined
      : `hex=${[...new Set(hexHits)].slice(0, 10).join(", ")}`,
  });

  // C2: No raw pixel spacing (encourage tokens/variables)
  const pxHits = text.match(/(?<!font-)\b\d+px\b/g) ?? [];
  const c2Pass = pxHits.length <= 2; // allow a tiny tail
  criteria.push({
    id: "no-raw-px-spacing",
    description: "Avoid raw px in spacing (use space tokens/vars)",
    weight: 0.25,
    passed: c2Pass,
    score: c2Pass ? 1 : 0,
    notes: c2Pass
      ? undefined
      : `px=${[...new Set(pxHits)].slice(0, 10).join(", ")}`,
  });

  // C3: Token coverage (presence of known tokens)
  const tokenKeys = [
    ...Object.keys(reg.colors || {}),
    ...Object.keys(reg.space || {}),
    ...Object.keys(reg.radius || {}),
  ];
  const usedTokens = tokenKeys.filter((k) => text.includes(k));
  const coverage = usedTokens.length / Math.max(1, tokenKeys.length);
  const c3Pass = coverage >= 0.1; // low bar: ensure tokens appear
  criteria.push({
    id: "token-coverage",
    description: "Some token usage present",
    weight: 0.15,
    passed: c3Pass,
    score: c3Pass ? 1 : 0,
    notes: `coverage≈${coverage.toFixed(2)} used=${usedTokens
      .slice(0, 10)
      .join(", ")}`,
  });

  // C4: Disallow ad-hoc color names (heuristic)
  const adHocColorNames =
    text.match(/\b(color|bg|background|fill|stroke)[-_:]\w+\b/g) ?? [];
  const c4Pass = adHocColorNames.filter((n) => !n.includes(".")).length === 0;
  criteria.push({
    id: "no-ad-hoc-color-names",
    description: "No ad-hoc color names; use semantic tokens",
    weight: 0.25,
    passed: c4Pass,
    score: c4Pass ? 1 : 0,
    notes: c4Pass ? undefined : adHocColorNames.slice(0, 10).join(", "),
  });

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

  const report: EvalReport = {
    taskId,
    artifactPaths: [path.resolve(artifactPath)],
    status: passCore ? "pass" : "iterate",
    score: Number(score.toFixed(3)),
    thresholdsMet,
    thresholdsMissed,
    criteria,
    iterations,
    stopReason: passCore ? "satisficed" : undefined,
    timestamp: new Date().toISOString(),
  };

  return report;
}
