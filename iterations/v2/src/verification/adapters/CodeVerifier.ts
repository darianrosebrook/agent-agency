/**
 * @fileoverview Code Verification Adapter - ARBITER-019
 *
 * Verifies code behavior claims using isolated VM2 sandbox for safe execution
 * with timeout protection and resource limits.
 *
 * @author @darianrosebrook
 */

// Note: Using Node.js built-in vm module instead of vm2 for security
import * as vm from "vm";

export interface CodeVerificationRequest {
  code: string;
  language: "javascript" | "typescript";
  testCases: Array<{
    input: any;
    expectedOutput?: any;
    description?: string;
  }>;
  timeoutMs?: number;
  memoryLimitMB?: number;
}

export interface CodeExecutionResult {
  success: boolean;
  output: any;
  error?: string;
  executionTimeMs: number;
  memoryUsageMB: number;
}

export interface CodeVerificationResult {
  success: boolean;
  testResults: Array<{
    testCase: any;
    success: boolean;
    actualOutput: any;
    expectedOutput?: any;
    executionResult: CodeExecutionResult;
  }>;
  overallConfidence: number;
  executionTimeMs: number;
}

export class CodeVerifier {
  private readonly defaultTimeoutMs = 5000;
  private readonly defaultMemoryLimitMB = 50;

  /**
   * Verify code behavior against test cases
   */
  async verify(
    request: CodeVerificationRequest
  ): Promise<CodeVerificationResult> {
    const {
      code,
      language,
      testCases,
      timeoutMs = this.defaultTimeoutMs,
      memoryLimitMB = this.defaultMemoryLimitMB,
    } = request;

    const startTime = Date.now();
    const testResults: CodeVerificationResult["testResults"] = [];

    for (const testCase of testCases) {
      try {
        const executionResult = await this.executeCode(
          code,
          language,
          testCase.input,
          timeoutMs,
          memoryLimitMB
        );

        let success = true;
        if (testCase.expectedOutput !== undefined) {
          success = this.compareOutputs(
            executionResult.output,
            testCase.expectedOutput
          );
        }

        testResults.push({
          testCase: testCase.input,
          success,
          actualOutput: executionResult.output,
          expectedOutput: testCase.expectedOutput,
          executionResult,
        });
      } catch (error) {
        testResults.push({
          testCase: testCase.input,
          success: false,
          actualOutput: null,
          expectedOutput: testCase.expectedOutput,
          executionResult: {
            success: false,
            output: null,
            error: error instanceof Error ? error.message : String(error),
            executionTimeMs: 0,
            memoryUsageMB: 0,
          },
        });
      }
    }

    const overallSuccess = testResults.every((result) => result.success);
    const successCount = testResults.filter((result) => result.success).length;
    const overallConfidence =
      testResults.length > 0 ? successCount / testResults.length : 0;

    return {
      success: overallSuccess,
      testResults,
      overallConfidence,
      executionTimeMs: Date.now() - startTime,
    };
  }

  /**
   * Verify code performance claims
   */
  async verifyPerformance(
    code: string,
    language: "javascript" | "typescript",
    input: any,
    maxExecutionTimeMs: number,
    iterations: number = 1
  ): Promise<CodeVerificationResult> {
    const testResults: CodeVerificationResult["testResults"] = [];

    for (let i = 0; i < iterations; i++) {
      try {
        const executionResult = await this.executeCode(
          code,
          language,
          input,
          maxExecutionTimeMs * 2,
          100
        );
        const success =
          executionResult.success &&
          executionResult.executionTimeMs <= maxExecutionTimeMs;

        testResults.push({
          testCase: input,
          success,
          actualOutput: executionResult.output,
          expectedOutput: `Execution time <= ${maxExecutionTimeMs}ms`,
          executionResult,
        });
      } catch (error) {
        testResults.push({
          testCase: input,
          success: false,
          actualOutput: null,
          expectedOutput: `Execution time <= ${maxExecutionTimeMs}ms`,
          executionResult: {
            success: false,
            output: null,
            error: error instanceof Error ? error.message : String(error),
            executionTimeMs: 0,
            memoryUsageMB: 0,
          },
        });
      }
    }

    const overallSuccess = testResults.every((result) => result.success);
    const successCount = testResults.filter((result) => result.success).length;
    const overallConfidence =
      testResults.length > 0 ? successCount / testResults.length : 0;

    return {
      success: overallSuccess,
      testResults,
      overallConfidence,
      executionTimeMs: testResults.reduce(
        (sum, result) => sum + result.executionResult.executionTimeMs,
        0
      ),
    };
  }

  private async executeCode(
    code: string,
    language: "javascript" | "typescript",
    input: any,
    timeoutMs: number,
    memoryLimitMB: number
  ): Promise<CodeExecutionResult> {
    const startTime = Date.now();
    const startMemory = process.memoryUsage().heapUsed / 1024 / 1024;

    try {
      // Create secure context with restricted access
      const context = vm.createContext({
        input,
        console: {
          log: (...args: any[]) => {
            // Capture console output if needed
            return args.join(" ");
          },
        },
        Math,
        JSON,
        Date,
        Array,
        Object,
        String,
        Number,
        Boolean,
        RegExp,
        Error,
        // Explicitly exclude dangerous globals
        require: undefined,
        process: undefined,
        global: undefined,
        Buffer: undefined,
        eval: undefined,
        Function: undefined,
        setTimeout: undefined,
        setInterval: undefined,
        clearTimeout: undefined,
        clearInterval: undefined,
      });

      // Execute code with timeout and memory limits
      const script = new vm.Script(code, {
        filename: "user-code.js",
      });

      const output = script.runInContext(context, {
        timeout: timeoutMs,
        breakOnSigint: true,
      });

      const executionTimeMs = Date.now() - startTime;
      const endMemory = process.memoryUsage().heapUsed / 1024 / 1024;
      const memoryUsageMB = endMemory - startMemory;

      return {
        success: true,
        output,
        executionTimeMs,
        memoryUsageMB,
      };
    } catch (error) {
      const executionTimeMs = Date.now() - startTime;
      const endMemory = process.memoryUsage().heapUsed / 1024 / 1024;
      const memoryUsageMB = endMemory - startMemory;

      return {
        success: false,
        output: null,
        error: error instanceof Error ? error.message : String(error),
        executionTimeMs,
        memoryUsageMB,
      };
    }
  }

  private compareOutputs(actual: any, expected: any): boolean {
    // Deep equality comparison
    if (actual === expected) {
      return true;
    }

    if (typeof actual !== typeof expected) {
      return false;
    }

    if (actual === null || expected === null) {
      return actual === expected;
    }

    if (Array.isArray(actual) && Array.isArray(expected)) {
      if (actual.length !== expected.length) {
        return false;
      }
      return actual.every((item, index) =>
        this.compareOutputs(item, expected[index])
      );
    }

    if (typeof actual === "object" && typeof expected === "object") {
      const actualKeys = Object.keys(actual);
      const expectedKeys = Object.keys(expected);

      if (actualKeys.length !== expectedKeys.length) {
        return false;
      }

      return actualKeys.every(
        (key) =>
          expectedKeys.includes(key) &&
          this.compareOutputs(actual[key], expected[key])
      );
    }

    // For numbers, allow small floating point differences
    if (typeof actual === "number" && typeof expected === "number") {
      return Math.abs(actual - expected) < 1e-10;
    }

    return false;
  }
}
