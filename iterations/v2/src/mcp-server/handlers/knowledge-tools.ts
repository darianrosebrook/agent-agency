/**
 * @fileoverview MCP Tool Handlers for Knowledge Seeker (ARBITER-006)
 *
 * Exposes knowledge research capabilities as discoverable MCP tools
 * that worker LLMs can invoke through the Model Context Protocol.
 *
 * @author @darianrosebrook
 */

import { ArbiterOrchestrator } from "../../orchestrator/ArbiterOrchestrator";
import { KnowledgeQuery, QueryType, SearchResult } from "../../types/knowledge";

/**
 * MCP Tool Definition for knowledge-search
 */
export const knowledgeSearchToolDefinition = {
  name: "knowledge-search",
  description:
    "Search for information using intelligent research capabilities. " +
    "Queries multiple search providers, processes results for relevance and credibility, " +
    "and returns high-quality research findings. Use for factual information, " +
    "technical documentation, comparisons, trends, or explanatory content.",
  inputSchema: {
    type: "object",
    properties: {
      query: {
        type: "string",
        description:
          "The search query or question to research. Be specific and clear.",
      },
      queryType: {
        type: "string",
        enum: ["factual", "explanatory", "comparative", "trend", "technical"],
        description:
          "Type of query: factual (facts/data), explanatory (how/why), " +
          "comparative (comparison), trend (patterns over time), technical (documentation)",
        default: "factual",
      },
      maxResults: {
        type: "number",
        description:
          "Maximum number of results to return. Higher values take longer.",
        minimum: 1,
        maximum: 20,
        default: 5,
      },
      relevanceThreshold: {
        type: "number",
        description:
          "Minimum relevance score (0-1). Higher values return fewer but more relevant results.",
        minimum: 0,
        maximum: 1,
        default: 0.7,
      },
      timeoutMs: {
        type: "number",
        description: "Query timeout in milliseconds. Default 10 seconds.",
        minimum: 1000,
        maximum: 30000,
        default: 10000,
      },
      context: {
        type: "object",
        description:
          "Additional context for the query (optional). Can include domain, audience, etc.",
        additionalProperties: true,
      },
    },
    required: ["query"],
  },
};

/**
 * MCP Tool Definition for knowledge-status
 */
export const knowledgeStatusToolDefinition = {
  name: "knowledge-status",
  description:
    "Get the current status of the Knowledge Seeker system, " +
    "including available search providers, cache statistics, and health metrics.",
  inputSchema: {
    type: "object",
    properties: {},
  },
};

/**
 * MCP Tool Definition for knowledge-clear-cache
 */
export const knowledgeClearCacheToolDefinition = {
  name: "knowledge-clear-cache",
  description:
    "Clear the knowledge query cache. Use when you need fresh results " +
    "or to free up memory. Requires appropriate permissions.",
  inputSchema: {
    type: "object",
    properties: {
      confirm: {
        type: "boolean",
        description: "Must be true to confirm cache clearing",
      },
    },
    required: ["confirm"],
  },
};

/**
 * MCP Resource Definition for knowledge status
 */
export const knowledgeStatusResourceDefinition = {
  uri: "knowledge://status",
  name: "Knowledge Seeker Status",
  description: "Real-time status of knowledge research capabilities",
  mimeType: "application/json",
};

/**
 * Knowledge Search Tool Handler
 *
 * Processes knowledge-search tool invocations from worker LLMs
 */
export async function handleKnowledgeSearch(
  orchestrator: ArbiterOrchestrator,
  args: any
): Promise<{
  success: boolean;
  data?: any;
  error?: string;
}> {
  try {
    // Validate required fields
    if (!args.query || typeof args.query !== "string") {
      return {
        success: false,
        error: "Query is required and must be a string",
      };
    }

    // Build knowledge query
    const query: KnowledgeQuery = {
      id: `mcp-query-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      query: args.query,
      queryType: (args.queryType as QueryType) || QueryType.FACTUAL,
      maxResults: args.maxResults || 5,
      relevanceThreshold: args.relevanceThreshold || 0.7,
      timeoutMs: args.timeoutMs || 10000,
      context: args.context || {},
      metadata: {
        requesterId: "mcp-tool",
        priority: 5,
        createdAt: new Date(),
        tags: ["mcp", "knowledge-search"],
      },
    };

    // Execute query through orchestrator
    const response = await orchestrator.processKnowledgeQuery(query);

    // Format response for MCP
    return {
      success: true,
      data: {
        query: response.query.query,
        summary: response.summary,
        confidence: response.confidence,
        results: response.results.map((r: SearchResult) => ({
          title: r.title,
          url: r.url,
          snippet: r.content.substring(0, 200) + "...",
          relevance: r.relevanceScore,
          credibility: r.credibilityScore,
          quality: r.quality,
          domain: r.domain,
          publishedAt: r.publishedAt,
        })),
        sourcesUsed: response.sourcesUsed,
        metadata: {
          totalResults: response.metadata.totalResultsFound,
          filtered: response.metadata.resultsFiltered,
          processingTime: response.metadata.processingTimeMs,
          cached: response.metadata.cacheUsed,
          providers: response.metadata.providersQueried,
        },
      },
    };
  } catch (error) {
    console.error("Knowledge search tool error:", error);
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Knowledge Status Tool Handler
 *
 * Returns current status of Knowledge Seeker system
 */
export async function handleKnowledgeStatus(
  orchestrator: ArbiterOrchestrator
): Promise<{
  success: boolean;
  data?: any;
  error?: string;
}> {
  try {
    const status = await orchestrator.getKnowledgeStatus();

    return {
      success: true,
      data: {
        enabled: status.enabled,
        providers: status.providers.map((p: any) => ({
          name: p.name,
          available: p.available,
          health: {
            responseTime: p.health.responseTimeMs,
            errorRate: p.health.errorRate,
            requestsThisMinute: p.health.requestsThisMinute,
          },
        })),
        cache: {
          queryCache: status.cacheStats.queryCacheSize,
          resultCache: status.cacheStats.resultCacheSize,
          hitRate: status.cacheStats.hitRate,
        },
        processing: {
          activeQueries: status.processingStats.activeQueries,
          queuedQueries: status.processingStats.queuedQueries,
        },
      },
    };
  } catch (error) {
    console.error("Knowledge status tool error:", error);
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Knowledge Clear Cache Tool Handler
 *
 * Clears knowledge query cache
 */
export async function handleKnowledgeClearCache(
  orchestrator: ArbiterOrchestrator,
  args: any
): Promise<{
  success: boolean;
  data?: any;
  error?: string;
}> {
  try {
    // Require explicit confirmation
    if (args.confirm !== true) {
      return {
        success: false,
        error: "Cache clearing requires explicit confirmation (confirm: true)",
      };
    }

    await orchestrator.clearKnowledgeCaches();

    return {
      success: true,
      data: {
        message: "Knowledge caches cleared successfully",
        timestamp: new Date().toISOString(),
      },
    };
  } catch (error) {
    console.error("Knowledge clear cache tool error:", error);
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Knowledge Status Resource Handler
 *
 * Returns knowledge status as an MCP resource
 */
export async function handleKnowledgeStatusResource(
  orchestrator: ArbiterOrchestrator
): Promise<{
  contents: Array<{
    uri: string;
    mimeType: string;
    text: string;
  }>;
}> {
  try {
    const status = await orchestrator.getKnowledgeStatus();

    return {
      contents: [
        {
          uri: "knowledge://status",
          mimeType: "application/json",
          text: JSON.stringify(status, null, 2),
        },
      ],
    };
  } catch (error) {
    console.error("Knowledge status resource error:", error);
    return {
      contents: [
        {
          uri: "knowledge://status",
          mimeType: "application/json",
          text: JSON.stringify(
            {
              error: error instanceof Error ? error.message : String(error),
            },
            null,
            2
          ),
        },
      ],
    };
  }
}

/**
 * Register all knowledge tools with MCP server
 */
export function registerKnowledgeTools(mcpServer: any): void {
  // Register tools
  mcpServer.tool(
    knowledgeSearchToolDefinition.name,
    knowledgeSearchToolDefinition.description,
    knowledgeSearchToolDefinition.inputSchema,
    async (args: any) => {
      const orchestrator = mcpServer.getOrchestrator();
      return await handleKnowledgeSearch(orchestrator, args);
    }
  );

  mcpServer.tool(
    knowledgeStatusToolDefinition.name,
    knowledgeStatusToolDefinition.description,
    knowledgeStatusToolDefinition.inputSchema,
    async () => {
      const orchestrator = mcpServer.getOrchestrator();
      return await handleKnowledgeStatus(orchestrator);
    }
  );

  mcpServer.tool(
    knowledgeClearCacheToolDefinition.name,
    knowledgeClearCacheToolDefinition.description,
    knowledgeClearCacheToolDefinition.inputSchema,
    async (args: any) => {
      const orchestrator = mcpServer.getOrchestrator();
      return await handleKnowledgeClearCache(orchestrator, args);
    }
  );

  // Register resources
  mcpServer.resource(
    knowledgeStatusResourceDefinition.uri,
    knowledgeStatusResourceDefinition.name,
    knowledgeStatusResourceDefinition.description,
    knowledgeStatusResourceDefinition.mimeType,
    async () => {
      const orchestrator = mcpServer.getOrchestrator();
      return await handleKnowledgeStatusResource(orchestrator);
    }
  );

  console.log("Knowledge tools registered with MCP server:");
  console.log("  - knowledge-search (tool)");
  console.log("  - knowledge-status (tool)");
  console.log("  - knowledge-clear-cache (tool)");
  console.log("  - knowledge://status (resource)");
}
