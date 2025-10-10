/**
 * Contract Testing Framework
 *
 * @author @darianrosebrook
 * @description Framework for testing API contracts, TypeScript interfaces, and protocol compliance
 */

import Ajv from "ajv";
import * as fs from "fs";
import yaml from "js-yaml";

export interface ContractTestResult {
  passed: boolean;
  contractType: string;
  contractPath: string;
  errors: string[];
  warnings: string[];
  coverage: number; // 0-1
}

export interface ContractDefinition {
  type: "typescript" | "openapi" | "jsonrpc" | "graphql";
  path: string;
  version: string;
  description?: string;
}

export class ContractTestFramework {
  private ajv: any;
  private results: ContractTestResult[] = [];

  constructor() {
    this.ajv = new Ajv({
      allErrors: true,
    });
  }

  /**
   * Test a TypeScript interface contract
   */
  async testTypeScriptContract(
    interfacePath: string,
    implementationPath: string,
    interfaceName: string
  ): Promise<ContractTestResult> {
    const result: ContractTestResult = {
      passed: true,
      contractType: "typescript",
      contractPath: interfacePath,
      errors: [],
      warnings: [],
      coverage: 1.0,
    };

    try {
      // Load the interface definition
      const interfaceContent = fs.readFileSync(interfacePath, "utf8");
      const implementationContent = fs.readFileSync(implementationPath, "utf8");

      // Basic checks - interface properties exist in implementation
      const interfaceMatches = interfaceContent.match(
        new RegExp(`interface ${interfaceName}\\s*{([^}]*)}`, "s")
      );
      if (!interfaceMatches) {
        result.errors.push(
          `Interface ${interfaceName} not found in ${interfacePath}`
        );
        result.passed = false;
        return result;
      }

      // Extract interface properties (simplified)
      const interfaceProps = interfaceMatches[1]
        .split("\n")
        .map((line) => line.trim())
        .filter((line) => line.includes(":") && !line.startsWith("//"))
        .map((line) => line.split(":")[0].trim());

      // Check if implementation mentions these properties
      for (const prop of interfaceProps) {
        if (!implementationContent.includes(prop)) {
          result.warnings.push(
            `Property '${prop}' from interface not found in implementation`
          );
          result.coverage -= 0.1;
        }
      }

      result.coverage = Math.max(0, result.coverage);
    } catch (error) {
      result.errors.push(`TypeScript contract test failed: ${error}`);
      result.passed = false;
    }

    this.results.push(result);
    return result;
  }

  /**
   * Test an OpenAPI contract
   */
  async testOpenAPIContract(
    specPath: string,
    _implementationBaseUrl: string = "http://localhost:3000"
  ): Promise<ContractTestResult> {
    const result: ContractTestResult = {
      passed: true,
      contractType: "openapi",
      contractPath: specPath,
      errors: [],
      warnings: [],
      coverage: 0.8, // Assume good coverage initially
    };

    try {
      const specContent = fs.readFileSync(specPath, "utf8");
      const spec = yaml.load(specContent) as any;

      // Validate OpenAPI structure
      if (!spec.openapi || !spec.info || !spec.paths) {
        result.errors.push("Invalid OpenAPI specification structure");
        result.passed = false;
        return result;
      }

      // Check required fields
      if (!spec.info.title || !spec.info.version) {
        result.warnings.push("Missing required OpenAPI info fields");
        result.coverage -= 0.1;
      }

      // Validate paths exist
      if (Object.keys(spec.paths).length === 0) {
        result.errors.push("No API paths defined in OpenAPI spec");
        result.passed = false;
        return result;
      }

      // Basic path validation
      const pathCount = Object.keys(spec.paths).length;
      if (pathCount < 3) {
        result.warnings.push("OpenAPI spec has very few paths defined");
        result.coverage -= 0.2;
      }
    } catch (error) {
      result.errors.push(`OpenAPI contract test failed: ${error}`);
      result.passed = false;
    }

    this.results.push(result);
    return result;
  }

  /**
   * Test JSON-RPC contract
   */
  async testJSONRPCContract(
    specPath: string,
    implementation: any
  ): Promise<ContractTestResult> {
    const result: ContractTestResult = {
      passed: true,
      contractType: "jsonrpc",
      contractPath: specPath,
      errors: [],
      warnings: [],
      coverage: 0.9,
    };

    try {
      const specContent = fs.readFileSync(specPath, "utf8");
      const spec = yaml.load(specContent) as any;

      // Validate JSON-RPC structure
      if (!spec.methods || !Array.isArray(spec.methods)) {
        result.errors.push(
          "Invalid JSON-RPC specification: missing methods array"
        );
        result.passed = false;
        return result;
      }

      // Check each method
      for (const method of spec.methods) {
        if (!method.name || !method.params || !method.result) {
          result.errors.push(
            `Method ${method.name || "unknown"} missing required fields`
          );
          result.passed = false;
        }

        // Check if method exists in implementation
        if (
          implementation &&
          typeof implementation[method.name] !== "function"
        ) {
          result.warnings.push(`Method ${method.name} not implemented`);
          result.coverage -= 0.1;
        }
      }
    } catch (error) {
      result.errors.push(`JSON-RPC contract test failed: ${error}`);
      result.passed = false;
    }

    this.results.push(result);
    return result;
  }

  /**
   * Test a complete contract suite
   */
  async testContractSuite(
    contracts: ContractDefinition[]
  ): Promise<ContractTestResult[]> {
    const results: ContractTestResult[] = [];

    for (const contract of contracts) {
      let result: ContractTestResult;

      switch (contract.type) {
        case "typescript": {
          // For TypeScript, we need to find the implementation
          const implPath = contract.path.replace(".ts", ".impl.ts");
          result = await this.testTypeScriptContract(
            contract.path,
            implPath,
            contract.description || "Interface"
          );
          break;
        }

        case "openapi":
          result = await this.testOpenAPIContract(contract.path);
          break;

        case "jsonrpc":
          result = await this.testJSONRPCContract(contract.path, null);
          break;

        default:
          result = {
            passed: false,
            contractType: contract.type,
            contractPath: contract.path,
            errors: [`Unsupported contract type: ${contract.type}`],
            warnings: [],
            coverage: 0,
          };
      }

      results.push(result);
    }

    return results;
  }

  /**
   * Get all test results
   */
  getResults(): ContractTestResult[] {
    return this.results;
  }

  /**
   * Get summary statistics
   */
  getSummary() {
    const total = this.results.length;
    const passed = this.results.filter((r) => r.passed).length;
    const avgCoverage =
      this.results.reduce((sum, r) => sum + r.coverage, 0) / total;

    return {
      total,
      passed,
      failed: total - passed,
      passRate: total > 0 ? (passed / total) * 100 : 0,
      averageCoverage: avgCoverage,
    };
  }

  /**
   * Reset results
   */
  reset(): void {
    this.results = [];
  }
}
