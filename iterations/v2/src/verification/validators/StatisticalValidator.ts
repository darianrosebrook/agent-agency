/**
 * @fileoverview Statistical Validator (ARBITER-007)
 *
 * Validates statistical claims, detects data manipulation,
 * and checks for proper statistical reasoning.
 *
 * @author @darianrosebrook
 */

import {
  VerificationMethodResult,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "../../types/verification";

/**
 * Configuration for statistical validation
 */
export interface StatisticalConfig {
  statisticalTests: string[];
  minSampleSize: number;
  significanceLevel?: number;
}

/**
 * Statistical claim extracted from content
 */
interface StatisticalClaim {
  text: string;
  type:
    | "percentage"
    | "mean"
    | "median"
    | "correlation"
    | "sample"
    | "probability"
    | "rate";
  value?: number;
  sampleSize?: number;
  context: string;
  confidence?: number;
  pValue?: number;
}

/**
 * Statistical issue detected
 */
interface StatisticalIssue {
  type: string;
  severity: "high" | "medium" | "low";
  description: string;
  claim: string;
}

/**
 * Statistical Validator
 *
 * Validates statistical claims and detects common statistical errors.
 */
export class StatisticalValidator {
  private config: StatisticalConfig;

  constructor(config: Partial<StatisticalConfig> = {}) {
    this.config = {
      statisticalTests: config.statisticalTests ?? [
        "chi_square",
        "correlation",
        "significance",
      ],
      minSampleSize: config.minSampleSize ?? 30,
      significanceLevel: config.significanceLevel ?? 0.05,
    };
  }

  /**
   * Verify statistical claims in content
   */
  async verify(
    request: VerificationRequest
  ): Promise<VerificationMethodResult> {
    const startTime = Date.now();

    try {
      // Extract statistical claims
      const claims = this.extractStatisticalClaims(request.content);

      if (claims.length === 0) {
        return {
          method: VerificationType.STATISTICAL_VALIDATION,
          verdict: VerificationVerdict.INSUFFICIENT_DATA,
          confidence: 0,
          reasoning: ["No statistical claims found in content"],
          processingTimeMs: Date.now() - startTime,
          evidenceCount: 0,
        };
      }

      // Validate sample sizes
      const sampleSizeIssues = this.validateSampleSizes(claims);

      // Detect data manipulation signals
      const manipulationSignals = this.detectManipulation(claims);

      // Check correlation vs causation
      const correlationIssues = this.checkCorrelationVsCausation(
        request.content,
        claims
      );

      // Assess overall statistical validity
      const assessment = this.assessStatistics(
        claims,
        sampleSizeIssues,
        manipulationSignals,
        correlationIssues
      );

      return {
        method: VerificationType.STATISTICAL_VALIDATION,
        verdict: assessment.verdict,
        confidence: assessment.confidence,
        reasoning: assessment.reasoning,
        processingTimeMs: Date.now() - startTime,
        evidenceCount: claims.length,
      };
    } catch (error) {
      return {
        method: VerificationType.STATISTICAL_VALIDATION,
        verdict: VerificationVerdict.ERROR,
        confidence: 0,
        reasoning: [
          `Statistical validation failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
        ],
        processingTimeMs: Date.now() - startTime,
        evidenceCount: 0,
      };
    }
  }

  /**
   * Extract statistical claims from content
   */
  private extractStatisticalClaims(content: string): StatisticalClaim[] {
    const claims: StatisticalClaim[] = [];

    // Split into sentences
    const sentences = content
      .split(/[.!?]+/)
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    for (const sentence of sentences) {
      // Check for percentages
      const percentageMatch = sentence.match(/(\d+(?:\.\d+)?)\s*%/);
      if (percentageMatch) {
        claims.push({
          text: sentence,
          type: "percentage",
          value: parseFloat(percentageMatch[1]),
          context: this.extractContext(sentence),
        });
      }

      // Check for means/averages
      const meanMatch = sentence.match(/\b(average|mean)\b.*?(\d+(?:\.\d+)?)/i);
      if (meanMatch) {
        claims.push({
          text: sentence,
          type: "mean",
          value: parseFloat(meanMatch[2]),
          context: this.extractContext(sentence),
        });
      }

      // Check for medians
      const medianMatch = sentence.match(/\bmedian\b.*?(\d+(?:\.\d+)?)/i);
      if (medianMatch) {
        claims.push({
          text: sentence,
          type: "median",
          value: parseFloat(medianMatch[1]),
          context: this.extractContext(sentence),
        });
      }

      // Check for correlations
      const correlationMatch = sentence.match(
        /\b(correlation|correlated|associated)\b/i
      );
      if (correlationMatch) {
        const valueMatch = sentence.match(/(\d+(?:\.\d+)?)/);
        claims.push({
          text: sentence,
          type: "correlation",
          value: valueMatch ? parseFloat(valueMatch[1]) : undefined,
          context: this.extractContext(sentence),
        });
      }

      // Check for sample sizes
      const sampleMatch = sentence.match(
        /\b(n|sample|participants?|subjects?)\s*=?\s*(\d+)/i
      );
      if (sampleMatch) {
        claims.push({
          text: sentence,
          type: "sample",
          sampleSize: parseInt(sampleMatch[2], 10),
          context: this.extractContext(sentence),
        });
      }

      // Check for probabilities
      const probabilityMatch = sentence.match(
        /\b(probability|likelihood|chance)\b.*?(\d+(?:\.\d+)?)\s*%/i
      );
      if (probabilityMatch) {
        claims.push({
          text: sentence,
          type: "probability",
          value: parseFloat(probabilityMatch[2]),
          context: this.extractContext(sentence),
        });
      }

      // Check for rates
      const rateMatch = sentence.match(/\b(rate|ratio)\b.*?(\d+(?:\.\d+)?)/i);
      if (rateMatch) {
        claims.push({
          text: sentence,
          type: "rate",
          value: parseFloat(rateMatch[2]),
          context: this.extractContext(sentence),
        });
      }

      // Check for p-values
      const pValueMatch = sentence.match(/p\s*[<>=]\s*(\d+(?:\.\d+)?)/i);
      if (pValueMatch) {
        const existingClaim = claims[claims.length - 1];
        if (existingClaim) {
          existingClaim.pValue = parseFloat(pValueMatch[1]);
        }
      }

      // Check for confidence intervals
      const confidenceMatch = sentence.match(/(\d+)%\s*confidence/i);
      if (confidenceMatch) {
        const existingClaim = claims[claims.length - 1];
        if (existingClaim) {
          existingClaim.confidence = parseInt(confidenceMatch[1], 10);
        }
      }
    }

    return claims;
  }

  /**
   * Extract context keywords from sentence
   */
  private extractContext(sentence: string): string {
    const words = sentence
      .toLowerCase()
      .replace(/[^\w\s]/g, "")
      .split(/\s+/)
      .filter((w) => w.length > 3);

    return words.slice(0, 5).join(" ");
  }

  /**
   * Validate sample sizes
   */
  private validateSampleSizes(claims: StatisticalClaim[]): StatisticalIssue[] {
    const issues: StatisticalIssue[] = [];

    for (const claim of claims) {
      if (claim.type === "sample" && claim.sampleSize) {
        if (claim.sampleSize < this.config.minSampleSize) {
          issues.push({
            type: "Insufficient Sample Size",
            severity: "high",
            description: `Sample size (n=${claim.sampleSize}) is below recommended minimum of ${this.config.minSampleSize}`,
            claim: claim.text,
          });
        }

        // Check for suspiciously small samples with precise statistics
        if (claim.sampleSize < 10) {
          issues.push({
            type: "Very Small Sample",
            severity: "high",
            description: `Sample size (n=${claim.sampleSize}) is too small for reliable statistical inference`,
            claim: claim.text,
          });
        }
      }

      // Check for claims without reported sample sizes
      if (
        (claim.type === "percentage" ||
          claim.type === "mean" ||
          claim.type === "median") &&
        !claim.sampleSize
      ) {
        const nextSampleClaim = claims.find(
          (c) =>
            c.type === "sample" && claims.indexOf(c) > claims.indexOf(claim)
        );

        if (!nextSampleClaim) {
          issues.push({
            type: "Missing Sample Size",
            severity: "medium",
            description: "Statistical claim made without reporting sample size",
            claim: claim.text,
          });
        }
      }
    }

    return issues;
  }

  /**
   * Detect potential data manipulation
   */
  private detectManipulation(claims: StatisticalClaim[]): StatisticalIssue[] {
    const issues: StatisticalIssue[] = [];

    // Check for suspiciously precise values
    for (const claim of claims) {
      if (claim.value !== undefined) {
        const decimalPlaces = this.countDecimalPlaces(claim.value);

        if (decimalPlaces > 4) {
          issues.push({
            type: "Excessive Precision",
            severity: "low",
            description: `Value ${claim.value} reported with ${decimalPlaces} decimal places, may indicate false precision`,
            claim: claim.text,
          });
        }

        // Check for suspiciously round numbers
        if (
          claim.type === "percentage" &&
          claim.value % 5 === 0 &&
          claim.value !== 0 &&
          claim.value !== 100
        ) {
          issues.push({
            type: "Suspiciously Round Number",
            severity: "low",
            description: `Percentage value ${claim.value}% is a multiple of 5, which may indicate rounding or estimation`,
            claim: claim.text,
          });
        }
      }

      // Check for cherry-picking indicators
      if (
        /\b(selected|chosen|specific|particular|certain)\b/i.test(claim.text)
      ) {
        issues.push({
          type: "Potential Cherry-Picking",
          severity: "medium",
          description: "Language suggests selective reporting of data",
          claim: claim.text,
        });
      }
    }

    // Check for p-value fishing
    const pValueClaims = claims.filter((c) => c.pValue !== undefined);
    if (pValueClaims.length > 3) {
      issues.push({
        type: "Multiple P-Values",
        severity: "medium",
        description: `${pValueClaims.length} p-values reported, which may indicate p-hacking or multiple testing`,
        claim: `Multiple statistical tests: ${pValueClaims.length} p-values`,
      });
    }

    // Check for suspiciously significant results
    for (const claim of pValueClaims) {
      if (claim.pValue && claim.pValue < 0.001) {
        issues.push({
          type: "Extremely Significant Result",
          severity: "low",
          description: `P-value ${claim.pValue} is extremely small, verify this is not a data entry error`,
          claim: claim.text,
        });
      }
    }

    return issues;
  }

  /**
   * Count decimal places in a number
   */
  private countDecimalPlaces(value: number): number {
    const str = value.toString();
    const decimalIndex = str.indexOf(".");

    if (decimalIndex === -1) {
      return 0;
    }

    return str.length - decimalIndex - 1;
  }

  /**
   * Check for correlation vs causation issues
   */
  private checkCorrelationVsCausation(
    content: string,
    claims: StatisticalClaim[]
  ): StatisticalIssue[] {
    const issues: StatisticalIssue[] = [];

    // Check for causal language with correlation claims
    const correlationClaims = claims.filter((c) => c.type === "correlation");

    const causalPatterns = [
      /\b(causes?|caused by|leads? to|results? in|due to|because of)\b/i,
      /\b(effect of|impact of|influence of)\b/i,
      /\b(makes?|produces?|creates?)\b/i,
    ];

    for (const claim of correlationClaims) {
      for (const pattern of causalPatterns) {
        if (pattern.test(claim.text) || pattern.test(content)) {
          issues.push({
            type: "Correlation vs Causation",
            severity: "high",
            description:
              "Causal language used with correlation data - correlation does not imply causation",
            claim: claim.text,
          });
          break;
        }
      }
    }

    // Check for confounding variable warnings
    if (correlationClaims.length > 0) {
      const hasConfoundingMention =
        /\b(controlled for|adjusted for|accounted for|confounding)\b/i.test(
          content
        );

      if (!hasConfoundingMention && correlationClaims.length > 1) {
        issues.push({
          type: "Missing Confounding Discussion",
          severity: "medium",
          description:
            "Multiple correlations reported without discussing potential confounding variables",
          claim: "Multiple correlation claims",
        });
      }
    }

    return issues;
  }

  /**
   * Assess overall statistical validity
   */
  private assessStatistics(
    claims: StatisticalClaim[],
    sampleSizeIssues: StatisticalIssue[],
    manipulationSignals: StatisticalIssue[],
    correlationIssues: StatisticalIssue[]
  ): {
    verdict: VerificationVerdict;
    confidence: number;
    reasoning: string[];
  } {
    const reasoning: string[] = [];

    reasoning.push(`Found ${claims.length} statistical claim(s)`);

    // Count by type
    const typeCount: Record<string, number> = {};
    for (const claim of claims) {
      typeCount[claim.type] = (typeCount[claim.type] ?? 0) + 1;
    }

    reasoning.push(
      `Types: ${Object.entries(typeCount)
        .map(([type, count]) => `${count} ${type}`)
        .join(", ")}`
    );

    const allIssues = [
      ...sampleSizeIssues,
      ...manipulationSignals,
      ...correlationIssues,
    ];

    // No issues found
    if (allIssues.length === 0) {
      reasoning.push("No statistical issues detected");
      reasoning.push("Claims appear methodologically sound");

      return {
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.7,
        reasoning,
      };
    }

    // Categorize issues by severity
    const highSeverity = allIssues.filter((i) => i.severity === "high");
    const mediumSeverity = allIssues.filter((i) => i.severity === "medium");
    const lowSeverity = allIssues.filter((i) => i.severity === "low");

    reasoning.push(
      `Found ${allIssues.length} statistical issue(s): ${highSeverity.length} high, ${mediumSeverity.length} medium, ${lowSeverity.length} low`
    );

    // Report top issues
    for (const issue of [...highSeverity, ...mediumSeverity].slice(0, 3)) {
      reasoning.push(
        `${issue.severity.toUpperCase()}: ${issue.type} - ${issue.description}`
      );
    }

    // High severity issues present
    if (highSeverity.length > 0) {
      return {
        verdict: VerificationVerdict.VERIFIED_FALSE,
        confidence: 0.7,
        reasoning,
      };
    }

    // Only medium severity issues
    if (mediumSeverity.length > 0 && lowSeverity.length === 0) {
      return {
        verdict: VerificationVerdict.PARTIALLY_TRUE,
        confidence: 0.5,
        reasoning,
      };
    }

    // Only low severity issues
    if (lowSeverity.length > 0 && mediumSeverity.length === 0) {
      return {
        verdict: VerificationVerdict.PARTIALLY_TRUE,
        confidence: 0.6,
        reasoning,
      };
    }

    // Mixed severity
    return {
      verdict: VerificationVerdict.PARTIALLY_TRUE,
      confidence: 0.4,
      reasoning,
    };
  }
}
