export type DatabaseEngine = "postgresql" | "mysql" | "sqlite";

export interface DatabaseConnection {
  id: string;
  name: string;
  engine: DatabaseEngine;
  host: string;
  port: number;
  database: string;
  schema?: string;
  status: "connected" | "disconnected" | "error";
  last_connected?: string;
  error?: string;
}

export interface DatabaseTable {
  name: string;
  schema: string;
  type: "table" | "view" | "materialized_view";
  row_count?: number;
  size_bytes?: number;
  created_at?: string;
  updated_at?: string;
  description?: string;
  columns: DatabaseColumn[];
  indexes: DatabaseIndex[];
  constraints: DatabaseConstraint[];
}

export interface DatabaseColumn {
  name: string;
  type: string;
  nullable: boolean;
  default_value?: string;
  primary_key: boolean;
  foreign_key?: {
    table: string;
    column: string;
  };
  description?: string;
  vector_dimension?: number; // For vector columns
}

export interface DatabaseIndex {
  name: string;
  type: "btree" | "hash" | "gist" | "gin" | "spgist" | "brin" | "bitmap";
  columns: string[];
  unique: boolean;
  partial?: string; // Partial index condition
}

export interface DatabaseConstraint {
  name: string;
  type: "primary_key" | "foreign_key" | "unique" | "check" | "not_null";
  columns: string[];
  referenced_table?: string;
  referenced_columns?: string[];
  check_expression?: string;
}

export interface QueryResult<T = any> {
  columns: QueryColumn[];
  rows: T[];
  row_count: number;
  execution_time_ms: number;
  query: string;
  warnings?: string[];
  errors?: string[];
}

export interface QueryColumn {
  name: string;
  type: string;
  nullable: boolean;
  description?: string;
}

export interface VectorSearchResult {
  id: string;
  vector: number[];
  similarity: number;
  metadata?: { [key: string]: any };
  content?: string;
}

export interface VectorSearchQuery {
  vector: number[];
  table: string;
  column: string;
  limit: number;
  threshold?: number; // Minimum similarity threshold
  metadata_filter?: { [key: string]: any };
}

export interface DataQualityMetric {
  name: string;
  value: number;
  unit: string;
  status: "good" | "warning" | "error";
  description: string;
  trend?: "up" | "down" | "stable";
  change_percent?: number;
}

export interface TableMetrics {
  table_name: string;
  row_count: number;
  size_bytes: number;
  last_vacuum?: string;
  dead_tuples?: number;
  data_quality: DataQualityMetric[];
  performance: {
    avg_query_time_ms: number;
    total_queries: number;
    slow_queries: number;
  };
}

export interface DatabaseMetrics {
  total_tables: number;
  total_rows: number;
  total_size_bytes: number;
  connections_active: number;
  connections_idle: number;
  cache_hit_ratio: number;
  table_metrics: TableMetrics[];
  overall_quality_score: number;
}

// API Response Types
export interface GetDatabaseTablesResponse {
  tables: DatabaseTable[];
  total_count: number;
}

export interface GetTableSchemaResponse {
  table: DatabaseTable;
}

export interface ExecuteQueryResponse {
  result: QueryResult;
  execution_stats: {
    rows_affected?: number;
    execution_time_ms: number;
    query_plan?: any;
  };
}

export interface VectorSearchResponse {
  results: VectorSearchResult[];
  total_count: number;
  search_time_ms: number;
  query: VectorSearchQuery;
}

export interface GetDatabaseMetricsResponse {
  metrics: DatabaseMetrics;
  last_updated: string;
}

// API Request Types
export interface QueryRequest {
  sql: string;
  parameters?: any[];
  timeout_ms?: number;
  read_only: boolean;
}

export interface TableQueryRequest {
  table: string;
  columns?: string[];
  where?: { [key: string]: any };
  order_by?: string;
  limit?: number;
  offset?: number;
}

// Component Props Types
export interface DatabaseExplorerProps {
  connections?: DatabaseConnection[];
  selectedConnectionId?: string;
  onConnectionSelect?: (connectionId: string) => void;
  onConnectionCreate?: () => void;
}

export interface TableViewerProps {
  table: DatabaseTable;
  data?: QueryResult;
  isLoading?: boolean;
  error?: string | null;
  onQuery?: (query: TableQueryRequest) => void;
  onExport?: (format: "csv" | "json" | "sql") => void;
}

export interface VectorSearchPanelProps {
  tables?: DatabaseTable[];
  onSearch?: (query: VectorSearchQuery) => void;
  results?: VectorSearchResult[];
  isSearching?: boolean;
  error?: string | null;
}

export interface QueryBuilderProps {
  tables: DatabaseTable[];
  onQueryExecute: (query: QueryRequest) => void;
  onQuerySave?: (name: string, query: QueryRequest) => void;
  savedQueries?: SavedQuery[];
  isExecuting?: boolean;
  error?: string | null;
}

export interface SavedQuery {
  id: string;
  name: string;
  query: QueryRequest;
  created_at: string;
  last_used?: string;
  use_count: number;
}

export interface DataQualityDashboardProps {
  metrics?: DatabaseMetrics;
  isLoading?: boolean;
  error?: string | null;
  onRefresh?: () => void;
}

// Error Types
export interface DatabaseError {
  code:
    | "connection_failed"
    | "query_failed"
    | "permission_denied"
    | "timeout"
    | "invalid_query"
    | "table_not_found"
    | "server_error";
  message: string;
  details?: string;
  sql_state?: string;
  position?: number;
}

// Filter and Search Types
export interface TableFilters {
  schema?: string[];
  type?: DatabaseTable["type"][];
  has_vector_columns?: boolean;
  min_rows?: number;
  max_rows?: number;
}

export interface ColumnFilters {
  type?: string[];
  nullable?: boolean;
  primary_key?: boolean;
  foreign_key?: boolean;
  has_vector?: boolean;
}

// Utility Types
export type DatabaseAction =
  | "SELECT"
  | "INSERT"
  | "UPDATE"
  | "DELETE"
  | "CREATE"
  | "DROP"
  | "ALTER";

export interface QueryHistoryEntry {
  id: string;
  query: string;
  timestamp: string;
  execution_time_ms?: number;
  row_count?: number;
  success: boolean;
  error?: string;
}
