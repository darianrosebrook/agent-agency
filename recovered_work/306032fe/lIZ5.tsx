"use client";

import React, { useState, useEffect, useCallback } from "react";
import { DatabaseExplorerProps, DatabaseConnection, DatabaseTable, TableFilters } from "@/types/database";
import { databaseApiClient, DatabaseApiError } from "@/lib/database-api";
import TableViewer from "./TableViewer";
import VectorSearchPanel from "./VectorSearchPanel";
import QueryBuilder from "./QueryBuilder";
import DataQualityDashboard from "./DataQualityDashboard";
import styles from "./DatabaseExplorer.module.scss";

interface DatabaseExplorerState {
  connections: DatabaseConnection[];
  selectedConnection: DatabaseConnection | null;
  tables: DatabaseTable[];
  selectedTable: DatabaseTable | null;
  filters: TableFilters;
  activeTab: "tables" | "query" | "search" | "quality";
  isLoading: boolean;
  error: string | null;
}

export default function DatabaseExplorer({
  connections: externalConnections,
  selectedConnectionId,
  onConnectionSelect,
  onConnectionCreate,
}: DatabaseExplorerProps) {
  const [state, setState] = useState<DatabaseExplorerState>({
    connections: externalConnections ?? [],
    selectedConnection: null,
    tables: [],
    selectedTable: null,
    filters: {},
    activeTab: "tables",
    isLoading: !externalConnections,
    error: null,
  });

  // Load connections if not provided externally
  const loadConnections = useCallback(async () => {
    if (externalConnections) return;

    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));
      const connections = await databaseApiClient.getConnections();
      setState((prev) => ({
        ...prev,
        connections,
        isLoading: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof DatabaseApiError
          ? error.message
          : "Failed to load database connections";
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      console.error("Failed to load connections:", error);
    }
  }, [externalConnections]);

  // Load tables for selected connection
  const loadTables = useCallback(async (connectionId?: string) => {
    if (!connectionId && !state.selectedConnection) return;

    const connId = connectionId || state.selectedConnection?.id;
    if (!connId) return;

    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));
      const response = await databaseApiClient.getTables(connId);
      setState((prev) => ({
        ...prev,
        tables: response.tables,
        isLoading: false,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof DatabaseApiError
          ? error.message
          : "Failed to load database tables";
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      console.error("Failed to load tables:", error);
    }
  }, [state.selectedConnection]);

  // Handle connection selection
  const handleConnectionSelect = useCallback((connectionId: string) => {
    const connection = state.connections.find((c) => c.id === connectionId);
    if (!connection) return;

    setState((prev) => ({
      ...prev,
      selectedConnection: connection,
      selectedTable: null,
      tables: [],
    }));

    onConnectionSelect?.(connectionId);
    loadTables(connectionId);
  }, [state.connections, onConnectionSelect, loadTables]);

  // Handle table selection
  const handleTableSelect = useCallback((table: DatabaseTable) => {
    setState((prev) => ({
      ...prev,
      selectedTable: table,
    }));
  }, []);

  // Handle tab changes
  const handleTabChange = useCallback((tab: DatabaseExplorerState["activeTab"]) => {
    setState((prev) => ({
      ...prev,
      activeTab: tab,
    }));
  }, []);

  // Filter tables based on current filters
  const filteredTables = React.useMemo(() => {
    return state.tables.filter((table) => {
      if (state.filters.schema?.length && !state.filters.schema.includes(table.schema)) {
        return false;
      }
      if (state.filters.type?.length && !state.filters.type.includes(table.type)) {
        return false;
      }
      if (state.filters.has_vector_columns !== undefined) {
        const hasVectors = table.columns.some((col) => col.vector_dimension);
        if (state.filters.has_vector_columns !== hasVectors) {
          return false;
        }
      }
      if (state.filters.min_rows !== undefined && (table.row_count ?? 0) < state.filters.min_rows) {
        return false;
      }
      if (state.filters.max_rows !== undefined && (table.row_count ?? 0) > state.filters.max_rows) {
        return false;
      }
      return true;
    });
  }, [state.tables, state.filters]);

  // Initialize connections on mount
  useEffect(() => {
    if (!externalConnections) {
      loadConnections();
    } else {
      setState((prev) => ({
        ...prev,
        connections: externalConnections,
        isLoading: false,
      }));
    }
  }, [externalConnections, loadConnections]);

  // Auto-select connection if specified
  useEffect(() => {
    if (selectedConnectionId && state.connections.length > 0) {
      handleConnectionSelect(selectedConnectionId);
    }
  }, [selectedConnectionId, state.connections, handleConnectionSelect]);

  return (
    <div className={styles.databaseExplorer}>
      <div className={styles.header}>
        <h1>Database Explorer</h1>
        <p className={styles.description}>
          Safely inspect database state for efficiency and robustness research.
          Query tables, run vector searches, and analyze data quality.
        </p>
      </div>

      {/* Connection Selector */}
      <div className={styles.connectionSection}>
        <div className={styles.connectionHeader}>
          <h2>Database Connection</h2>
          {onConnectionCreate && (
            <button
              onClick={onConnectionCreate}
              className={styles.createConnectionButton}
            >
              + New Connection
            </button>
          )}
        </div>

        {state.connections.length === 0 ? (
          <div className={styles.noConnections}>
            <div className={styles.emptyIcon}>🗄️</div>
            <h3>No Database Connections</h3>
            <p>Connect to a database to start exploring data.</p>
            {onConnectionCreate && (
              <button
                onClick={onConnectionCreate}
                className={styles.primaryButton}
              >
                Create Connection
              </button>
            )}
          </div>
        ) : (
          <div className={styles.connectionList}>
            {state.connections.map((connection) => (
              <div
                key={connection.id}
                className={`${styles.connectionCard} ${
                  state.selectedConnection?.id === connection.id ? styles.selected : ""
                }`}
                onClick={() => handleConnectionSelect(connection.id)}
              >
                <div className={styles.connectionInfo}>
                  <h3>{connection.name}</h3>
                  <p className={styles.connectionDetails}>
                    {connection.engine} • {connection.host}:{connection.port} • {connection.database}
                  </p>
                </div>
                <div className={`${styles.connectionStatus} ${styles[connection.status]}`}>
                  <span className={styles.statusDot}></span>
                  {connection.status}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Main Content Area */}
      {state.selectedConnection ? (
        <div className={styles.contentArea}>
          {/* Tab Navigation */}
          <div className={styles.tabNavigation}>
            <button
              className={`${styles.tabButton} ${
                state.activeTab === "tables" ? styles.active : ""
              }`}
              onClick={() => handleTabChange("tables")}
            >
              Tables ({filteredTables.length})
            </button>
            <button
              className={`${styles.tabButton} ${
                state.activeTab === "query" ? styles.active : ""
              }`}
              onClick={() => handleTabChange("query")}
            >
              Query Builder
            </button>
            <button
              className={`${styles.tabButton} ${
                state.activeTab === "search" ? styles.active : ""
              }`}
              onClick={() => handleTabChange("search")}
            >
              Vector Search
            </button>
            <button
              className={`${styles.tabButton} ${
                state.activeTab === "quality" ? styles.active : ""
              }`}
              onClick={() => handleTabChange("quality")}
            >
              Data Quality
            </button>
          </div>

          {/* Tab Content */}
          <div className={styles.tabContent}>
            {state.activeTab === "tables" && (
              <div className={styles.tablesTab}>
                {/* Table Filters */}
                <div className={styles.filtersBar}>
                  <div className={styles.filterGroup}>
                    <label>Schema:</label>
                    <select
                      value={state.filters.schema?.[0] || ""}
                      onChange={(e) =>
                        setState((prev) => ({
                          ...prev,
                          filters: {
                            ...prev.filters,
                            schema: e.target.value ? [e.target.value] : undefined,
                          },
                        }))
                      }
                    >
                      <option value="">All Schemas</option>
                      {Array.from(new Set(state.tables.map((t) => t.schema))).map((schema) => (
                        <option key={schema} value={schema}>
                          {schema}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className={styles.filterGroup}>
                    <label>Type:</label>
                    <select
                      value={state.filters.type?.[0] || ""}
                      onChange={(e) =>
                        setState((prev) => ({
                          ...prev,
                          filters: {
                            ...prev.filters,
                            type: e.target.value ? [e.target.value as any] : undefined,
                          },
                        }))
                      }
                    >
                      <option value="">All Types</option>
                      <option value="table">Tables</option>
                      <option value="view">Views</option>
                      <option value="materialized_view">Materialized Views</option>
                    </select>
                  </div>
                </div>

                {/* Tables List */}
                {state.isLoading ? (
                  <div className={styles.loading}>
                    <div className={styles.spinner}></div>
                    <p>Loading tables...</p>
                  </div>
                ) : state.error ? (
                  <div className={styles.error}>
                    <h3>Failed to load tables</h3>
                    <p>{state.error}</p>
                    <button onClick={() => loadTables()}>Retry</button>
                  </div>
                ) : (
                  <div className={styles.tablesGrid}>
                    {filteredTables.map((table) => (
                      <div
                        key={`${table.schema}.${table.name}`}
                        className={`${styles.tableCard} ${
                          state.selectedTable?.name === table.name &&
                          state.selectedTable?.schema === table.schema
                            ? styles.selected
                            : ""
                        }`}
                        onClick={() => handleTableSelect(table)}
                      >
                        <div className={styles.tableHeader}>
                          <h3 className={styles.tableName}>{table.name}</h3>
                          <span className={`${styles.tableType} ${styles[table.type]}`}>
                            {table.type}
                          </span>
                        </div>
                        <div className={styles.tableMeta}>
                          <span className={styles.schema}>{table.schema}</span>
                          {table.row_count !== undefined && (
                            <span className={styles.rowCount}>
                              {table.row_count.toLocaleString()} rows
                            </span>
                          )}
                          {table.columns.some((col) => col.vector_dimension) && (
                            <span className={styles.vectorBadge}>🔍 Vectors</span>
                          )}
                        </div>
                        {table.description && (
                          <p className={styles.tableDescription}>{table.description}</p>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {state.activeTab === "query" && (
              <QueryBuilder
                tables={state.tables}
                onQueryExecute={(query) => {
                  console.log("Execute query:", query);
                  // TODO: Handle query execution
                }}
                onQuerySave={(name, query) => {
                  console.log("Save query:", name, query);
                  // TODO: Handle query saving
                }}
                isExecuting={false}
                error={null}
              />
            )}

            {state.activeTab === "search" && (
              <VectorSearchPanel
                tables={state.tables}
                onSearch={(query) => {
                  console.log("Vector search:", query);
                  // TODO: Handle vector search
                }}
                results={[]}
                isSearching={false}
                error={null}
              />
            )}

            {state.activeTab === "quality" && (
              <DataQualityDashboard
                isLoading={false}
                error={null}
                onRefresh={() => {
                  console.log("Refresh quality metrics");
                  // TODO: Handle refresh
                }}
              />
            )}
          </div>

          {/* Table Viewer Sidebar */}
          {state.selectedTable && (
            <div className={styles.sidebar}>
              <TableViewer
                table={state.selectedTable}
                isLoading={false}
                error={null}
                onQuery={(query) => {
                  console.log("Table query:", query);
                  // TODO: Handle table query
                }}
                onExport={(format) => {
                  console.log("Export table:", format);
                  // TODO: Handle table export
                }}
              />
            </div>
          )}
        </div>
      ) : (
        <div className={styles.noSelection}>
          <div className={styles.emptyIcon}>🔍</div>
          <h2>Select a Database Connection</h2>
          <p>Choose a database connection from the list above to start exploring.</p>
        </div>
      )}
    </div>
  );
}
