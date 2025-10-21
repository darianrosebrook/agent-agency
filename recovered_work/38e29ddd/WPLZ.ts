import { apiClient } from "@/lib/api-client";
import {
  DatabaseConnection,
  DatabaseTable,
  QueryResult,
  VectorSearchResult,
  VectorSearchQuery,
  DatabaseMetrics,
  GetDatabaseTablesResponse,
  GetTableSchemaResponse,
  ExecuteQueryResponse,
  VectorSearchResponse,
  GetDatabaseMetricsResponse,
  QueryRequest,
  TableQueryRequest,
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
    this.baseUrl = baseUrl ?? "/api/proxy";
  }

  /**
   * Fetches a list of available database connections.
   * @returns A promise that resolves to an array of DatabaseConnection objects.
   */
  async getConnections(): Promise<DatabaseConnection[]> {
    console.warn(
      "getConnections using mock implementation - V3 database connection management API not available"
    );
    // TODO: Milestone 4 - Database Connection Management
    // - [ ] Implement V3 GET /api/v1/database/connections endpoint
    // - [ ] Add connection status monitoring
    // - [ ] Implement connection pool management
    // - [ ] Add connection health checks

    // Mock implementation for development
    return [
      {
        id: "main_db",
        name: "Main Database",
        host: "localhost",
        port: 5432,
        database: "agent_agency",
        status: "connected",
        connectionPoolSize: 10,
        activeConnections: 3,
        lastHealthCheck: new Date().toISOString(),
      },
      {
        id: "vector_db",
        name: "Vector Database",
        host: "localhost",
        port: 5432,
        database: "agent_agency_vectors",
        status: "connected",
        connectionPoolSize: 5,
        activeConnections: 1,
        lastHealthCheck: new Date().toISOString(),
      },
    ];
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
        connection_id: connectionId || "main_db",
        row_count: 1250,
        size_bytes: 52428800, // 50MB
        created_at: "2025-01-15T10:00:00Z",
        updated_at: "2025-01-22T14:30:00Z",
        indexes: ["idx_execution_artifacts_task_id", "idx_execution_artifacts_created_at"],
        primary_key: "id",
        columns: [
          { name: "id", type: "uuid", nullable: false, is_primary: true },
          { name: "task_id", type: "uuid", nullable: false, is_primary: false },
          { name: "artifact_type", type: "varchar(50)", nullable: false, is_primary: false },
          { name: "size_bytes", type: "bigint", nullable: false, is_primary: false },
        ],
      },
      {
        name: "artifact_versions",
        schema: "public",
        connection_id: connectionId || "main_db",
        row_count: 340,
        size_bytes: 15728640, // 15MB
        created_at: "2025-01-15T10:00:00Z",
        updated_at: "2025-01-22T11:15:00Z",
        indexes: ["idx_artifact_versions_task_id", "idx_artifact_versions_version_number"],
        primary_key: "id",
        columns: [
          { name: "id", type: "uuid", nullable: false, is_primary: true },
          { name: "task_id", type: "uuid", nullable: false, is_primary: false },
          { name: "version_number", type: "integer", nullable: false, is_primary: false },
          { name: "change_summary", type: "text", nullable: true, is_primary: false },
        ],
      },
      {
        name: "tasks",
        schema: "public",
        connection_id: connectionId || "main_db",
        row_count: 89,
        size_bytes: 2097152, // 2MB
        created_at: "2025-01-10T09:00:00Z",
        updated_at: "2025-01-22T16:45:00Z",
        indexes: ["idx_tasks_status", "idx_tasks_created_at"],
        primary_key: "id",
        columns: [
          { name: "id", type: "uuid", nullable: false, is_primary: true },
          { name: "title", type: "varchar(255)", nullable: false, is_primary: false },
          { name: "status", type: "varchar(50)", nullable: false, is_primary: false },
          { name: "created_at", type: "timestamptz", nullable: false, is_primary: false },
        ],
      },
    ];

    return {
      tables: mockTables,
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
    try {
      const payload = {
        ...queryRequest,
        connection_id: connectionId,
      };

      const response = await apiClient.request<ExecuteQueryResponse>(
        "/database/query",
        {
          method: "POST",
          body: JSON.stringify(payload),
        }
      );

      return response;
    } catch (error) {
      console.error("Failed to execute query:", error);
      throw new DatabaseApiError(
        "query_failed",
        "Failed to execute database query",
        true
      );
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
    connectionId?: string
  ): Promise<ExecuteQueryResponse> {
    console.warn(
      "queryTable not implemented - requires V3 table query API"
    );
    // TODO: Milestone 4 - Table Query API Implementation
    // - [ ] Implement V3 POST /api/v1/database/tables/:table/query endpoint
    // - [ ] Add automatic SQL generation from query parameters
    // - [ ] Implement column selection and filtering
    // - [ ] Add pagination and sorting
    // - [ ] Enforce safety constraints (row limits, timeouts)
    try {
      const payload = {
        ...tableQuery,
        connection_id: connectionId,
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
    connectionId?: string
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
    try {
      const payload = {
        ...searchQuery,
        connection_id: connectionId,
      };

      const response = await apiClient.request<VectorSearchResponse>(
        "/database/vector/search",
        {
          method: "POST",
          body: JSON.stringify(payload),
        }
      );

      return response;
    } catch (error) {
      console.error("Failed to perform vector search:", error);
      throw new DatabaseApiError(
        "query_failed",
        "Failed to perform vector search",
        true
      );
    }
  }

  /**
   * Fetches database metrics and statistics.
   * @param connectionId - The ID of the database connection (optional).
   * @returns A promise that resolves to a GetDatabaseMetricsResponse.
   */
  async getDatabaseMetrics(connectionId?: string): Promise<GetDatabaseMetricsResponse> {
    console.warn(
      "getDatabaseMetrics not implemented - requires V3 database metrics API"
    );
    // TODO: Milestone 4 - Database Metrics API Implementation
    // - [ ] Implement V3 GET /api/v1/database/metrics endpoint
    // - [ ] Add table statistics (row counts, sizes, dead tuples)
    // - [ ] Include connection pool metrics
    // - [ ] Add query performance statistics
    // - [ ] Implement data quality scoring
    try {
      const params = connectionId ? `?connection_id=${encodeURIComponent(connectionId)}` : "";
      const response = await apiClient.request<GetDatabaseMetricsResponse>(
        `/database/metrics${params}`
      );
      return response;
    } catch (error) {
      console.error("Failed to get database metrics:", error);
      throw new DatabaseApiError(
        "server_error",
        "Failed to retrieve database metrics",
        true
      );
    }
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
    query?: Partial<TableQueryRequest>,
    connectionId?: string
  ): Promise<Blob | string> {
    console.warn(
      "exportTable not implemented - requires V3 data export API"
    );
    // TODO: Milestone 4 - Data Export API Implementation
    // - [ ] Implement V3 GET /api/v1/database/tables/:table/export endpoint
    // - [ ] Add CSV, JSON, and SQL export formats
    // - [ ] Implement streaming for large datasets
    // - [ ] Add export size limits and rate limiting
    // - [ ] Include column headers and metadata
    throw new DatabaseApiError(
      "server_error",
      "Table export API not yet implemented",
      false
    );
  }
}

// Default database API client instance
export const databaseApiClient = new DatabaseApiClient();
