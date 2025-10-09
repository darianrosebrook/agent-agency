/**
 * JSON Schema Validator for Working Specifications
 *
 * Validates working specs against their JSON schemas to ensure
 * contract compliance and data integrity
 *
 * @author @darianrosebrook
 */

import Ajv from "ajv";
import addFormats from "ajv-formats";
import * as fs from "fs";
import * as path from "path";

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

export class SchemaValidator {
  private ajv: Ajv;
  private schemas: Map<string, any> = new Map();

  constructor() {
    this.ajv = new Ajv({
      allErrors: true,
      verbose: true,
      strict: false,
      allowUnionTypes: true,
    });

    addFormats(this.ajv);

    // Load schemas
    this.loadSchemas();
  }

  /**
   * Validate a working spec against its schema
   */
  validateWorkingSpec(
    spec: any,
    schemaType: "working-spec" | "waivers" = "working-spec"
  ): ValidationResult {
    const schema = this.schemas.get(schemaType);
    if (!schema) {
      return {
        valid: false,
        errors: [`Schema ${schemaType} not found`],
        warnings: [],
      };
    }

    const validate = this.ajv.compile(schema);
    const valid = validate(spec);

    return {
      valid: valid,
      errors: validate.errors
        ? validate.errors.map((err) => this.formatError(err))
        : [],
      warnings: this.extractWarnings(spec),
    };
  }

  /**
   * Validate multiple working specs
   */
  validateWorkingSpecs(
    specs: Array<{ spec: any; path: string }>
  ): Array<{ path: string; result: ValidationResult }> {
    return specs.map(({ spec, path }) => ({
      path,
      result: this.validateWorkingSpec(spec),
    }));
  }

  private loadSchemas(): void {
    const schemasDir = path.join(__dirname, "../../../apps/tools/caws/schemas");

    try {
      // Load working spec schema
      const workingSpecPath = path.join(schemasDir, "working-spec.schema.json");
      if (fs.existsSync(workingSpecPath)) {
        const workingSpecSchema = JSON.parse(
          fs.readFileSync(workingSpecPath, "utf-8")
        );
        this.schemas.set("working-spec", workingSpecSchema);
      }

      // Load waivers schema
      const waiversPath = path.join(schemasDir, "waivers.schema.json");
      if (fs.existsSync(waiversPath)) {
        const waiversSchema = JSON.parse(fs.readFileSync(waiversPath, "utf-8"));
        this.schemas.set("waivers", waiversSchema);
      }
    } catch (error) {
      console.error("Error loading schemas:", error);
    }
  }

  private formatError(error: any): string {
    const path = error.instancePath || error.dataPath || "";
    const message = error.message || "Unknown error";

    if (path) {
      return `${path}: ${message}`;
    }

    return message;
  }

  private extractWarnings(spec: any): string[] {
    const warnings: string[] = [];

    // Check for Tier 1 specs with fewer than 5 acceptance criteria
    if (spec.risk_tier === 1) {
      const acceptanceCount = spec.acceptance ? spec.acceptance.length : 0;
      if (acceptanceCount < 5) {
        warnings.push(
          `Tier 1 spec should have at least 5 acceptance criteria, found ${acceptanceCount}`
        );
      }
    }

    // Check for missing contracts section
    if (!spec.contracts || spec.contracts.length === 0) {
      warnings.push(
        "Working spec should define contracts for API compatibility"
      );
    }

    // Check for missing non-functional requirements
    if (!spec.non_functional) {
      warnings.push("Working spec should define non-functional requirements");
    } else {
      if (!spec.non_functional.perf) {
        warnings.push(
          "Non-functional requirements should include performance targets"
        );
      }
      if (!spec.non_functional.security) {
        warnings.push(
          "Non-functional requirements should include security requirements"
        );
      }
    }

    // Check for overly broad scope
    if (spec.scope?.out) {
      const broadExclusions = ["node_modules", "dist", "build", ".git"];
      const hasBroadExclusions = spec.scope.out.some((pattern: string) =>
        broadExclusions.some((exclusion) => pattern.includes(exclusion))
      );
      if (!hasBroadExclusions) {
        warnings.push(
          "Scope.out should include standard exclusions (node_modules, dist, etc.)"
        );
      }
    }

    // Check for missing operational rollback SLO
    if (!spec.operational_rollback_slo) {
      warnings.push("Working spec should define operational rollback SLO");
    }

    return warnings;
  }
}

/**
 * Utility function to validate a working spec file
 */
export async function validateWorkingSpecFile(
  filePath: string
): Promise<ValidationResult> {
  try {
    const content = fs.readFileSync(filePath, "utf-8");
    const spec = JSON.parse(content);

    const validator = new SchemaValidator();
    return validator.validateWorkingSpec(spec);
  } catch (error) {
    return {
      valid: false,
      errors: [`Failed to validate ${filePath}: ${error.message}`],
      warnings: [],
    };
  }
}

/**
 * Utility function to validate all working specs in a directory
 */
export async function validateAllWorkingSpecs(
  directory: string
): Promise<Array<{ file: string; result: ValidationResult }>> {
  const results: Array<{ file: string; result: ValidationResult }> = [];
  const validator = new SchemaValidator();

  function scanDirectory(dir: string): void {
    const items = fs.readdirSync(dir);

    for (const item of items) {
      const fullPath = path.join(dir, item);
      const stat = fs.statSync(fullPath);

      if (
        stat.isDirectory() &&
        !item.startsWith(".") &&
        item !== "node_modules"
      ) {
        scanDirectory(fullPath);
      } else if (
        item.endsWith(".caws/working-spec.yaml") ||
        item === "working-spec.yaml"
      ) {
        try {
          const content = fs.readFileSync(fullPath, "utf-8");
          const spec = JSON.parse(content); // Assuming YAML is parsed to JSON

          const result = validator.validateWorkingSpec(spec);
          results.push({ file: fullPath, result });
        } catch (error) {
          results.push({
            file: fullPath,
            result: {
              valid: false,
              errors: [`Failed to parse ${fullPath}: ${error.message}`],
              warnings: [],
            },
          });
        }
      }
    }
  }

  scanDirectory(directory);
  return results;
}
