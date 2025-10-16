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
    | "rate"
    | "distribution"
    | "confidence_interval";
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
  private healthMetrics: {
    totalRequests: number;
    successfulRequests: number;
    failedRequests: number;
    responseTimes: number[];
    lastHealthCheck: Date;
    consecutiveFailures: number;
    lastResponseTime: number;
    errorRate: number;
  } = {
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    responseTimes: [],
    lastHealthCheck: new Date(),
    consecutiveFailures: 0,
    lastResponseTime: 0,
    errorRate: 0,
  };

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
        const implicitIssues = this.detectImplicitStatisticalIssues(
          request.content
        );

        if (implicitIssues.length === 0) {
          return {
            method: VerificationType.STATISTICAL_VALIDATION,
            verdict: VerificationVerdict.UNVERIFIED,
            confidence: 0,
            reasoning: ["No statistical claims found in content"],
            processingTimeMs: Date.now() - startTime,
            evidenceCount: 0,
          };
        }

        const highSeverityImplicit = implicitIssues.filter(
          (issue) => issue.severity === "high"
        );

        const verdict =
          highSeverityImplicit.length > 0
            ? VerificationVerdict.VERIFIED_FALSE
            : VerificationVerdict.UNVERIFIED;

        const reasoning = implicitIssues.map((issue) => issue.description);
        if (verdict === VerificationVerdict.UNVERIFIED) {
          reasoning.unshift(
            "Statistical-style claim lacks supporting quantitative evidence"
          );
        }

        return {
          method: VerificationType.STATISTICAL_VALIDATION,
          verdict,
          confidence:
            verdict === VerificationVerdict.VERIFIED_FALSE ? 0.6 : 0.2,
          reasoning,
          processingTimeMs: Date.now() - startTime,
          evidenceCount: 0,
          metadata: {
            issues: implicitIssues.map((issue) => issue.type),
            totalIssues: implicitIssues.length,
            highSeverityIssues: highSeverityImplicit.length,
            mediumSeverityIssues: implicitIssues.filter(
              (issue) => issue.severity === "medium"
            ).length,
            claimsAnalyzed: 0,
          },
        };
      }

      // Validate sample sizes
      const sampleSizeIssues = this.validateSampleSizes(claims);
      console.log(
        "Sample size issues:",
        sampleSizeIssues.map((i) => ({ type: i.type, severity: i.severity }))
      );

      // Validate percentages
      const percentageIssues = this.validatePercentages(claims);
      console.log(
        "Percentage issues:",
        percentageIssues.map((i) => ({ type: i.type, severity: i.severity }))
      );

      // Detect data manipulation signals
      const manipulationSignals = this.detectManipulation(claims);
      console.log(
        "Manipulation signals:",
        manipulationSignals.map((i) => ({ type: i.type, severity: i.severity }))
      );

      // Check correlation vs causation
      const correlationIssues = this.checkCorrelationVsCausation(
        request.content,
        claims
      );

      // Validate statistical significance reporting
      const significanceIssues = this.validateSignificance(claims);
      console.log(
        "Significance issues:",
        significanceIssues.map((i) => ({ type: i.type, severity: i.severity }))
      );
      console.log(
        "Correlation issues:",
        correlationIssues.map((i) => ({ type: i.type, severity: i.severity }))
      );

      // Assess overall statistical validity
      const assessment = this.assessStatistics(
        claims,
        sampleSizeIssues,
        percentageIssues,
        manipulationSignals,
        correlationIssues,
        significanceIssues
      );
      console.log("Final assessment:", assessment);

      return {
        method: VerificationType.STATISTICAL_VALIDATION,
        verdict: assessment.verdict,
        confidence: assessment.confidence,
        reasoning: assessment.reasoning,
        processingTimeMs: Date.now() - startTime,
        evidenceCount: claims.length,
        metadata: {
          issues: [
            ...sampleSizeIssues,
            ...percentageIssues,
            ...manipulationSignals,
            ...correlationIssues,
            ...significanceIssues,
          ].map((issue) => issue.type),
          totalIssues:
            sampleSizeIssues.length +
            percentageIssues.length +
            manipulationSignals.length +
            correlationIssues.length +
            significanceIssues.length,
          highSeverityIssues: [
            ...sampleSizeIssues,
            ...percentageIssues,
            ...manipulationSignals,
            ...correlationIssues,
            ...significanceIssues,
          ].filter((issue) => issue.severity === "high").length,
          mediumSeverityIssues: [
            ...sampleSizeIssues,
            ...percentageIssues,
            ...manipulationSignals,
            ...correlationIssues,
            ...significanceIssues,
          ].filter((issue) => issue.severity === "medium").length,
          claimsAnalyzed: claims.length,
        },
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
      .split(/(?<!\d)[.!?]+(?!\d)/)
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    for (const sentence of sentences) {
      // First, check for sample sizes in the sentence
      // Try pattern: "500 participants" (number before word)
      let sampleMatch = sentence.match(
        /(\d+)\s+(n|sample|participants?|subjects?|people|respondents?|patients?)\b/i
      );
      if (!sampleMatch) {
        // Try pattern: "participants = 500" (word before number)
        sampleMatch = sentence.match(
          /\b(n|sample|participants?|subjects?|people|respondents?|patients?)\s*=?\s*(\d+)/i
        );
      }

      let sampleSize: number | undefined;
      if (sampleMatch) {
        // Extract the number (could be in group 1 or group 2 depending on pattern)
        sampleSize = parseInt(sampleMatch[1] || sampleMatch[2], 10);
        claims.push({
          text: sentence,
          type: "sample",
          sampleSize: sampleSize,
          context: this.extractContext(sentence),
        });
      }

      // Check for percentages
      const percentageMatches = sentence.matchAll(/(\d+(?:\.\d+)?)\s*%/g);
      for (const match of percentageMatches) {
        const value = parseFloat(match[1]);
        const matchIndex = match.index ?? sentence.indexOf(match[0]);
        const windowStart = Math.max(0, matchIndex - 15);
        const windowEnd = Math.min(
          sentence.length,
          matchIndex + match[0].length + 15
        );
        const windowText = sentence.slice(windowStart, windowEnd);
        const isConfidenceInterval = /\b(ci|confidence interval)\b/i.test(
          windowText
        );

        if (isConfidenceInterval) {
          claims.push({
            text: sentence,
            type: "confidence_interval",
            value,
            sampleSize,
            context: this.extractContext(sentence),
            confidence: value,
          });
        } else {
          claims.push({
            text: sentence,
            type: "percentage",
            value,
            sampleSize,
            context: this.extractContext(sentence),
          });
        }
      }

      // Check for means/averages
      const meanMatch = sentence.match(/\b(average|mean)\b.*?(\d+(?:\.\d+)?)/i);
      if (meanMatch) {
        claims.push({
          text: sentence,
          type: "mean",
          value: parseFloat(meanMatch[2]),
          sampleSize: sampleSize, // Link to sample size if found in same sentence
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
          sampleSize: sampleSize, // Link to sample size if found in same sentence
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
          sampleSize: sampleSize, // Link to sample size if found in same sentence
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

      // Detect explicit numeric sequences that imply data distributions
      const sequenceMatch = sentence.match(
        /(\d+(?:\.\d+)?)(?:\s*(?:,|and)\s*(\d+(?:\.\d+)?)){2,}/
      );
      if (
        sequenceMatch &&
        /\b(measurements?|values?|numbers?|scores?|data)\b/i.test(sentence)
      ) {
        const numericValues = sentence
          .match(/\d+(?:\.\d+)?/g)
          ?.map((value) => parseFloat(value));

        if (numericValues && numericValues.length >= 3) {
          claims.push({
            text: sentence,
            type: "distribution",
            context: this.extractContext(sentence),
            value:
              numericValues.reduce((acc, value) => acc + value, 0) /
              numericValues.length,
          });
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

    // Find all claims with sample sizes (both sample claims and statistical claims with linked sample sizes)
    const claimsWithSampleSize = claims.filter((c) => c.sampleSize);

    // Check sample size claims directly
    for (const claim of claimsWithSampleSize) {
      if (claim.sampleSize! < this.config.minSampleSize) {
        issues.push({
          type: "inadequate_sample_size",
          severity: "high",
          description: `Sample size (n=${claim.sampleSize}) is below recommended minimum of ${this.config.minSampleSize}`,
          claim: claim.text,
        });
      }

      // Check for suspiciously small samples with precise statistics
      if (claim.sampleSize! < 10) {
        issues.push({
          type: "very_small_sample",
          severity: "high",
          description: `Sample size (n=${claim.sampleSize}) is too small for reliable statistical inference`,
          claim: claim.text,
        });
      }
    }

    // Check for statistical claims without sample sizes
    const statisticalClaims = claims.filter(
      (c) =>
        (c.type === "percentage" || c.type === "mean" || c.type === "median") &&
        !c.sampleSize
    );

    for (const statClaim of statisticalClaims) {
      // Look for a sample claim in the same context or nearby
      const sampleClaims = claims.filter(
        (c) => c.type === "sample" && c.sampleSize
      );
      const hasSampleSize = sampleClaims.some((sampleClaim) =>
        this.areClaimsRelated(statClaim, sampleClaim)
      );

      if (!hasSampleSize) {
        issues.push({
          type: "missing_sample_size",
          severity: "medium",
          description: "Statistical claim made without reporting sample size",
          claim: statClaim.text,
        });
      }
    }

    return issues;
  }

  private areClaimsRelated(
    claim1: StatisticalClaim,
    claim2: StatisticalClaim
  ): boolean {
    // Check if claims are in the same sentence or adjacent sentences
    const text1 = claim1.text.toLowerCase();
    const text2 = claim2.text.toLowerCase();

    // Simple heuristic: if they share common words or are in the same context
    const words1 = text1.split(/\s+/);
    const words2 = text2.split(/\s+/);

    const commonWords = words1.filter((word) => words2.includes(word));
    return commonWords.length > 0;
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
            type: "excessive_precision",
            severity: "low",
            description: `Value ${claim.value} reported with ${decimalPlaces} decimal places, may indicate false precision`,
            claim: claim.text,
          });
        }

        // Check for suspiciously round numbers (only flag if sample size is inadequate)
        if (
          claim.type === "percentage" &&
          claim.value % 5 === 0 &&
          claim.value !== 0 &&
          claim.value !== 100 &&
          (!claim.sampleSize || claim.sampleSize < this.config.minSampleSize)
        ) {
          issues.push({
            type: "suspicious_round_number",
            severity: "low",
            description: `Percentage value ${claim.value}% is a multiple of 5, which may indicate rounding or estimation`,
            claim: claim.text,
          });
        }
      }

      // Check for cherry-picking indicators
      if (
        /\b(selected|chosen|specific|particular|certain)\b/i.test(claim.text) ||
        /\bother studies\b/i.test(claim.text) ||
        /\bnote\b.*\bother\b.*\bstudies\b/i.test(claim.text)
      ) {
        issues.push({
          type: "cherry_picking",
          severity: "high",
          description: "Language suggests selective reporting of data",
          claim: claim.text,
        });
      }
    }

    // Check for p-value fishing
    const pValueClaims = claims.filter((c) => c.pValue !== undefined);
    if (pValueClaims.length > 3) {
      issues.push({
        type: "multiple_p_values",
        severity: "medium",
        description: `${pValueClaims.length} p-values reported, which may indicate p-hacking or multiple testing`,
        claim: `Multiple statistical tests: ${pValueClaims.length} p-values`,
      });
    }

    // Check for explicit p-hacking language (high severity)
    const content = claims
      .map((c) => c.text)
      .join(" ")
      .toLowerCase();
    const pHackingIndicators = [
      /excluding outliers/i,
      /removing.*participants/i,
      /trying.*different.*analyses/i,
      /multiple.*analyses/i,
      /data.*manipulation/i,
      /selective.*reporting/i,
      /cherry.*picking/i,
    ];

    const pHackingMatches = pHackingIndicators.filter((pattern) =>
      pattern.test(content)
    );
    if (pHackingMatches.length > 0) {
      issues.push({
        type: "p_hacking",
        severity: "high",
        description: `Detected ${
          pHackingMatches.length
        } indicators of p-hacking: ${pHackingMatches
          .map((p) => p.source)
          .join(", ")}`,
        claim: claims.map((c) => c.text).join(" | "),
      });
    }

    // Check for suspiciously significant results
    for (const claim of pValueClaims) {
      if (claim.pValue && claim.pValue < 0.001) {
        issues.push({
          type: "extremely_significant_result",
          severity: "low",
          description: `P-value ${claim.pValue} is extremely small, verify this is not a data entry error`,
          claim: claim.text,
        });
      }
    }

    // Check for suspicious data patterns (high severity)
    const fullText = claims
      .map((c) => c.text)
      .join(" ")
      .toLowerCase();
    const suspiciousPatterns = [
      /exactly.*10\.0.*20\.0.*30\.0.*40\.0.*50\.0/i, // Arithmetic progression
      /no variation/i,
      /identical.*values/i,
      /same.*number/i,
      /perfect.*correlation/i,
      /zero.*variance/i,
    ];

    const suspiciousMatches = suspiciousPatterns.filter((pattern) =>
      pattern.test(fullText)
    );
    if (suspiciousMatches.length > 0) {
      issues.push({
        type: "suspicious_data_pattern",
        severity: "high",
        description: `Detected ${
          suspiciousMatches.length
        } indicators of suspicious data patterns: ${suspiciousMatches
          .map((p) => p.source)
          .join(", ")}`,
        claim: claims.map((c) => c.text).join(" | "),
      });
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
            type: "correlation_causation_confusion",
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
          type: "missing_confounding_discussion",
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
   * Validate percentage claims and check for sum consistency
   */
  private validatePercentages(claims: StatisticalClaim[]): StatisticalIssue[] {
    const issues: StatisticalIssue[] = [];
    const percentageClaims = claims.filter((c) => c.type === "percentage");

    if (percentageClaims.length < 2) {
      // Still validate individual percentage math if possible
      for (const claim of percentageClaims) {
        this.evaluatePercentageAccuracy(claim, issues);
      }
      return issues;
    }

    // Group percentages by context (same sentence or related context)
    const contextGroups: { [key: string]: StatisticalClaim[] } = {};
    for (const claim of percentageClaims) {
      const contextKey = claim.context || "general";
      if (!contextGroups[contextKey]) {
        contextGroups[contextKey] = [];
      }
      contextGroups[contextKey].push(claim);
      this.evaluatePercentageAccuracy(claim, issues);
    }

    // Check each context group for sum validation
    for (const [context, groupClaims] of Object.entries(contextGroups)) {
      if (groupClaims.length < 2) continue;

      const sum = groupClaims.reduce(
        (acc, claim) => acc + (claim.value || 0),
        0
      );

      // Check if sum is approximately 100% (allow small tolerance)
      const tolerance = 1.0; // Allow 1% tolerance
      const isValidSum = Math.abs(sum - 100) <= tolerance;

      if (!isValidSum) {
        issues.push({
          type: "percentage_sum_error",
          severity: "high",
          description: `Percentages sum to ${sum.toFixed(
            1
          )}% instead of 100%. Individual percentages: ${groupClaims
            .map((c) => `${c.value}%`)
            .join(", ")}`,
          claim: groupClaims.map((c) => c.text).join(" | "),
        });
      }
    }

    return issues;
  }

  private evaluatePercentageAccuracy(
    claim: StatisticalClaim,
    issues: StatisticalIssue[]
  ): void {
    if (claim.value === undefined) {
      return;
    }

    const sampleSize =
      claim.sampleSize ?? this.extractSampleSizeFromText(claim.text);
    if (!sampleSize || sampleSize <= 0) {
      return;
    }

    const numerator = this.extractNumeratorFromText(claim.text);
    if (numerator === undefined) {
      return;
    }

    const expectedPercentage = (numerator / sampleSize) * 100;
    const tolerance = 1;

    if (Math.abs(expectedPercentage - claim.value) > tolerance) {
      issues.push({
        type: "incorrect_percentage",
        severity: "high",
        description: `Reported ${
          claim.value
        }% does not match ${numerator}/${sampleSize} (expected ${expectedPercentage.toFixed(
          1
        )}%)`,
        claim: claim.text,
      });
    }
  }

  private validateSignificance(claims: StatisticalClaim[]): StatisticalIssue[] {
    const issues: StatisticalIssue[] = [];

    for (const claim of claims) {
      if (claim.pValue !== undefined) {
        if (claim.pValue > (this.config.significanceLevel ?? 0.05)) {
          issues.push({
            type: "weak_significance",
            severity: "high",
            description: `Reported p-value ${claim.pValue} exceeds significance threshold ${this.config.significanceLevel}`,
            claim: claim.text,
          });
        }

        if (
          claim.pValue <= (this.config.significanceLevel ?? 0.05) &&
          claim.sampleSize &&
          claim.sampleSize < this.config.minSampleSize
        ) {
          issues.push({
            type: "underpowered_significance",
            severity: "medium",
            description: `Result marked significant with small sample size (n=${claim.sampleSize})`,
            claim: claim.text,
          });
        }
      } else if (/statistically significant/i.test(claim.text)) {
        issues.push({
          type: "missing_significance_details",
          severity: "medium",
          description:
            "Claim mentions statistical significance without reporting p-value",
          claim: claim.text,
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
    percentageIssues: StatisticalIssue[],
    manipulationSignals: StatisticalIssue[],
    correlationIssues: StatisticalIssue[],
    significanceIssues: StatisticalIssue[]
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
      ...percentageIssues,
      ...manipulationSignals,
      ...correlationIssues,
      ...significanceIssues,
    ];

    // No issues found
    if (allIssues.length === 0) {
      reasoning.push("No statistical issues detected");
      reasoning.push("Claims appear methodologically sound");

      return {
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.8,
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

  private detectImplicitStatisticalIssues(content: string): StatisticalIssue[] {
    const issues: StatisticalIssue[] = [];
    const normalized = content.toLowerCase();
    const trimmedContent = content.trim();

    if (
      /\b(most|majority|nearly all|almost all|more than half|many|often)\b/.test(
        normalized
      )
    ) {
      issues.push({
        type: "missing_sample_size",
        severity: "medium",
        description:
          "Qualitative statistical claim provided without sample size or numeric support",
        claim: trimmedContent,
      });
    }

    const fallbackClaims: StatisticalClaim[] = [
      {
        text: trimmedContent,
        type: "distribution",
        context: this.extractContext(trimmedContent),
      },
    ];

    const manipulationIssues = this.detectManipulation(fallbackClaims).filter(
      (issue) =>
        issue.type === "suspicious_data_pattern" || issue.type === "p_hacking"
    );

    issues.push(...manipulationIssues);

    return issues;
  }

  private extractSampleSizeFromText(text: string): number | undefined {
    const normalized = text.toLowerCase();

    const ofPattern = normalized.match(
      /\b(?:of|out of)\s+(\d+(?:\.\d+)?)\s*(?:participants?|respondents?|people|subjects?|patients?)?/i
    );
    if (ofPattern) {
      return parseFloat(ofPattern[1]);
    }

    const nPattern = normalized.match(/\bn\s*=?\s*(\d+(?:\.\d+)?)/i);
    if (nPattern) {
      return parseFloat(nPattern[1]);
    }

    return undefined;
  }

  private extractNumeratorFromText(text: string): number | undefined {
    const parenthetical = text.match(
      /(\d+(?:\.\d+)?)\s*\(\s*(\d+(?:\.\d+)?)%\s*\)/
    );
    if (parenthetical) {
      return parseFloat(parenthetical[1]);
    }

    const ofPattern = text.match(
      /(\d+(?:\.\d+)?)\s+(?:of|out of)\s+(\d+(?:\.\d+)?)/i
    );
    if (ofPattern) {
      return parseFloat(ofPattern[1]);
    }

    return undefined;
  }

  /**
   * Get method health status
   */
  getHealth(): { available: boolean; responseTime: number; errorRate: number } {
    // Update error rate based on recent metrics
    this.updateErrorRate();

    // Check availability based on consecutive failures and recent activity
    const now = new Date();
    const timeSinceLastCheck =
      now.getTime() - this.healthMetrics.lastHealthCheck.getTime();
    const available: boolean =
      this.healthMetrics.consecutiveFailures < 3 && timeSinceLastCheck < 300000; // 5 minutes

    // Calculate average response time
    const avgResponseTime =
      this.healthMetrics.responseTimes.length > 0
        ? this.healthMetrics.responseTimes.reduce(
            (sum, time) => sum + time,
            0
          ) / this.healthMetrics.responseTimes.length
        : this.healthMetrics.lastResponseTime || 0;

    return {
      available,
      responseTime: Math.round(avgResponseTime),
      errorRate: Math.round(this.healthMetrics.errorRate * 100) / 100,
    };
  }

  /**
   * Record a successful verification request
   */
  private recordSuccess(responseTime: number): void {
    this.healthMetrics.totalRequests++;
    this.healthMetrics.successfulRequests++;
    this.healthMetrics.consecutiveFailures = 0;
    this.healthMetrics.lastResponseTime = responseTime;
    this.healthMetrics.responseTimes.push(responseTime);

    // Keep only last 100 response times for rolling average
    if (this.healthMetrics.responseTimes.length > 100) {
      this.healthMetrics.responseTimes.shift();
    }

    this.healthMetrics.lastHealthCheck = new Date();
  }

  /**
   * Record a failed verification request
   */
  private recordFailure(responseTime: number): void {
    this.healthMetrics.totalRequests++;
    this.healthMetrics.failedRequests++;
    this.healthMetrics.consecutiveFailures++;
    this.healthMetrics.lastResponseTime = responseTime;
    this.healthMetrics.responseTimes.push(responseTime);

    // Keep only last 100 response times for rolling average
    if (this.healthMetrics.responseTimes.length > 100) {
      this.healthMetrics.responseTimes.shift();
    }

    this.healthMetrics.lastHealthCheck = new Date();
  }

  /**
   * Update error rate based on recent metrics
   */
  private updateErrorRate(): void {
    if (this.healthMetrics.totalRequests > 0) {
      this.healthMetrics.errorRate =
        this.healthMetrics.failedRequests / this.healthMetrics.totalRequests;
    } else {
      this.healthMetrics.errorRate = 0;
    }
  }
}
