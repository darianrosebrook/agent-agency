"use client";

import React, { useState, useCallback } from "react";
import { QueryBuilderProps, QueryRequest, DatabaseTable, SavedQuery } from "@/types/database";
import { databaseApiClient, DatabaseApiError } from "@/lib/database-api";
import styles from "./QueryBuilder.module.scss";

interface QueryBuilderState {
  sqlQuery: string;
  parameters: any[];
  isReadOnly: boolean;
  isExecuting: boolean;
  error: string | null;
  executionResult: any;
  savedQueries: SavedQuery[];
  showSaveDialog: boolean;
  saveQueryName: string;
}

export default function QueryBuilder({
  tables,
  onQueryExecute,
  onQuerySave,
  savedQueries: externalSavedQueries,
  isExecuting: externalExecuting,
  error: externalError,
}: QueryBuilderProps) {
  const [state, setState] = useState<QueryBuilderState>({
    sqlQuery: "",
    parameters: [],
    isReadOnly: true,
    isExecuting: externalExecuting ?? false,
    error: externalError ?? null,
    executionResult: null,
    savedQueries: externalSavedQueries ?? [],
    showSaveDialog: false,
    saveQueryName: "",
  });

  // Handle query execution
  const handleExecuteQuery = useCallback(async () => {
    if (!state.sqlQuery.trim()) {
      setState((prev) => ({ ...prev, error: "Please enter a SQL query" }));
      return;
    }

    try {
      setState((prev) => ({ ...prev, isExecuting: true, error: null }));

      const queryRequest: QueryRequest = {
        sql: state.sqlQuery,
        parameters: state.parameters.length > 0 ? state.parameters : undefined,
        read_only: state.isReadOnly,
      };

      const result = await databaseApiClient.executeQuery(queryRequest);
      setState((prev) => ({
        ...prev,
        executionResult: result,
        isExecuting: false,
      }));

      onQueryExecute(queryRequest);
    } catch (error) {
      const errorMessage =
        error instanceof DatabaseApiError
          ? error.message
          : "Query execution failed";
      setState((prev) => ({
        ...prev,
        isExecuting: false,
        error: errorMessage,
      }));
      console.error("Query execution error:", error);
    }
  }, [state.sqlQuery, state.parameters, state.isReadOnly, onQueryExecute]);

  // Handle query saving
  const handleSaveQuery = useCallback(() => {
    if (!state.saveQueryName.trim()) {
      setState((prev) => ({ ...prev, error: "Please enter a query name" }));
      return;
    }

    const queryRequest: QueryRequest = {
      sql: state.sqlQuery,
      parameters: state.parameters.length > 0 ? state.parameters : undefined,
      read_only: state.isReadOnly,
    };

    onQuerySave?.(state.saveQueryName, queryRequest);
    setState((prev) => ({
      ...prev,
      showSaveDialog: false,
      saveQueryName: "",
    }));
  }, [state.saveQueryName, state.sqlQuery, state.parameters, state.isReadOnly, onQuerySave]);

  // Load saved query
  const handleLoadQuery = useCallback((savedQuery: SavedQuery) => {
    setState((prev) => ({
      ...prev,
      sqlQuery: savedQuery.query.sql,
      parameters: savedQuery.query.parameters || [],
      isReadOnly: savedQuery.query.read_only,
    }));
  }, []);

  // Handle parameter input
  const handleParameterChange = useCallback((index: number, value: string) => {
    setState((prev) => {
      const newParameters = [...prev.parameters];
      newParameters[index] = value;
      return { ...prev, parameters: newParameters };
    });
  }, []);

  // Add parameter
  const handleAddParameter = useCallback(() => {
    setState((prev) => ({
      ...prev,
      parameters: [...prev.parameters, ""],
    }));
  }, []);

  // Remove parameter
  const handleRemoveParameter = useCallback((index: number) => {
    setState((prev) => ({
      ...prev,
      parameters: prev.parameters.filter((_, i) => i !== index),
    }));
  }, []);

  // Insert table name into query
  const handleInsertTable = useCallback((tableName: string) => {
    const insertText = `"${tableName}"`;
    setState((prev) => ({
      ...prev,
      sqlQuery: prev.sqlQuery + insertText,
    }));
  }, []);

  // Insert column name into query
  const handleInsertColumn = useCallback((tableName: string, columnName: string) => {
    const insertText = `"${tableName}"."${columnName}"`;
    setState((prev) => ({
      ...prev,
      sqlQuery: prev.sqlQuery + insertText,
    }));
  }, []);

  // Generate sample SELECT query
  const generateSelectQuery = useCallback((table: DatabaseTable) => {
    const columns = table.columns.slice(0, 5).map((col) => `"${col.name}"`).join(", ");
    const query = `SELECT ${columns}\nFROM "${table.schema}"."${table.name}"\nLIMIT 100;`;
    setState((prev) => ({ ...prev, sqlQuery: query }));
  }, []);

  // Update state when external props change
  React.useEffect(() => {
    setState((prev) => ({
      ...prev,
      isExecuting: externalExecuting ?? prev.isExecuting,
      error: externalError ?? prev.error,
      savedQueries: externalSavedQueries ?? prev.savedQueries,
    }));
  }, [externalExecuting, externalError, externalSavedQueries]);

  return (
    <div className={styles.queryBuilder}>
      <div className={styles.builderHeader}>
        <h2>SQL Query Builder</h2>
        <p className={styles.description}>
          Build and execute SQL queries against your database with safety constraints.
        </p>
      </div>

      <div className={styles.builderLayout}>
        {/* Query Editor */}
        <div className={styles.querySection}>
          <div className={styles.queryHeader}>
            <h3>SQL Query</h3>
            <div className={styles.queryControls}>
              <label className={styles.readOnlyToggle}>
                <input
                  type="checkbox"
                  checked={state.isReadOnly}
                  onChange={(e) => setState((prev) => ({ ...prev, isReadOnly: e.target.checked }))}
                />
                Read-only query
              </label>
              <button
                onClick={handleExecuteQuery}
                disabled={state.isExecuting || !state.sqlQuery.trim()}
                className={styles.executeButton}
              >
                {state.isExecuting ? (
                  <>
                    <div className={styles.spinner}></div>
                    Executing...
                  </>
                ) : (
                  <>
                    ▶️ Execute Query
                  </>
                )}
              </button>
            </div>
          </div>

          <textarea
            value={state.sqlQuery}
            onChange={(e) => setState((prev) => ({ ...prev, sqlQuery: e.target.value }))}
            placeholder="Enter your SQL query here..."
            className={styles.queryTextarea}
            rows={10}
          />

          {/* Parameters */}
          {state.parameters.length > 0 && (
            <div className={styles.parametersSection}>
              <h4>Query Parameters</h4>
              <div className={styles.parametersList}>
                {state.parameters.map((param, index) => (
                  <div key={index} className={styles.parameterItem}>
                    <label>${index + 1}:</label>
                    <input
                      type="text"
                      value={param}
                      onChange={(e) => handleParameterChange(index, e.target.value)}
                      placeholder="Parameter value"
                      className={styles.parameterInput}
                    />
                    <button
                      onClick={() => handleRemoveParameter(index)}
                      className={styles.removeParameterButton}
                    >
                      ✕
                    </button>
                  </div>
                ))}
              </div>
              <button onClick={handleAddParameter} className={styles.addParameterButton}>
                + Add Parameter
              </button>
            </div>
          )}

          {/* Query Actions */}
          <div className={styles.queryActions}>
            <button
              onClick={() => setState((prev) => ({ ...prev, showSaveDialog: true }))}
              disabled={!state.sqlQuery.trim()}
              className={styles.saveButton}
            >
              💾 Save Query
            </button>
            <button
              onClick={handleAddParameter}
              className={styles.addParameterButton}
            >
              📝 Add Parameter
            </button>
          </div>
        </div>

        {/* Sidebar */}
        <div className={styles.sidebar}>
          {/* Table Browser */}
          <div className={styles.sidebarSection}>
            <h3>Tables</h3>
            <div className={styles.tableList}>
              {tables.map((table) => (
                <details key={`${table.schema}.${table.name}`} className={styles.tableItem}>
                  <summary className={styles.tableSummary}>
                    <span className={styles.tableName}>{table.name}</span>
                    <span className={styles.tableType}>{table.type}</span>
                  </summary>
                  <div className={styles.tableDetails}>
                    <div className={styles.tableActions}>
                      <button
                        onClick={() => generateSelectQuery(table)}
                        className={styles.quickSelectButton}
                      >
                        SELECT
                      </button>
                      <button
                        onClick={() => handleInsertTable(table.name)}
                        className={styles.insertButton}
                      >
                        Insert
                      </button>
                    </div>
                    <div className={styles.columnList}>
                      <h4>Columns</h4>
                      {table.columns.map((column) => (
                        <div key={column.name} className={styles.columnItem}>
                          <span className={styles.columnName}>{column.name}</span>
                          <span className={styles.columnType}>{column.type}</span>
                          {column.primary_key && <span className={styles.pkBadge}>PK</span>}
                          {column.foreign_key && <span className={styles.fkBadge}>FK</span>}
                          <button
                            onClick={() => handleInsertColumn(table.name, column.name)}
                            className={styles.insertColumnButton}
                          >
                            +
                          </button>
                        </div>
                      ))}
                    </div>
                  </div>
                </details>
              ))}
            </div>
          </div>

          {/* Saved Queries */}
          {state.savedQueries.length > 0 && (
            <div className={styles.sidebarSection}>
              <h3>Saved Queries</h3>
              <div className={styles.savedQueriesList}>
                {state.savedQueries.map((query) => (
                  <div key={query.id} className={styles.savedQueryItem}>
                    <div className={styles.savedQueryHeader}>
                      <span className={styles.savedQueryName}>{query.name}</span>
                      <span className={styles.savedQueryDate}>
                        {new Date(query.created_at).toLocaleDateString()}
                      </span>
                    </div>
                    <div className={styles.savedQueryActions}>
                      <button
                        onClick={() => handleLoadQuery(query)}
                        className={styles.loadQueryButton}
                      >
                        Load
                      </button>
                      <span className={styles.queryStats}>
                        Used {query.use_count} times
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Query Templates */}
          <div className={styles.sidebarSection}>
            <h3>Query Templates</h3>
            <div className={styles.templatesList}>
              <button
                onClick={() => setState((prev) => ({
                  ...prev,
                  sqlQuery: "SELECT * FROM table_name LIMIT 100;",
                  parameters: [],
                }))}
                className={styles.templateButton}
              >
                Basic SELECT
              </button>
              <button
                onClick={() => setState((prev) => ({
                  ...prev,
                  sqlQuery: "SELECT COUNT(*) FROM table_name WHERE column_name = $1;",
                  parameters: [""],
                }))}
                className={styles.templateButton}
              >
                COUNT with WHERE
              </button>
              <button
                onClick={() => setState((prev) => ({
                  ...prev,
                  sqlQuery: "SELECT * FROM table_name ORDER BY column_name DESC LIMIT 10;",
                  parameters: [],
                }))}
                className={styles.templateButton}
              >
                TOP 10 Ordered
              </button>
              <button
                onClick={() => setState((prev) => ({
                  ...prev,
                  sqlQuery: "SELECT column_name, COUNT(*) FROM table_name GROUP BY column_name ORDER BY COUNT(*) DESC;",
                  parameters: [],
                }))}
                className={styles.templateButton}
              >
                GROUP BY Count
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Error Display */}
      {state.error && (
        <div className={styles.errorBanner}>
          <span className={styles.errorIcon}>⚠️</span>
          <span>{state.error}</span>
        </div>
      )}

      {/* Execution Results */}
      {state.executionResult && (
        <div className={styles.resultsSection}>
          <div className={styles.resultsHeader}>
            <h3>Query Results</h3>
            <div className={styles.resultsMeta}>
              <span>
                {state.executionResult.result?.row_count || 0} rows returned
              </span>
              <span>
                Executed in {(state.executionResult.execution_stats?.execution_time_ms || 0).toFixed(2)}ms
              </span>
            </div>
          </div>

          {state.executionResult.result?.rows && state.executionResult.result.rows.length > 0 ? (
            <div className={styles.resultsTable}>
              <table className={styles.dataTable}>
                <thead>
                  <tr>
                    {state.executionResult.result.columns.map((col: any) => (
                      <th key={col.name}>{col.name}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {state.executionResult.result.rows.slice(0, 100).map((row: any, index: number) => (
                    <tr key={index}>
                      {state.executionResult.result.columns.map((col: any) => (
                        <td key={col.name}>
                          {row[col.name] !== null ? String(row[col.name]) : "NULL"}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
              {state.executionResult.result.rows.length > 100 && (
                <div className={styles.resultsNote}>
                  Showing first 100 rows of {state.executionResult.result.row_count} total rows.
                </div>
              )}
            </div>
          ) : (
            <div className={styles.noResults}>
              <div className={styles.emptyIcon}>📋</div>
              <h3>Query executed successfully</h3>
              <p>
                {state.executionResult.result?.row_count === 0
                  ? "No rows returned"
                  : "Query completed without returning data"}
              </p>
            </div>
          )}
        </div>
      )}

      {/* Save Query Dialog */}
      {state.showSaveDialog && (
        <div className={styles.modalOverlay}>
          <div className={styles.modal}>
            <h3>Save Query</h3>
            <div className={styles.modalContent}>
              <div className={styles.inputGroup}>
                <label>Query Name:</label>
                <input
                  type="text"
                  value={state.saveQueryName}
                  onChange={(e) => setState((prev) => ({ ...prev, saveQueryName: e.target.value }))}
                  placeholder="Enter a name for this query"
                  className={styles.modalInput}
                />
              </div>
            </div>
            <div className={styles.modalActions}>
              <button
                onClick={() => setState((prev) => ({ ...prev, showSaveDialog: false }))}
                className={styles.cancelButton}
              >
                Cancel
              </button>
              <button
                onClick={handleSaveQuery}
                disabled={!state.saveQueryName.trim()}
                className={styles.saveButton}
              >
                Save Query
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
