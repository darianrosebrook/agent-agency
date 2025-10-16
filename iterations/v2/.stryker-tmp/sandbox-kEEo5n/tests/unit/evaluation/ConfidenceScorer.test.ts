/**
 * @fileoverview
 * Unit tests for ConfidenceScorer
 * Tests confidence calculation logic
 */
// @ts-nocheck


import { beforeEach, describe, expect, it } from "@jest/globals";
import { ConfidenceScorer } from "../../../src/evaluation/ConfidenceScorer";
import type { LLMResponse } from "../../../src/types/judge";

describe("ConfidenceScorer", () => {
  let scorer: ConfidenceScorer;

  beforeEach(() => {
    scorer = new ConfidenceScorer();
  });

  describe("Explicit Confidence", () => {
    it("should use explicit confidence when provided", () => {
      const response: LLMResponse = {
        criterion: "faithfulness",
        score: 0.8,
        confidence: 0.95,
        reasoning: "Test reasoning",
      };

      const confidence = scorer.calculateConfidence(response);

      expect(confidence).toBe(0.95);
    });

    it("should clamp explicit confidence to [0, 1]", () => {
      const response: LLMResponse = {
        criterion: "faithfulness",
        score: 0.8,
        confidence: 1.5,
        reasoning: "Test",
      };

      const confidence = scorer.calculateConfidence(response);

      expect(confidence).toBe(1.0);
    });
  });

  describe("Heuristic Confidence", () => {
    it("should calculate confidence from reasoning length", () => {
      const shortResponse: LLMResponse = {
        criterion: "faithfulness",
        score: 0.8,
        confidence: -1,
        reasoning: "Short",
      };

      const longResponse: LLMResponse = {
        criterion: "faithfulness",
        score: 0.8,
        confidence: -1,
        reasoning:
          "This is a much longer reasoning that demonstrates confidence because it provides detailed explanation and analysis",
      };

      const shortConf = scorer.calculateConfidence(shortResponse);
      const longConf = scorer.calculateConfidence(longResponse);

      expect(longConf).toBeGreaterThan(shortConf);
    });

    it("should factor in score extremity", () => {
      const middleScore: LLMResponse = {
        criterion: "faithfulness",
        score: 0.5,
        confidence: -1,
        reasoning: "Reasoning text here that is of moderate length",
      };

      const extremeScore: LLMResponse = {
        criterion: "faithfulness",
        score: 0.95,
        confidence: -1,
        reasoning: "Reasoning text here that is of moderate length",
      };

      const middleConf = scorer.calculateConfidence(middleScore);
      const extremeConf = scorer.calculateConfidence(extremeScore);

      expect(extremeConf).toBeGreaterThan(middleConf);
    });

    it("should detect quality indicators", () => {
      const withIndicators: LLMResponse = {
        criterion: "faithfulness",
        score: 0.8,
        confidence: -1,
        reasoning:
          "Output is correct because it clearly demonstrates the concept therefore",
      };

      const withoutIndicators: LLMResponse = {
        criterion: "faithfulness",
        score: 0.8,
        confidence: -1,
        reasoning: "Output looks good",
      };

      const withConf = scorer.calculateConfidence(withIndicators);
      const withoutConf = scorer.calculateConfidence(withoutIndicators);

      expect(withConf).toBeGreaterThan(withoutConf);
    });
  });

  describe("Confidence Aggregation", () => {
    it("should aggregate multiple confidences", () => {
      const confidences = [0.8, 0.9, 0.7];

      const aggregated = scorer.aggregateConfidences(confidences);

      expect(aggregated).toBeCloseTo(0.8, 5);
    });

    it("should handle empty array", () => {
      const aggregated = scorer.aggregateConfidences([]);

      expect(aggregated).toBe(0);
    });

    it("should handle single confidence", () => {
      const aggregated = scorer.aggregateConfidences([0.75]);

      expect(aggregated).toBe(0.75);
    });
  });

  describe("Confidence Bounds", () => {
    it("should never return confidence above 1.0", () => {
      const response: LLMResponse = {
        criterion: "faithfulness",
        score: 1.0,
        confidence: -1,
        reasoning:
          "This is an extremely long reasoning with many quality indicators because it demonstrates clear evidence and therefore specifically shows understanding",
      };

      const confidence = scorer.calculateConfidence(response);

      expect(confidence).toBeLessThanOrEqual(1.0);
    });

    it("should never return confidence below 0.0", () => {
      const response: LLMResponse = {
        criterion: "faithfulness",
        score: 0.5,
        confidence: -1,
        reasoning: "",
      };

      const confidence = scorer.calculateConfidence(response);

      expect(confidence).toBeGreaterThanOrEqual(0.0);
    });
  });
});
