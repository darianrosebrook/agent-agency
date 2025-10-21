/**
 * @fileoverview Tests for MockCodeQualityGate
 * Tests detection of mocked/placeholder implementations
 * @module caws-validator/__tests__/mock-code-quality-gate
 */

import * as fs from "fs";
import * as path from "path";
import { WorkingSpec } from "../../types/caws-types";
import { CAWSValidator } from "../CAWSValidator";

// Mock fs and path for testing
jest.mock("fs");
jest.mock("path");

const mockedFs = fs as jest.Mocked<typeof fs>;
const mockedPath = path as jest.Mocked<typeof path>;

describe("MockCodeQualityGate", () => {
  let validator: CAWSValidator;
  let mockSpec: WorkingSpec;

  beforeEach(() => {
    // Reset mocks
    jest.clearAllMocks();

    // Mock path.join to return the input path
    mockedPath.join.mockImplementation((...args) => args[args.length - 1]);

    // Create validator instance
    validator = new CAWSValidator();

    // Create mock working spec
    mockSpec = {
      id: "TEST-001",
      title: "Test Spec",
      risk_tier: 2,
      mode: "feature",
      change_budget: {
        max_files: 10,
        max_loc: 500,
      },
      blast_radius: {
        modules: [],
        data_migration: false,
      },
      operational_rollback_slo: "5m",
      scope: {
        in: ["src/service-file.ts"],
        out: [],
      },
      invariants: [],
      acceptance: [],
      non_functional: {
        perf: {},
        security: [],
      },
      contracts: [],
    };
  });

  describe("executeMockCodeQualityGate", () => {
    it("should pass when no mock code is found", async () => {
      // Mock clean file content
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        export class TestService {
          async processData(data: any) {
            return data.map(item => item.value);
          }
        }
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.gate).toBe("mock-code-detection");
      expect(result.passed).toBe(true);
      expect(result.score).toBe(100);
      expect(result.message).toContain(
        "✅ No mocked or placeholder implementations found"
      );
      expect(result.evidence).toEqual([]);
    });

    it("should detect TODO comments", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        export class TestService {
          // TODO: Implement proper error handling
          async processData(data: any) {
            return [];
          }
        }
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(false);
      expect(result.score).toBe(80); // 100 - 20 points for 2 findings (TODO + return [])
      expect(result.message).toContain(
        "🚫 Found 2 instances of mocked/placeholder code"
      );
      expect(result.evidence).toHaveLength(2);
      expect(result.evidence[0]).toMatchObject({
        file: "src/service-file.ts",
        line: 3,
        pattern: "\\/\\/\\s*TODO:",
        content: "// TODO: Implement proper error handling",
      });
    });

    it("should detect PLACEHOLDER comments", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        export class TestService {
          // PLACEHOLDER: Add real validation logic here
          validateInput(input: any) {
            return true;
          }
        }
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(false);
      expect(result.score).toBe(80); // 100 - 20 points for 2 findings
      expect(
        result.evidence.some((e: any) => e.content.includes("PLACEHOLDER:"))
      ).toBe(true);
    });

    it("should detect 'In a real implementation' comments", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        export class TestService {
          // In a real implementation, this would validate against the database
          validateUser(userId: string) {
            return true;
          }
        }
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(false);
      expect(result.score).toBe(80); // 100 - 20 points for 2 findings
      expect(
        result.evidence.some((e: any) =>
          e.content.includes("In a real implementation")
        )
      ).toBe(true);
    });

    it("should detect MOCK DATA comments", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        export class TestService {
          // MOCK DATA: Using fake users for development
          getUsers() {
            return [
              { id: "user-1", name: "Fake User" }
            ];
          }
        }
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(false);
      expect(result.score).toBe(90);
      expect(result.evidence[0].content).toContain("MOCK DATA:");
    });

    it("should detect hardcoded mock agent IDs", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        export class TaskRouter {
          routeToAgent(taskId: string) {
            // TODO: Implement sophisticated mock task routing with realistic scenarios
            // - Support multiple routing strategies (load balancing, capability matching, etc.)
            // - Add configurable routing rules and agent availability simulation
            // - Implement routing failure scenarios and fallback mechanisms
            // - Support task complexity assessment and appropriate agent selection
            // - Add routing performance metrics and decision tracking
            // - Implement routing consistency and determinism for testing
            // - Support routing configuration and scenario customization
            // - Add routing validation and correctness checking
            return "agent-1";
          }
        }
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(false);
      expect(result.score).toBe(80); // 100 - 20 points for 2 findings
      expect(
        result.evidence.some((e: any) => e.content.includes("agent-1"))
      ).toBe(true);
    });

    it("should detect placeholder return statements", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        export class DataService {
          getAllUsers() {
            return [];
          }

          getUserConfig() {
            return {};
          }

          getDefaultValue() {
            return "default";
          }

          isValid() {
            return true;
          }
        }
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(false);
      expect(result.score).toBe(60); // 100 - 40 points for 4 findings
      expect(result.evidence).toHaveLength(4);
      expect(result.evidence.map((e: any) => e.content)).toEqual(
        expect.arrayContaining([
          "return [];",
          "return {};",
          'return "default";',
          "return true;",
        ])
      );
    });

    it("should detect console logging implementations", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        export class Logger {
          logError(message: string, error: any) {
            console.error("Error occurred:", error);
          }

          logWarning(message: string) {
            console.warn("Warning:", message);
          }

          logInfo(message: string) {
            console.log("Info:", message);
          }
        }
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(false);
      expect(result.score).toBe(70); // 100 - 30 points for 3 findings
      expect(result.evidence).toHaveLength(3);
    });

    it("should handle multiple mock patterns in one file", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        export class ProblematicService {
          // TODO: Fix this implementation
          // In a real implementation, this would connect to database
          async getData() {
            // MOCK DATA: Fake data for development
            return [
              { id: "agent-1", name: "Mock Agent" }
            ];
          }

          validate() {
            console.log("Validating...");
            return true;
          }
        }
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(false);
      expect(result.score).toBe(40); // 100 - 60 points for 6 findings
      expect(result.evidence).toHaveLength(6);
    });

    it("should skip non-existent files", async () => {
      mockedFs.existsSync.mockReturnValue(false);

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(true);
      expect(result.score).toBe(100);
      expect(result.evidence).toEqual([]);
    });

    it("should skip test files", async () => {
      const testSpec = {
        ...mockSpec,
        scope: {
          in: ["src/test-file.test.ts", "src/__tests__/file.ts"],
          out: [],
        },
      };

      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue(`
        // This should be ignored because it's in a test file
        // TODO: This is a test TODO that should not be flagged
        export const testData = [];
      `);

      const result = await (validator as any).executeMockCodeQualityGate(
        testSpec,
        Date.now()
      );

      expect(result.passed).toBe(true);
      expect(result.score).toBe(100);
      expect(result.evidence).toEqual([]);
    });

    it("should handle file read errors gracefully", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockImplementation(() => {
        throw new Error("File read error");
      });

      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        Date.now()
      );

      expect(result.passed).toBe(true);
      expect(result.score).toBe(100);
      expect(result.evidence).toEqual([]);
    });

    it("should scan files in both scope.in and scope.out (if they contain src/)", async () => {
      const complexSpec = {
        ...mockSpec,
        scope: {
          in: ["src/main-file.ts"],
          out: ["src/out-file.ts", "node_modules/dep.ts"],
        },
      };

      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValueOnce(`
          // Clean file
          export const clean = true;
        `).mockReturnValueOnce(`
          // File with mock code
          // TODO: Implement this
          export const mock = [];
        `);

      const result = await (validator as any).executeMockCodeQualityGate(
        complexSpec,
        Date.now()
      );

      expect(result.passed).toBe(false);
      expect(result.score).toBe(90);
      expect(result.evidence).toHaveLength(1);
      expect(result.evidence[0].file).toBe("src/out-file.ts");
    });

    it("should include execution time", async () => {
      mockedFs.existsSync.mockReturnValue(true);
      mockedFs.readFileSync.mockReturnValue("// Clean file");

      const startTime = Date.now();
      const result = await (validator as any).executeMockCodeQualityGate(
        mockSpec,
        startTime
      );

      expect(result.executionTime).toBeGreaterThanOrEqual(0);
      expect(typeof result.executionTime).toBe("number");
    });
  });
});
