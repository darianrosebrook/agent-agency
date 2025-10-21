"use client";

import React, { useState, useEffect } from "react";
import { TableViewerProps, QueryResult, TableQueryRequest } from "@/types/database";
import { databaseApiClient, DatabaseApiError } from "@/lib/database-api";
import styles from "./TableViewer.module.scss";

interface TableViewerState {
  data: QueryResult | null;
  isLoading: boolean;
  error: string | null;
  currentPage: number;
  pageSize: number;
  sortColumn: string | null;
  sortDirection: "asc" | "desc";
}

export default function TableViewer({
  table,
  data: externalData,
  isLoading: externalLoading,
  error: externalError,
  onQuery,
  onExport,
}: TableViewerProps) {
  const [state, setState] = useState<TableViewerState>({
    data: externalData ?? null,
    isLoading: externalLoading ?? false,
    error: externalError ?? null,
    currentPage: 1,
    pageSize: 100,
    sortColumn: null,
    sortDirection: "asc",
  });

  const [selectedColumns, setSelectedColumns] = useState<string[]>(
    table.columns.map((col) => col.name)
  );

  // Load table data if not provided externally
  const loadTableData = async () => {
    if (externalData) return;

    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      const queryRequest: TableQueryRequest = {
        table: table.name,
        columns: selectedColumns,
        limit: state.pageSize,
        offset: (state.currentPage - 1) * state.pageSize,
        order_by: state.sortColumn
          ? `${state.sortColumn} ${state.sortDirection}`
          : undefined,
      };

      const response = await databaseApiClient.queryTable(queryRequest);
      setState((prev) => ({
        ...prev,
        data: response.result,
        isLoading: false,
      }));

      onQuery?.(queryRequest);
    } catch (error) {
      const errorMessage =
        error instanceof DatabaseApiError
          ? error.message
          : "Failed to load table data";
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      console.error("Failed to load table data:", error);
    }
  };

  // Handle column selection
  const handleColumnToggle = (columnName: string) => {
    setSelectedColumns((prev) => {
      if (prev.includes(columnName)) {
        return prev.filter((col) => col !== columnName);
      } else {
        return [...prev, columnName];
      }
    });
  };

  // Handle sorting
  const handleSort = (columnName: string) => {
    setState((prev) => ({
      ...prev,
      sortColumn: columnName,
      sortDirection:
        prev.sortColumn === columnName && prev.sortDirection === "asc"
          ? "desc"
          : "asc",
    }));
  };

  // Handle pagination
  const handlePageChange = (newPage: number) => {
    setState((prev) => ({ ...prev, currentPage: newPage }));
  };

  // Handle page size change
  const handlePageSizeChange = (newPageSize: number) => {
    setState((prev) => ({ ...prev, pageSize: newPageSize, currentPage: 1 }));
  };

  // Reload data when dependencies change
  useEffect(() => {
    loadTableData();
  }, [table.name, selectedColumns, state.currentPage, state.pageSize, state.sortColumn, state.sortDirection]);

  // Update state when external props change
  useEffect(() => {
    setState((prev) => ({
      ...prev,
      data: externalData ?? prev.data,
      isLoading: externalLoading ?? prev.isLoading,
      error: externalError ?? prev.error,
    }));
  }, [externalData, externalLoading, externalError]);

  const renderCellValue = (value: any, _columnType: string) => {
    if (value === null || value === undefined) {
      return <span className={styles.nullValue}>NULL</span>;
    }

    // Handle different data types
    if (typeof value === "boolean") {
      return <span className={styles.booleanValue}>{value ? "TRUE" : "FALSE"}</span>;
    }

    if (typeof value === "object") {
      // Handle arrays and objects
      if (Array.isArray(value)) {
        return (
          <span className={styles.arrayValue} title={JSON.stringify(value)}>
            [{value.length} items]
          </span>
        );
      }
      return (
        <span className={styles.objectValue} title={JSON.stringify(value)}>
          Object
        </span>
      );
    }

    // Handle large text
    const stringValue = String(value);
    if (stringValue.length > 100) {
      return (
        <span className={styles.truncatedValue} title={stringValue}>
          {stringValue.substring(0, 100)}...
        </span>
      );
    }

    return <span>{stringValue}</span>;
  };

  return (
    <div className={styles.tableViewer}>
      <div className={styles.viewerHeader}>
        <h2>{table.schema}.{table.name}</h2>
        <div className={styles.viewerActions}>
          {onExport && (
            <div className={styles.exportDropdown}>
              <button className={styles.exportButton}>
                Export ▼
              </button>
              <div className={styles.exportMenu}>
                <button onClick={() => onExport("csv")}>Export as CSV</button>
                <button onClick={() => onExport("json")}>Export as JSON</button>
                <button onClick={() => onExport("sql")}>Export as SQL</button>
              </div>
            </div>
          )}
          <button
            onClick={() => loadTableData()}
            className={styles.refreshButton}
            disabled={state.isLoading}
          >
            🔄 Refresh
          </button>
        </div>
      </div>

      {/* Table Schema Overview */}
      <div className={styles.schemaOverview}>
        <div className={styles.schemaStats}>
          <span className={styles.stat}>
            {table.columns.length} columns
          </span>
          {table.row_count !== undefined && (
            <span className={styles.stat}>
              {table.row_count.toLocaleString()} rows
            </span>
          )}
          {table.size_bytes !== undefined && (
            <span className={styles.stat}>
              {(table.size_bytes / 1024 / 1024).toFixed(2)} MB
            </span>
          )}
        </div>
      </div>

      {/* Column Selector */}
      <div className={styles.columnSelector}>
        <h3>Columns</h3>
        <div className={styles.columnList}>
          {table.columns.map((column) => (
            <label key={column.name} className={styles.columnCheckbox}>
              <input
                type="checkbox"
                checked={selectedColumns.includes(column.name)}
                onChange={() => handleColumnToggle(column.name)}
              />
              <span className={styles.columnName}>{column.name}</span>
              <span className={styles.columnType}>{column.type}</span>
              {column.primary_key && <span className={styles.primaryKeyBadge}>PK</span>}
              {column.foreign_key && <span className={styles.foreignKeyBadge}>FK</span>}
              {column.vector_dimension && (
                <span className={styles.vectorBadge} title={`Vector dimension: ${column.vector_dimension}`}>
                  🔍
                </span>
              )}
            </label>
          ))}
        </div>
      </div>

      {/* Data Table */}
      <div className={styles.dataSection}>
        {state.isLoading ? (
          <div className={styles.loading}>
            <div className={styles.spinner}></div>
            <p>Loading table data...</p>
          </div>
        ) : state.error ? (
          <div className={styles.error}>
            <h3>Failed to load data</h3>
            <p>{state.error}</p>
            <button onClick={() => loadTableData()}>Retry</button>
          </div>
        ) : state.data ? (
          <>
            {/* Table Controls */}
            <div className={styles.tableControls}>
              <div className={styles.pageSizeSelector}>
                <label>Rows per page:</label>
                <select
                  value={state.pageSize}
                  onChange={(e) => handlePageSizeChange(Number(e.target.value))}
                >
                  <option value={25}>25</option>
                  <option value={50}>50</option>
                  <option value={100}>100</option>
                  <option value={250}>250</option>
                  <option value={500}>500</option>
                </select>
              </div>
              <div className={styles.executionStats}>
                <span>
                  Query executed in {(state.data.execution_time_ms || 0).toFixed(2)}ms
                </span>
                <span>
                  {state.data.row_count} of {state.data.row_count} rows shown
                </span>
              </div>
            </div>

            {/* Data Table */}
            <div className={styles.dataTableContainer}>
              <table className={styles.dataTable}>
                <thead>
                  <tr>
                    {state.data.columns.map((column) => (
                      <th
                        key={column.name}
                        className={`${styles.tableHeader} ${
                          state.sortColumn === column.name ? styles.sorted : ""
                        }`}
                        onClick={() => handleSort(column.name)}
                      >
                        <div className={styles.headerContent}>
                          <span className={styles.columnName}>{column.name}</span>
                          <span className={styles.columnType}>{column.type}</span>
                          {state.sortColumn === column.name && (
                            <span className={styles.sortIndicator}>
                              {state.sortDirection === "asc" ? "↑" : "↓"}
                            </span>
                          )}
                        </div>
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {state.data.rows.map((row, index) => (
                    <tr key={index} className={index % 2 === 0 ? styles.evenRow : styles.oddRow}>
                      {state.data!.columns.map((column) => (
                        <td key={column.name} className={styles.tableCell}>
                          {renderCellValue(row[column.name], column.type)}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Pagination */}
            {state.data.row_count > state.pageSize && (
              <div className={styles.pagination}>
                <button
                  className={styles.pageButton}
                  disabled={state.currentPage <= 1}
                  onClick={() => handlePageChange(state.currentPage - 1)}
                >
                  Previous
                </button>

                <div className={styles.pageInfo}>
                  Page {state.currentPage} of {Math.ceil(state.data.row_count / state.pageSize)}
                </div>

                <button
                  className={styles.pageButton}
                  disabled={state.currentPage >= Math.ceil(state.data.row_count / state.pageSize)}
                  onClick={() => handlePageChange(state.currentPage + 1)}
                >
                  Next
                </button>
              </div>
            )}
          </>
        ) : (
          <div className={styles.noData}>
            <div className={styles.emptyIcon}>📋</div>
            <h3>No Data Available</h3>
            <p>Select columns and click refresh to load table data.</p>
            <button onClick={() => loadTableData()} className={styles.primaryButton}>
              Load Data
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
