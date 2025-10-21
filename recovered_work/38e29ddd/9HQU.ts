import { apiClient } from "@/lib/api-client";
import {
  DatabaseConnection,
  GetDatabaseTablesResponse,
  GetTableSchemaResponse,
  ExecuteQueryResponse,
  VectorSearchResponse,
  GetDatabaseMetricsResponse,
  QueryRequest,
  TableQueryRequest,
  VectorSearchQuery,
  DatabaseError,
} from "@/types/database";

// Database API Error Class
export class DatabaseApiError extends Error {
  constructor(
    public code: DatabaseError["code"],
    message: string,
    public retryable: boolean = false,
    public details?: string
  ) {
    super(message);
    this.name = "DatabaseApiError";
  }
}

class DatabaseApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl ?? "/api/database";
  }

  /**
   * Fetches a list of available database connections.
   * @returns A promise that resolves to an array of DatabaseConnection objects.
   */
  async getConnections(): Promise<DatabaseConnection[]> {
    try {
      const response = await apiClient.request<{ connections: DatabaseConnection[] }>(
        `${this.baseUrl}/connections`
      );

      return response.connections || [];
    } catch (error) {
      console.error("Failed to get database connections:", error);
      throw new DatabaseApiError(
        "server_error",
        "Failed to retrieve database connections",
        true,
        error instanceof Error ? error.message : "Unknown error"
      );
    }
  }
  }

  /**
   * Fetches a list of tables from the specified database connection.
   * @param connectionId - The ID of the database connection.
   * @returns A promise that resolves to a GetDatabaseTablesResponse.
   */
  async getTables(connectionId?: string): Promise<GetDatabaseTablesResponse> {
    console.warn(
      "getTables using mock implementation - V3 database tables API not available"
    );
    // TODO: Milestone 4 - Database Tables API Implementation
    // - [ ] Implement V3 GET /api/v1/database/tables endpoint
    // - [ ] Add table metadata (row count, size, indexes)
    // - [ ] Implement table filtering and sorting
    // - [ ] Add table schema information

    // Mock implementation for development
    const mockTables = [
      {
        name: "execution_artifacts",
        schema: "public",
        connection_id: connectionId ?? "main_db",
        row_count: 1250,
        size_bytes: 52428800, // 50MB
        created_at: "2025-01-15T10:00:00Z",
        updated_at: "2025-01-22T14:30:00Z",
        indexes: [
          {
            name: "idx_execution_artifacts_task_id",
            type: "btree" as const,
            columns: ["task_id"],
            unique: false,
          },
          {
            name: "idx_execution_artifacts_created_at",
            type: "btree" as const,
            columns: ["created_at"],
            unique: false,
          },
        ],
        primary_key: "id",
        columns: [
          { name: "id", type: "uuid", nullable: false, primary_key: true },
          {
            name: "task_id",
            type: "uuid",
            nullable: false,
            primary_key: false,
          },
          {
            name: "artifact_type",
            type: "varchar(50)",
            nullable: false,
            primary_key: false,
          },
          {
            name: "size_bytes",
            type: "bigint",
            nullable: false,
            primary_key: false,
          },
        ],
      },
      {
        name: "artifact_versions",
        schema: "public",
        connection_id: connectionId ?? "main_db",
        row_count: 340,
        size_bytes: 15728640, // 15MB
        created_at: "2025-01-15T10:00:00Z",
        updated_at: "2025-01-22T11:15:00Z",
        indexes: [
          {
            name: "idx_artifact_versions_task_id",
            type: "btree" as const,
            columns: ["task_id"],
            unique: false,
          },
          {
            name: "idx_artifact_versions_version_number",
            type: "btree" as const,
            columns: ["version_number"],
            unique: false,
          },
        ],
        primary_key: "id",
        columns: [
          { name: "id", type: "uuid", nullable: false, primary_key: true },
          {
            name: "task_id",
            type: "uuid",
            nullable: false,
            primary_key: false,
          },
          {
            name: "version_number",
            type: "integer",
            nullable: false,
            primary_key: false,
          },
          {
            name: "change_summary",
            type: "text",
            nullable: true,
            primary_key: false,
          },
        ],
      },
      {
        name: "tasks",
        schema: "public",
        connection_id: connectionId ?? "main_db",
        row_count: 89,
        size_bytes: 2097152, // 2MB
        created_at: "2025-01-10T09:00:00Z",
        updated_at: "2025-01-22T16:45:00Z",
        indexes: [
          {
            name: "idx_tasks_status",
            type: "btree" as const,
            columns: ["status"],
            unique: false,
          },
          {
            name: "idx_tasks_created_at",
            type: "btree" as const,
            columns: ["created_at"],
            unique: false,
          },
        ],
        primary_key: "id",
        columns: [
          { name: "id", type: "uuid", nullable: false, primary_key: true },
          {
            name: "title",
            type: "varchar(255)",
            nullable: false,
            primary_key: false,
          },
          {
            name: "status",
            type: "varchar(50)",
            nullable: false,
            primary_key: false,
          },
          {
            name: "created_at",
            type: "timestamptz",
            nullable: false,
            primary_key: false,
          },
        ],
      },
    ];

    return {
      tables: mockTables.map((table) => ({
        ...table,
        type: "table" as const,
        constraints: [],
      })),
      total_count: mockTables.length,
    };
  }

  /**
   * Fetches detailed schema information for a specific table.
   * @param tableName - The name of the table.
   * @param schema - The schema name (optional).
   * @param connectionId - The ID of the database connection (optional).
   * @returns A promise that resolves to a GetTableSchemaResponse.
   */
  async getTableSchema(
    tableName: string,
    schema?: string,
    connectionId?: string
  ): Promise<GetTableSchemaResponse> {
    console.warn(
      "getTableSchema not implemented - requires V3 table schema API"
    );
    // TODO: Milestone 4 - Table Schema API Implementation
    // - [ ] Implement V3 GET /api/v1/database/tables/:table/schema endpoint
    // - [ ] Add column details (types, constraints, indexes)
    // - [ ] Include foreign key relationships
    // - [ ] Add table statistics and metadata
    try {
      const params = new URLSearchParams();
      if (schema) params.append("schema", schema);
      if (connectionId) params.append("connection_id", connectionId);

      const queryString = params.toString();
      const url = `/database/tables/${encodeURIComponent(tableName)}/schema${
        queryString ? `?${queryString}` : ""
      }`;

      const response = await apiClient.request<GetTableSchemaResponse>(url);
      return response;
    } catch (error) {
      console.error("Failed to get table schema:", error);
      if (error instanceof Error && error.message.includes("404")) {
        throw new DatabaseApiError("table_not_found", "Table not found", false);
      }
      throw new DatabaseApiError(
        "server_error",
        "Failed to retrieve table schema",
        true
      );
    }
  }

  /**
   * Executes a SQL query against the database.
   * @param queryRequest - The query request containing SQL and parameters.
   * @param connectionId - The ID of the database connection (optional).
   * @returns A promise that resolves to an ExecuteQueryResponse.
   */
  async executeQuery(
    queryRequest: QueryRequest,
    connectionId?: string
  ): Promise<ExecuteQueryResponse> {
    console.warn(
      "executeQuery not implemented - requires V3 query execution API"
    );
    // TODO: Milestone 4 - Query Execution API Implementation
    // - [ ] Implement V3 POST /api/v1/database/query endpoint
    // - [ ] Add SQL injection protection
    // - [ ] Implement query timeout and resource limits
    // - [ ] Add query result pagination
    // - [ ] Include execution statistics and query plans
    // Mock implementation for development
    // Simulate query execution time
    await new Promise((resolve) =>
      setTimeout(resolve, 200 + Math.random() * 800)
    );

    // Mock response based on query type
    const sql = queryRequest.sql.toLowerCase();

    if (sql.includes("select")) {
      // Mock SELECT query results
      const mockRows = [
        {
          id: "550e8400-e29b-41d4-a716-446655440000",
          task_id: "660e8400-e29b-41d4-a716-446655440001",
          artifact_type: "unit_tests",
          size_bytes: 245760,
        },
        {
          id: "550e8400-e29b-41d4-a716-446655440001",
          task_id: "660e8400-e29b-41d4-a716-446655440001",
          artifact_type: "linting",
          size_bytes: 15360,
        },
      ];

      return {
        result: {
          columns: [
            { name: "id", type: "uuid", nullable: false },
            { name: "task_id", type: "uuid", nullable: false },
            { name: "artifact_type", type: "varchar", nullable: false },
            { name: "size_bytes", type: "bigint", nullable: true },
          ],
          rows: mockRows,
          row_count: mockRows.length,
          execution_time_ms: 45.2,
          query: "SELECT * FROM task_artifacts LIMIT 10;",
        },
        execution_stats: {
          execution_time_ms: 45.2,
          query_plan: {
            "Node Type": "Seq Scan",
            "Relation Name": "execution_artifacts",
            Alias: "execution_artifacts",
            "Startup Cost": 0.0,
            "Total Cost": 25.0,
            "Plan Rows": 1000,
            "Plan Width": 128,
          },
        },
      };
    } else if (
      sql.includes("insert") ||
      sql.includes("update") ||
      sql.includes("delete")
    ) {
      // Mock write operation
      return {
        result: {
          columns: [{ name: "count", type: "bigint", nullable: false }],
          rows: [],
          row_count: 0,
          execution_time_ms: 12.5,
          query: "SELECT COUNT(*) FROM users WHERE active = true;",
        },
        execution_stats: {
          rows_affected: 1,
          execution_time_ms: 12.5,
        },
      };
    } else {
      // Mock DDL or other operations
      const query = "CREATE INDEX idx_example ON example_table(column);";
      return {
        result: {
          columns: [{ name: "count", type: "bigint", nullable: false }],
          rows: [],
          row_count: 0,
          execution_time_ms: 8.3,
          query,
        },
        execution_stats: {
          execution_time_ms: 8.3,
        },
      };
    }
  }

  /**
   * Executes a table query with automatic SQL generation.
   * @param tableQuery - The table query request.
   * @param connectionId - The ID of the database connection (optional).
   * @returns A promise that resolves to an ExecuteQueryResponse.
   */
  async queryTable(
    tableQuery: TableQueryRequest,
    _connectionId?: string // eslint-disable-line @typescript-eslint/no-unused-vars
  ): Promise<ExecuteQueryResponse> {
    console.warn("queryTable not implemented - requires V3 table query API");
    // TODO: Milestone 4 - Table Query API Implementation
    // - [ ] Implement V3 POST /api/v1/database/tables/:table/query endpoint
    // - [ ] Add automatic SQL generation from query parameters
    // - [ ] Implement column selection and filtering
    // - [ ] Add pagination and sorting
    // - [ ] Enforce safety constraints (row limits, timeouts)
    try {
      const payload = {
        ...tableQuery,
        connection_id: _connectionId,
      };

      const response = await apiClient.request<ExecuteQueryResponse>(
        `/database/tables/${encodeURIComponent(tableQuery.table)}/query`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        }
      );

      return response;
    } catch (error) {
      console.error("Failed to query table:", error);
      throw new DatabaseApiError(
        "query_failed",
        "Failed to query database table",
        true
      );
    }
  }

  /**
   * Performs a vector similarity search.
   * @param searchQuery - The vector search query parameters.
   * @param connectionId - The ID of the database connection (optional).
   * @returns A promise that resolves to a VectorSearchResponse.
   */
  async vectorSearch(
    searchQuery: VectorSearchQuery,
    _connectionId?: string // eslint-disable-line @typescript-eslint/no-unused-vars
  ): Promise<VectorSearchResponse> {
    console.warn(
      "vectorSearch not implemented - requires V3 vector search API"
    );
    // TODO: Milestone 4 - Vector Search API Implementation
    // - [ ] Implement V3 POST /api/v1/database/vector/search endpoint
    // - [ ] Add vector similarity calculations (cosine, euclidean, etc.)
    // - [ ] Implement metadata filtering
    // - [ ] Add search result ranking and scoring
    // - [ ] Include performance optimizations for high-dimensional vectors
    // Mock implementation for development
    // Simulate vector search time
    await new Promise((resolve) =>
      setTimeout(resolve, 300 + Math.random() * 1200)
    );

    // Mock vector search results
    const mockResults = [
      {
        id: "vec_001",
        vector: [0.1, 0.2, 0.3, 0.4, 0.5], // Simplified for display
        metadata: {
          task_id: "550e8400-e29b-41d4-a716-446655440000",
          content: "User authentication implementation with JWT tokens",
          type: "task_embedding",
        },
        score: 0.92,
        distance: 0.08,
      },
      {
        id: "vec_002",
        vector: [0.15, 0.25, 0.35, 0.45, 0.55],
        metadata: {
          task_id: "550e8400-e29b-41d4-a716-446655440001",
          content: "Database schema design for artifact storage",
          type: "task_embedding",
        },
        score: 0.87,
        distance: 0.13,
      },
      {
        id: "vec_003",
        vector: [0.05, 0.15, 0.25, 0.35, 0.45],
        metadata: {
          task_id: "550e8400-e29b-41d4-a716-446655440002",
          content: "API endpoint development and testing",
          type: "task_embedding",
        },
        score: 0.81,
        distance: 0.19,
      },
    ];

    return {
      results: mockResults.map((result) => ({
        ...result,
        similarity: result.score ?? result.distance ?? 0.85,
      })),
      total_count: mockResults.length,
      search_time_ms: 145.7,
      query: searchQuery,
    };
  }

  /**
   * Fetches database metrics and statistics.
   * @param connectionId - The ID of the database connection (optional).
   * @returns A promise that resolves to a GetDatabaseMetricsResponse.
   */
  async getDatabaseMetrics(
    _connectionId?: string // eslint-disable-line @typescript-eslint/no-unused-vars
  ): Promise<GetDatabaseMetricsResponse> {
    console.warn(
      "getDatabaseMetrics not implemented - requires V3 database metrics API"
    );
    // TODO: Milestone 4 - Database Metrics API Implementation
    // - [ ] Implement V3 GET /api/v1/database/metrics endpoint
    // - [ ] Add table statistics (row counts, sizes, dead tuples)
    // - [ ] Include connection pool metrics
    // - [ ] Add query performance statistics
    // - [ ] Implement data quality scoring
    // Mock implementation for development
    return {
      metrics: {
        total_tables: 12,
        total_rows: 45230,
        total_size_bytes: 2147483648, // 2GB
        connections_active: 5,
        connections_idle: 10,
        cache_hit_ratio: 0.87,
        table_metrics: [],
        overall_quality_score: 85.5,
      },
      last_updated: new Date().toISOString(),
    };
  }

  /**
   * Exports table data in various formats.
   * @param tableName - The name of the table to export.
   * @param format - The export format (csv, json, sql).
   * @param query - Optional query parameters to filter the data.
   * @param connectionId - The ID of the database connection (optional).
   * @returns A promise that resolves to a Blob or string containing the exported data.
   */
  async exportTable(
    tableName: string,
    format: "csv" | "json" | "sql",
    _query?: Partial<TableQueryRequest>, // eslint-disable-line @typescript-eslint/no-unused-vars
    _connectionId?: string // eslint-disable-line @typescript-eslint/no-unused-vars
  ): Promise<Blob | string> {
    console.warn(
      "exportTable using mock implementation - V3 data export API not available"
    );
    // TODO: Milestone 4 - Data Export API Implementation
    // - [ ] Implement V3 GET /api/v1/database/tables/:table/export endpoint
    // - [ ] Add CSV, JSON, and SQL export formats
    // - [ ] Implement streaming for large datasets
    // - [ ] Add export size limits and rate limiting
    // - [ ] Include column headers and metadata

    // Mock implementation for development
    // Simulate export processing time
    await new Promise((resolve) =>
      setTimeout(resolve, 500 + Math.random() * 1500)
    );

    const mockData = [
      {
        id: "550e8400-e29b-41d4-a716-446655440000",
        task_id: "660e8400-e29b-41d4-a716-446655440001",
        artifact_type: "unit_tests",
        size_bytes: 245760,
        created_at: "2025-01-20T14:30:00Z",
      },
      {
        id: "550e8400-e29b-41d4-a716-446655440001",
        task_id: "660e8400-e29b-41d4-a716-446655440001",
        artifact_type: "linting",
        size_bytes: 15360,
        created_at: "2025-01-20T14:35:00Z",
      },
    ];

    switch (format) {
      case "csv": {
        const csvHeader = "id,task_id,artifact_type,size_bytes,created_at\n";
        const csvRows = mockData
          .map(
            (row) =>
              `${row.id},${row.task_id},${row.artifact_type},${row.size_bytes},${row.created_at}`
          )
          .join("\n");
        return csvHeader + csvRows;
      }

      case "json":
        return JSON.stringify(mockData, null, 2);

      case "sql": {
        const sqlInserts = mockData
          .map(
            (row) =>
              `INSERT INTO ${tableName} (id, task_id, artifact_type, size_bytes, created_at) VALUES ('${row.id}', '${row.task_id}', '${row.artifact_type}', ${row.size_bytes}, '${row.created_at}');`
          )
          .join("\n");
        return `-- Export of ${tableName}\n${sqlInserts}`;
      }

      default:
        throw new DatabaseApiError(
          "invalid_query",
          `Export format '${format}' is not supported`,
          false
        );
    }
  }
}

// Default database API client instance
export const databaseApiClient = new DatabaseApiClient();
