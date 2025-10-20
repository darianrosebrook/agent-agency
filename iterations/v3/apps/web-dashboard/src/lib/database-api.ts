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

  /**
   * Fetches a list of tables from the specified database connection.
   * @param connectionId - The ID of the database connection.
   * @returns A promise that resolves to a GetDatabaseTablesResponse.
   */
  async getTables(connectionId?: string): Promise<GetDatabaseTablesResponse> {
    try {
      if (!connectionId) {
        throw new DatabaseApiError(
          "validation_error",
          "connectionId is required for table listing",
          false
        );
      }

      const response = await apiClient.request<GetDatabaseTablesResponse>(
        `${this.baseUrl}/tables?connection_id=${encodeURIComponent(connectionId)}`
      );

      return response;
    } catch (error) {
      console.error("Failed to get database tables:", error);
      throw new DatabaseApiError(
        "server_error",
        "Failed to retrieve database tables",
        true,
        error instanceof Error ? error.message : "Unknown error"
      );
    }
  }

  /**
   * Fetches detailed schema information for a specific table.
   * @param tableName - The name of the table.
   * @param connectionId - The ID of the database connection.
   * @param schema - The schema name (optional).
   * @returns A promise that resolves to a GetTableSchemaResponse.
   */
  async getTableSchema(
    tableName: string,
    connectionId: string,
    schema?: string
  ): Promise<GetTableSchemaResponse> {
    try {
      const params = new URLSearchParams();
      params.append("connection_id", connectionId);
      if (schema) params.append("schema", schema);

      const response = await apiClient.request<GetTableSchemaResponse>(
        `${this.baseUrl}/tables/${encodeURIComponent(tableName)}/schema?${params}`
      );

      return response;
    } catch (error) {
      console.error("Failed to get table schema:", error);
      throw new DatabaseApiError(
        "server_error",
        "Failed to retrieve table schema",
        true,
        error instanceof Error ? error.message : "Unknown error"
      );
    }
  }

  /**
   * Executes a SQL query safely against the specified database connection.
   * @param query - The query request containing connection ID, SQL, and parameters.
   * @returns A promise that resolves to an ExecuteQueryResponse.
   */
  async executeQuery(query: QueryRequest): Promise<ExecuteQueryResponse> {
    try {
      const response = await apiClient.request<ExecuteQueryResponse>(
        `${this.baseUrl}/query`,
        {
          method: "POST",
          body: JSON.stringify(query),
        }
      );

      return response;
    } catch (error) {
      console.error("Failed to execute query:", error);
      throw new DatabaseApiError(
        "query_failed",
        "Failed to execute database query",
        true,
        error instanceof Error ? error.message : "Unknown error"
      );
    }
  }

  async vectorSearch(searchQuery: VectorSearchQuery): Promise<VectorSearchResponse> {
    try {
      const response = await apiClient.request<VectorSearchResponse>(
        `${this.baseUrl}/vector-search`,
        {
          method: "POST",
          body: JSON.stringify(searchQuery),
        }
      );

      return response;
    } catch (error) {
      console.error("Failed to perform vector search:", error);
      throw new DatabaseApiError(
        "search_failed",
        "Failed to perform vector similarity search",
        true,
        error instanceof Error ? error.message : "Unknown error"
      );
    }
  }
}
