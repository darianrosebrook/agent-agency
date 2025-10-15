/**
 * @fileoverview Test helper functions for verification tests
 *
 * Provides type-safe factory functions for creating test data
 *
 * @author @darianrosebrook
 */

import {
  Evidence,
  VerificationMethodResult,
  VerificationPriority,
  VerificationRequest,
  VerificationResult,
  VerificationType,
  VerificationVerdict,
} from "@/types/verification";
import { v4 as uuidv4 } from "uuid";

/**
 * Create a test verification request with defaults
 */
export function createTestRequest(
  overrides: Partial<VerificationRequest> = {}
): VerificationRequest {
  return {
    id: uuidv4(),
    content: "Test content for verification",
    source: "https://example.com",
    context: "Test context",
    priority: VerificationPriority.MEDIUM,
    timeoutMs: 30000,
    verificationTypes: [VerificationType.FACT_CHECKING],
    metadata: {},
    ...overrides,
  };
}

/**
 * Create a test verification result with defaults
 */
export function createTestResult(
  overrides: Partial<VerificationResult> = {}
): VerificationResult {
  return {
    requestId: uuidv4(),
    verdict: VerificationVerdict.VERIFIED_TRUE,
    confidence: 0.8,
    reasoning: ["Test verification passed through automated checks"],
    supportingEvidence: [],
    contradictoryEvidence: [],
    verificationMethods: [],
    processingTimeMs: 100,
    ...overrides,
  };
}

/**
 * Create test evidence with defaults
 */
export function createTestEvidence(
  overrides: Partial<Evidence> = {}
): Evidence {
  return {
    source: "https://example.com/source",
    content: "Test evidence content",
    relevance: 0.8,
    credibility: 0.7,
    supporting: true,
    verificationDate: new Date(),
    metadata: {
      type: "factual", // Set default evidence type
      ...overrides.metadata,
    },
    ...overrides,
  };
}

/**
 * Create test method result with defaults
 */
export function createTestMethodResult(
  overrides: Partial<VerificationMethodResult> = {}
): VerificationMethodResult {
  return {
    method: VerificationType.FACT_CHECKING,
    verdict: VerificationVerdict.VERIFIED_TRUE,
    confidence: 0.8,
    reasoning: "Test method verification passed",
    processingTimeMs: 50,
    evidenceCount: 1,
    ...overrides,
  };
}

/**
 * Create a complete verification result with evidence and method results
 */
export function createCompleteTestResult(
  requestId: string,
  verdict: VerificationVerdict = VerificationVerdict.VERIFIED_TRUE,
  evidenceCount = 3
): VerificationResult {
  const supportingEvidence: Evidence[] = Array.from(
    { length: evidenceCount },
    (_, i) =>
      createTestEvidence({
        source: `https://source${i + 1}.com`,
        content: `Supporting evidence ${i + 1}`,
        relevance: 0.9 - i * 0.1,
        credibility: 0.8 - i * 0.05,
        supporting: true,
      })
  );

  const methodResults: VerificationMethodResult[] = [
    createTestMethodResult({
      method: VerificationType.FACT_CHECKING,
      verdict,
      confidence: 0.85,
      evidenceCount: evidenceCount,
    }),
    createTestMethodResult({
      method: VerificationType.SOURCE_CREDIBILITY,
      verdict,
      confidence: 0.75,
      evidenceCount: evidenceCount,
    }),
  ];

  return createTestResult({
    requestId,
    verdict,
    confidence: 0.8,
    reasoning: [
      "Verified through fact-checking",
      "Source credibility confirmed",
      `${evidenceCount} pieces of supporting evidence found`,
    ],
    supportingEvidence,
    contradictoryEvidence: [],
    verificationMethods: methodResults,
    processingTimeMs: 250,
  });
}

/**
 * Create multiple test requests with varying priorities
 */
export function createTestRequests(
  count: number,
  baseId = "test"
): VerificationRequest[] {
  const priorities = [
    VerificationPriority.LOW,
    VerificationPriority.MEDIUM,
    VerificationPriority.HIGH,
    VerificationPriority.CRITICAL,
  ];

  return Array.from({ length: count }, (_, i) =>
    createTestRequest({
      id: uuidv4(), // Use UUID instead of baseId pattern
      content: `Test content ${i}`,
      source: `https://example${i}.com`,
      priority: priorities[i % priorities.length],
      verificationTypes: [
        i % 2 === 0
          ? VerificationType.FACT_CHECKING
          : VerificationType.CROSS_REFERENCE,
      ],
    })
  );
}

/**
 * Create multiple test results with varying verdicts
 */
export function createTestResults(
  requestIds: string[],
  verdictPattern: VerificationVerdict[] = [
    VerificationVerdict.VERIFIED_TRUE,
    VerificationVerdict.VERIFIED_FALSE,
  ]
): VerificationResult[] {
  return requestIds.map((requestId, i) =>
    createTestResult({
      requestId,
      verdict: verdictPattern[i % verdictPattern.length],
      confidence: 0.7 + (i % 3) * 0.1,
      reasoning: [`Result ${i} reasoning`],
      supportingEvidence: [
        createTestEvidence({
          source: `https://evidence${i}.com`,
          supporting:
            verdictPattern[i % verdictPattern.length] ===
            VerificationVerdict.VERIFIED_TRUE,
        }),
      ],
      verificationMethods: [
        createTestMethodResult({
          method: VerificationType.FACT_CHECKING,
          verdict: verdictPattern[i % verdictPattern.length],
          confidence: 0.75 + (i % 2) * 0.1,
        }),
      ],
      processingTimeMs: 100 + i * 10,
    })
  );
}

/**
 * Assert that a verification result has the expected structure
 */
export function assertValidVerificationResult(
  result: any
): asserts result is VerificationResult {
  if (!result) {
    throw new Error("Result is null or undefined");
  }
  if (typeof result.requestId !== "string") {
    throw new Error("Result missing requestId");
  }
  if (!result.verdict) {
    throw new Error("Result missing verdict");
  }
  if (typeof result.confidence !== "number") {
    throw new Error("Result missing confidence");
  }
  if (!Array.isArray(result.reasoning)) {
    throw new Error("Result missing reasoning array");
  }
  if (!Array.isArray(result.supportingEvidence)) {
    throw new Error("Result missing supportingEvidence array");
  }
  if (!Array.isArray(result.contradictoryEvidence)) {
    throw new Error("Result missing contradictoryEvidence array");
  }
  if (!Array.isArray(result.verificationMethods)) {
    throw new Error("Result missing verificationMethods array");
  }
  if (typeof result.processingTimeMs !== "number") {
    throw new Error("Result missing processingTimeMs");
  }
}

/**
 * Assert that evidence has the expected structure
 */
export function assertValidEvidence(
  evidence: any
): asserts evidence is Evidence {
  if (!evidence) {
    throw new Error("Evidence is null or undefined");
  }
  if (typeof evidence.source !== "string") {
    throw new Error("Evidence missing source");
  }
  if (typeof evidence.content !== "string") {
    throw new Error("Evidence missing content");
  }
  if (typeof evidence.relevance !== "number") {
    throw new Error("Evidence missing relevance");
  }
  if (typeof evidence.credibility !== "number") {
    throw new Error("Evidence missing credibility");
  }
  if (typeof evidence.supporting !== "boolean") {
    throw new Error("Evidence missing supporting boolean");
  }
  if (!(evidence.verificationDate instanceof Date)) {
    throw new Error("Evidence missing verificationDate");
  }
}
