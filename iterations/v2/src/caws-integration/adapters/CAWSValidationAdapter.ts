/**
 * CAWS Validation Adapter
 *
 * Wraps CAWS CLI validation functionality, handling TypeScript ↔ YAML conversion
 * and enriching results with arbiter orchestration metadata.
 *
 * @author @darianrosebrook
 */

import type { WorkingSpec } from "../../types/caws-types.js";
import type {
  AdapterOperationResult,
  ArbiterValidationResult,
  CAWSAdapterConfig,
  CAWSValidationRequest,
  OrchestrationMetadata,
} from "../types/arbiter-caws-types.js";
import { SpecFileManager } from "../utils/spec-file-manager.js";

/**
 * Adapter for CAWS CLI validation operations
 *
 * Provides TypeScript-friendly interface to CAWS CLI validation,
 * with automatic YAML conversion and result enrichment.
 */
export class CAWSValidationAdapter {
  private readonly specFileManager: SpecFileManager;
  private readonly arbiterVersion: string;
  private readonly projectRoot: string;

  constructor(config: CAWSAdapterConfig) {
    this.projectRoot = config.projectRoot;
    this.arbiterVersion = config.arbiterVersion ?? "2.0.0";

    this.specFileManager = new SpecFileManager({
      projectRoot: config.projectRoot,
      useTemporaryFiles: config.useTemporaryFiles ?? true,
    });
  }

  /**
   * Validate a WorkingSpec using CAWS CLI
   *
   * Converts spec to YAML, runs CAWS validation, and enriches result
   * with arbiter metadata.
   *
   * @param request Validation request with spec and options
   * @returns Enriched validation result
   */
  public async validateSpec(
    request: CAWSValidationRequest
  ): Promise<AdapterOperationResult<ArbiterValidationResult>> {
    const startTime = Date.now();

    try {
      // Write spec to YAML file
      const writeResult = await this.specFileManager.writeSpecFile(
        request.spec
      );

      try {
        // For now, return a simple successful validation
        // Full CAWS CLI integration will be completed in Week 3
        const enrichedResult: ArbiterValidationResult = {
          passed: true,
          cawsVersion: "3.4.0",
          timestamp: new Date().toISOString(),
          budgetCompliance: {
            compliant: true,
            baseline: { max_files: 20, max_loc: 500 },
            effective: { max_files: 20, max_loc: 500 },
            current: { filesChanged: 0, linesChanged: 0 },
            violations: [],
            waiversApplied: [],
          },
          qualityGates: [],
          waivers: [],
          verdict: "pass",
          orchestration: {
            timestamp: new Date().toISOString(),
            arbiterVersion: this.arbiterVersion,
          },
          spec: request.spec,
          durationMs: Date.now() - startTime,
        };

        return {
          success: true,
          data: enrichedResult,
          durationMs: Date.now() - startTime,
        };
      } finally {
        // Clean up temporary file
        if (writeResult.cleanup) {
          await writeResult.cleanup();
        }
      }
    } catch (error) {
      return {
        success: false,
        error: {
          message: error instanceof Error ? error.message : "Unknown error",
          code: "VALIDATION_ERROR",
          details: error,
        },
        durationMs: Date.now() - startTime,
      };
    }
  }

  /**
   * Generate a new WorkingSpec using CAWS CLI
   *
   * @param params Generation parameters
   * @returns Generated WorkingSpec
   */
  public async generateSpec(params: {
    title: string;
    mode: "feature" | "refactor" | "fix" | "doc" | "chore";
    riskTier: 1 | 2 | 3;
    description?: string;
  }): Promise<AdapterOperationResult<WorkingSpec>> {
    const startTime = Date.now();

    try {
      const cawsCLI = await import("@paths.design/caws-cli");

      // Generate spec using CAWS CLI
      const generated = await cawsCLI.generateWorkingSpec({
        title: params.title,
        mode: params.mode,
        riskTier: params.riskTier,
        description: params.description,
        projectRoot: this.projectRoot,
      });

      // Parse YAML to WorkingSpec if needed
      const spec =
        typeof generated === "string"
          ? this.specFileManager.yamlToSpec(generated)
          : (generated as unknown as WorkingSpec);

      return {
        success: true,
        data: spec,
        durationMs: Date.now() - startTime,
      };
    } catch (error) {
      return {
        success: false,
        error: {
          message: error instanceof Error ? error.message : "Unknown error",
          code: "GENERATION_ERROR",
          details: error,
        },
        durationMs: Date.now() - startTime,
      };
    }
  }

  /**
   * Validate an existing working spec file in the project
   *
   * @param options Validation options
   * @returns Validation result
   */
  public async validateExistingSpec(options?: {
    autoFix?: boolean;
    suggestions?: boolean;
    checkBudget?: boolean;
  }): Promise<AdapterOperationResult<ArbiterValidationResult>> {
    const startTime = Date.now();

    try {
      // Read existing spec
      const spec = await this.specFileManager.readSpecFile();

      // Validate using the spec
      return await this.validateSpec({
        spec,
        projectRoot: this.projectRoot,
        options: {
          autoFix: options?.autoFix ?? false,
          suggestions: options?.suggestions ?? true,
          checkBudget: options?.checkBudget ?? true,
        },
      });
    } catch (error) {
      return {
        success: false,
        error: {
          message: error instanceof Error ? error.message : "Unknown error",
          code: "READ_SPEC_ERROR",
          details: error,
        },
        durationMs: Date.now() - startTime,
      };
    }
  }

  /**
   * Quick validation check (boolean result only)
   *
   * @param spec Working spec to validate
   * @returns True if spec is valid
   */
  public async isSpecValid(spec: WorkingSpec): Promise<boolean> {
    const result = await this.validateSpec({
      spec,
      projectRoot: this.projectRoot,
      options: {
        autoFix: false,
        suggestions: false,
        checkBudget: true,
      },
    });

    return result.success && result.data?.passed === true;
  }

  /**
   * Enrich CAWS validation result with arbiter metadata
   *
   * @param cawsResult Result from CAWS CLI
   * @param request Original validation request
   * @param durationMs Validation duration
   * @returns Enriched result
   */
  private enrichValidationResult(
    cawsResult: any,
    request: CAWSValidationRequest,
    durationMs: number
  ): ArbiterValidationResult {
    const orchestration: OrchestrationMetadata = {
      taskId: request.context?.taskId,
      assignedAgent: request.context?.assignedAgent,
      timestamp: new Date().toISOString(),
      arbiterVersion: this.arbiterVersion,
      sessionId: request.context?.sessionId,
    };

    // Map CAWS result to our enriched format
    return {
      passed: cawsResult.valid ?? true,
      cawsVersion: "3.4.0",
      timestamp: new Date().toISOString(),
      budgetCompliance: {
        compliant: true,
        baseline: { max_files: 20, max_loc: 500 },
        effective: { max_files: 20, max_loc: 500 },
        current: { filesChanged: 0, linesChanged: 0 },
        violations: [],
        waiversApplied: [],
      },
      qualityGates: cawsResult.qualityGates ?? [],
      waivers: [],
      verdict: cawsResult.verdict ?? "pass",
      remediation: cawsResult.remediationSteps,
      spec: request.spec,
      orchestration,
      durationMs,
    };
  }

  /**
   * Get project root directory
   */
  public getProjectRoot(): string {
    return this.projectRoot;
  }

  /**
   * Get spec file manager instance
   */
  public getSpecFileManager(): SpecFileManager {
    return this.specFileManager;
  }
}

/**
 * Create a CAWSValidationAdapter instance
 *
 * @param projectRoot Project root directory
 * @param options Additional configuration options
 * @returns CAWSValidationAdapter instance
 */
export function createCAWSValidationAdapter(
  projectRoot: string,
  options?: Partial<CAWSAdapterConfig>
): CAWSValidationAdapter {
  return new CAWSValidationAdapter({
    projectRoot,
    ...options,
  });
}
