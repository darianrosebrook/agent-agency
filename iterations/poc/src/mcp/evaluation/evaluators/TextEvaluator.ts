/**
 * Text Evaluator for Content Quality Assessment
 *
 * @author @darianrosebrook
 * @description Evaluates text quality and adherence to requirements
 */

import * as fs from "fs";
import { Logger } from "../../../utils/Logger.js";
import {
  BaseEvaluator,
  EvalCriterion,
  EvaluationParams,
  EvaluationReport,
} from "../EvaluationOrchestrator.js";

export interface TextEvaluationConfig {
  style?: "concise" | "formal" | "neutral";
  maxChars?: number;
  minChars?: number;
  bannedPhrases?: string[];
  requiredPhrases?: string[];
  readingGradeMax?: number;
  allowContractions?: boolean;
}

export class TextEvaluator extends BaseEvaluator {
  constructor(private logger: Logger) {
    super();
  }

  async evaluate(params: EvaluationParams): Promise<EvaluationReport> {
    const { taskId, artifactPath, iterations, acceptance, config } = params;
    const textConfig = config as TextEvaluationConfig | undefined;

    this.logger.debug(`Evaluating text for task ${taskId} at ${artifactPath}`);

    // Read the text file
    let text: string;
    try {
      text = fs.readFileSync(artifactPath, "utf8").trim();
    } catch (error) {
      throw new Error(`Failed to read text file: ${artifactPath}`);
    }

    const criteria: EvalCriterion[] = [];

    // Length constraints
    const withinMax = textConfig?.maxChars
      ? text.length <= textConfig.maxChars
      : true;
    const withinMin = textConfig?.minChars
      ? text.length >= textConfig.minChars
      : true;
    criteria.push({
      id: "length-band",
      description: "Text length within target bounds",
      weight: 0.15,
      passed: withinMax && withinMin,
      score: withinMax && withinMin ? 1 : 0,
      notes: `length=${text.length}${
        textConfig?.maxChars ? ` (max: ${textConfig.maxChars})` : ""
      }${textConfig?.minChars ? ` (min: ${textConfig.minChars})` : ""}`,
    });

    // Banned phrases
    const bannedPhrases = textConfig?.bannedPhrases || [];
    const bannedHits = bannedPhrases.filter((phrase) =>
      text.toLowerCase().includes(phrase.toLowerCase())
    );
    criteria.push({
      id: "no-banned-phrases",
      description: "Avoid banned phrases",
      weight: 0.15,
      passed: bannedHits.length === 0,
      score: bannedHits.length === 0 ? 1 : 0,
      notes: bannedHits.length ? `found: ${bannedHits.join(", ")}` : undefined,
    });

    // Required phrases
    const requiredPhrases = textConfig?.requiredPhrases || [];
    const missingReq = requiredPhrases.filter(
      (phrase) => !text.toLowerCase().includes(phrase.toLowerCase())
    );
    criteria.push({
      id: "required-phrases",
      description: "Include required phrases",
      weight: 0.15,
      passed: missingReq.length === 0,
      score: missingReq.length === 0 ? 1 : 0,
      notes: missingReq.length
        ? `missing: ${missingReq.join(", ")}`
        : undefined,
    });

    // Readability (optional)
    let readabilityPass = true;
    let grade = 0;
    if (textConfig?.readingGradeMax) {
      grade = this.calculateFleschKincaid(text);
      readabilityPass = grade <= textConfig.readingGradeMax;
    }
    criteria.push({
      id: "readability",
      description: "Reading grade at or below max",
      weight: 0.1,
      passed: readabilityPass,
      score: readabilityPass ? 1 : 0,
      notes: textConfig?.readingGradeMax
        ? `grade≈${grade.toFixed(1)} (max: ${textConfig.readingGradeMax})`
        : undefined,
    });

    // Style heuristics
    const style = textConfig?.style || "neutral";
    const allowContractions =
      textConfig?.allowContractions ?? style !== "formal";
    const hasContractions = /n't|'re|'s|'d|'ll|'ve|'m\b/.test(text);
    const contractionsOk = allowContractions || !hasContractions;

    criteria.push({
      id: "style-heuristic",
      description: `Style conforms (${style})`,
      weight: 0.1,
      passed: contractionsOk,
      score: contractionsOk ? 1 : 0,
      notes:
        hasContractions && !allowContractions
          ? "Contains contractions in formal context"
          : undefined,
    });

    // Structure markers (paragraphs, headings)
    const paras = text.split(/\n{2,}/g).length;
    const structurePass = paras >= 1;
    criteria.push({
      id: "structure",
      description: "Basic structure present (≥1 paragraph)",
      weight: 0.1,
      passed: structurePass,
      score: structurePass ? 1 : 0,
      notes: `paragraphs=${paras}`,
    });

    // Grammar and spelling (basic heuristics)
    const basicGrammarIssues = this.checkBasicGrammar(text);
    criteria.push({
      id: "basic-grammar",
      description: "No obvious grammar/spelling issues",
      weight: 0.1,
      passed: basicGrammarIssues.length === 0,
      score: basicGrammarIssues.length === 0 ? 1 : 0.5, // Partial credit for minor issues
      notes: basicGrammarIssues.length
        ? `issues: ${basicGrammarIssues.slice(0, 3).join(", ")}`
        : undefined,
    });

    // Language variety (sentence length variation)
    const sentenceVariation = this.calculateSentenceVariation(text);
    const varietyPass = sentenceVariation > 0.3; // Some variation is good
    criteria.push({
      id: "sentence-variation",
      description: "Good sentence length variation",
      weight: 0.1,
      passed: varietyPass,
      score: varietyPass ? 1 : 0,
      notes: `variation=${sentenceVariation.toFixed(2)}`,
    });

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
      nextActions: passCore
        ? []
        : this.generateImprovementActions(criteria, textConfig),
      timestamp: new Date().toISOString(),
    };

    this.logger.debug(
      `Text evaluation completed for ${taskId}: score=${report.score}, status=${report.status}`
    );
    return report;
  }

  private calculateFleschKincaid(text: string): number {
    const sentences = Math.max(1, text.match(/[.!?]+/g)?.length ?? 1);
    const words = Math.max(1, text.trim().split(/\s+/g).length);
    const syllables = Math.max(
      1,
      text.match(/[aeiouy]+/gi)?.length ?? Math.ceil(words * 1.3)
    );

    // Flesch-Kincaid Grade Level
    return 0.39 * (words / sentences) + 11.8 * (syllables / words) - 15.59;
  }

  private checkBasicGrammar(text: string): string[] {
    const issues: string[] = [];

    // Check for double spaces
    if (text.includes("  ")) {
      issues.push("double spaces");
    }

    // Check for multiple consecutive punctuation
    if (/[.!?]{2,}/.test(text)) {
      issues.push("multiple punctuation");
    }

    // Check for common spacing issues
    if (/\s+[.!?]/.test(text)) {
      issues.push("space before punctuation");
    }

    return issues;
  }

  private calculateSentenceVariation(text: string): number {
    const sentences = text.split(/[.!?]+/).filter((s) => s.trim().length > 0);
    if (sentences.length < 2) return 0;

    const lengths = sentences.map((s) => s.trim().split(/\s+/).length);
    const mean = lengths.reduce((a, b) => a + b, 0) / lengths.length;
    const variance =
      lengths.reduce((acc, len) => acc + Math.pow(len - mean, 2), 0) /
      lengths.length;
    const stdDev = Math.sqrt(variance);

    // Coefficient of variation (relative standard deviation)
    return mean > 0 ? stdDev / mean : 0;
  }

  private generateImprovementActions(
    criteria: EvalCriterion[],
    config?: TextEvaluationConfig
  ): string[] {
    const actions: string[] = [];

    const failedCriteria = criteria.filter((c) => !c.passed);

    for (const criterion of failedCriteria) {
      switch (criterion.id) {
        case "length-band":
          if (config?.maxChars && criterion.notes?.includes("length=")) {
            const currentLength = parseInt(
              criterion.notes.split(" ")[0].split("=")[1]
            );
            if (currentLength > config.maxChars) {
              actions.push(
                `Reduce text length from ${currentLength} to under ${config.maxChars} characters`
              );
            }
          }
          if (config?.minChars && criterion.notes?.includes("length=")) {
            const currentLength = parseInt(
              criterion.notes.split(" ")[0].split("=")[1]
            );
            if (currentLength < config.minChars) {
              actions.push(
                `Increase text length from ${currentLength} to at least ${config.minChars} characters`
              );
            }
          }
          break;
        case "no-banned-phrases":
          actions.push("Remove or replace banned phrases from the text");
          break;
        case "required-phrases":
          actions.push("Add missing required phrases to the text");
          break;
        case "readability":
          actions.push("Simplify language to improve readability score");
          break;
        case "style-heuristic":
          actions.push(
            `Adjust writing style to match ${
              config?.style || "neutral"
            } requirements`
          );
          break;
        case "structure":
          actions.push("Add paragraph breaks and improve text structure");
          break;
        case "basic-grammar":
          actions.push("Fix grammar and punctuation issues");
          break;
        case "sentence-variation":
          actions.push("Vary sentence lengths for better readability");
          break;
        default:
          actions.push(`Address ${criterion.description.toLowerCase()}`);
      }
    }

    if (actions.length === 0) {
      actions.push("Review content and improve overall text quality");
    }

    return actions;
  }
}
