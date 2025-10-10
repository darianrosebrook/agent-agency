/**
 * Query Optimizer - Intelligent query analysis and optimization
 *
 * @author @darianrosebrook
 * @description Analyzes database queries and applies optimization strategies
 */

import { Logger } from "../utils/Logger.js";

export interface QueryAnalysis {
  query: string;
  type: "select" | "insert" | "update" | "delete" | "complex";
  tables: string[];
  conditions: QueryCondition[];
  joins: JoinInfo[];
  aggregations: string[];
  sorting: SortInfo[];
  estimatedComplexity: number; // 1-10 scale
  recommendedIndexes: IndexRecommendation[];
  optimizationOpportunities: OptimizationOpportunity[];
}

export interface QueryCondition {
  column: string;
  operator: string;
  value?: any;
  selectivity: number; // Estimated selectivity (0-1)
}

export interface JoinInfo {
  table1: string;
  table2: string;
  type: "inner" | "left" | "right" | "full";
  condition: string;
}

export interface SortInfo {
  column: string;
  direction: "asc" | "desc";
  nullable: boolean;
}

export interface IndexRecommendation {
  table: string;
  columns: string[];
  type: "btree" | "hash" | "gin" | "gist";
  reason: string;
  estimatedImprovement: number; // Percentage improvement
}

export interface OptimizationOpportunity {
  type: "index" | "query_rewrite" | "denormalization" | "caching";
  description: string;
  estimatedBenefit: number;
  complexity: "low" | "medium" | "high";
}

export interface QueryPlan {
  originalQuery: string;
  optimizedQuery: string;
  analysis: QueryAnalysis;
  estimatedCost: number;
  estimatedRows: number;
  executionTime?: number;
  actualRows?: number;
}

export class QueryOptimizer {
  private logger: Logger;
  private queryHistory: Map<string, QueryPlan> = new Map();
  private indexRecommendations: Map<string, IndexRecommendation[]> = new Map();

  constructor(logger?: Logger) {
    this.logger = logger || new Logger("QueryOptimizer");
  }

  async analyzeQuery(
    query: string,
    executionStats?: {
      executionTime: number;
      actualRows: number;
      estimatedRows: number;
    }
  ): Promise<QueryAnalysis> {
    const analysis: QueryAnalysis = {
      query,
      type: this.determineQueryType(query),
      tables: this.extractTables(query),
      conditions: this.extractConditions(query),
      joins: this.extractJoins(query),
      aggregations: this.extractAggregations(query),
      sorting: this.extractSorting(query),
      estimatedComplexity: 1,
      recommendedIndexes: [],
      optimizationOpportunities: [],
    };

    // Calculate complexity
    analysis.estimatedComplexity = this.calculateComplexity(analysis);

    // Generate recommendations
    analysis.recommendedIndexes = this.generateIndexRecommendations(analysis);
    analysis.optimizationOpportunities =
      this.identifyOptimizationOpportunities(analysis);

    // Store in history
    const queryHash = this.hashQuery(query);
    this.queryHistory.set(queryHash, {
      originalQuery: query,
      optimizedQuery: query, // Will be updated if optimized
      analysis,
      estimatedCost: this.estimateCost(analysis),
      estimatedRows: executionStats?.estimatedRows || 100,
      executionTime: executionStats?.executionTime,
      actualRows: executionStats?.actualRows,
    });

    this.logger.debug("Query analyzed", {
      type: analysis.type,
      complexity: analysis.estimatedComplexity,
      tables: analysis.tables.length,
      recommendations: analysis.recommendedIndexes.length,
    });

    return analysis;
  }

  optimizeQuery(query: string): string {
    let optimizedQuery = query;

    // Apply various optimizations
    optimizedQuery = this.optimizeJoins(optimizedQuery);
    optimizedQuery = this.optimizeConditions(optimizedQuery);
    optimizedQuery = this.optimizeAggregations(optimizedQuery);
    optimizedQuery = this.addQueryHints(optimizedQuery);

    // Store the optimized version
    const queryHash = this.hashQuery(query);
    const existing = this.queryHistory.get(queryHash);
    if (existing) {
      existing.optimizedQuery = optimizedQuery;
    }

    if (optimizedQuery !== query) {
      this.logger.info("Query optimized", {
        originalLength: query.length,
        optimizedLength: optimizedQuery.length,
      });
    }

    return optimizedQuery;
  }

  getQueryStats(query: string): QueryPlan | null {
    const queryHash = this.hashQuery(query);
    return this.queryHistory.get(queryHash) || null;
  }

  getIndexRecommendations(table?: string): IndexRecommendation[] {
    if (table) {
      return this.indexRecommendations.get(table) || [];
    }

    // Return all recommendations
    const allRecommendations: IndexRecommendation[] = [];
    for (const recommendations of this.indexRecommendations.values()) {
      allRecommendations.push(...recommendations);
    }
    return allRecommendations;
  }

  private determineQueryType(query: string): QueryAnalysis["type"] {
    const lowerQuery = query.toLowerCase().trim();

    if (lowerQuery.startsWith("select")) return "select";
    if (lowerQuery.startsWith("insert")) return "insert";
    if (lowerQuery.startsWith("update")) return "update";
    if (lowerQuery.startsWith("delete")) return "delete";

    return "complex";
  }

  private extractTables(query: string): string[] {
    // Simple regex-based table extraction
    const tableRegex = /\bfrom\s+(\w+)|join\s+(\w+)/gi;
    const tables: string[] = [];
    let match;

    while ((match = tableRegex.exec(query)) !== null) {
      const table = match[1] || match[2];
      if (table && !tables.includes(table)) {
        tables.push(table);
      }
    }

    return tables;
  }

  private extractConditions(query: string): QueryCondition[] {
    // Extract WHERE conditions
    const whereMatch = query.match(
      /where\s+(.+?)(?:group by|order by|limit|$)/i
    );
    if (!whereMatch) return [];

    const whereClause = whereMatch[1];
    const conditions: QueryCondition[] = [];

    // Simple condition parsing (column = value, column > value, etc.)
    const conditionRegex =
      /(\w+)\s*([=<>!]+(?:\s*(?:like|ilike|in|between))?)\s*([^,\s]+(?:\s+and\s+\w+\s*[=<>!]+\s*[^,\s]+)*)/gi;
    let match;

    while ((match = conditionRegex.exec(whereClause)) !== null) {
      conditions.push({
        column: match[1],
        operator: match[2],
        value: match[3],
        selectivity: this.estimateSelectivity(match[2], match[3]),
      });
    }

    return conditions;
  }

  private extractJoins(query: string): JoinInfo[] {
    const joins: JoinInfo[] = [];
    const joinRegex =
      /(inner\s+join|left\s+join|right\s+join|full\s+join|join)\s+(\w+)\s+on\s+(.+?)(?=join|$)/gi;
    let match;

    while ((match = joinRegex.exec(query)) !== null) {
      const joinType = match[1].toLowerCase().replace(/\s+join$/, "");
      const _table = match[2];
      const condition = match[3];

      // Extract table names from condition
      const tableRegex = /\b(\w+)\.\w+\s*=\s*\w+\.(\w+)\b/;
      const tableMatch = condition.match(tableRegex);

      if (tableMatch) {
        joins.push({
          table1: tableMatch[1],
          table2: tableMatch[2],
          type: joinType as any,
          condition,
        });
      }
    }

    return joins;
  }

  private extractAggregations(query: string): string[] {
    const aggregations: string[] = [];
    const aggRegex = /\b(count|sum|avg|min|max|group_concat)\s*\(/gi;
    let match;

    while ((match = aggRegex.exec(query)) !== null) {
      aggregations.push(match[1]);
    }

    return aggregations;
  }

  private extractSorting(query: string): SortInfo[] {
    const sorting: SortInfo[] = [];
    const orderMatch = query.match(/order by\s+(.+?)(?:limit|$)/i);

    if (orderMatch) {
      const orderClause = orderMatch[1];
      const sortRegex = /(\w+)\s*(asc|desc)?/gi;
      let match;

      while ((match = sortRegex.exec(orderClause)) !== null) {
        sorting.push({
          column: match[1],
          direction: (match[2] || "asc") as "asc" | "desc",
          nullable: true, // Assume nullable unless we can determine otherwise
        });
      }
    }

    return sorting;
  }

  private calculateComplexity(analysis: QueryAnalysis): number {
    let complexity = 1;

    // Base complexity by query type
    switch (analysis.type) {
      case "select":
        complexity = 2;
        break;
      case "insert":
        complexity = 1;
        break;
      case "update":
        complexity = 3;
        break;
      case "delete":
        complexity = 3;
        break;
      case "complex":
        complexity = 5;
        break;
    }

    // Add complexity for joins
    complexity += analysis.joins.length * 2;

    // Add complexity for conditions
    complexity += Math.min(analysis.conditions.length, 5);

    // Add complexity for aggregations
    complexity += analysis.aggregations.length;

    // Add complexity for sorting
    complexity += analysis.sorting.length;

    return Math.min(Math.max(complexity, 1), 10);
  }

  private generateIndexRecommendations(
    analysis: QueryAnalysis
  ): IndexRecommendation[] {
    const recommendations: IndexRecommendation[] = [];

    // Recommend indexes for WHERE conditions
    for (const condition of analysis.conditions) {
      if (condition.selectivity < 0.1) {
        // High selectivity = good candidate for index
        recommendations.push({
          table: analysis.tables[0] || "unknown", // Assume first table
          columns: [condition.column],
          type: "btree",
          reason: `High selectivity condition on ${condition.column}`,
          estimatedImprovement: Math.round((1 - condition.selectivity) * 80), // Rough estimate
        });
      }
    }

    // Recommend indexes for JOIN conditions
    for (const join of analysis.joins) {
      recommendations.push({
        table: join.table1,
        columns: [
          join.condition.split(".")[1]?.split("=")[0]?.trim() || "unknown",
        ],
        type: "btree",
        reason: `JOIN condition on ${join.table1}`,
        estimatedImprovement: 60,
      });

      recommendations.push({
        table: join.table2,
        columns: [
          join.condition.split(".")[3]?.split("=")[0]?.trim() || "unknown",
        ],
        type: "btree",
        reason: `JOIN condition on ${join.table2}`,
        estimatedImprovement: 60,
      });
    }

    // Recommend indexes for ORDER BY
    for (const sort of analysis.sorting) {
      recommendations.push({
        table: analysis.tables[0] || "unknown",
        columns: [sort.column],
        type: "btree",
        reason: `ORDER BY on ${sort.column}`,
        estimatedImprovement: 40,
      });
    }

    return recommendations;
  }

  private identifyOptimizationOpportunities(
    analysis: QueryAnalysis
  ): OptimizationOpportunity[] {
    const opportunities: OptimizationOpportunity[] = [];

    // Check for missing indexes
    if (analysis.recommendedIndexes.length > 0) {
      opportunities.push({
        type: "index",
        description: `Create ${analysis.recommendedIndexes.length} recommended indexes`,
        estimatedBenefit:
          analysis.recommendedIndexes.reduce(
            (sum, rec) => sum + rec.estimatedImprovement,
            0
          ) / analysis.recommendedIndexes.length,
        complexity: "medium",
      });
    }

    // Check for complex joins
    if (analysis.joins.length > 3) {
      opportunities.push({
        type: "denormalization",
        description: "Consider denormalizing frequently joined tables",
        estimatedBenefit: 30,
        complexity: "high",
      });
    }

    // Check for expensive aggregations
    if (analysis.aggregations.length > 0 && analysis.conditions.length === 0) {
      opportunities.push({
        type: "caching",
        description: "Cache aggregation results for frequently accessed data",
        estimatedBenefit: 50,
        complexity: "low",
      });
    }

    return opportunities;
  }

  private estimateCost(analysis: QueryAnalysis): number {
    let cost = 1;

    // Base cost by query type
    switch (analysis.type) {
      case "select":
        cost = 10;
        break;
      case "insert":
        cost = 5;
        break;
      case "update":
        cost = 15;
        break;
      case "delete":
        cost = 12;
        break;
      case "complex":
        cost = 50;
        break;
    }

    // Add cost for joins
    cost += analysis.joins.length * 20;

    // Add cost for conditions (more selective = lower cost)
    for (const condition of analysis.conditions) {
      cost += (1 - condition.selectivity) * 10;
    }

    // Add cost for aggregations
    cost += analysis.aggregations.length * 25;

    return cost;
  }

  private estimateSelectivity(operator: string, value: string): number {
    // Rough selectivity estimation
    if (operator.includes("=") && !value.includes("%")) {
      return 0.01; // Very selective
    } else if (operator.includes("like") && value.includes("%")) {
      return 0.3; // Less selective
    } else if (operator.includes(">") || operator.includes("<")) {
      return 0.5; // Range queries
    }

    return 0.1; // Default
  }

  private optimizeJoins(query: string): string {
    // Simple optimization: reorder joins for better performance
    // In a real implementation, this would use join ordering algorithms
    return query;
  }

  private optimizeConditions(query: string): string {
    // Optimize WHERE conditions
    return query;
  }

  private optimizeAggregations(query: string): string {
    // Optimize aggregation queries
    return query;
  }

  private addQueryHints(query: string): string {
    // Add database-specific query hints
    return query;
  }

  private hashQuery(query: string): string {
    // Simple hash for query identification
    let hash = 0;
    for (let i = 0; i < query.length; i++) {
      const char = query.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return hash.toString();
  }
}
