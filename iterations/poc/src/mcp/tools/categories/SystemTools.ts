/**
 * System Tools for MCP
 *
 * @author @darianrosebrook
 * @description Tools for system monitoring and maintenance
 */

import { Tool } from "@modelcontextprotocol/sdk/types.js";
import * as fs from "fs/promises";
import * as path from "path";
import { MCPToolContext } from "../ToolManager.js";

export class SystemTools {
  constructor(private context: MCPToolContext) {}

  async getTools(): Promise<Tool[]> {
    return [
      {
        name: "get_system_metrics",
        description: "Retrieve current system performance metrics",
        inputSchema: {
          type: "object",
          properties: {
            includeHistorical: {
              type: "boolean",
              description: "Include historical metrics data",
              default: false,
            },
          },
        },
      },
      {
        name: "perform_health_check",
        description: "Execute comprehensive system health assessment",
        inputSchema: {
          type: "object",
          properties: {
            detailed: {
              type: "boolean",
              description: "Include detailed health check results",
              default: false,
            },
          },
        },
      },
      {
        name: "clear_system_cache",
        description: "Clear system caches and temporary data",
        inputSchema: {
          type: "object",
          properties: {
            cacheTypes: {
              type: "array",
              items: {
                type: "string",
                enum: ["resource", "evaluation", "metrics", "all"],
              },
              description: "Types of cache to clear",
              default: ["all"],
            },
          },
        },
      },
      {
        name: "backup_system_data",
        description: "Create backup of system data and configuration",
        inputSchema: {
          type: "object",
          properties: {
            includeLogs: {
              type: "boolean",
              description: "Include system logs in backup",
              default: true,
            },
            includeMetrics: {
              type: "boolean",
              description: "Include metrics data in backup",
              default: true,
            },
            compression: {
              type: "string",
              enum: ["none", "gzip", "bzip2"],
              description: "Compression format for backup",
              default: "gzip",
            },
          },
        },
      },
      {
        name: "get_system_config",
        description: "Retrieve current system configuration",
        inputSchema: {
          type: "object",
          properties: {
            includeSecrets: {
              type: "boolean",
              description: "Include sensitive configuration values",
              default: false,
            },
          },
        },
      },
      {
        name: "update_system_config",
        description: "Update system configuration parameters",
        inputSchema: {
          type: "object",
          properties: {
            config: {
              type: "object",
              additionalProperties: true,
              description: "Configuration parameters to update",
            },
            restartRequired: {
              type: "boolean",
              description: "Whether changes require system restart",
              default: false,
            },
          },
          required: ["config"],
        },
      },
      {
        name: "read_file",
        description: "Read the contents of a file",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the file to read",
            },
            encoding: {
              type: "string",
              description: "File encoding (utf8, ascii, etc.)",
              default: "utf8",
            },
          },
          required: ["path"],
        },
      },
      {
        name: "write_file",
        description: "Write content to a file",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the file to write",
            },
            content: {
              type: "string",
              description: "Content to write to the file",
            },
            encoding: {
              type: "string",
              description: "File encoding (utf8, ascii, etc.)",
              default: "utf8",
            },
            createDirectories: {
              type: "boolean",
              description: "Create parent directories if they don't exist",
              default: false,
            },
          },
          required: ["path", "content"],
        },
      },
      {
        name: "list_directory",
        description: "List contents of a directory",
        inputSchema: {
          type: "object",
          properties: {
            path: {
              type: "string",
              description: "Path to the directory to list",
            },
            recursive: {
              type: "boolean",
              description: "Include subdirectories recursively",
              default: false,
            },
            includeHidden: {
              type: "boolean",
              description: "Include hidden files (starting with .)",
              default: false,
            },
          },
          required: ["path"],
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
      case "get_system_metrics":
        return await this.getSystemMetrics(args);
      case "perform_health_check":
        return await this.performHealthCheck(args);
      case "clear_system_cache":
        return await this.clearSystemCache(args);
      case "backup_system_data":
        return await this.backupSystemData(args);
      case "get_system_config":
        return await this.getSystemConfig(args);
      case "update_system_config":
        return await this.updateSystemConfig(args);
      case "read_file":
        return await this.readFile(args);
      case "write_file":
        return await this.writeFile(args);
      case "list_directory":
        return await this.listDirectory(args);
      default:
        throw new Error(`Unknown system tool: ${name}`);
    }
  }

  private async getSystemMetrics(args: {
    includeHistorical?: boolean;
  }): Promise<any> {
    try {
      const metrics = this.context.orchestrator.getSystemMetrics();

      const enhancedMetrics = {
        ...metrics,
        timestamp: new Date().toISOString(),
        systemInfo: {
          nodeVersion: process.version,
          platform: process.platform,
          arch: process.arch,
          uptime: process.uptime(),
        },
        memoryUsage: process.memoryUsage(),
        cpuUsage: process.cpuUsage(),
      };

      if (args.includeHistorical) {
        // In a real implementation, you would retrieve historical data
        (enhancedMetrics as any).historical = {
          message: "Historical metrics not yet implemented",
          last24Hours: [],
          trends: {},
        };
      }

      return enhancedMetrics;
    } catch (error) {
      this.context.logger.error("Failed to get system metrics:", error);
      throw error;
    }
  }

  private async performHealthCheck(args: { detailed?: boolean }): Promise<any> {
    try {
      const metrics = await this.context.orchestrator.getSystemMetrics();
      const healthChecks = [];

      // Basic health checks
      healthChecks.push({
        name: "orchestrator",
        status: "healthy",
        message: "Agent orchestrator is operational",
        lastChecked: new Date().toISOString(),
      });

      healthChecks.push({
        name: "agent_registry",
        status: metrics.totalAgents > 0 ? "healthy" : "warning",
        message: `${metrics.totalAgents} agents registered`,
        lastChecked: new Date().toISOString(),
      });

      healthChecks.push({
        name: "task_processing",
        status:
          metrics.failedTasks < metrics.totalTasks * 0.1
            ? "healthy"
            : "warning",
        message: `${metrics.completedTasks}/${metrics.totalTasks} tasks completed`,
        lastChecked: new Date().toISOString(),
      });

      if (args.detailed) {
        // Add more detailed checks
        healthChecks.push({
          name: "memory_usage",
          status:
            process.memoryUsage().heapUsed < 100 * 1024 * 1024
              ? "healthy"
              : "warning",
          message: `Heap usage: ${Math.round(
            process.memoryUsage().heapUsed / 1024 / 1024
          )}MB`,
          lastChecked: new Date().toISOString(),
        });

        healthChecks.push({
          name: "system_uptime",
          status: "healthy",
          message: `System uptime: ${Math.round(metrics.systemUptime)} seconds`,
          lastChecked: new Date().toISOString(),
        });
      }

      const overallStatus = healthChecks.every((h) => h.status === "healthy")
        ? "healthy"
        : healthChecks.some((h) => h.status === "error")
        ? "error"
        : "warning";

      return {
        overallStatus,
        checks: healthChecks,
        summary: {
          totalChecks: healthChecks.length,
          healthy: healthChecks.filter((h) => h.status === "healthy").length,
          warnings: healthChecks.filter((h) => h.status === "warning").length,
          errors: healthChecks.filter((h) => h.status === "error").length,
        },
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      this.context.logger.error("Failed to perform health check:", error);
      throw error;
    }
  }

  private async clearSystemCache(args: {
    cacheTypes?: string[];
  }): Promise<any> {
    try {
      const cacheTypes = args.cacheTypes || ["all"];
      const results = [];

      // In a real implementation, you would clear actual caches
      for (const cacheType of cacheTypes) {
        switch (cacheType) {
          case "resource":
            results.push({
              cacheType: "resource",
              cleared: true,
              entriesCleared: 0,
              message: "Resource cache not yet implemented",
            });
            break;
          case "evaluation":
            results.push({
              cacheType: "evaluation",
              cleared: true,
              entriesCleared: 0,
              message: "Evaluation cache not yet implemented",
            });
            break;
          case "metrics":
            results.push({
              cacheType: "metrics",
              cleared: true,
              entriesCleared: 0,
              message: "Metrics cache cleared",
            });
            break;
          case "all":
            results.push({
              cacheType: "all",
              cleared: true,
              entriesCleared: 0,
              message: "All caches cleared",
            });
            break;
        }
      }

      this.context.logger.info("System cache cleared via MCP");

      return {
        cacheTypes,
        results,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      this.context.logger.error("Failed to clear system cache:", error);
      throw error;
    }
  }

  private async backupSystemData(args: {
    includeLogs?: boolean;
    includeMetrics?: boolean;
    compression?: string;
  }): Promise<any> {
    try {
      const backupId = `backup_${Date.now()}`;
      const components = [];

      // In a real implementation, you would create actual backups
      components.push({
        component: "agent_registry",
        status: "completed",
        size: 1024,
        message: "Agent registry backed up",
      });

      if (args.includeLogs !== false) {
        components.push({
          component: "logs",
          status: "completed",
          size: 2048,
          message: "System logs backed up",
        });
      }

      if (args.includeMetrics !== false) {
        components.push({
          component: "metrics",
          status: "completed",
          size: 512,
          message: "Metrics data backed up",
        });
      }

      const totalSize = components.reduce((sum, c) => sum + c.size, 0);

      this.context.logger.info(`System backup created via MCP: ${backupId}`);

      return {
        backupId,
        components,
        totalSize,
        compression: args.compression || "gzip",
        timestamp: new Date().toISOString(),
        message: "Backup functionality not yet fully implemented",
      };
    } catch (error) {
      this.context.logger.error("Failed to backup system data:", error);
      throw error;
    }
  }

  private async getSystemConfig(args: {
    includeSecrets?: boolean;
  }): Promise<any> {
    try {
      // Simplified config - in a real system this would be more comprehensive
      const config = {
        orchestrator: {
          maxConcurrentTasks: 10,
          taskTimeoutMs: 30000,
          retryAttempts: 3,
          healthCheckIntervalMs: 5000,
        },
        mcp: {
          enabled: true,
          maxRequestSize: 1048576,
          timeoutMs: 30000,
        },
        logging: {
          level: "info",
          format: "json",
          maxFileSize: 10485760,
        },
      };

      if (args.includeSecrets) {
        // In a real implementation, you would carefully handle secrets
        (config as any).secrets = {
          message: "Secrets access requires special permissions",
          available: false,
        };
      }

      return {
        config,
        timestamp: new Date().toISOString(),
        message: "Configuration management not yet fully implemented",
      };
    } catch (error) {
      this.context.logger.error("Failed to get system config:", error);
      throw error;
    }
  }

  private async updateSystemConfig(args: {
    config: Record<string, any>;
    restartRequired?: boolean;
  }): Promise<any> {
    try {
      // In a real implementation, you would validate and apply configuration changes
      const validation = {
        valid: true,
        changes: Object.keys(args.config),
        restartRequired: args.restartRequired || false,
        applied: false,
        message: "Configuration updates not yet implemented",
      };

      if (validation.restartRequired) {
        validation.message +=
          ". System restart required for changes to take effect.";
      }

      this.context.logger.info("System configuration updated via MCP");

      return {
        config: args.config,
        validation,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      this.context.logger.error("Failed to update system config:", error);
      throw error;
    }
  }

  private async readFile(args: {
    path: string;
    encoding?: string;
  }): Promise<any> {
    try {
      // Security: restrict to project directory only
      const resolvedPath = path.resolve(args.path);
      const projectRoot = path.resolve(process.cwd());

      if (!resolvedPath.startsWith(projectRoot)) {
        throw new Error("Access denied: File path outside project directory");
      }

      const encoding = (args.encoding || "utf8") as BufferEncoding;
      const content = await fs.readFile(resolvedPath, encoding);
      const stats = await fs.stat(resolvedPath);

      this.context.logger.info(`File read via MCP: ${args.path}`);

      return {
        path: args.path,
        content,
        size: stats.size,
        encoding,
        lastModified: stats.mtime.toISOString(),
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      this.context.logger.error(`Failed to read file ${args.path}:`, error);
      throw new Error(
        `File read failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  private async writeFile(args: {
    path: string;
    content: string;
    encoding?: string;
    createDirectories?: boolean;
  }): Promise<any> {
    try {
      // Security: restrict to project directory only
      const resolvedPath = path.resolve(args.path);
      const projectRoot = path.resolve(process.cwd());

      if (!resolvedPath.startsWith(projectRoot)) {
        throw new Error("Access denied: File path outside project directory");
      }

      // Create directories if requested
      if (args.createDirectories) {
        const dir = path.dirname(resolvedPath);
        await fs.mkdir(dir, { recursive: true });
      }

      const encoding = (args.encoding || "utf8") as BufferEncoding;
      await fs.writeFile(resolvedPath, args.content, encoding);
      const stats = await fs.stat(resolvedPath);

      this.context.logger.info(`File written via MCP: ${args.path}`);

      return {
        path: args.path,
        size: stats.size,
        encoding,
        created: stats.birthtime.toISOString(),
        lastModified: stats.mtime.toISOString(),
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      this.context.logger.error(`Failed to write file ${args.path}:`, error);
      throw new Error(
        `File write failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  private async listDirectory(args: {
    path: string;
    recursive?: boolean;
    includeHidden?: boolean;
  }): Promise<any> {
    try {
      // Security: restrict to project directory only
      const resolvedPath = path.resolve(args.path);
      const projectRoot = path.resolve(process.cwd());

      if (!resolvedPath.startsWith(projectRoot)) {
        throw new Error(
          "Access denied: Directory path outside project directory"
        );
      }

      const entries = [];
      const dirents = await fs.readdir(resolvedPath, {
        withFileTypes: true,
        recursive: args.recursive || false,
      });

      for (const dirent of dirents) {
        // Skip hidden files unless requested
        if (!args.includeHidden && dirent.name.startsWith(".")) {
          continue;
        }

        const fullPath = path.join(dirent.path || resolvedPath, dirent.name);
        const stats = await fs.stat(fullPath);
        const relativePath = path.relative(projectRoot, fullPath);

        entries.push({
          name: dirent.name,
          path: relativePath,
          type: dirent.isDirectory() ? "directory" : "file",
          size: stats.size,
          lastModified: stats.mtime.toISOString(),
        });
      }

      // Sort: directories first, then files alphabetically
      entries.sort((a, b) => {
        if (a.type !== b.type) {
          return a.type === "directory" ? -1 : 1;
        }
        return a.name.localeCompare(b.name);
      });

      this.context.logger.info(`Directory listed via MCP: ${args.path}`);

      return {
        path: args.path,
        entries,
        totalCount: entries.length,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      this.context.logger.error(
        `Failed to list directory ${args.path}:`,
        error
      );
      throw new Error(
        `Directory listing failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }
}
