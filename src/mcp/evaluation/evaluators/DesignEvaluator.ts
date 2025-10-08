/**
 * Design Evaluator for Token Compliance Assessment
 *
 * @author @darianrosebrook
 * @description Evaluates design token usage and consistency
 */

import * as fs from "fs";
import * as path from "path";
import { Logger } from "../../../utils/Logger.js";
import {
  BaseEvaluator,
  EvaluationParams,
  EvaluationReport,
  EvalCriterion,
} from "../EvaluationOrchestrator.js";

export interface DesignTokenRegistry {
  colors?: Record<string, string>;
  space?: Record<string, string>;
  radius?: Record<string, string>;
  typography?: Record<string, string>;
  shadows?: Record<string, string>;
  [key: string]: Record<string, string> | undefined;
}

export interface DesignEvaluationConfig {
  tokenRegistry?: string; // Path to token registry file
  allowedRawValues?: {
    colors?: string[];
    spacing?: string[];
  };
  strictMode?: boolean; // Fail on any raw values if true
}

export class DesignEvaluator extends BaseEvaluator {
  constructor(private logger: Logger) {
    super();
  }

  async evaluate(params: EvaluationParams): Promise<EvaluationReport> {
    const { taskId, artifactPath, iterations, acceptance, config } = params;
    const designConfig = config as DesignEvaluationConfig | undefined;

    this.logger.debug(
      `Evaluating design for task ${taskId} at ${artifactPath}`
    );

    // Read the design file
    let content: string;
    try {
      content = fs.readFileSync(artifactPath, "utf8");
    } catch (error) {
      throw new Error(`Failed to read design file: ${artifactPath}`);
    }

    // Load token registry if specified
    let tokenRegistry: DesignTokenRegistry = {};
    if (designConfig?.tokenRegistry) {
      try {
        tokenRegistry = JSON.parse(
          fs.readFileSync(designConfig.tokenRegistry, "utf8")
        );
      } catch (error) {
        this.logger.warn(
          `Failed to load token registry: ${designConfig.tokenRegistry}`
        );
      }
    }

    const criteria: EvalCriterion[] = [];

    // No hard-coded hex colors
    const hexColors = content.match(/#[0-9a-f]{3,8}\b/gi) || [];
    const allowedHexColors = designConfig?.allowedRawValues?.colors || [];
    const invalidHexColors = hexColors.filter(
      (hex) => !allowedHexColors.includes(hex.toLowerCase())
    );
    const strictMode = designConfig?.strictMode ?? false;
    const noHardcodedHex = strictMode
      ? invalidHexColors.length === 0
      : hexColors.length <= 2;

    criteria.push({
      id: "no-hardcoded-hex",
      description: "No raw hex color values (use tokens/vars)",
      weight: 0.35,
      passed: noHardcodedHex,
      score: noHardcodedHex ? 1 : 0,
      notes: invalidHexColors.length
        ? `invalid hex: ${invalidHexColors.slice(0, 5).join(", ")}`
        : undefined,
    });

    // No raw pixel spacing
    const pixelSpacing = content.match(/(?<!font-)\b\d+px\b/g) || [];
    const allowedPixelSpacing = designConfig?.allowedRawValues?.spacing || [];
    const invalidPixelSpacing = pixelSpacing.filter(
      (px) => !allowedPixelSpacing.includes(px)
    );
    const noRawPxSpacing = strictMode
      ? invalidPixelSpacing.length === 0
      : invalidPixelSpacing.length <= 2;

    criteria.push({
      id: "no-raw-px-spacing",
      description: "Avoid raw px in spacing (use space tokens/vars)",
      weight: 0.25,
      passed: noRawPxSpacing,
      score: noRawPxSpacing ? 1 : 0,
      notes: invalidPixelSpacing.length
        ? `raw px: ${invalidPixelSpacing.slice(0, 5).join(", ")}`
        : undefined,
    });

    // Token coverage (presence of known tokens)
    const allTokenKeys = this.getAllTokenKeys(tokenRegistry);
    const usedTokens = allTokenKeys.filter((key) => content.includes(key));
    const coverage =
      allTokenKeys.length > 0 ? usedTokens.length / allTokenKeys.length : 1;
    const tokenCoverageGood = coverage >= 0.1; // Low bar: ensure some tokens appear

    criteria.push({
      id: "token-coverage",
      description: "Some token usage present",
      weight: 0.15,
      passed: tokenCoverageGood,
      score: tokenCoverageGood ? 1 : 0,
      notes: `coverage≈${coverage.toFixed(2)} (${usedTokens.length}/${
        allTokenKeys.length
      } tokens)`,
    });

    // No ad-hoc color names (heuristic)
    const colorNames =
      content.match(/\b(color|bg|background|fill|stroke)[-_:]\w+\b/g) || [];
    const adHocColorNames = colorNames.filter(
      (name) =>
        !name.includes(".") && // Not a token reference
        !this.isKnownColorName(name, tokenRegistry)
    );
    const noAdHocColors = adHocColorNames.length === 0;

    criteria.push({
      id: "no-ad-hoc-color-names",
      description: "No ad-hoc color names; use semantic tokens",
      weight: 0.25,
      passed: noAdHocColors,
      score: noAdHocColors ? 1 : 0,
      notes: adHocColorNames.length
        ? `ad-hoc: ${adHocColorNames.slice(0, 5).join(", ")}`
        : undefined,
    });

    // Consistent spacing scale (check if spacing values follow a pattern)
    const spacingPattern = this.analyzeSpacingPattern(content, tokenRegistry);
    criteria.push({
      id: "consistent-spacing",
      description: "Spacing values follow consistent scale",
      weight: 0.1,
      passed: spacingPattern.consistent,
      score: spacingPattern.consistent ? 1 : 0.5,
      notes: `pattern: ${spacingPattern.description}`,
    });

    // Color contrast considerations (basic check)
    const contrastIssues = this.checkColorContrast(content);
    criteria.push({
      id: "color-contrast",
      description: "Color combinations meet basic contrast guidelines",
      weight: 0.1,
      passed: contrastIssues.length === 0,
      score: contrastIssues.length === 0 ? 1 : 0.5,
      notes: contrastIssues.length
        ? `issues: ${contrastIssues.slice(0, 3).join(", ")}`
        : undefined,
    });

    // Responsive design considerations
    const responsiveFeatures = this.checkResponsiveDesign(content);
    criteria.push({
      id: "responsive-design",
      description: "Includes responsive design considerations",
      weight: 0.1,
      passed: responsiveFeatures.present,
      score: responsiveFeatures.present ? 1 : 0,
      notes: responsiveFeatures.description,
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
        : this.generateImprovementActions(criteria, tokenRegistry),
      timestamp: new Date().toISOString(),
    };

    this.logger.debug(
      `Design evaluation completed for ${taskId}: score=${report.score}, status=${report.status}`
    );
    return report;
  }

  private getAllTokenKeys(registry: DesignTokenRegistry): string[] {
    const keys: string[] = [];
    for (const category of Object.values(registry)) {
      if (category) {
        keys.push(...Object.keys(category));
      }
    }
    return keys;
  }

  private isKnownColorName(
    name: string,
    registry: DesignTokenRegistry
  ): boolean {
    if (!registry.colors) return false;

    const colorName = name.replace(
      /^(color|bg|background|fill|stroke)[-_:]/,
      ""
    );
    return Object.keys(registry.colors).some(
      (key) => key.includes(colorName) || colorName.includes(key)
    );
  }

  private analyzeSpacingPattern(
    content: string,
    registry: DesignTokenRegistry
  ): {
    consistent: boolean;
    description: string;
  } {
    const spacingTokens = registry.space || {};
    const tokenValues = Object.values(spacingTokens);

    if (tokenValues.length === 0) {
      return { consistent: true, description: "No spacing tokens defined" };
    }

    // Check if spacing values in content follow token patterns
    const spacingPattern = /\b(\d+)(px|rem|em)\b/g;
    const matches = content.match(spacingPattern) || [];

    if (matches.length === 0) {
      return { consistent: true, description: "No spacing values found" };
    }

    // Check if most values could be mapped to tokens
    const uniqueValues = [...new Set(matches)];
    const mappableValues = uniqueValues.filter((value) =>
      tokenValues.some((token) => token.includes(value))
    );

    const consistencyRatio = mappableValues.length / uniqueValues.length;
    const consistent = consistencyRatio >= 0.7;

    return {
      consistent,
      description: `${mappableValues.length}/${uniqueValues.length} spacing values match tokens`,
    };
  }

  private checkColorContrast(content: string): string[] {
    const issues: string[] = [];

    // Basic heuristics for potential contrast issues
    // This is a simplified check - real contrast analysis would need more context
    const lightColors = content.match(/#(f{3,8}|e[0-9a-f]{2,7})/gi) || [];
    const darkColors = content.match(/#(0{3,8}|1[0-9a-f]{2,7})/gi) || [];

    // Flag potential low contrast combinations
    if (lightColors.length > 0 && darkColors.length === 0) {
      issues.push("only light colors detected");
    }

    if (darkColors.length > 0 && lightColors.length === 0) {
      issues.push("only dark colors detected");
    }

    return issues;
  }

  private checkResponsiveDesign(content: string): {
    present: boolean;
    description: string;
  } {
    const hasMediaQueries = /@media/.test(content);
    const hasFlexbox =
      /\b(display:\s*flex|flex-direction|justify-content|align-items)\b/.test(
        content
      );
    const hasGrid =
      /\b(display:\s*grid|grid-template|grid-column|grid-row)\b/.test(content);

    const responsiveFeatures = [hasMediaQueries, hasFlexbox, hasGrid].filter(
      Boolean
    ).length;

    return {
      present: responsiveFeatures >= 1,
      description:
        responsiveFeatures === 0
          ? "No responsive features detected"
          : `Responsive features: ${hasMediaQueries ? "media-queries " : ""}${
              hasFlexbox ? "flexbox " : ""
            }${hasGrid ? "grid " : ""}`.trim(),
    };
  }

  private generateImprovementActions(
    criteria: EvalCriterion[],
    registry: DesignTokenRegistry
  ): string[] {
    const actions: string[] = [];

    const failedCriteria = criteria.filter((c) => !c.passed);

    for (const criterion of failedCriteria) {
      switch (criterion.id) {
        case "no-hardcoded-hex":
          actions.push("Replace hard-coded hex colors with design tokens");
          break;
        case "no-raw-px-spacing":
          actions.push("Replace raw pixel values with spacing tokens");
          break;
        case "token-coverage":
          actions.push("Use more design tokens from the token registry");
          break;
        case "no-ad-hoc-color-names":
          actions.push("Replace ad-hoc color names with semantic token names");
          break;
        case "consistent-spacing":
          actions.push("Use consistent spacing scale from design tokens");
          break;
        case "color-contrast":
          actions.push("Review color combinations for sufficient contrast");
          break;
        case "responsive-design":
          actions.push(
            "Add responsive design features (media queries, flexbox, or grid)"
          );
          break;
        default:
          actions.push(`Address ${criterion.description.toLowerCase()}`);
      }
    }

    if (actions.length === 0) {
      actions.push("Review design implementation and improve token usage");
    }

    return actions;
  }
}

