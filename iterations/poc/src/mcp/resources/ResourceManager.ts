/**
 * MCP Resource Manager for Agent Agency
 *
 * @author @darianrosebrook
 * @description Manages MCP resource access and retrieval for agent data
 */

import { Resource } from "@modelcontextprotocol/sdk/types.js";
import { AgentOrchestrator } from "../../services/AgentOrchestrator.js";
import { Logger } from "../../utils/Logger.js";

export interface MCPResource {
  uri: string;
  name: string;
  description: string;
  mimeType: string;
}

export interface ResourceReadResult {
  contents: Array<{
    uri: string;
    mimeType: string;
    text: string;
    blob?: string; // Optional for binary content
  }>;
}

export class MCPResourceManager {
  private readonly logger: Logger;
  private readonly orchestrator: AgentOrchestrator;

  constructor(orchestrator: AgentOrchestrator, logger: Logger) {
    this.orchestrator = orchestrator;
    this.logger = logger;
  }

  /**
   * List all available resources
   */
  async listResources(): Promise<{ resources: Resource[] }> {
    try {
      const resources: Resource[] = [];

      // Agent resources
      resources.push(...(await this.getAgentResources()));

      // Task resources
      resources.push(...(await this.getTaskResources()));

      // System resources
      resources.push(...(await this.getSystemResources()));

      // Memory resources (if available)
      resources.push(...(await this.getMemoryResources()));

      this.logger.debug(`Listed ${resources.length} MCP resources`);
      return { resources };
    } catch (error) {
      this.logger.error("Failed to list resources:", error);
      throw error;
    }
  }

  /**
   * Read a specific resource by URI
   */
  async readResource(uri: string): Promise<ResourceReadResult> {
    try {
      const [scheme, type, ...pathParts] = uri.split("/");

      this.logger.debug(`Reading resource: ${uri}`);

      switch (scheme) {
        case "agent:":
          return await this.handleAgentResource(type, pathParts);
        case "task:":
          return await this.handleTaskResource(type, pathParts);
        case "system:":
          return await this.handleSystemResource(type, pathParts);
        case "memory:":
          return await this.handleMemoryResource(type, pathParts);
        default:
          throw new Error(`Unknown resource scheme: ${scheme}`);
      }
    } catch (error) {
      this.logger.error(`Failed to read resource ${uri}:`, error);
      throw error;
    }
  }

  /**
   * Get all agent-related resources
   */
  private async getAgentResources(): Promise<Resource[]> {
    const resources: Resource[] = [];

    try {
      // Get all agents (this is a simplified approach - in production you'd want pagination)
      // For now, we'll create static resource templates
      resources.push({
        uri: "agents://list",
        name: "Agent List",
        description: "Complete list of all registered agents",
        mimeType: "application/json",
      });

      // Template resources for individual agents
      resources.push({
        uri: "agent://{agentId}",
        name: "Individual Agent",
        description: "Detailed information for a specific agent",
        mimeType: "application/json",
      });

      resources.push({
        uri: "agents://capabilities/{agentId}",
        name: "Agent Capabilities",
        description: "Capabilities and proficiency levels for a specific agent",
        mimeType: "application/json",
      });

      resources.push({
        uri: "agents://relationships/{agentId}",
        name: "Agent Relationships",
        description:
          "Collaboration history and relationships for a specific agent",
        mimeType: "application/json",
      });
    } catch (error) {
      this.logger.warn("Failed to get agent resources:", error);
    }

    return resources;
  }

  /**
   * Get all task-related resources
   */
  private async getTaskResources(): Promise<Resource[]> {
    const resources: Resource[] = [];

    try {
      resources.push({
        uri: "tasks://queue",
        name: "Task Queue",
        description: "Current task queue with pending and running tasks",
        mimeType: "application/json",
      });

      resources.push({
        uri: "tasks://history/{agentId}",
        name: "Task History",
        description: "Task execution history for a specific agent",
        mimeType: "application/json",
      });

      resources.push({
        uri: "task://{taskId}",
        name: "Individual Task",
        description: "Detailed information for a specific task",
        mimeType: "application/json",
      });

      resources.push({
        uri: "tasks://metrics",
        name: "Task Metrics",
        description: "Task performance and success metrics",
        mimeType: "application/json",
      });
    } catch (error) {
      this.logger.warn("Failed to get task resources:", error);
    }

    return resources;
  }

  /**
   * Get all system-related resources
   */
  private async getSystemResources(): Promise<Resource[]> {
    const resources: Resource[] = [];

    try {
      resources.push({
        uri: "system://metrics",
        name: "System Metrics",
        description: "Real-time system health and performance metrics",
        mimeType: "application/json",
      });

      resources.push({
        uri: "system://config",
        name: "System Configuration",
        description: "Current system configuration settings",
        mimeType: "application/json",
      });

      resources.push({
        uri: "system://health",
        name: "System Health",
        description: "Comprehensive system health assessment",
        mimeType: "application/json",
      });

      resources.push({
        uri: "system://logs",
        name: "System Logs",
        description: "Recent system activity and error logs",
        mimeType: "application/json",
      });
    } catch (error) {
      this.logger.warn("Failed to get system resources:", error);
    }

    return resources;
  }

  /**
   * Get all memory-related resources (placeholders for future memory system)
   */
  private async getMemoryResources(): Promise<Resource[]> {
    const resources: Resource[] = [];

    try {
      resources.push({
        uri: "memory://experiences/{agentId}",
        name: "Agent Experiences",
        description:
          "Task execution experiences and outcomes for a specific agent",
        mimeType: "application/json",
      });

      resources.push({
        uri: "memory://conversations/{agentId}",
        name: "Agent Conversations",
        description: "Conversation history and context for a specific agent",
        mimeType: "application/json",
      });

      resources.push({
        uri: "memory://capabilities/{agentId}",
        name: "Agent Capability Evolution",
        description: "Capability development and proficiency changes over time",
        mimeType: "application/json",
      });

      resources.push({
        uri: "memory://relationships",
        name: "Agent Relationship Network",
        description: "Cross-agent collaboration and relationship network",
        mimeType: "application/json",
      });
    } catch (error) {
      this.logger.warn("Failed to get memory resources:", error);
    }

    return resources;
  }

  /**
   * Handle agent resource requests
   */
  private async handleAgentResource(
    type: string,
    pathParts: string[]
  ): Promise<ResourceReadResult> {
    switch (type) {
      case "list":
        return await this.getAgentList();
      default:
        // Individual agent by ID
        const agentId = type;
        if (!agentId) {
          throw new Error("Agent ID required");
        }
        return await this.getAgentById(agentId);
    }
  }

  /**
   * Handle task resource requests
   */
  private async handleTaskResource(
    type: string,
    pathParts: string[]
  ): Promise<ResourceReadResult> {
    switch (type) {
      case "queue":
        return await this.getTaskQueue();
      case "history":
        const agentId = pathParts[0];
        if (!agentId) {
          throw new Error("Agent ID required for task history");
        }
        return await this.getTaskHistory(agentId);
      case "metrics":
        return await this.getTaskMetrics();
      default:
        // Individual task by ID
        const taskId = type;
        if (!taskId) {
          throw new Error("Task ID required");
        }
        return await this.getTaskById(taskId);
    }
  }

  /**
   * Handle system resource requests
   */
  private async handleSystemResource(
    type: string,
    pathParts: string[]
  ): Promise<ResourceReadResult> {
    switch (type) {
      case "metrics":
        return await this.getSystemMetrics();
      case "config":
        return await this.getSystemConfig();
      case "health":
        return await this.getSystemHealth();
      case "logs":
        return await this.getSystemLogs();
      default:
        throw new Error(`Unknown system resource type: ${type}`);
    }
  }

  /**
   * Handle memory resource requests
   */
  private async handleMemoryResource(
    type: string,
    pathParts: string[]
  ): Promise<ResourceReadResult> {
    // Placeholder implementations for memory resources
    // These would integrate with the actual memory system when implemented
    switch (type) {
      case "experiences":
        const agentId = pathParts[0];
        return await this.getAgentExperiences(agentId);
      case "conversations":
        const convAgentId = pathParts[0];
        return await this.getAgentConversations(convAgentId);
      case "capabilities":
        const capAgentId = pathParts[0];
        return await this.getAgentCapabilityEvolution(capAgentId);
      case "relationships":
        return await this.getAgentRelationshipNetwork();
      default:
        throw new Error(`Unknown memory resource type: ${type}`);
    }
  }

  // Resource implementation methods

  private async getAgentList(): Promise<ResourceReadResult> {
    // This is a simplified implementation - in a real system you'd want to
    // get all agents from the orchestrator or database
    const agents = [
      {
        id: "agent_001",
        name: "Data Processor",
        type: "worker",
        status: "active",
        capabilities: ["process", "analyze"],
        createdAt: new Date().toISOString(),
      },
    ];

    return {
      contents: [
        {
          uri: "agents://list",
          mimeType: "application/json",
          text: JSON.stringify({ agents, total: agents.length }, null, 2),
        },
      ],
    };
  }

  private async getAgentById(agentId: string): Promise<ResourceReadResult> {
    const agent = this.orchestrator.getAgent(agentId);

    if (!agent) {
      throw new Error(`Agent not found: ${agentId}`);
    }

    return {
      contents: [
        {
          uri: `agent://${agentId}`,
          mimeType: "application/json",
          text: JSON.stringify(agent, null, 2),
        },
      ],
    };
  }

  private async getTaskQueue(): Promise<ResourceReadResult> {
    // Simplified implementation - get pending tasks
    const metrics = await this.orchestrator.getSystemMetrics();

    const queueData = {
      pendingTasks:
        metrics.totalTasks - metrics.completedTasks - metrics.failedTasks,
      runningTasks: 0, // Would need to track this in orchestrator
      totalTasks: metrics.totalTasks,
      timestamp: new Date().toISOString(),
    };

    return {
      contents: [
        {
          uri: "tasks://queue",
          mimeType: "application/json",
          text: JSON.stringify(queueData, null, 2),
        },
      ],
    };
  }

  private async getTaskHistory(agentId: string): Promise<ResourceReadResult> {
    // Placeholder - would need to implement task history tracking
    const history = {
      agentId,
      tasks: [],
      totalTasks: 0,
      successRate: 0,
      timestamp: new Date().toISOString(),
    };

    return {
      contents: [
        {
          uri: `tasks://history/${agentId}`,
          mimeType: "application/json",
          text: JSON.stringify(history, null, 2),
        },
      ],
    };
  }

  private async getTaskById(taskId: string): Promise<ResourceReadResult> {
    const task = this.orchestrator.getTask(taskId);

    if (!task) {
      throw new Error(`Task not found: ${taskId}`);
    }

    return {
      contents: [
        {
          uri: `task://${taskId}`,
          mimeType: "application/json",
          text: JSON.stringify(task, null, 2),
        },
      ],
    };
  }

  private async getTaskMetrics(): Promise<ResourceReadResult> {
    const metrics = await this.orchestrator.getSystemMetrics();

    const taskMetrics = {
      totalTasks: metrics.totalTasks,
      completedTasks: metrics.completedTasks,
      failedTasks: metrics.failedTasks,
      successRate:
        metrics.totalTasks > 0
          ? (metrics.completedTasks / metrics.totalTasks) * 100
          : 0,
      averageTaskDuration: metrics.averageTaskDuration,
      timestamp: new Date().toISOString(),
    };

    return {
      contents: [
        {
          uri: "tasks://metrics",
          mimeType: "application/json",
          text: JSON.stringify(taskMetrics, null, 2),
        },
      ],
    };
  }

  private async getSystemMetrics(): Promise<ResourceReadResult> {
    const metrics = await this.orchestrator.getSystemMetrics();

    return {
      contents: [
        {
          uri: "system://metrics",
          mimeType: "application/json",
          text: JSON.stringify(metrics, null, 2),
        },
      ],
    };
  }

  private async getSystemConfig(): Promise<ResourceReadResult> {
    // Simplified config - would need to expose actual configuration
    const config = {
      maxConcurrentTasks: 10,
      taskTimeoutMs: 30000,
      retryAttempts: 3,
      healthCheckIntervalMs: 5000,
      timestamp: new Date().toISOString(),
    };

    return {
      contents: [
        {
          uri: "system://config",
          mimeType: "application/json",
          text: JSON.stringify(config, null, 2),
        },
      ],
    };
  }

  private async getSystemHealth(): Promise<ResourceReadResult> {
    const metrics = await this.orchestrator.getSystemMetrics();

    const health = {
      status: "healthy",
      uptime: metrics.systemUptime,
      activeAgents: metrics.activeAgents,
      totalAgents: metrics.totalAgents,
      pendingTasks:
        metrics.totalTasks - metrics.completedTasks - metrics.failedTasks,
      timestamp: new Date().toISOString(),
      checks: {
        orchestrator: "healthy",
        database: "unknown", // Would need actual health checks
        cache: "unknown",
      },
    };

    return {
      contents: [
        {
          uri: "system://health",
          mimeType: "application/json",
          text: JSON.stringify(health, null, 2),
        },
      ],
    };
  }

  private async getSystemLogs(): Promise<ResourceReadResult> {
    // Simplified logs - would need actual log aggregation
    const logs = {
      recent: [
        {
          level: "info",
          message: "Agent orchestrator initialized",
          timestamp: new Date().toISOString(),
        },
      ],
      total: 1,
      timestamp: new Date().toISOString(),
    };

    return {
      contents: [
        {
          uri: "system://logs",
          mimeType: "application/json",
          text: JSON.stringify(logs, null, 2),
        },
      ],
    };
  }

  // Memory resource placeholders (for future implementation)

  private async getAgentExperiences(
    agentId: string
  ): Promise<ResourceReadResult> {
    const experiences = {
      agentId,
      experiences: [],
      totalExperiences: 0,
      successRate: 0,
      message: "Memory system not yet implemented",
      timestamp: new Date().toISOString(),
    };

    return {
      contents: [
        {
          uri: `memory://experiences/${agentId}`,
          mimeType: "application/json",
          text: JSON.stringify(experiences, null, 2),
        },
      ],
    };
  }

  private async getAgentConversations(
    agentId: string
  ): Promise<ResourceReadResult> {
    const conversations = {
      agentId,
      conversations: [],
      totalConversations: 0,
      message: "Memory system not yet implemented",
      timestamp: new Date().toISOString(),
    };

    return {
      contents: [
        {
          uri: `memory://conversations/${agentId}`,
          mimeType: "application/json",
          text: JSON.stringify(conversations, null, 2),
        },
      ],
    };
  }

  private async getAgentCapabilityEvolution(
    agentId: string
  ): Promise<ResourceReadResult> {
    const evolution = {
      agentId,
      capabilities: [],
      evolution: [],
      message: "Memory system not yet implemented",
      timestamp: new Date().toISOString(),
    };

    return {
      contents: [
        {
          uri: `memory://capabilities/${agentId}`,
          mimeType: "application/json",
          text: JSON.stringify(evolution, null, 2),
        },
      ],
    };
  }

  private async getAgentRelationshipNetwork(): Promise<ResourceReadResult> {
    const network = {
      relationships: [],
      totalRelationships: 0,
      message: "Memory system not yet implemented",
      timestamp: new Date().toISOString(),
    };

    return {
      contents: [
        {
          uri: "memory://relationships",
          mimeType: "application/json",
          text: JSON.stringify(network, null, 2),
        },
      ],
    };
  }
}
