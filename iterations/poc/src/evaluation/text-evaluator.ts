/**
 * Text Transformation Evaluator
 *
 * @author @darianrosebrook
 * @description Evaluates text transformation tasks (rewrite, formalize, etc.)
 */

import fs from "node:fs";
import path from "node:path";
import { EvalCriterion, EvalReport } from "./types";

interface TextEvalConfig {
  style: "concise" | "formal" | "neutral";
  maxChars?: number;
  minChars?: number;
  bannedPhrases?: string[];
  requiredPhrases?: string[];
  readingGradeMax?: number; // optional: rough heuristic
}

function fleschKincaidApprox(text: string): number {
  // Lightweight heuristic: sentences, words, syllables (very rough)
  const sentences = Math.max(1, text.match(/[.!?]+/g)?.length ?? 1);
  const words = Math.max(1, text.trim().split(/\s+/g).length);
  const syllables = Math.max(
    1,
    text.match(/[aeiouy]+/gi)?.length ?? Math.ceil(words * 1.3)
  );
  // Flesch-Kincaid Grade (approximate)
  return 0.39 * (words / sentences) + 11.8 * (syllables / words) - 15.59;
}

function scoreBoolean(passed: boolean) {
  return passed ? 1 : 0;
}

export async function evaluateText(params: {
  taskId: string;
  artifactPath: string; // path to .txt or .md produced by the agent
  iterations: number;
  config: TextEvalConfig;
  acceptance: { minScore: number; mandatoryGates: string[] };
}): Promise<EvalReport> {
  const { taskId, artifactPath, iterations, config, acceptance } = params;
  const raw = fs.readFileSync(artifactPath, "utf8");
  const text = raw.trim();

  const criteria: EvalCriterion[] = [];

  // C1: length band
  const withinMax = config.maxChars ? text.length <= config.maxChars : true;
  const withinMin = config.minChars ? text.length >= config.minChars : true;
  criteria.push({
    id: "length-band",
    description: "Text length within target bounds",
    weight: 0.15,
    passed: withinMax && withinMin,
    score: scoreBoolean(withinMax && withinMin),
    notes: `len=${text.length}`,
  });

  // C2: banned phrases
  const banned = config.bannedPhrases ?? [];
  const bannedHits = banned.filter((p) =>
    text.toLowerCase().includes(p.toLowerCase())
  );
  criteria.push({
    id: "no-banned-phrases",
    description: "Avoid banned phrases",
    weight: 0.15,
    passed: bannedHits.length === 0,
    score: scoreBoolean(bannedHits.length === 0),
    notes: bannedHits.length ? `hits=${bannedHits.join(",")}` : undefined,
  });

  // C3: required phrases
  const required = config.requiredPhrases ?? [];
  const missingReq = required.filter(
    (p) => !text.toLowerCase().includes(p.toLowerCase())
  );
  criteria.push({
    id: "required-phrases",
    description: "Include required phrases",
    weight: 0.15,
    passed: missingReq.length === 0,
    score: scoreBoolean(missingReq.length === 0),
    notes: missingReq.length ? `missing=${missingReq.join(",")}` : undefined,
  });

  // C4: readability ceiling (optional)
  let grade = 0;
  let readabilityPass = true;
  if (config.readingGradeMax != null) {
    grade = fleschKincaidApprox(text);
    readabilityPass = grade <= config.readingGradeMax;
  }
  criteria.push({
    id: "readability",
    description: "Reading grade at or below max",
    weight: 0.1,
    passed: readabilityPass,
    score: scoreBoolean(readabilityPass),
    notes: `grade≈${grade.toFixed(1)}`,
  });

  // C5: style heuristic (very light)
  const style = config.style;
  const hasContractions = /n't|'re|'s|'d|'ll|'ve|'m\b/.test(text);
  const stylePass = style === "formal" ? !hasContractions : true;
  criteria.push({
    id: "style-heuristic",
    description: `Style conforms (${style})`,
    weight: 0.1,
    passed: stylePass,
    score: scoreBoolean(stylePass),
    notes: `contractions=${hasContractions}`,
  });

  // C6: structure markers (paragraphs, headings)
  const paras = text.split(/\n{2,}/g).length;
  const structurePass = paras >= 1;
  criteria.push({
    id: "structure",
    description: "Basic structure present (≥1 paragraph)",
    weight: 0.1,
    passed: structurePass,
    score: scoreBoolean(structurePass),
    notes: `paragraphs=${paras}`,
  });

  // "Mandatory gates" can be mapped to criteria IDs for text tasks if you wish
  const scores = criteria.reduce((s, c) => s + c.score * c.weight, 0);
  const thresholdsMet: string[] = [];
  const thresholdsMissed: string[] = [];

  // Example: treat readability & no-banned as gates if listed
  for (const g of acceptance.mandatoryGates) {
    const crit = criteria.find((c) => c.id === g);
    if (!crit) continue;
    (crit.passed ? thresholdsMet : thresholdsMissed).push(g);
  }

  const passCore =
    scores >= acceptance.minScore && thresholdsMissed.length === 0;
  const report: EvalReport = {
    taskId,
    artifactPaths: [path.resolve(artifactPath)],
    status: passCore ? "pass" : "iterate",
    score: Number(scores.toFixed(3)),
    thresholdsMet,
    thresholdsMissed,
    criteria,
    iterations,
    stopReason: passCore ? "satisficed" : undefined,
    nextActions: passCore ? [] : ["Tighten style or fix gates, then re-run."],
    timestamp: new Date().toISOString(),
    logs: [],
  };

  return report;
}
