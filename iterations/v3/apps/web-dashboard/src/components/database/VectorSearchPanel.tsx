"use client";

import React, { useState, useCallback } from "react";
import { VectorSearchPanelProps, VectorSearchQuery, VectorSearchResult } from "@/types/database";
import { databaseApiClient, DatabaseApiError } from "@/lib/database-api";
import styles from "./VectorSearchPanel.module.scss";

interface VectorSearchPanelState {
  selectedTable: string;
  selectedColumn: string;
  searchVector: number[];
  vectorInput: string;
  limit: number;
  threshold: number;
  isSearching: boolean;
  error: string | null;
  results: VectorSearchResult[];
}

export default function VectorSearchPanel({
  tables = [],
  onSearch,
  results: externalResults,
  isSearching: externalSearching,
  error: externalError,
}: VectorSearchPanelProps) {
  const [state, setState] = useState<VectorSearchPanelState>({
    selectedTable: "",
    selectedColumn: "",
    searchVector: [],
    vectorInput: "",
    limit: 10,
    threshold: 0.1,
    isSearching: externalSearching ?? false,
    error: externalError ?? null,
    results: externalResults ?? [],
  });

  // Get vector columns from available tables
  const vectorColumns = React.useMemo(() => {
    const columns: { table: string; column: string; dimension: number }[] = [];
    tables.forEach((table) => {
      table.columns.forEach((column) => {
        if (column.vector_dimension) {
          columns.push({
            table: table.name,
            column: column.name,
            dimension: column.vector_dimension,
          });
        }
      });
    });
    return columns;
  }, [tables]);

  // Parse vector input
  const parseVectorInput = useCallback((input: string): number[] => {
    try {
      // Try to parse as JSON array first
      const parsed = JSON.parse(input);
      if (Array.isArray(parsed) && parsed.every((n) => typeof n === "number")) {
        return parsed;
      }

      // Try to parse as comma-separated values
      const values = input.split(",").map((s) => {
        const trimmed = s.trim();
        const num = parseFloat(trimmed);
        if (isNaN(num)) {
          throw new Error(`Invalid number: ${trimmed}`);
        }
        return num;
      });

      return values;
    } catch (error) {
      throw new Error(`Invalid vector format: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }, []);

  // Handle vector input change
  const handleVectorInputChange = useCallback((input: string) => {
    setState((prev) => ({ ...prev, vectorInput: input }));

    try {
      const vector = parseVectorInput(input);
      setState((prev) => ({
        ...prev,
        searchVector: vector,
        error: null,
      }));
    } catch (error) {
      setState((prev) => ({
        ...prev,
        searchVector: [],
        error: error instanceof Error ? error.message : "Invalid vector input",
      }));
    }
  }, [parseVectorInput]);

  // Handle search
  const handleSearch = useCallback(async () => {
    if (!state.selectedTable || !state.selectedColumn || state.searchVector.length === 0) {
      setState((prev) => ({
        ...prev,
        error: "Please select a table, column, and provide a search vector",
      }));
      return;
    }

    try {
      setState((prev) => ({ ...prev, isSearching: true, error: null }));

      const searchQuery: VectorSearchQuery = {
        table: state.selectedTable,
        column: state.selectedColumn,
        vector: state.searchVector,
        limit: state.limit,
        threshold: state.threshold,
      };

      const response = await databaseApiClient.vectorSearch(searchQuery);
      setState((prev) => ({
        ...prev,
        results: response.results,
        isSearching: false,
      }));

      onSearch?.(searchQuery);
    } catch (error) {
      const errorMessage =
        error instanceof DatabaseApiError
          ? error.message
          : "Vector search failed";
      setState((prev) => ({
        ...prev,
        isSearching: false,
        error: errorMessage,
      }));
      console.error("Vector search error:", error);
    }
  }, [state.selectedTable, state.selectedColumn, state.searchVector, state.limit, state.threshold, onSearch]);

  // Handle table/column selection
  const handleTableChange = useCallback((tableName: string) => {
    setState((prev) => ({
      ...prev,
      selectedTable: tableName,
      selectedColumn: "",
    }));
  }, []);

  const handleColumnChange = useCallback((columnName: string) => {
    setState((prev) => ({
      ...prev,
      selectedColumn: columnName,
    }));
  }, []);

  // Generate sample vectors for testing
  const generateSampleVector = useCallback((dimension: number) => {
    const vector = Array.from({ length: dimension }, () => Math.random() * 2 - 1); // Random values between -1 and 1
    const vectorString = JSON.stringify(vector);
    setState((prev) => ({ ...prev, vectorInput: vectorString }));
    handleVectorInputChange(vectorString);
  }, [handleVectorInputChange]);

  // Update state when external props change
  React.useEffect(() => {
    setState((prev) => ({
      ...prev,
      isSearching: externalSearching ?? prev.isSearching,
      error: externalError ?? prev.error,
      results: externalResults ?? prev.results,
    }));
  }, [externalSearching, externalError, externalResults]);

  return (
    <div className={styles.vectorSearchPanel}>
      <div className={styles.panelHeader}>
        <h2>Vector Similarity Search</h2>
        <p className={styles.description}>
          Search for similar vectors in your database using cosine similarity or other distance metrics.
        </p>
      </div>

      {/* Search Configuration */}
      <div className={styles.searchConfig}>
        <div className={styles.configSection}>
          <h3>Search Target</h3>
          <div className={styles.inputGroup}>
            <label>Table:</label>
            <select
              value={state.selectedTable}
              onChange={(e) => handleTableChange(e.target.value)}
              className={styles.selectInput}
            >
              <option value="">Select a table...</option>
              {tables.map((table) => (
                <option key={table.name} value={table.name}>
                  {table.schema}.{table.name}
                </option>
              ))}
            </select>
          </div>

          <div className={styles.inputGroup}>
            <label>Vector Column:</label>
            <select
              value={state.selectedColumn}
              onChange={(e) => handleColumnChange(e.target.value)}
              disabled={!state.selectedTable}
              className={styles.selectInput}
            >
              <option value="">Select a vector column...</option>
              {vectorColumns
                .filter((col) => col.table === state.selectedTable)
                .map((col) => (
                  <option key={col.column} value={col.column}>
                    {col.column} (dimension: {col.dimension})
                  </option>
                ))}
            </select>
          </div>
        </div>

        <div className={styles.configSection}>
          <h3>Search Vector</h3>
          <div className={styles.inputGroup}>
            <label>Vector Values:</label>
            <textarea
              value={state.vectorInput}
              onChange={(e) => handleVectorInputChange(e.target.value)}
              placeholder="Enter vector as JSON array [1.0, 2.0, 3.0] or comma-separated values 1.0, 2.0, 3.0"
              className={`${styles.vectorInput} ${state.searchVector.length === 0 ? styles.invalid : ""}`}
              rows={3}
            />
            {state.selectedColumn && (
              <div className={styles.vectorHelpers}>
                <button
                  onClick={() => {
                    const dimension = vectorColumns.find(
                      (col) => col.table === state.selectedTable && col.column === state.selectedColumn
                    )?.dimension;
                    if (dimension) {
                      generateSampleVector(dimension);
                    }
                  }}
                  className={styles.helperButton}
                >
                  Generate Sample Vector
                </button>
                <span className={styles.vectorInfo}>
                  Expected dimension: {
                    vectorColumns.find(
                      (col) => col.table === state.selectedTable && col.column === state.selectedColumn
                    )?.dimension || "unknown"
                  }
                </span>
              </div>
            )}
          </div>
        </div>

        <div className={styles.configSection}>
          <h3>Search Parameters</h3>
          <div className={styles.parameterGrid}>
            <div className={styles.inputGroup}>
              <label>Limit:</label>
              <input
                type="number"
                value={state.limit}
                onChange={(e) => setState((prev) => ({ ...prev, limit: Number(e.target.value) }))}
                min={1}
                max={1000}
                className={styles.numberInput}
              />
            </div>

            <div className={styles.inputGroup}>
              <label>Threshold:</label>
              <input
                type="number"
                value={state.threshold}
                onChange={(e) => setState((prev) => ({ ...prev, threshold: Number(e.target.value) }))}
                min={0}
                max={1}
                step={0.01}
                className={styles.numberInput}
              />
              <span className={styles.inputHelp}>Minimum similarity score (0-1)</span>
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

      {/* Search Button */}
      <div className={styles.searchActions}>
        <button
          onClick={handleSearch}
          disabled={state.isSearching || !state.selectedTable || !state.selectedColumn || state.searchVector.length === 0}
          className={styles.searchButton}
        >
          {state.isSearching ? (
            <>
              <div className={styles.spinner}></div>
              Searching...
            </>
          ) : (
            <>
              🔍 Search Vectors
            </>
          )}
        </button>
      </div>

      {/* Search Results */}
      {state.results.length > 0 && (
        <div className={styles.resultsSection}>
          <div className={styles.resultsHeader}>
            <h3>Search Results ({state.results.length})</h3>
            <div className={styles.resultsMeta}>
              <span>Query executed in search</span>
            </div>
          </div>

          <div className={styles.resultsTable}>
            <div className={styles.tableHeader}>
              <div className={styles.headerCell}>ID</div>
              <div className={styles.headerCell}>Similarity</div>
              <div className={styles.headerCell}>Vector Preview</div>
              <div className={styles.headerCell}>Metadata</div>
            </div>

            {state.results.map((result, index) => (
              <div key={index} className={styles.resultRow}>
                <div className={styles.cell}>
                  <code className={styles.idCell}>{result.id}</code>
                </div>
                <div className={styles.cell}>
                  <div className={styles.similarityScore}>
                    {(result.similarity * 100).toFixed(2)}%
                  </div>
                  <div className={styles.similarityBar}>
                    <div
                      className={styles.similarityFill}
                      style={{ width: `${result.similarity * 100}%` }}
                    ></div>
                  </div>
                </div>
                <div className={styles.cell}>
                  <span className={styles.vectorPreview} title={JSON.stringify(result.vector)}>
                    [{result.vector.slice(0, 5).join(", ")}{result.vector.length > 5 ? "..." : ""}]
                  </span>
                  <span className={styles.dimensionBadge}>
                    {result.vector.length}D
                  </span>
                </div>
                <div className={styles.cell}>
                  {result.metadata ? (
                    <details className={styles.metadataDetails}>
                      <summary>View metadata</summary>
                      <pre className={styles.metadataContent}>
                        {JSON.stringify(result.metadata, null, 2)}
                      </pre>
                    </details>
                  ) : (
                    <span className={styles.noMetadata}>No metadata</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* No Results State */}
      {!state.isSearching && state.results.length === 0 && !state.error && (
        <div className={styles.noResults}>
          <div className={styles.emptyIcon}>🔍</div>
          <h3>No Search Results</h3>
          <p>
            Configure your search parameters and click "Search Vectors" to find similar vectors in your database.
          </p>
        </div>
      )}
    </div>
  );
}
