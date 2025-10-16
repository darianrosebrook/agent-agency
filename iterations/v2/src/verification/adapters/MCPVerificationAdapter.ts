/**
 * @fileoverview MCP Verification Adapter - ARBITER-021
 *
 * Provides MCP protocol interface for agent-driven verification tools
 * enabling remote verification capabilities during task execution.
 *
 * @author @darianrosebrook
 */

export interface MCPVerificationToolAdapter {
  /**
   * Unique identifier for the verification tool
   */
  readonly toolId: string;

  /**
   * Human-readable name for the tool
   */
  readonly name: string;

  /**
   * Description of what the tool verifies
   */
  readonly description: string;

  /**
   * Supported verification types
   */
  readonly supportedTypes: string[];

  /**
   * Execute verification using the MCP tool
   */
  verify(request: MCPVerificationRequest): Promise<MCPVerificationResult>;

  /**
   * Check if tool is available and responsive
   */
  healthCheck(): Promise<boolean>;

  /**
   * Get tool capabilities and configuration
   */
  getCapabilities(): Promise<MCPToolCapabilities>;
}

export interface MCPVerificationRequest {
  toolId: string;
  content: string;
  verificationType: string;
  context?: Record<string, any>;
  options?: Record<string, any>;
  timeoutMs?: number;
}

export interface MCPVerificationResult {
  success: boolean;
  confidence: number;
  result: any;
  evidence: Array<{
    type: string;
    content: string;
    confidence: number;
  }>;
  error?: string;
  executionTimeMs: number;
  toolMetadata: {
    toolId: string;
    version: string;
    timestamp: Date;
  };
}

export interface MCPToolCapabilities {
  supportedTypes: string[];
  maxContentSize: number;
  timeoutMs: number;
  rateLimitPerMinute: number;
  requiresAuthentication: boolean;
  supportedOptions: string[];
}

/**
 * MCP Verification Adapter Registry
 */
export class MCPVerificationRegistry {
  private adapters: Map<string, MCPVerificationToolAdapter> = new Map();
  private healthStatus: Map<string, boolean> = new Map();

  /**
   * Register a new MCP verification tool adapter
   */
  register(adapter: MCPVerificationToolAdapter): void {
    this.adapters.set(adapter.toolId, adapter);
    this.healthStatus.set(adapter.toolId, false);
  }

  /**
   * Unregister a verification tool adapter
   */
  unregister(toolId: string): void {
    this.adapters.delete(toolId);
    this.healthStatus.delete(toolId);
  }

  /**
   * Get available verification tools for a specific type
   */
  getToolsForType(verificationType: string): MCPVerificationToolAdapter[] {
    return Array.from(this.adapters.values()).filter((adapter) =>
      adapter.supportedTypes.includes(verificationType)
    );
  }

  /**
   * Execute verification using the best available tool
   */
  async verify(
    request: MCPVerificationRequest
  ): Promise<MCPVerificationResult> {
    const adapter = this.adapters.get(request.toolId);

    if (!adapter) {
      throw new Error(`MCP verification tool not found: ${request.toolId}`);
    }

    // Check health status
    const isHealthy = await this.checkToolHealth(request.toolId);
    if (!isHealthy) {
      throw new Error(
        `MCP verification tool is not healthy: ${request.toolId}`
      );
    }

    return await adapter.verify(request);
  }

  /**
   * Check health of all registered tools
   */
  async checkAllToolsHealth(): Promise<Map<string, boolean>> {
    const healthChecks = Array.from(this.adapters.entries()).map(
      async ([toolId, adapter]) => {
        try {
          const isHealthy = await adapter.healthCheck();
          this.healthStatus.set(toolId, isHealthy);
          return [toolId, isHealthy] as const;
        } catch (error) {
          this.healthStatus.set(toolId, false);
          return [toolId, false] as const;
        }
      }
    );

    const results = await Promise.all(healthChecks);
    return new Map(results);
  }

  /**
   * Check health of a specific tool
   */
  async checkToolHealth(toolId: string): Promise<boolean> {
    const adapter = this.adapters.get(toolId);
    if (!adapter) {
      return false;
    }

    try {
      const isHealthy = await adapter.healthCheck();
      this.healthStatus.set(toolId, isHealthy);
      return isHealthy;
    } catch (error) {
      this.healthStatus.set(toolId, false);
      return false;
    }
  }

  /**
   * Get all registered tool capabilities
   */
  async getAllCapabilities(): Promise<Map<string, MCPToolCapabilities>> {
    const capabilities = new Map<string, MCPToolCapabilities>();

    for (const [toolId, adapter] of this.adapters.entries()) {
      try {
        const caps = await adapter.getCapabilities();
        capabilities.set(toolId, caps);
      } catch (error) {
        // Skip tools that fail capability check
        console.warn(`Failed to get capabilities for tool ${toolId}:`, error);
      }
    }

    return capabilities;
  }

  /**
   * Get registry status
   */
  getStatus(): {
    totalTools: number;
    healthyTools: number;
    unhealthyTools: string[];
    availableTypes: string[];
  } {
    const totalTools = this.adapters.size;
    const healthyTools = Array.from(this.healthStatus.values()).filter(
      Boolean
    ).length;
    const unhealthyTools = Array.from(this.healthStatus.entries())
      .filter(([, healthy]) => !healthy)
      .map(([toolId]) => toolId);

    const allTypes = new Set<string>();
    for (const adapter of this.adapters.values()) {
      adapter.supportedTypes.forEach((type) => allTypes.add(type));
    }

    return {
      totalTools,
      healthyTools,
      unhealthyTools,
      availableTypes: Array.from(allTypes),
    };
  }
}

/**
 * Base implementation for MCP verification tools
 */
export abstract class BaseMCPVerificationTool
  implements MCPVerificationToolAdapter
{
  constructor(
    public readonly toolId: string,
    public readonly name: string,
    public readonly description: string,
    public readonly supportedTypes: string[]
  ) {}

  abstract verify(
    request: MCPVerificationRequest
  ): Promise<MCPVerificationResult>;

  async healthCheck(): Promise<boolean> {
    try {
      // Default health check - can be overridden
      const capabilities = await this.getCapabilities();
      return capabilities.timeoutMs > 0;
    } catch (error) {
      return false;
    }
  }

  async getCapabilities(): Promise<MCPToolCapabilities> {
    return {
      supportedTypes: this.supportedTypes,
      maxContentSize: 1024 * 1024, // 1MB default
      timeoutMs: 30000, // 30 seconds default
      rateLimitPerMinute: 60, // 60 requests per minute default
      requiresAuthentication: false,
      supportedOptions: [],
    };
  }

  protected createResult(
    success: boolean,
    confidence: number,
    result: any,
    evidence: MCPVerificationResult["evidence"],
    executionTimeMs: number,
    error?: string
  ): MCPVerificationResult {
    return {
      success,
      confidence,
      result,
      evidence,
      error,
      executionTimeMs,
      toolMetadata: {
        toolId: this.toolId,
        version: "1.0.0", // Default version
        timestamp: new Date(),
      },
    };
  }
}

