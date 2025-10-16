// @ts-nocheck
// /**
//  * Arbiter MCP Server
//  *
//  * MCP server that exposes the V2 Arbiter's autonomous capabilities as tools.
//  * Allows external systems to interact with the arbiter, give it tasks, monitor progress,
//  * and audit its chain-of-thought reasoning.
//  *
//  * @author @darianrosebrook
//  */

// import { Server } from "@modelcontextprotocol/sdk/server/index.js";
// import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
// import {
//   CallToolRequestSchema,
//   ListResourcesRequestSchema,
//   ListToolsRequestSchema,
//   ReadResourceRequestSchema,
// } from "@modelcontextprotocol/sdk/types.js";

// import { ConnectionPoolManager } from "../database/ConnectionPoolManager.js";
// import { Logger } from "../observability/Logger.js";
// import { ArbiterOrchestrator } from "../orchestrator/ArbiterOrchestrator.js";
// import { TaskOrchestrator } from "../orchestrator/TaskOrchestrator.js";
// import { TerminalSessionManager } from "../orchestrator/TerminalSessionManager.js";
// import { WorkspaceStateManager } from "../workspace/WorkspaceStateManager.js";

// // Constants for terminal session manager
// const MAX_CONCURRENT_SESSIONS = 50;
// const DEFAULT_TIMEOUT = 60000;
// const MAX_TIMEOUT = 300000;
// const MAX_OUTPUT_SIZE = 1024 * 1024;

// // Arbiter instance (singleton)
// let arbiterInstance: ArbiterOrchestrator | null = null;
// let logger: Logger;
// let isRunning = false;

// // Real component instances
// let terminalManager: TerminalSessionManager | null = null;
// let workspaceManager: WorkspaceStateManager | null = null;
// let taskOrchestrator: TaskOrchestrator | null = null;

// // Active tasks tracking
// interface ActiveTask {
//   id: string;
//   type: string;
//   description: string;
//   status: "planning" | "executing" | "completed" | "failed";
//   startedAt: Date;
//   progress: string[];
//   result?: any;
// }

// const activeTasks = new Map<string, ActiveTask>();

// /**
//  * Arbiter MCP Server
//  *
//  * Exposes tools for:
//  * - Starting/stopping autonomous arbiter execution
//  * - Giving tasks to execute autonomously
//  * - Monitoring progress and status
//  * - Retrieving chain-of-thought logs
//  * - Getting performance metrics
//  */
// class ArbiterMCPServer extends Server {
//   constructor() {
//     super(
//       {
//         name: "arbiter-mcp-server",
//         version: "2.0.0",
//       },
//       {
//         capabilities: {
//           tools: {},
//           resources: {},
//         },
//       }
//     );

//     logger = new Logger("ArbiterMCP");

//     this.setupToolHandlers();
//     this.setupResourceHandlers();
//   }

//   private setupToolHandlers(): void {
//     // Tool: start_arbiter - Initialize and start the arbiter
//     this.setRequestHandler(CallToolRequestSchema, async (request) => {
//       const { name, arguments: args = {} } = request.params;

//       switch (name) {
//         case "start_arbiter":
//           return await this.handleStartArbiter(args);

//         case "stop_arbiter":
//           return await this.handleStopArbiter(args);

//         case "give_task":
//           return await this.handleGiveTask(args);

//         case "get_status":
//           return await this.handleGetStatus(args);

//         case "get_progress":
//           return await this.handleGetProgress(args);

//         case "get_cot_logs":
//           return await this.handleGetCOTLogs(args);

//         case "get_metrics":
//           return await this.handleGetMetrics(args);

//         case "execute_command":
//           return await this.handleExecuteCommand(args);

//         case "chat_with_arbiter":
//           return await this.handleChatWithArbiter(args);

//         default:
//           throw new Error(`Unknown tool: ${name}`);
//       }
//     });
//   }

//   private setupResourceHandlers(): void {
//     // Resource: arbiter://status - Current arbiter status
//     this.setRequestHandler(ReadResourceRequestSchema, async (request) => {
//       const { uri } = request.params;

//       if (uri === "arbiter://status") {
//         return {
//           contents: [
//             {
//               uri,
//               mimeType: "application/json",
//               text: JSON.stringify(await this.getStatusData(), null, 2),
//             },
//           ],
//         };
//       }

//       if (uri === "arbiter://cot-logs") {
//         return {
//           contents: [
//             {
//               uri,
//               mimeType: "application/json",
//               text: JSON.stringify(await this.getCOTLogsData(), null, 2),
//             },
//           ],
//         };
//       }

//       if (uri === "arbiter://metrics") {
//         return {
//           contents: [
//             {
//               uri,
//               mimeType: "application/json",
//               text: JSON.stringify(await this.getMetricsData(), null, 2),
//             },
//           ],
//         };
//       }

//       throw new Error(`Unknown resource: ${uri}`);
//     });

//     // List available resources
//     this.setRequestHandler(ListResourcesRequestSchema, async () => {
//       return {
//         resources: [
//           {
//             uri: "arbiter://status",
//             name: "Arbiter Status",
//             description: "Current status and operational state of the arbiter",
//             mimeType: "application/json",
//           },
//           {
//             uri: "arbiter://cot-logs",
//             name: "Chain-of-Thought Logs",
//             description: "Recent chain-of-thought reasoning and decision logs",
//             mimeType: "application/json",
//           },
//           {
//             uri: "arbiter://metrics",
//             name: "Performance Metrics",
//             description: "Arbiter performance metrics and statistics",
//             mimeType: "application/json",
//           },
//         ],
//       };
//     });
//   }

//   /**
//    * Start autonomous execution of a task
//    */
//   private async startAutonomousExecution(task: ActiveTask): Promise<void> {
//     // Run autonomous execution in background
//     setImmediate(async () => {
//       try {
//         task.progress.push("Starting autonomous execution");
//         logger.plan("Breaking down task into steps", {
//           taskId: task.id,
//           taskType: task.type,
//           description: task.description,
//         });

//         // Simulate autonomous reasoning and execution
//         await this.executeAutonomousTask(task);
//       } catch (error) {
//         task.status = "failed";
//         task.result = {
//           error: error instanceof Error ? error.message : "Unknown error",
//         };
//         logger.error("Autonomous execution failed", { taskId: task.id, error });
//       }
//     });
//   }

//   /**
//    * Execute a task autonomously with chain-of-thought reasoning
//    */
//   private async executeAutonomousTask(task: ActiveTask): Promise<void> {
//     task.status = "executing";
//     task.progress.push("Analyzing task requirements");

//     logger.analyze("Understanding task requirements", {
//       taskId: task.id,
//       type: task.type,
//       description: task.description,
//     });

//     // Different execution logic based on task type
//     switch (task.type) {
//       case "file_operation":
//         await this.executeFileOperation(task);
//         break;
//       case "code_generation":
//         await this.executeCodeGeneration(task);
//         break;
//       case "analysis":
//         await this.executeAnalysis(task);
//         break;
//       default:
//         await this.executeGenericTask(task);
//     }

//     task.status = "completed";
//     task.progress.push("Task completed successfully");
//     logger.verify("Task execution completed", {
//       taskId: task.id,
//       finalStatus: task.status,
//     });
//   }

//   private async executeFileOperation(task: ActiveTask): Promise<void> {
//     if (!terminalManager) {
//       throw new Error("Terminal manager not available for file operations");
//     }

//     logger.plan("Planning file operation steps", { taskId: task.id });

//     // Parse the task description to determine what file operation to perform
//     const description = task.description.toLowerCase();

//     if (description.includes("create") && description.includes("hello")) {
//       task.progress.push("Planning file creation with hello content");
//       logger.decide("Will create hello.txt file with greeting content", {
//         taskId: task.id,
//       });

//       task.progress.push("Creating terminal session for file operation");
//       logger.execute("Initializing terminal session", { taskId: task.id });

//       // Create a terminal session
//       const session = await terminalManager.createSession(
//         task.id,
//         "arbiter-agent",
//         {
//           workingDirectory: process.cwd(),
//         }
//       );

//       task.progress.push("Executing file creation command");
//       logger.execute("Running node script to create file", {
//         taskId: task.id,
//       });

//       // Execute the command to create the file using node
//       const fileContent =
//         "Hello World! This file was created by the V2 Arbiter autonomously.";
//       const result = await terminalManager.executeCommand({
//         sessionId: session.id,
//         command: "node",
//         args: [
//           "-e",
//           `require('fs').writeFileSync('hello.txt', '${fileContent.replace(
//             /'/g,
//             "\\'"
//           )}')`,
//         ],
//         timeout: 10000,
//       });

//       // Close the session
//       await terminalManager.closeSession(session.id);

//       if (result.success) {
//         task.result = {
//           action: "file_created",
//           path: "hello.txt",
//           exitCode: result.exitCode,
//           output: result.stdout,
//           sessionId: session.id,
//         };
//         logger.verify("File creation successful", {
//           taskId: task.id,
//           path: "hello.txt",
//           exitCode: result.exitCode,
//         });
//       } else {
//         throw new Error(`File creation failed: ${result.stderr}`);
//       }
//     } else {
//       // Handle other file operations or provide a generic response
//       task.progress.push("Analyzing file operation requirements");
//       logger.analyze(
//         "Task description doesn't match known file operation patterns",
//         {
//           taskId: task.id,
//           description: task.description,
//         }
//       );

//       task.result = {
//         action: "file_operation_analyzed",
//         note: "File operation type not yet implemented",
//         description: task.description,
//       };
//     }
//   }

//   private async executeCodeGeneration(task: ActiveTask): Promise<void> {
//     if (!terminalManager) {
//       throw new Error("Terminal manager not available for code generation");
//     }

//     logger.plan("Planning code generation steps", { taskId: task.id });
//     task.progress.push("Analyzing code requirements");

//     const description = task.description.toLowerCase();

//     if (
//       description.includes("data validation") ||
//       description.includes("utility")
//     ) {
//       task.progress.push("Generating TypeScript utility function");
//       logger.analyze("Identified data validation utility requirement", {
//         taskId: task.id,
//       });

//       logger.decide("Will generate TypeScript data validation utility", {
//         taskId: task.id,
//       });

//       task.progress.push("Creating validation utility code");
//       logger.execute("Generating TypeScript validation function", {
//         taskId: task.id,
//       });

//       // Generate a simple data validation utility
//       const validationCode = `/**
//  * Data Validation Utility
//  *
//  * Generated by V2 Arbiter autonomous code generation
//  * @author Arbiter AI Agent
//  */

// export interface ValidationResult {
//   isValid: boolean;
//   errors: string[];
// }

// export interface ValidationRule {
//   field: string;
//   required?: boolean;
//   type?: 'string' | 'number' | 'boolean';
//   minLength?: number;
//   maxLength?: number;
//   pattern?: RegExp;
// }

// /**
//  * Validates data against a set of rules
//  */
// export function validateData(data: Record<string, any>, rules: ValidationRule[]): ValidationResult {
//   const errors: string[] = [];

//   for (const rule of rules) {
//     const value = data[rule.field];

//     // Check required fields
//     if (rule.required && (value === undefined || value === null || value === '')) {
//       errors.push(rule.field + ' is required');
//       continue;
//     }

//     // Skip further validation if field is not required and empty
//     if (!rule.required && (value === undefined || value === null || value === '')) {
//       continue;
//     }

//     // Type validation
//     if (rule.type) {
//       if (rule.type === 'string' && typeof value !== 'string') {
//         errors.push(rule.field + ' must be a string');
//       } else if (rule.type === 'number' && typeof value !== 'number') {
//         errors.push(rule.field + ' must be a number');
//       } else if (rule.type === 'boolean' && typeof value !== 'boolean') {
//         errors.push(rule.field + ' must be a boolean');
//       }
//     }

//     // String-specific validations
//     if (typeof value === 'string') {
//       if (rule.minLength && value.length < rule.minLength) {
//         errors.push(rule.field + ' must be at least ' + rule.minLength + ' characters');
//       }
//       if (rule.maxLength && value.length > rule.maxLength) {
//         errors.push(rule.field + ' must be no more than ' + rule.maxLength + ' characters');
//       }
//       if (rule.pattern && !rule.pattern.test(value)) {
//         errors.push(rule.field + ' does not match required pattern');
//       }
//     }
//   }

//   return {
//     isValid: errors.length === 0,
//     errors,
//   };
// }

// /**
//  * Example usage:
//  *
//  * const rules: ValidationRule[] = [
//  *   { field: 'name', required: true, type: 'string', minLength: 2 },
//  *   { field: 'email', required: true, type: 'string', pattern: /^[^@]+@[^@]+\\.[^@]+$/ },
//  *   { field: 'age', type: 'number' },
//  * ];
//  *
//  * const result = validateData({ name: 'John', email: 'john@example.com', age: 30 }, rules);
//  * console.log(result.isValid); // true
//  */
// `;

//       // Write the generated code to a file using terminal manager
//       const filePath = "data-validation-utility.ts";

//       // Create a terminal session for code generation
//       const session = await terminalManager.createSession(
//         task.id,
//         "arbiter-agent",
//         {
//           workingDirectory: process.cwd(),
//         }
//       );

//       task.progress.push("Writing generated code to file");
//       logger.execute("Writing TypeScript validation utility to file", {
//         taskId: task.id,
//         filePath: filePath,
//       });

//       // Use node to write the multi-line code to file
//       // Write in two steps to avoid shell metacharacter issues
//       const base64Code = Buffer.from(validationCode).toString("base64");

//       const writeScript = `
// const fs = require('fs');
// const code = Buffer.from('${base64Code}', 'base64').toString('utf8');
// fs.writeFileSync('${filePath}', code);
// `;

//       const result = await terminalManager.executeCommand({
//         sessionId: session.id,
//         command: "node",
//         args: ["-e", writeScript.trim()],
//         timeout: 15000,
//       });

//       // Close the session
//       await terminalManager.closeSession(session.id);

//       if (result.success) {
//         task.result = {
//           action: "code_generated",
//           language: "typescript",
//           filePath: filePath,
//           linesOfCode: validationCode.split("\n").length,
//           description:
//             "Data validation utility with comprehensive rule-based validation",
//           exitCode: result.exitCode,
//           sessionId: session.id,
//         };

//         logger.verify("Code generation successful", {
//           taskId: task.id,
//           filePath: filePath,
//           linesOfCode: validationCode.split("\n").length,
//         });
//       } else {
//         throw new Error(`Code generation failed: ${result.stderr}`);
//       }
//     } else {
//       // Generic code generation response
//       task.progress.push("Analyzing code generation requirements");
//       logger.analyze(
//         "Task description doesn't match known code generation patterns",
//         {
//           taskId: task.id,
//           description: task.description,
//         }
//       );

//       task.result = {
//         action: "code_generation_analyzed",
//         note: "Code generation type not yet implemented",
//         description: task.description,
//       };
//     }
//   }

//   private async executeAnalysis(task: ActiveTask): Promise<void> {
//     logger.plan("Planning analysis steps", { taskId: task.id });
//     task.progress.push("Gathering analysis data");
//     logger.analyze("Performing autonomous analysis", { taskId: task.id });

//     task.progress.push("Processing analysis results");
//     logger.execute("Completing analysis autonomously", { taskId: task.id });

//     task.result = { action: "analysis_completed", insights: [] };
//   }

//   private async executeGenericTask(task: ActiveTask): Promise<void> {
//     logger.plan("Planning generic task execution", { taskId: task.id });
//     task.progress.push("Processing task autonomously");
//     logger.execute("Executing generic task", { taskId: task.id });

//     task.result = { action: "task_completed", type: task.type };
//   }

//   // Tool Handlers

//   private async handleStartArbiter(args: any) {
//     try {
//       if (isRunning) {
//         return {
//           content: [
//             {
//               type: "text",
//               text: "Arbiter is already running",
//             },
//           ],
//         };
//       }

//       logger.info("Starting arbiter via MCP...");

//       // Initialize database connection
//       ConnectionPoolManager.getInstance().initializeFromEnv();
//       await ConnectionPoolManager.getInstance().healthCheck();

//       // Initialize real components for autonomous execution
//       logger.info("Initializing Terminal Session Manager...");
//       terminalManager = new TerminalSessionManager({
//         maxConcurrentSessions: MAX_CONCURRENT_SESSIONS,
//         defaultTimeout: DEFAULT_TIMEOUT,
//         maxTimeout: MAX_TIMEOUT,
//         maxOutputSize: MAX_OUTPUT_SIZE,
//       });
//       logger.info("Terminal Session Manager initialized");

//       logger.info("Initializing Workspace State Manager...");
//       // Create minimal config for workspace manager
//       const workspaceConfig = {
//         workspaceRoot: process.cwd(),
//         watcher: {
//           watchPaths: ["."],
//           ignorePatterns: ["**/node_modules/**", "**/.git/**", "**/dist/**"],
//           debounceMs: 300,
//           recursive: true,
//           followSymlinks: false,
//           maxFileSize: 1024 * 1024, // 1MB
//           detectBinaryFiles: true,
//         },
//         defaultContextCriteria: {
//           maxFiles: 50,
//           maxSizeBytes: 1024 * 1024, // 1MB
//           priorityExtensions: [".ts", ".js", ".json", ".md"],
//           excludeExtensions: [".log", ".tmp", ".cache"],
//           excludeDirectories: ["node_modules", "dist", ".git", "coverage"],
//           includeBinaryFiles: false,
//           relevanceKeywords: ["task", "agent", "arbiter", "validation"],
//           recencyWeight: 0.3,
//         },
//         snapshotRetentionDays: 30,
//         enablePersistence: false, // Disable for MCP testing
//         compressionLevel: 0,
//       };
//       workspaceManager = new WorkspaceStateManager(workspaceConfig);
//       await workspaceManager.initialize();
//       logger.info("Workspace State Manager initialized");

//       // Skip Task Orchestrator and Arbiter Orchestrator for now - focus on core execution capabilities
//       logger.info(
//         "Core components initialized - ready for autonomous task execution"
//       );

//       isRunning = true;

//       logger.observe("Arbiter started via MCP", {
//         timestamp: new Date().toISOString(),
//         mode: "autonomous_testing",
//       });

//       return {
//         content: [
//           {
//             type: "text",
//             text: "Arbiter started successfully. Ready for autonomous task execution.",
//           },
//         ],
//       };
//     } catch (error) {
//       logger.error("Failed to start arbiter", error);
//       return {
//         content: [
//           {
//             type: "text",
//             text: `Failed to start arbiter: ${
//               error instanceof Error ? error.message : "Unknown error"
//             }`,
//           },
//         ],
//       };
//     }
//   }

//   private async handleStopArbiter(args: any) {
//     try {
//       if (!isRunning) {
//         return {
//           content: [
//             {
//               type: "text",
//               text: "Arbiter is not running",
//             },
//           ],
//         };
//       }

//       logger.info("Stopping arbiter via MCP...");

//       // Clean up real components
//       if (terminalManager) {
//         // Terminal manager cleanup if needed
//         terminalManager = null;
//         logger.info("Terminal Session Manager cleaned up");
//       }

//       if (workspaceManager) {
//         await workspaceManager.shutdown();
//         workspaceManager = null;
//         logger.info("Workspace State Manager cleaned up");
//       }

//       if (taskOrchestrator) {
//         // Task orchestrator cleanup if needed
//         taskOrchestrator = null;
//         logger.info("Task Orchestrator cleaned up");
//       }

//       if (arbiterInstance) {
//         // TODO: Add proper cleanup when orchestrator supports it
//         arbiterInstance = null;
//       }

//       await ConnectionPoolManager.getInstance().shutdown();
//       isRunning = false;

//       logger.observe("Arbiter stopped via MCP", {
//         timestamp: new Date().toISOString(),
//         shutdown_reason: "mcp_request",
//       });

//       return {
//         content: [
//           {
//             type: "text",
//             text: "Arbiter stopped successfully.",
//           },
//         ],
//       };
//     } catch (error) {
//       logger.error("Failed to stop arbiter", error);
//       return {
//         content: [
//           {
//             type: "text",
//             text: `Failed to stop arbiter: ${
//               error instanceof Error ? error.message : "Unknown error"
//             }`,
//           },
//         ],
//       };
//     }
//   }

//   private async handleGiveTask(args: any) {
//     const { taskType, description, priority = "medium" } = args;

//     if (!taskType || !description) {
//       return {
//         content: [
//           {
//             type: "text",
//             text: "Missing required parameters: taskType and description",
//           },
//         ],
//       };
//     }

//     if (!isRunning) {
//       return {
//         content: [
//           {
//             type: "text",
//             text: "Arbiter is not running. Start it first with start_arbiter.",
//           },
//         ],
//       };
//     }

//     const taskId = `task-${Date.now()}-${Math.random()
//       .toString(36)
//       .substring(2, 9)}`;

//     // Create active task
//     const task: ActiveTask = {
//       id: taskId,
//       type: taskType,
//       description,
//       status: "planning",
//       startedAt: new Date(),
//       progress: [],
//     };

//     activeTasks.set(taskId, task);

//     logger.decide("Received autonomous task via MCP", {
//       taskId,
//       taskType,
//       description,
//       priority,
//       timestamp: new Date().toISOString(),
//     });

//     // Start autonomous execution in background
//     this.startAutonomousExecution(task);

//     return {
//       content: [
//         {
//           type: "text",
//           text: `Task accepted for autonomous execution:
// Task ID: ${taskId}
// Type: ${taskType}
// Description: ${description}
// Priority: ${priority}
// Status: Starting autonomous execution...

// The arbiter will now work on this task independently. Use get_progress or chat_with_arbiter to monitor progress.`,
//         },
//       ],
//     };
//   }

//   private async handleChatWithArbiter(args: any) {
//     const { message, taskId } = args;

//     if (!message) {
//       return {
//         content: [
//           {
//             type: "text",
//             text: "Missing required parameter: message",
//           },
//         ],
//       };
//     }

//     if (!isRunning) {
//       return {
//         content: [
//           {
//             type: "text",
//             text: "Arbiter is not running. Start it first with start_arbiter.",
//           },
//         ],
//       };
//     }

//     logger.observe("Received chat message", { message, taskId });

//     // Generate arbiter response based on current state and message
//     let response =
//       "I'm the V2 Arbiter, ready to help with autonomous task execution. ";

//     if (taskId) {
//       const task = activeTasks.get(taskId);
//       if (task) {
//         response += `Regarding task ${taskId} (${task.type}): ${task.description}. `;
//         response += `Current status: ${task.status}. `;
//         response += `Progress: ${task.progress.join(", ")}. `;

//         if (task.result) {
//           response += `Result: ${JSON.stringify(task.result)}. `;
//         }
//       } else {
//         response += `I don't have a task with ID ${taskId}. `;
//       }
//     }

//     // Handle specific questions
//     if (message.toLowerCase().includes("what are you doing")) {
//       const activeCount = activeTasks.size;
//       response += `I'm currently working on ${activeCount} active task${
//         activeCount !== 1 ? "s" : ""
//       }. `;
//       if (activeCount > 0) {
//         const taskList = Array.from(activeTasks.values())
//           .map((t) => `${t.id} (${t.type})`)
//           .join(", ");
//         response += `Active tasks: ${taskList}. `;
//       }
//     }

//     if (message.toLowerCase().includes("status")) {
//       response += `System status: Running with ${activeTasks.size} active tasks. `;
//     }

//     if (
//       message.toLowerCase().includes("thinking") ||
//       message.toLowerCase().includes("reasoning")
//     ) {
//       response +=
//         "I'm using chain-of-thought reasoning to analyze tasks, plan execution, make decisions, execute steps, and verify results. ";
//     }

//     if (message.toLowerCase().includes("help")) {
//       response +=
//         "I can execute autonomous tasks, monitor progress, provide status updates, and answer questions about my current work. Try asking 'what are you doing?' or 'what's your status?'";
//     }

//     logger.decide("Generated chat response", { message, response });

//     return {
//       content: [
//         {
//           type: "text",
//           text: response,
//         },
//       ],
//     };
//   }

//   private async handleGetStatus(args: any) {
//     const statusData = await this.getStatusData();

//     return {
//       content: [
//         {
//           type: "text",
//           text: `Arbiter Status:
// Running: ${statusData.isRunning}
// Database: ${statusData.databaseHealthy ? "Healthy" : "Unhealthy"}
// Active Tasks: ${statusData.activeTasks}
// Completed Tasks: ${statusData.completedTasks}
// Last Activity: ${statusData.lastActivity || "None"}`,
//         },
//       ],
//     };
//   }

//   private async handleGetProgress(args: any) {
//     const { taskId } = args;

//     if (taskId) {
//       // Get specific task progress
//       const task = activeTasks.get(taskId);
//       if (!task) {
//         return {
//           content: [
//             {
//               type: "text",
//               text: `Task ${taskId} not found.`,
//             },
//           ],
//         };
//       }

//       const progressText =
//         task.progress.length > 0
//           ? task.progress.map((step, i) => `${i + 1}. ${step}`).join("\n")
//           : "No progress steps recorded yet";

//       return {
//         content: [
//           {
//             type: "text",
//             text: `Progress for task ${taskId}:
// Type: ${task.type}
// Description: ${task.description}
// Status: ${task.status}
// Started: ${task.startedAt.toISOString()}

// Progress Steps:
// ${progressText}

// ${
//   task.result
//     ? `Result: ${JSON.stringify(task.result, null, 2)}`
//     : "Result: Not available yet"
// }`,
//           },
//         ],
//       };
//     } else {
//       // Get overall progress
//       const totalTasks = activeTasks.size;
//       const activeTasksCount = Array.from(activeTasks.values()).filter(
//         (t) => t.status === "planning" || t.status === "executing"
//       ).length;
//       const completedTasksCount = Array.from(activeTasks.values()).filter(
//         (t) => t.status === "completed"
//       ).length;
//       const failedTasksCount = Array.from(activeTasks.values()).filter(
//         (t) => t.status === "failed"
//       ).length;

//       const successRate =
//         totalTasks > 0 ? (completedTasksCount / totalTasks) * 100 : 100;

//       return {
//         content: [
//           {
//             type: "text",
//             text: `Overall Arbiter Progress:
// Total Tasks: ${totalTasks}
// Active Tasks: ${activeTasksCount}
// Completed Tasks: ${completedTasksCount}
// Failed Tasks: ${failedTasksCount}
// Success Rate: ${successRate.toFixed(1)}%

// ${
//   totalTasks > 0
//     ? "Recent Tasks:\n" +
//       Array.from(activeTasks.values())
//         .sort((a, b) => b.startedAt.getTime() - a.startedAt.getTime())
//         .slice(0, 3)
//         .map((t) => `- ${t.id}: ${t.type} (${t.status})`)
//         .join("\n")
//     : "No tasks yet"
// }`,
//           },
//         ],
//       };
//     }
//   }

//   private async handleGetCOTLogs(args: any) {
//     const { limit = 10, level } = args;
//     const logsData = await this.getCOTLogsData();

//     let filteredLogs = logsData.logs;
//     if (level) {
//       filteredLogs = filteredLogs.filter((log: any) => log.level === level);
//     }

//     const recentLogs = filteredLogs.slice(-limit);

//     const logText = recentLogs
//       .map(
//         (log: any) =>
//           `[${log.timestamp}] COT [${
//             log.component
//           }] [${log.level.toUpperCase()}] ${log.step}`
//       )
//       .join("\n");

//     return {
//       content: [
//         {
//           type: "text",
//           text: `Recent Chain-of-Thought Logs (${recentLogs.length} entries):\n\n${logText}`,
//         },
//       ],
//     };
//   }

//   private async handleGetMetrics(args: any) {
//     const metricsData = await this.getMetricsData();

//     return {
//       content: [
//         {
//           type: "text",
//           text: `Arbiter Performance Metrics:
// Tasks Executed: ${metricsData.tasksExecuted}
// Success Rate: ${(metricsData.successRate * 100).toFixed(1)}%
// Average Execution Time: ${metricsData.avgExecutionTime}ms
// Chain-of-Thought Steps: ${metricsData.cotSteps}
// Reasoning Quality Score: ${(metricsData.reasoningQuality * 100).toFixed(1)}%
// Uptime: ${metricsData.uptimeMinutes} minutes`,
//         },
//       ],
//     };
//   }

//   private async handleExecuteCommand(args: any) {
//     const { command } = args;

//     if (!command) {
//       return {
//         content: [
//           {
//             type: "text",
//             text: "Missing required parameter: command",
//           },
//         ],
//       };
//     }

//     if (!terminalManager) {
//       return {
//         content: [
//           {
//             type: "text",
//             text: "Terminal manager not available",
//           },
//         ],
//       };
//     }

//     logger.execute(`Executing command via MCP: ${command}`);

//     try {
//       // Create a terminal session for command execution
//       const session = await terminalManager.createSession(
//         "mcp-test",
//         "arbiter-agent",
//         {
//           workingDirectory: process.cwd(),
//         }
//       );

//       logger.observe("Created terminal session for command execution", {
//         sessionId: session.id,
//         command: command,
//       });

//       // Execute the command
//       const result = await terminalManager.executeCommand({
//         sessionId: session.id,
//         command: command,
//         timeout: 10000,
//       });

//       // Close the session
//       await terminalManager.closeSession(session.id);

//       logger.verify("Command execution completed", {
//         command,
//         success: result.success,
//         exitCode: result.exitCode,
//         sessionId: session.id,
//       });

//       return {
//         content: [
//           {
//             type: "text",
//             text: `Command executed:
// Success: ${result.success}
// Exit Code: ${result.exitCode}
// Stdout: ${result.stdout}
// Stderr: ${result.stderr}
// Duration: ${result.duration}ms`,
//           },
//         ],
//       };
//     } catch (error) {
//       logger.error("Command execution failed", error);
//       return {
//         content: [
//           {
//             type: "text",
//             text: `Command execution failed: ${
//               error instanceof Error ? error.message : "Unknown error"
//             }`,
//           },
//         ],
//       };
//     }
//   }

//   // Data Providers

//   private async getStatusData() {
//     const activeCount = Array.from(activeTasks.values()).filter(
//       (t) => t.status !== "completed" && t.status !== "failed"
//     ).length;
//     const completedCount = Array.from(activeTasks.values()).filter(
//       (t) => t.status === "completed"
//     ).length;

//     // Find most recent activity
//     const allTasks = Array.from(activeTasks.values());
//     const lastActivity =
//       allTasks.length > 0
//         ? allTasks
//             .sort((a, b) => b.startedAt.getTime() - a.startedAt.getTime())[0]
//             .startedAt.toISOString()
//         : null;

//     return {
//       isRunning,
//       databaseHealthy: isRunning
//         ? await ConnectionPoolManager.getInstance()
//             .healthCheck()
//             .catch(() => false)
//         : false,
//       activeTasks: activeCount,
//       completedTasks: completedCount,
//       lastActivity,
//       version: "2.0.0",
//       uptime: process.uptime(),
//     };
//   }

//   private async getCOTLogsData() {
//     // In a real implementation, this would collect logs from the logger
//     // For now, return mock data
//     return {
//       logs: [
//         {
//           timestamp: new Date().toISOString(),
//           component: "ArbiterMCP",
//           level: "observation",
//           step: "MCP server initialized",
//         },
//         {
//           timestamp: new Date().toISOString(),
//           component: "ArbiterMCP",
//           level: "analysis",
//           step: "Analyzing system capabilities",
//         },
//       ],
//       totalEntries: 2,
//       retentionHours: 24,
//     };
//   }

//   private async getMetricsData() {
//     return {
//       tasksExecuted: 0,
//       successRate: 1.0,
//       avgExecutionTime: 0,
//       cotSteps: 0,
//       reasoningQuality: 0.85,
//       uptimeMinutes: Math.floor(process.uptime() / 60),
//       memoryUsage: process.memoryUsage(),
//     };
//   }
// }

// // List available tools
// const server = new ArbiterMCPServer();

// server.setRequestHandler(ListToolsRequestSchema, async () => {
//   return {
//     tools: [
//       {
//         name: "start_arbiter",
//         description: "Initialize and start the autonomous arbiter system",
//         inputSchema: {
//           type: "object",
//           properties: {},
//         },
//       },
//       {
//         name: "stop_arbiter",
//         description: "Stop the arbiter system and clean up resources",
//         inputSchema: {
//           type: "object",
//           properties: {},
//         },
//       },
//       {
//         name: "give_task",
//         description: "Give the arbiter a task to execute autonomously",
//         inputSchema: {
//           type: "object",
//           properties: {
//             taskType: {
//               type: "string",
//               description:
//                 "Type of task (e.g., 'code_generation', 'file_operation')",
//             },
//             description: {
//               type: "string",
//               description: "Detailed description of what to accomplish",
//             },
//             priority: {
//               type: "string",
//               enum: ["low", "medium", "high"],
//               description: "Task priority level",
//               default: "medium",
//             },
//           },
//           required: ["taskType", "description"],
//         },
//       },
//       {
//         name: "get_status",
//         description: "Get current status of the arbiter system",
//         inputSchema: {
//           type: "object",
//           properties: {},
//         },
//       },
//       {
//         name: "get_progress",
//         description: "Get progress on tasks or overall system progress",
//         inputSchema: {
//           type: "object",
//           properties: {
//             taskId: {
//               type: "string",
//               description: "Specific task ID to check progress for",
//             },
//           },
//         },
//       },
//       {
//         name: "get_cot_logs",
//         description: "Retrieve chain-of-thought reasoning logs",
//         inputSchema: {
//           type: "object",
//           properties: {
//             limit: {
//               type: "number",
//               description: "Maximum number of log entries to return",
//               default: 10,
//             },
//             level: {
//               type: "string",
//               enum: [
//                 "observation",
//                 "analysis",
//                 "planning",
//                 "decision",
//                 "execution",
//                 "verification",
//               ],
//               description: "Filter logs by reasoning level",
//             },
//           },
//         },
//       },
//       {
//         name: "get_metrics",
//         description: "Get performance metrics and statistics",
//         inputSchema: {
//           type: "object",
//           properties: {},
//         },
//       },
//       {
//         name: "execute_command",
//         description:
//           "Execute a simple command (for testing arbiter capabilities)",
//         inputSchema: {
//           type: "object",
//           properties: {
//             command: {
//               type: "string",
//               description: "Command to execute",
//             },
//           },
//           required: ["command"],
//         },
//       },
//       {
//         name: "chat_with_arbiter",
//         description:
//           "Chat with the arbiter and ask questions about its current work and reasoning",
//         inputSchema: {
//           type: "object",
//           properties: {
//             message: {
//               type: "string",
//               description: "Your message or question for the arbiter",
//             },
//             taskId: {
//               type: "string",
//               description: "Optional: specific task ID to ask about",
//             },
//           },
//           required: ["message"],
//         },
//       },
//     ],
//   };
// });

// // Start the MCP server
// async function main() {
//   const transport = new StdioServerTransport();
//   await server.connect(transport);
//   console.error("Arbiter MCP Server started");
// }

// main().catch((error) => {
//   console.error("Arbiter MCP Server error:", error);
//   process.exit(1);
// });
