/**
 * Task Management Tools for MCP
 *
 * @author @darianrosebrook
 * @description Tools for managing task lifecycle and operations
 */

import { Tool } from "@modelcontextprotocol/sdk/types.js";
import { TaskType } from "../../../types/index.js";
import { MCPToolContext } from "../ToolManager.js";

export class TaskManagementTools {
  constructor(private context: MCPToolContext) {}

  async getTools(): Promise<Tool[]> {
    return [
      {
        name: "submit_task",
        description: "Submit a new task for execution by an agent",
        inputSchema: {
          type: "object",
          properties: {
            agentId: {
              type: "string",
              description: "ID of the agent to assign the task to",
            },
            type: {
              type: "string",
              enum: ["process", "analyze", "coordinate", "monitor"],
              description: "Type of task to execute",
            },
            payload: {
              type: "object",
              additionalProperties: true,
              description: "Task input data and parameters",
            },
            priority: {
              type: "string",
              enum: ["low", "normal", "high", "critical"],
              description: "Task priority level",
              default: "normal",
            },
          },
          required: ["agentId", "type", "payload"],
        },
      },
      {
        name: "get_task",
        description: "Retrieve detailed information about a specific task",
        inputSchema: {
          type: "object",
          properties: {
            taskId: {
              type: "string",
              description: "ID of the task to retrieve",
            },
          },
          required: ["taskId"],
        },
      },
      {
        name: "cancel_task",
        description: "Cancel a pending or running task",
        inputSchema: {
          type: "object",
          properties: {
            taskId: {
              type: "string",
              description: "ID of the task to cancel",
            },
            reason: {
              type: "string",
              description: "Reason for cancellation",
            },
          },
          required: ["taskId"],
        },
      },
      {
        name: "list_tasks",
        description: "List tasks with optional filtering",
        inputSchema: {
          type: "object",
          properties: {
            agentId: {
              type: "string",
              description: "Filter by agent ID",
            },
            status: {
              type: "string",
              enum: ["pending", "running", "completed", "failed", "cancelled"],
              description: "Filter by task status",
            },
            type: {
              type: "string",
              enum: ["process", "analyze", "coordinate", "monitor"],
              description: "Filter by task type",
            },
            limit: {
              type: "number",
              description: "Maximum number of tasks to return",
              default: 50,
              minimum: 1,
              maximum: 1000,
            },
          },
        },
      },
      {
        name: "retry_task",
        description: "Retry a failed task with optional modifications",
        inputSchema: {
          type: "object",
          properties: {
            taskId: {
              type: "string",
              description: "ID of the failed task to retry",
            },
            payload: {
              type: "object",
              additionalProperties: true,
              description: "Modified task payload (optional)",
            },
            priority: {
              type: "string",
              enum: ["low", "normal", "high", "critical"],
              description: "New priority for retried task",
            },
          },
          required: ["taskId"],
        },
      },
    ];
  }

  async hasTool(name: string): Promise<boolean> {
    const tools = await this.getTools();
    return tools.some((tool) => tool.name === name);
  }

  async executeTool(name: string, args: any): Promise<any> {
    switch (name) {
      case "submit_task":
        return await this.submitTask(args);
      case "get_task":
        return await this.getTask(args);
      case "cancel_task":
        return await this.cancelTask(args);
      case "list_tasks":
        return await this.listTasks(args);
      case "retry_task":
        return await this.retryTask(args);
      default:
        throw new Error(`Unknown task management tool: ${name}`);
    }
  }

  private async submitTask(args: {
    agentId: string;
    type: TaskType;
    payload: Record<string, unknown>;
    priority?: string;
  }): Promise<{ taskId: string; task: any }> {
    try {
      // Validate agent exists and has required capabilities
      const agent = this.context.orchestrator.getAgent(args.agentId);
      if (!agent) {
        throw new Error(`Agent not found: ${args.agentId}`);
      }

      if (!agent.capabilities.includes(args.type)) {
        throw new Error(
          `Agent ${args.agentId} does not have capability: ${args.type}`
        );
      }

      // Add priority to payload if specified
      const taskPayload = {
        ...args.payload,
        ...(args.priority && { priority: args.priority }),
      };

      const taskId = await this.context.orchestrator.submitTask({
        agentId: args.agentId,
        type: args.type,
        description: `Task of type ${args.type}`,
        priority: (args.priority || "normal") as "low" | "normal" | "high",
        payload: taskPayload,
      });

      const task = this.context.orchestrator.getTask(taskId);

      if (!task) {
        throw new Error("Failed to retrieve submitted task");
      }

      this.context.logger.info(
        `Task submitted via MCP: ${taskId} to agent ${args.agentId}`
      );

      return { taskId, task };
    } catch (error) {
      this.context.logger.error("Failed to submit task:", error);
      throw error;
    }
  }

  private async getTask(args: { taskId: string }): Promise<any> {
    try {
      const task = this.context.orchestrator.getTask(args.taskId);

      if (!task) {
        throw new Error(`Task not found: ${args.taskId}`);
      }

      return task;
    } catch (error) {
      this.context.logger.error("Failed to get task:", error);
      throw error;
    }
  }

  private async cancelTask(args: {
    taskId: string;
    reason?: string;
  }): Promise<{ task: any; cancelled: boolean }> {
    try {
      const task = this.context.orchestrator.getTask(args.taskId);

      if (!task) {
        throw new Error(`Task not found: ${args.taskId}`);
      }

      if (task.status === "completed" || task.status === "failed") {
        throw new Error(`Cannot cancel task in status: ${task.status}`);
      }

      // In a real implementation, you would call orchestrator.cancelTask()
      // For now, we'll simulate the cancellation
      const cancelledTask = {
        ...task,
        status: "cancelled" as const,
        updatedAt: new Date(),
        cancelledAt: new Date(),
        cancellationReason: args.reason || "Cancelled via MCP",
      };

      this.context.logger.info(`Task cancelled via MCP: ${args.taskId}`);

      return {
        task: cancelledTask,
        cancelled: true,
      };
    } catch (error) {
      this.context.logger.error("Failed to cancel task:", error);
      throw error;
    }
  }

  private async listTasks(args: {
    agentId?: string;
    status?: string;
    type?: TaskType;
    limit?: number;
  }): Promise<{ tasks: any[]; total: number; filters: any }> {
    try {
      // Simplified implementation - in a real system you'd have proper querying
      // For now, we'll return mock data
      const allTasks = [
        {
          id: "task_001",
          agentId: "agent_001",
          type: "process",
          status: "completed",
          payload: { data: "sample" },
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
          completedAt: new Date().toISOString(),
        },
      ];

      let filteredTasks = allTasks;

      // Apply filters
      if (args.agentId) {
        filteredTasks = filteredTasks.filter((t) => t.agentId === args.agentId);
      }

      if (args.status) {
        filteredTasks = filteredTasks.filter((t) => t.status === args.status);
      }

      if (args.type) {
        filteredTasks = filteredTasks.filter((t) => t.type === args.type);
      }

      // Apply limit
      const limit = args.limit || 50;
      filteredTasks = filteredTasks.slice(0, limit);

      return {
        tasks: filteredTasks,
        total: filteredTasks.length,
        filters: args,
      };
    } catch (error) {
      this.context.logger.error("Failed to list tasks:", error);
      throw error;
    }
  }

  private async retryTask(args: {
    taskId: string;
    payload?: Record<string, unknown>;
    priority?: string;
  }): Promise<{ originalTaskId: string; newTaskId: string; newTask: any }> {
    try {
      const originalTask = this.context.orchestrator.getTask(args.taskId);

      if (!originalTask) {
        throw new Error(`Task not found: ${args.taskId}`);
      }

      if (originalTask.status !== "failed") {
        throw new Error(
          `Can only retry failed tasks. Current status: ${originalTask.status}`
        );
      }

      // Create retry task with modified payload if provided
      const retryPayload = {
        ...originalTask.payload,
        ...args.payload,
        isRetry: true,
        originalTaskId: args.taskId,
        ...(args.priority && { priority: args.priority }),
      };

      const newTaskId = await this.context.orchestrator.submitTask({
        agentId: originalTask.agentId,
        type: originalTask.type,
        description: `Retry of task ${args.taskId}: ${originalTask.description}`,
        priority: (args.priority || originalTask.priority) as
          | "low"
          | "normal"
          | "high",
        payload: retryPayload,
      });

      const newTask = this.context.orchestrator.getTask(newTaskId);

      if (!newTask) {
        throw new Error("Failed to retrieve retried task");
      }

      this.context.logger.info(
        `Task retried via MCP: ${args.taskId} -> ${newTaskId}`
      );

      return {
        originalTaskId: args.taskId,
        newTaskId,
        newTask,
      };
    } catch (error) {
      this.context.logger.error("Failed to retry task:", error);
      throw error;
    }
  }
}
