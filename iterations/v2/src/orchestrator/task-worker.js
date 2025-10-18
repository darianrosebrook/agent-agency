/**
 * Task Worker - ARBITER-014
 *
 * Worker thread implementation for isolated task execution.
 *
 * @author @darianrosebrook
 */

import { performance } from "perf_hooks";
import { parentPort, workerData } from "worker_threads";
// Import will be handled dynamically since we can't import TS directly in JS worker
// We'll create a proper sandbox implementation below

const {
  workerId,
  capabilities,
  artifactConfig = {
    rootPath: "./output/artifacts",
    maxFileSizeBytes: 10 * 1024 * 1024, // 10MB
    maxTotalFiles: 100,
    maxPathLength: 255,
  },
} = workerData;

let isRunning = true;
let currentTask = null;
let currentSandbox = null;

/**
 * Create a proper artifact sandbox for task execution.
 * This implements the same interface as ArtifactSandbox but in JavaScript.
 */
async function createArtifactSandbox(taskId, config) {
  const fs = await import("fs/promises");
  const path = await import("path");
  const crypto = await import("crypto");

  const artifactDir = path.resolve(config.rootPath, taskId);
  const files = new Map();
  let totalSize = 0;

  return {
    rootPath: config.rootPath,
    taskId: taskId,
    artifactDir: artifactDir,

    async initialize() {
      try {
        await fs.mkdir(artifactDir, { recursive: true });
      } catch (error) {
        throw new Error(
          `Failed to create artifact directory: ${error.message}`
        );
      }
    },

    async writeFile(relativePath, content) {
      // Validate path
      if (!relativePath || relativePath.trim() === "") {
        throw new Error("Path cannot be empty");
      }

      // Check for path traversal
      if (relativePath.includes("..") || path.isAbsolute(relativePath)) {
        throw new Error(
          "Invalid path: path traversal or absolute path not allowed"
        );
      }

      const fullPath = path.join(artifactDir, relativePath);
      const contentBuffer = Buffer.isBuffer(content)
        ? content
        : Buffer.from(content, "utf8");

      // Check file size quota
      if (contentBuffer.length > config.maxFileSizeBytes) {
        throw new Error(
          `File size ${contentBuffer.length} bytes exceeds limit of ${config.maxFileSizeBytes} bytes`
        );
      }

      // Check total size quota
      const newTotalSize = totalSize + contentBuffer.length;
      const maxTotalSize = config.maxFileSizeBytes * config.maxTotalFiles;
      if (newTotalSize > maxTotalSize) {
        throw new Error(
          `Total artifact size would exceed quota (${newTotalSize} > ${maxTotalSize})`
        );
      }

      // Check file count quota
      if (files.size >= config.maxTotalFiles) {
        throw new Error(
          `File count ${files.size} exceeds limit of ${config.maxTotalFiles} files`
        );
      }

      // Ensure directory exists
      const dir = path.dirname(fullPath);
      if (dir !== artifactDir) {
        await fs.mkdir(dir, { recursive: true });
      }

      // Write file
      await fs.writeFile(fullPath, contentBuffer);

      // Generate SHA256 digest
      const sha256 = crypto
        .createHash("sha256")
        .update(contentBuffer)
        .digest("hex");

      // Detect MIME type (basic)
      const ext = path.extname(relativePath).toLowerCase();
      const mimeTypes = {
        ".json": "application/json",
        ".js": "application/javascript",
        ".ts": "application/typescript",
        ".html": "text/html",
        ".css": "text/css",
        ".md": "text/markdown",
        ".txt": "text/plain",
        ".yml": "text/yaml",
        ".yaml": "text/yaml",
        ".xml": "text/xml",
        ".csv": "text/csv",
        ".png": "image/png",
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".gif": "image/gif",
        ".svg": "image/svg+xml",
        ".pdf": "application/pdf",
      };
      const mimeType = mimeTypes[ext];

      // Track file metadata
      const fileEntry = {
        path: relativePath,
        size: contentBuffer.length,
        sha256: sha256,
        mimeType: mimeType,
        createdAt: new Date().toISOString(),
      };

      files.set(relativePath, fileEntry);
      totalSize += contentBuffer.length;

      console.log(
        `Artifact written: ${relativePath} (${contentBuffer.length} bytes)`
      );
      return { success: true, path: relativePath };
    },

    async mkdir(relativePath) {
      // Validate path
      if (!relativePath || relativePath.trim() === "") {
        throw new Error("Path cannot be empty");
      }

      if (relativePath.includes("..") || path.isAbsolute(relativePath)) {
        throw new Error(
          "Invalid path: path traversal or absolute path not allowed"
        );
      }

      const fullPath = path.join(artifactDir, relativePath);
      await fs.mkdir(fullPath, { recursive: true });
      console.log(`Artifact directory created: ${relativePath}`);
    },

    async readdir(relativePath) {
      // Validate path
      if (
        relativePath &&
        (relativePath.includes("..") || path.isAbsolute(relativePath))
      ) {
        throw new Error(
          "Invalid path: path traversal or absolute path not allowed"
        );
      }

      const fullPath = relativePath
        ? path.join(artifactDir, relativePath)
        : artifactDir;
      return await fs.readdir(fullPath);
    },

    async stat(relativePath) {
      // Validate path
      if (!relativePath || relativePath.trim() === "") {
        throw new Error("Path cannot be empty");
      }

      if (relativePath.includes("..") || path.isAbsolute(relativePath)) {
        throw new Error(
          "Invalid path: path traversal or absolute path not allowed"
        );
      }

      const fullPath = path.join(artifactDir, relativePath);
      const stats = await fs.stat(fullPath);

      return {
        size: stats.size,
        isFile: stats.isFile(),
        isDirectory: stats.isDirectory(),
      };
    },

    async rename(oldPath, newPath) {
      // Validate both paths
      if (
        !oldPath ||
        !newPath ||
        oldPath.trim() === "" ||
        newPath.trim() === ""
      ) {
        throw new Error("Paths cannot be empty");
      }

      if (
        oldPath.includes("..") ||
        newPath.includes("..") ||
        path.isAbsolute(oldPath) ||
        path.isAbsolute(newPath)
      ) {
        throw new Error(
          "Invalid path: path traversal or absolute path not allowed"
        );
      }

      const oldFullPath = path.join(artifactDir, oldPath);
      const newFullPath = path.join(artifactDir, newPath);

      await fs.rename(oldFullPath, newFullPath);

      // Update tracking if this was a tracked file
      if (files.has(oldPath)) {
        const fileEntry = files.get(oldPath);
        fileEntry.path = newPath;
        files.delete(oldPath);
        files.set(newPath, fileEntry);
      }

      console.log(`Artifact renamed: ${oldPath} -> ${newPath}`);
    },

    generateManifest() {
      return {
        taskId: taskId,
        files: Array.from(files.values()),
        totalSize: totalSize,
        createdAt: new Date().toISOString(),
      };
    },

    getRootPath() {
      return artifactDir;
    },

    getManifest() {
      return {
        taskId: taskId,
        files: Array.from(files.values()),
        totalSize: totalSize,
        createdAt: new Date().toISOString(),
      };
    },
  };
}

// Task execution functions
const taskExecutors = {
  script: executeScriptTask,
  api_call: executeApiCallTask,
  data_processing: executeDataProcessingTask,
  ai_inference: executeAIInferenceTask,
  file_editing: executeFileEditingTask,
};

async function executeScriptTask(task) {
  const { code, args = [], timeout = 30000 } = task.payload;

  // Initialize proper sandbox for this task
  if (!currentSandbox) {
    currentSandbox = await createArtifactSandbox(task.id, artifactConfig);
    await currentSandbox.initialize();
  }

  // Create isolated context
  const context = {
    console: {
      log: (...args) => logs.push(`[LOG] ${args.join(" ")}`),
      error: (...args) => logs.push(`[ERROR] ${args.join(" ")}`),
      warn: (...args) => logs.push(`[WARN] ${args.join(" ")}`),
    },
    args,
    result: null,
    artifacts: {
      writeFile: async (path, content) =>
        await currentSandbox.writeFile(path, content),
      mkdir: async (path) => await currentSandbox.mkdir(path),
      readdir: async (path) => await currentSandbox.readdir(path),
      stat: async (path) => await currentSandbox.stat(path),
      rename: async (oldPath, newPath) =>
        await currentSandbox.rename(oldPath, newPath),
    },
  };

  const logs = [];
  const startTime = performance.now();

  try {
    // Execute code with timeout

    // Use Function constructor for isolated execution
    const executeFn = new Function(
      "context",
      `
      const { console, args, artifacts } = context;
      let result = context.result;
      
      // Execute the user code directly
      ${code}
      
      // Update context result
      context.result = result;
    `
    );

    const result = await Promise.race([
      executeFn(context),
      new Promise((_, reject) =>
        setTimeout(() => reject(new Error("Script execution timeout")), timeout)
      ),
    ]);

    const executionTime = performance.now() - startTime;
    const manifest = currentSandbox ? currentSandbox.generateManifest() : null;

    return {
      success: true,
      result: context.result || result,
      logs,
      metrics: {
        executionTime,
        cpuUsage: process.cpuUsage().user / 1000, // microseconds to milliseconds
        memoryUsage: process.memoryUsage().heapUsed,
        outputSize: manifest ? manifest.totalSize : 0,
      },
      artifacts: manifest
        ? {
            manifest,
            rootPath: currentSandbox.getRootPath(),
          }
        : undefined,
    };
  } catch (error) {
    const executionTime = performance.now() - startTime;
    const manifest = currentSandbox ? currentSandbox.generateManifest() : null;

    return {
      success: false,
      error: error.message,
      logs,
      metrics: {
        executionTime,
        cpuUsage: process.cpuUsage().user / 1000,
        memoryUsage: process.memoryUsage().heapUsed,
        outputSize: manifest ? manifest.totalSize : 0,
      },
      artifacts: manifest
        ? {
            manifest,
            rootPath: currentSandbox.getRootPath(),
          }
        : undefined,
    };
  }
}

async function executeApiCallTask(task) {
  const { method, url, headers = {}, body, timeout = 30000 } = task.payload;
  const logs = [];
  const startTime = performance.now();

  try {
    // Basic HTTP client (in real implementation, use axios or fetch)
    const http = await import(
      method.toLowerCase().startsWith("http") ? "http" : "https"
    );
    const urlObj = new URL(url);

    const options = {
      hostname: urlObj.hostname,
      port: urlObj.port || (urlObj.protocol === "https:" ? 443 : 80),
      path: urlObj.pathname + urlObj.search,
      method: method.toUpperCase(),
      headers: {
        "User-Agent": "TaskWorker/1.0",
        ...headers,
      },
      timeout,
    };

    const result = await new Promise((resolve, reject) => {
      const req = http.default.request(options, (res) => {
        let data = "";

        res.on("data", (chunk) => {
          data += chunk;
        });

        res.on("end", () => {
          try {
            const parsed = JSON.parse(data);
            resolve({
              status: res.statusCode,
              headers: res.headers,
              data: parsed,
            });
          } catch {
            resolve({
              status: res.statusCode,
              headers: res.headers,
              data,
            });
          }
        });
      });

      req.on("error", reject);
      req.on("timeout", () => {
        req.destroy();
        reject(new Error("Request timeout"));
      });

      if (body) {
        req.write(typeof body === "string" ? body : JSON.stringify(body));
      }

      req.end();
    });

    const executionTime = performance.now() - startTime;

    return {
      success: true,
      result,
      logs,
      metrics: {
        executionTime,
        cpuUsage: process.cpuUsage().user / 1000,
        memoryUsage: process.memoryUsage().heapUsed,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: error.message,
      logs,
      metrics: {
        executionTime: performance.now() - startTime,
        cpuUsage: process.cpuUsage().user / 1000,
        memoryUsage: process.memoryUsage().heapUsed,
      },
    };
  }
}

async function executeDataProcessingTask(task) {
  const { operation, data, config = {} } = task.payload;
  const logs = [];
  const startTime = performance.now();

  try {
    let result;

    switch (operation) {
      case "filter":
        result = data.filter((item) => {
          try {
            return new Function("item", `return ${config.filter}`)(item);
          } catch (error) {
            logs.push(`Filter error for item: ${error.message}`);
            return false;
          }
        });
        break;

      case "map":
        result = data.map((item) => {
          try {
            return new Function("item", `return ${config.map}`)(item);
          } catch (error) {
            logs.push(`Map error for item: ${error.message}`);
            return item;
          }
        });
        break;

      case "reduce":
        result = data.reduce((acc, item) => {
          try {
            return new Function("acc", "item", `return ${config.reduce}`)(
              acc,
              item
            );
          } catch (error) {
            logs.push(`Reduce error for item: ${error.message}`);
            return acc;
          }
        }, config.initialValue);
        break;

      case "sort":
        result = [...data].sort((a, b) => {
          try {
            return new Function("a", "b", `return ${config.sort}`)(a, b);
          } catch (error) {
            logs.push(`Sort error: ${error.message}`);
            return 0;
          }
        });
        break;

      default:
        throw new Error(`Unknown data processing operation: ${operation}`);
    }

    const executionTime = performance.now() - startTime;

    return {
      success: true,
      result,
      logs,
      metrics: {
        executionTime,
        cpuUsage: process.cpuUsage().user / 1000,
        memoryUsage: process.memoryUsage().heapUsed,
        outputSize: JSON.stringify(result).length,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: error.message,
      logs,
      metrics: {
        executionTime: performance.now() - startTime,
        cpuUsage: process.cpuUsage().user / 1000,
        memoryUsage: process.memoryUsage().heapUsed,
      },
    };
  }
}

async function executeAIInferenceTask(task) {
  const { model, prompt, parameters = {} } = task.payload;
  const logs = [];
  const startTime = performance.now();

  try {
    // Placeholder for AI inference - in real implementation,
    // this would integrate with actual AI services
    logs.push(`AI inference requested for model: ${model}`);

    // Simulate AI processing time
    await new Promise((resolve) => setTimeout(resolve, 1000));

    const mockResponse = {
      model,
      prompt,
      response: `Mock AI response for prompt: ${prompt.substring(0, 50)}...`,
      confidence: 0.85,
      tokens: prompt.split(" ").length + 50,
    };

    const executionTime = performance.now() - startTime;

    return {
      success: true,
      result: mockResponse,
      logs,
      metrics: {
        executionTime,
        cpuUsage: process.cpuUsage().user / 1000,
        memoryUsage: process.memoryUsage().heapUsed,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: error.message,
      logs,
      metrics: {
        executionTime: performance.now() - startTime,
        cpuUsage: process.cpuUsage().user / 1000,
        memoryUsage: process.memoryUsage().heapUsed,
      },
    };
  }
}

async function executeFileEditingTask(task) {
  const { operations, projectRoot, timeout = 60000 } = task.payload;

  const logs = [];
  const startTime = performance.now();

  try {
    // Import required modules
    const fs = await import("fs/promises");
    const path = await import("path");
    const { exec } = await import("child_process");
    const util = await import("util");
    const execAsync = util.promisify(exec);

    // Set project root (default to current working directory)
    const workingDir = projectRoot || process.cwd();

    // Create file editing context
    const context = {
      console: {
        log: (...args) => logs.push(`[LOG] ${args.join(" ")}`),
        error: (...args) => logs.push(`[ERROR] ${args.join(" ")}`),
        warn: (...args) => logs.push(`[WARN] ${args.join(" ")}`),
      },
      projectRoot: workingDir,
      operations: operations || [],
      results: [],

      // File editing tools
      tools: {
        file_read: async (targetFile, offset, limit) => {
          const fullPath = path.resolve(workingDir, targetFile);
          let content = await fs.readFile(fullPath, "utf-8");

          if (offset !== undefined || limit !== undefined) {
            const lines = content.split("\n");
            const start = offset ? offset - 1 : 0;
            const end = limit ? start + limit : lines.length;
            content = lines.slice(start, end).join("\n");
          }

          return content;
        },

        file_search_replace: async (
          filePath,
          oldString,
          newString,
          replaceAll = false
        ) => {
          const fullPath = path.resolve(workingDir, filePath);
          let content = await fs.readFile(fullPath, "utf-8");

          if (replaceAll) {
            content = content.replaceAll(oldString, newString);
          } else {
            content = content.replace(oldString, newString);
          }

          await fs.writeFile(fullPath, content, "utf-8");
          return `Successfully updated file: ${filePath}`;
        },

        file_write: async (filePath, contents) => {
          const fullPath = path.resolve(workingDir, filePath);
          await fs.writeFile(fullPath, contents, "utf-8");
          return `Successfully wrote file: ${filePath}`;
        },

        run_terminal_cmd: async (command, isBackground = false) => {
          // Basic security checks
          const dangerousPatterns = [
            /rm\s+-rf\s+\//,
            />/,
            /sudo/,
            /chmod\s+777/,
            /dd\s+if=/,
          ];

          if (dangerousPatterns.some((pattern) => pattern.test(command))) {
            throw new Error(
              "Command contains potentially dangerous operations"
            );
          }

          if (isBackground) {
            // Run in background
            const child = exec(command, { cwd: workingDir });
            return `Command started in background: ${command}`;
          } else {
            // Run synchronously
            const { stdout, stderr } = await execAsync(command, {
              cwd: workingDir,
            });
            return `Command: ${command}\nOutput:\n${stdout}${
              stderr ? `\nErrors:\n${stderr}` : ""
            }`;
          }
        },
      },
    };

    // Execute operations
    for (const operation of operations) {
      try {
        let result;

        switch (operation.type) {
          case "file_read":
            result = await context.tools.file_read(
              operation.target_file,
              operation.offset,
              operation.limit
            );
            break;

          case "file_search_replace":
            result = await context.tools.file_search_replace(
              operation.file_path,
              operation.old_string,
              operation.new_string,
              operation.replace_all
            );
            break;

          case "file_write":
            result = await context.tools.file_write(
              operation.file_path,
              operation.contents
            );
            break;

          case "run_terminal_cmd":
            result = await context.tools.run_terminal_cmd(
              operation.command,
              operation.is_background
            );
            break;

          default:
            throw new Error(`Unknown operation type: ${operation.type}`);
        }

        context.results.push({
          operation: operation.type,
          success: true,
          result: result,
        });

        context.console.log(
          `Operation ${operation.type} completed successfully`
        );
      } catch (error) {
        context.results.push({
          operation: operation.type,
          success: false,
          error: error.message,
        });

        context.console.error(
          `Operation ${operation.type} failed: ${error.message}`
        );
      }
    }

    const executionTime = performance.now() - startTime;

    return {
      success: true,
      executionTime,
      logs,
      results: context.results,
      summary: {
        totalOperations: operations.length,
        successfulOperations: context.results.filter((r) => r.success).length,
        failedOperations: context.results.filter((r) => !r.success).length,
      },
    };
  } catch (error) {
    const executionTime = performance.now() - startTime;

    return {
      success: false,
      executionTime,
      logs,
      error: error.message,
      results: [],
    };
  }
}

// Message handling
parentPort.on("message", async (message) => {
  try {
    switch (message.type) {
      case "execute_task":
        if (currentTask) {
          parentPort.postMessage({
            type: "task_failed",
            taskId: message.task.id,
            error: "Worker already executing a task",
          });
          return;
        }

        currentTask = message.task;

        const executor = taskExecutors[message.task.type];
        if (!executor) {
          parentPort.postMessage({
            type: "task_failed",
            taskId: message.task.id,
            error: `Unsupported task type: ${message.task.type}`,
          });
          currentTask = null;
          currentSandbox = null; // Reset sandbox on error
          return;
        }

        const result = await executor(message.task);

        parentPort.postMessage({
          type: "task_completed",
          taskId: message.task.id,
          result,
        });

        currentTask = null;
        currentSandbox = null; // Reset sandbox for next task
        break;

      case "shutdown":
        isRunning = false;
        parentPort.postMessage({ type: "worker_shutdown" });
        process.exit(0);
        break;

      default:
        parentPort.postMessage({
          type: "error",
          error: `Unknown message type: ${message.type}`,
        });
    }
  } catch (error) {
    parentPort.postMessage({
      type: "task_failed",
      taskId: currentTask?.id,
      error: error.message,
    });
    currentTask = null;
    currentSandbox = null; // Reset sandbox on error
  }
});

// Send ready signal
parentPort.postMessage({ type: "worker_ready" });

// Periodic metrics reporting
const metricsInterval = setInterval(() => {
  if (!isRunning) {
    clearInterval(metricsInterval);
    return;
  }

  const memUsage = process.memoryUsage();
  const cpuUsage = process.cpuUsage();

  parentPort.postMessage({
    type: "worker_metrics",
    metrics: {
      memoryUsage: memUsage.heapUsed,
      cpuUsage: cpuUsage.user / 1000, // microseconds to milliseconds
      uptime: process.uptime(),
    },
  });
}, 5000); // Every 5 seconds

// Graceful shutdown
process.on("SIGTERM", () => {
  isRunning = false;
  clearInterval(metricsInterval);
  process.exit(0);
});

process.on("SIGINT", () => {
  isRunning = false;
  clearInterval(metricsInterval);
  process.exit(0);
});
