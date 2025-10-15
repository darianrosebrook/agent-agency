/**
 * Task Worker - ARBITER-014
 *
 * Worker thread implementation for isolated task execution.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { performance } from "perf_hooks";
import { parentPort, workerData } from "worker_threads";

const { workerId, capabilities } = workerData;

let isRunning = true;
let currentTask = null;

// Task execution functions
const taskExecutors = {
  script: executeScriptTask,
  api_call: executeApiCallTask,
  data_processing: executeDataProcessingTask,
  ai_inference: executeAIInferenceTask,
};

async function executeScriptTask(task) {
  const { code, args = [], timeout = 30000 } = task.payload;

  // Create isolated context
  const context = {
    console: {
      log: (...args) => logs.push(`[LOG] ${args.join(" ")}`),
      error: (...args) => logs.push(`[ERROR] ${args.join(" ")}`),
      warn: (...args) => logs.push(`[WARN] ${args.join(" ")}`),
    },
    args,
    result: null,
  };

  const logs = [];

  try {
    // Execute code with timeout
    const startTime = performance.now();

    const script = `
      (async () => {
        try {
          ${code}
        } catch (error) {
          throw new Error(\`Script execution failed: \${error.message}\`);
        }
      })()
    `;

    // Use Function constructor for isolated execution
    const executeFn = new Function(
      "context",
      "require",
      `
      const { console, args, result } = context;
      const module = { exports: {} };

      // Limited require function
      const safeRequire = (moduleName) => {
        const allowedModules = ['crypto', 'util', 'path', 'url'];
        if (!allowedModules.includes(moduleName)) {
          throw new Error(\`Module \${moduleName} not allowed in task execution\`);
        }
        return require(moduleName);
      };

      ${script}
    `
    );

    const result = await Promise.race([
      executeFn(context, safeRequire),
      new Promise((_, reject) =>
        setTimeout(() => reject(new Error("Script execution timeout")), timeout)
      ),
    ]);

    const executionTime = performance.now() - startTime;

    return {
      success: true,
      result: context.result || result,
      logs,
      metrics: {
        executionTime,
        cpuUsage: process.cpuUsage().user / 1000, // microseconds to milliseconds
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

async function executeApiCallTask(task) {
  const { method, url, headers = {}, body, timeout = 30000 } = task.payload;
  const logs = [];

  try {
    const startTime = performance.now();

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

  try {
    const startTime = performance.now();

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

  try {
    const startTime = performance.now();

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
          return;
        }

        const result = await executor(message.task);

        parentPort.postMessage({
          type: "task_completed",
          taskId: message.task.id,
          result,
        });

        currentTask = null;
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
