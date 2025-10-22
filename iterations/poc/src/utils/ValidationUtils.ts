/**
 * Central Validation Utilities
 *
 * Extracted from CAWS validation logic to provide generic validation
 * capabilities across the agent agency system.
 *
 * @author @darianrosebrook
 */

import Ajv, { type Ajv as AjvType } from "ajv";
import addFormats from "ajv-formats";
import * as fs from "fs";
import * as yaml from "js-yaml";
// import * as path from "path";
import type {
  ValidationResult,
  ContractValidationResult,
  // ContractDetails,
  // NonFunctionalRequirements,
  TrustScoreResult,
  TrustScoreComponents,
} from "../types/index.js";

export class ValidationUtils {
  private ajv: AjvType;

  constructor() {
    this.ajv = new Ajv({
      allErrors: true,
      verbose: true,
      strict: false, // Allow draft schemas
    });
    addFormats(this.ajv); // Add format support for date-time, uri, etc.
  }

  /**
   * Validate JSON against a schema
   */
  validateJsonAgainstSchema(jsonData: any, schema: any): ValidationResult {
    try {
      const validate = this.ajv.compile(schema);
      const valid = validate(jsonData);

      if (!valid) {
        return {
          passed: false,
          score: 0,
          details: {},
          errors:
            validate.errors?.map(
              (err: any) => `${err.instancePath}: ${err.message}`
            ) ?? [],
        };
      }

      return {
        passed: true,
        score: 1,
        details: {},
      };
    } catch (error) {
      return {
        passed: false,
        score: 0,
        details: {},
        errors: [`Schema validation failed: ${error}`],
      };
    }
  }

  /**
   * Validate YAML against a schema
   */
  validateYamlAgainstSchema(
    yamlPath: string,
    schemaPath: string
  ): ValidationResult {
    try {
      // Read YAML file
      const yamlContent = fs.readFileSync(yamlPath, "utf-8");
      const yamlData = yaml.load(yamlContent);

      // Read schema file
      const schemaContent = fs.readFileSync(schemaPath, "utf-8");
      const schema = JSON.parse(schemaContent);

      // Validate
      const validate = this.ajv.compile(schema);
      const valid = validate(yamlData);

      if (!valid) {
        return {
          passed: false,
          score: 0,
          details: {},
          errors:
            validate.errors?.map(
              (err: any) => `${err.instancePath}: ${err.message}`
            ) ?? [],
        };
      }

      return {
        passed: true,
        score: 1,
        details: {},
      };
    } catch (error) {
      return {
        passed: false,
        score: 0,
        details: {},
        errors: [`YAML schema validation failed: ${error}`],
      };
    }
  }

  /**
   * Validate file exists and is readable
   */
  validateFileExists(filePath: string): ValidationResult {
    try {
      if (!fs.existsSync(filePath)) {
        return {
          passed: false,
          score: 0,
          details: {},
          errors: [`File not found: ${filePath}`],
        };
      }

      // Try to read the file
      fs.accessSync(filePath, fs.constants.R_OK);
      return {
        passed: true,
        score: 1,
        details: {},
      };
    } catch {
      return {
        passed: false,
        score: 0,
        details: {},
        errors: [`File not readable: ${filePath}`],
      };
    }
  }

  /**
   * Validate directory exists and is writable
   */
  validateDirectoryExists(dirPath: string): ValidationResult {
    try {
      if (!fs.existsSync(dirPath)) {
        return {
          passed: false,
          score: 0,
          details: {},
          errors: [`Directory not found: ${dirPath}`],
        };
      }

      // Try to write to the directory
      fs.accessSync(dirPath, fs.constants.W_OK);
      return {
        passed: true,
        score: 1,
        details: {},
      };
    } catch {
      return {
        passed: false,
        score: 0,
        details: {},
        errors: [`Directory not writable: ${dirPath}`],
      };
    }
  }

  /**
   * Calculate trust score from components
   */
  calculateTrustScore(components: TrustScoreComponents): TrustScoreResult {
    // Weights for each component (total = 100)
    const weights = {
      coverage: 25, // Branch coverage
      mutation: 20, // Mutation testing
      contracts: 15, // Contract validation
      a11y: 10, // Accessibility
      perf: 15, // Performance
      flake: 15, // Test stability
    };

    // Normalize and score each component
    const coverage = Math.min(components.coverage_branch / 90, 1) * 100;
    const mutation = Math.min(components.mutation_score / 70, 1) * 100;
    const contracts =
      components.contracts_consumer && components.contracts_provider
        ? 100
        : components.contracts_consumer || components.contracts_provider
        ? 50
        : 0;
    const a11y = components.a11y_passed ? 100 : 0;
    const perf = components.perf_within_budget ? 100 : 0;
    const flake = Math.max(0, (1 - components.flake_rate) * 100);

    // Calculate weighted score
    const total_score =
      (coverage * weights.coverage +
        mutation * weights.mutation +
        contracts * weights.contracts +
        a11y * weights.a11y +
        perf * weights.perf +
        flake * weights.flake) /
      100;

    // Determine tier
    let tier: string;
    if (total_score >= 85) tier = "platinum";
    else if (total_score >= 70) tier = "gold";
    else if (total_score >= 55) tier = "silver";
    else if (total_score >= 40) tier = "bronze";
    else tier = "needs-improvement";

    // Generate recommendations
    const recommendations: string[] = [];
    if (coverage < 80)
      recommendations.push("Increase branch coverage to at least 80%");
    if (mutation < 50)
      recommendations.push("Improve mutation score to at least 50%");
    if (contracts === 0) recommendations.push("Add contract tests for APIs");
    if (!components.a11y_passed)
      recommendations.push("Fix accessibility violations");
    if (!components.perf_within_budget)
      recommendations.push("Optimize performance to meet budgets");
    if (flake > 10) recommendations.push("Reduce test flakiness below 10%");

    return {
      total_score: Math.round(total_score * 100) / 100,
      tier,
      components,
      breakdown: {
        coverage: Math.round(coverage * 100) / 100,
        mutation: Math.round(mutation * 100) / 100,
        contracts: Math.round(contracts * 100) / 100,
        a11y: Math.round(a11y * 100) / 100,
        perf: Math.round(perf * 100) / 100,
        flake: Math.round(flake * 100) / 100,
      },
      recommendations,
    };
  }

  /**
   * Validate contract test results
   */
  validateContractResults(
    consumerResults?: { numPassed: number; numTotal: number },
    providerResults?: { numPassed: number; numTotal: number }
  ): ContractValidationResult {
    const errors: Array<{
      type: "request" | "response" | "schema";
      endpoint: string;
      message: string;
      details?: any;
    }> = [];

    let score = 0;
    const details: Record<string, any> = {};

    // Validate consumer contracts
    if (consumerResults) {
      const consumerPassRate =
        consumerResults.numTotal > 0
          ? consumerResults.numPassed / consumerResults.numTotal
          : 0;
      details.consumer_pass_rate = consumerPassRate;

      if (consumerPassRate < 0.9) {
        errors.push({
          type: "request",
          endpoint: "consumer-contracts",
          message: `Consumer contract pass rate too low: ${(
            consumerPassRate * 100
          ).toFixed(1)}%`,
        });
      } else {
        score += 0.5;
      }
    }

    // Validate provider contracts
    if (providerResults) {
      const providerPassRate =
        providerResults.numTotal > 0
          ? providerResults.numPassed / providerResults.numTotal
          : 0;
      details.provider_pass_rate = providerPassRate;

      if (providerPassRate < 0.9) {
        errors.push({
          type: "response",
          endpoint: "provider-contracts",
          message: `Provider contract pass rate too low: ${(
            providerPassRate * 100
          ).toFixed(1)}%`,
        });
      } else {
        score += 0.5;
      }
    }

    return {
      passed: errors.length === 0,
      score,
      details,
      errors,
      coverage: {
        endpointsTested:
          (consumerResults?.numTotal ?? 0) + (providerResults?.numTotal ?? 0),
        totalEndpoints:
          (consumerResults?.numTotal ?? 0) + (providerResults?.numTotal ?? 0),
        schemasValidated:
          (consumerResults?.numPassed ?? 0) + (providerResults?.numPassed ?? 0),
      },
    };
  }

  /**
   * Parse and validate configuration file
   */
  validateConfigFile(
    configPath: string,
    schemaPath?: string
  ): ValidationResult {
    try {
      // Read config file
      const configContent = fs.readFileSync(configPath, "utf-8");
      let config: any;

      // Try to parse as YAML first, then JSON
      try {
        config = yaml.load(configContent);
      } catch {
        try {
          config = JSON.parse(configContent);
        } catch {
          return {
            passed: false,
            score: 0,
            details: {},
            errors: ["Invalid JSON/YAML format in config file"],
          };
        }
      }

      // If schema provided, validate against it
      if (schemaPath) {
        const schemaContent = fs.readFileSync(schemaPath, "utf-8");
        const schema = JSON.parse(schemaContent);

        const validate = this.ajv.compile(schema);
        const valid = validate(config);

        if (!valid) {
          return {
            passed: false,
            score: 0,
            details: {},
            errors:
              validate.errors?.map(
                (err: any) => `${err.instancePath}: ${err.message}`
              ) ?? [],
          };
        }
      }

      return {
        passed: true,
        score: 1,
        details: { config },
      };
    } catch (error) {
      return {
        passed: false,
        score: 0,
        details: {},
        errors: [`Config validation failed: ${error}`],
      };
    }
  }
}
