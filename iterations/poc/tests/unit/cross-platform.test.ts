/**
 * Cross-Platform Tests
 *
 * @author @darianrosebrook
 * @description Tests ensuring compatibility across different operating systems and environments
 */

import {
  jest,
  describe,
  beforeEach,
  afterEach,
  it,
  expect,
} from "@jest/globals";
// import * as path from "path";
import * as os from "os";
import { AgentOrchestrator } from "../../src/services/AgentOrchestrator";
import { Logger } from "../../src/utils/Logger";

// Mock all dependencies
jest.mock("../../src/memory/MultiTenantMemoryManager");
jest.mock("../../src/services/AdvancedTaskRouter");
jest.mock("../../src/services/ErrorPatternAnalyzer");
jest.mock("../../src/services/CawsConstitutionalEnforcer");
jest.mock("../../src/utils/Logger");

describe("Cross-Platform Tests", () => {
  let orchestrator: AgentOrchestrator;
  let mockLogger: any;

  beforeEach(() => {
    jest.clearAllMocks();

    mockLogger = {
      info: jest.fn(),
      warn: jest.fn(),
      error: jest.fn(),
      debug: jest.fn(),
    };
    (Logger as jest.Mock).mockImplementation(() => mockLogger);

    // Mock minimal dependencies
    const {
      MultiTenantMemoryManager,
    } = require("../../src/memory/MultiTenantMemoryManager");
    MultiTenantMemoryManager.mockImplementation(() => ({
      initialize: jest.fn().mockResolvedValue(undefined),
      registerTenant: jest.fn().mockResolvedValue(undefined),
      storeExperience: jest.fn().mockResolvedValue(undefined),
      retrieveExperiences: jest.fn().mockResolvedValue([]),
    }));

    const {
      AdvancedTaskRouter,
    } = require("../../src/services/AdvancedTaskRouter");
    AdvancedTaskRouter.mockImplementation(() => ({
      submitTask: jest.fn().mockResolvedValue({
        selectedAgentId: "agent-1",
        routingStrategy: "load_balanced",
        confidence: 0.95,
        estimatedLatency: 100,
        expectedQuality: 0.9,
      }),
    }));

    const {
      ErrorPatternAnalyzer,
    } = require("../../src/services/ErrorPatternAnalyzer");
    ErrorPatternAnalyzer.mockImplementation(() => ({
      initialize: jest.fn().mockResolvedValue(undefined),
    }));

    const {
      CawsConstitutionalEnforcer,
    } = require("../../src/services/CawsConstitutionalEnforcer");
    CawsConstitutionalEnforcer.mockImplementation(() => ({
      initialize: jest.fn().mockResolvedValue(undefined),
      enforceConstitution: jest
        .fn()
        .mockResolvedValue({ allowed: true, violations: [] }),
      startBudgetTracking: jest.fn(),
    }));
  });

  afterEach(async () => {
    if (orchestrator) {
      orchestrator = null as any;
    }
    jest.restoreAllMocks();
  });

  describe("Path Handling", () => {
    it("should handle POSIX-style paths (Linux/macOS)", async () => {
      // Mock platform detection
      const originalPlatform = process.platform;
      Object.defineProperty(process, "platform", {
        value: "linux",
        writable: true,
      });

      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test with POSIX paths",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          filePath: "/home/user/project/src/main.ts",
          includePaths: ["/usr/local/include", "/opt/homebrew/include"],
          pathSeparator: "/", // POSIX separator
          pathDelimiter: ":", // POSIX delimiter
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");

      // Restore original platform
      Object.defineProperty(process, "platform", {
        value: originalPlatform,
        writable: true,
      });
    });

    it("should handle Windows-style paths", async () => {
      // Mock platform detection
      const originalPlatform = process.platform;
      Object.defineProperty(process, "platform", {
        value: "win32",
        writable: true,
      });

      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test with Windows paths",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          filePath: "C:\\Users\\user\\project\\src\\main.ts",
          includePaths: [
            "C:\\Program Files\\include",
            "C:\\vcpkg\\installed\\x64-windows\\include",
          ],
          pathSeparator: "\\", // Windows separator
          pathDelimiter: ";", // Windows delimiter
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");

      // Restore original platform
      Object.defineProperty(process, "platform", {
        value: originalPlatform,
        writable: true,
      });
    });

    it("should handle UNC paths (Windows network paths)", async () => {
      const originalPlatform = process.platform;
      Object.defineProperty(process, "platform", {
        value: "win32",
        writable: true,
      });

      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test with UNC paths",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          filePath: "\\\\server\\share\\project\\src\\main.ts",
          networkPaths: ["\\\\nas\\shared", "\\\\backup\\archive"],
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");

      Object.defineProperty(process, "platform", {
        value: originalPlatform,
        writable: true,
      });
    });
  });

  describe("Environment Variables", () => {
    it("should handle environment variables with different naming conventions", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      // Test with various environment variable patterns
      const testEnvs = {
        // Unix-style (uppercase with underscores)
        DATABASE_URL: "postgresql://localhost:5432/db",
        REDIS_HOST: "localhost",
        API_KEY: "sk-123456",

        // Windows-style (may have different conventions)
        "PROGRAMFILES(X86)": "C:\\Program Files (x86)",
        USERPROFILE: "C:\\Users\\testuser",
        APPDATA: "C:\\Users\\testuser\\AppData\\Roaming",

        // Case variations that might occur
        database_url: "postgresql://localhost:5432/db",
        Database_Url: "postgresql://localhost:5432/db",

        // Special characters in values
        COMPLEX_PATH: "/path/with spaces/and/special-chars:!@#$%^&*()",
        JSON_CONFIG: '{"host":"localhost","port":8080}',
      };

      const task = {
        type: "code_generation" as const,
        description: "Test with environment variables",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          environment: testEnvs,
          config: {
            useEnvVars: true,
            envVarMapping: {
              dbUrl: "DATABASE_URL",
              redisHost: "REDIS_HOST",
              apiKey: "API_KEY",
            },
          },
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });

    it("should handle missing environment variables gracefully", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test with missing env vars",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          requiredEnvVars: ["NONEXISTENT_VAR", "MISSING_CONFIG"],
          optionalEnvVars: ["MAYBE_HERE", "OPTIONAL_SETTING"],
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });
  });

  describe("File System Operations", () => {
    it("should handle different file permission models", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      // Test with various file permission scenarios
      const task = {
        type: "code_generation" as const,
        description: "Test file permissions",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          files: [
            // POSIX permissions (rw-r--r--)
            {
              path: "/etc/passwd",
              permissions: 0o644,
              owner: "root",
              group: "root",
            },
            // Windows permissions (different model)
            {
              path: "C:\\Windows\\System32\\config",
              permissions: "SYSTEM:FULL",
              owner: "SYSTEM",
            },
            // Executable files
            { path: "/usr/bin/node", permissions: 0o755, executable: true },
            {
              path: "C:\\Program Files\\nodejs\\node.exe",
              permissions: "Everyone:READ",
              executable: true,
            },
          ],
          operations: [
            "read_config",
            "write_cache",
            "execute_binary",
            "create_temp_file",
          ],
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });

    it("should handle different line ending conventions", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      // Test with different line ending styles
      const task = {
        type: "text_transformation" as const,
        description: "Test line endings",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          content: "Line 1\r\nLine 2\nLine 3\rLine 4", // Mixed line endings
          expectedOutput: "Line 1\nLine 2\nLine 3\nLine 4", // Normalized
          preserveOriginal: false,
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });

    it("should handle different case sensitivity rules", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      // Test case sensitivity differences
      const task = {
        type: "code_generation" as const,
        description: "Test case sensitivity",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          // POSIX is case-sensitive
          posixImports: [
            { from: "./utils/Logger", to: "Logger" },
            { from: "./Utils/Logger", to: "Logger" }, // Different case
          ],
          // Windows is case-insensitive
          windowsImports: [
            { from: ".\\utils\\Logger", to: "Logger" },
            { from: ".\\Utils\\Logger", to: "Logger" }, // Same file on Windows
          ],
          caseSensitive: process.platform !== "win32",
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });
  });

  describe("Process and Command Execution", () => {
    it("should handle different shell command syntax", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      // Test commands for different platforms
      const task = {
        type: "code_generation" as const,
        description: "Test shell commands",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          commands: {
            // Unix/Linux/macOS
            unix: {
              listFiles: "ls -la",
              createDir: "mkdir -p /tmp/test",
              findFiles: "find /tmp -name '*.log'",
              pipeCommands: "cat file.txt | grep 'pattern' | wc -l",
            },
            // Windows (cmd.exe)
            windows: {
              listFiles: "dir /a",
              createDir: "mkdir C:\\temp\\test",
              findFiles:
                'forfiles /P "C:\\temp" /M "*.log" /C "cmd /c echo @file"',
              pipeCommands:
                'type file.txt | findstr "pattern" | find /c "pattern"',
            },
            // PowerShell (Windows)
            powershell: {
              listFiles: "Get-ChildItem -Force",
              createDir:
                "New-Item -ItemType Directory -Path 'C:\\temp\\test' -Force",
              findFiles:
                "Get-ChildItem -Path 'C:\\temp' -Filter '*.log' -Recurse",
              pipeCommands:
                "Get-Content file.txt | Select-String 'pattern' | Measure-Object | Select-Object -ExpandProperty Count",
            },
          },
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });

    it("should handle different process spawning patterns", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test process spawning",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          processes: [
            // Node.js processes
            {
              command: "node",
              args: ["--version"],
              platform: "all",
            },
            // Python processes
            {
              command: process.platform === "win32" ? "python.exe" : "python3",
              args: ["--version"],
              platform: "all",
            },
            // System commands
            {
              command: process.platform === "win32" ? "cmd.exe" : "/bin/sh",
              args:
                process.platform === "win32"
                  ? ["/c", "echo hello"]
                  : ["-c", "echo hello"],
              platform: "all",
            },
          ],
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });
  });

  describe("Network Operations", () => {
    it("should handle different network address formats", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test network addresses",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          endpoints: [
            // IPv4
            "http://192.168.1.100:8080/api",
            "http://10.0.0.1:3000/health",
            // IPv6
            "http://[2001:db8::1]:8080/api",
            "http://[::1]:3000/health",
            // Hostnames
            "http://localhost:8080/api",
            "http://example.com:443/api",
            // Windows named pipes (local only)
            "\\\\.\\pipe\\agent-orchestrator",
            // Unix domain sockets
            "/tmp/agent-orchestrator.sock",
          ],
          timeouts: {
            connection: 5000,
            read: 10000,
            write: 5000,
          },
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });

    it("should handle different TLS/SSL configurations", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test SSL configurations",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          sslConfigs: [
            // Standard HTTPS
            {
              url: "https://api.example.com",
              rejectUnauthorized: true,
            },
            // Self-signed certificates (development)
            {
              url: "https://localhost:8443",
              rejectUnauthorized: false,
              ca: "/path/to/ca.crt",
            },
            // Client certificates
            {
              url: "https://secure-api.example.com",
              cert: "/path/to/client.crt",
              key: "/path/to/client.key",
              ca: "/path/to/ca.crt",
            },
            // Windows certificate store
            {
              url: "https://windows-api.example.com",
              pfx: "C:\\path\\to\\cert.pfx",
              passphrase: "cert_password",
            },
          ],
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });
  });

  describe("Time and Date Handling", () => {
    it("should handle different timezone configurations", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test timezones",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          timestamps: [
            // ISO 8601 with timezone
            "2024-01-15T10:30:00Z",
            "2024-01-15T10:30:00+05:00",
            "2024-01-15T10:30:00-08:00",
            // Unix timestamps
            1705312200000,
            // Locale-specific formats
            "2024-01-15 10:30:00 GMT",
            "Mon, 15 Jan 2024 10:30:00 GMT",
          ],
          timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
          locale: Intl.DateTimeFormat().resolvedOptions().locale,
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });

    it("should handle daylight saving time transitions", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test DST transitions",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          dstTransitions: [
            // Spring forward (lose an hour)
            {
              before: "2024-03-10T01:59:59",
              after: "2024-03-10T03:00:01",
              timezone: "America/New_York",
            },
            // Fall back (gain an hour)
            {
              before: "2024-11-03T01:59:59",
              after: "2024-11-03T01:00:01",
              timezone: "America/New_York",
            },
            // No DST
            {
              before: "2024-06-15T12:00:00",
              after: "2024-06-15T13:00:00",
              timezone: "UTC",
            },
          ],
          scheduleTasks: [
            {
              cron: "0 2 * * *", // 2 AM daily - problematic during DST
              timezone: "America/New_York",
              description: "Daily maintenance",
            },
          ],
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });
  });

  describe("Character Encoding", () => {
    it("should handle different text encodings", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "text_transformation" as const,
        description: "Test text encodings",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          encodings: [
            // UTF-8 (most common)
            { encoding: "utf8", content: "Hello 🌍" },
            // ASCII (limited)
            { encoding: "ascii", content: "Hello World" },
            // Latin-1 (Western European)
            { encoding: "latin1", content: "Café résumé" },
            // UTF-16 (Windows default for some operations)
            { encoding: "utf16le", content: "Hello 🌍" },
            // Base64 (common for data transfer)
            { encoding: "base64", content: "SGVsbG8g8J+RiyA=" },
          ],
          transformations: [
            "normalize_unicode",
            "convert_encoding",
            "detect_encoding",
            "handle_bom", // Byte Order Mark
          ],
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });

    it("should handle platform-specific filename encodings", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test filename encodings",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          filenames: [
            // ASCII-safe names
            "config.json",
            "main.ts",
            // Unicode names (different handling per platform)
            "café.config.json",
            "测试文件.txt",
            "файл.txt",
            "📁.config",
            // Names with special characters
            "file with spaces.txt",
            "file-with-dashes.txt",
            "file_with_underscores.txt",
            // Platform-specific issues
            "aux.txt", // Reserved on Windows
            "COM1.txt", // Reserved on Windows
            ".hidden", // Hidden files
          ],
          operations: [
            "create_files",
            "read_files",
            "list_directory",
            "resolve_paths",
          ],
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });
  });

  describe("Resource Limits and System Constraints", () => {
    it("should handle different memory limits", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test memory limits",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          memoryConstraints: {
            // Node.js default heap limit
            nodeDefault: "1.4GB",
            // System memory
            systemTotal: `${os.totalmem() / 1024 / 1024}MB`,
            systemFree: `${os.freemem() / 1024 / 1024}MB`,
            // Container limits
            containerLimit: "512MB",
            // Platform-specific limits
            platform: process.platform,
            arch: process.arch,
          },
          largeDataStructures: [
            // Large arrays
            Array.from({ length: 100000 }, (_, i) => ({
              id: i,
              data: "x".repeat(100),
            })),
            // Deep nesting
            createDeepObject(50),
            // Mixed types
            {
              strings: Array.from({ length: 1000 }, () => "test string"),
              numbers: Array.from({ length: 1000 }, (_, i) => i),
              objects: Array.from({ length: 100 }, () => ({
                nested: { deep: { value: Math.random() } },
              })),
            },
          ],
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });

    it("should handle different CPU core configurations", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Test CPU configurations",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {
          cpuInfo: {
            cores: os.cpus().length,
            architecture: process.arch,
            platform: process.platform,
          },
          concurrency: {
            // Default concurrency based on CPU cores
            default: Math.max(1, os.cpus().length - 1),
            // Platform-specific recommendations
            recommendations: {
              linux: Math.max(1, os.cpus().length),
              darwin: Math.max(1, os.cpus().length - 1),
              win32: Math.max(1, os.cpus().length - 1),
            },
          },
          parallelTasks: Array.from(
            { length: Math.min(10, os.cpus().length) },
            (_, i) => ({
              id: `task-${i}`,
              type: "cpu_intensive",
              priority: 1,
            })
          ),
        },
      };

      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });
  });
});

// Helper function for creating deep objects
function createDeepObject(depth: number): any {
  if (depth === 0) return { value: "leaf" };
  return { nested: createDeepObject(depth - 1) };
}
