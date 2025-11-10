'use client';

import React, { useState } from 'react';
import { Card, Button, Table, Loading } from '@/components/ui';
import { databaseApi } from '@/lib/api';
import type { DatabaseQueryResult } from '@/types';
import styles from './page.module.scss';

export default function QueryPage() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<DatabaseQueryResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleExecute = async () => {
    if (!query.trim()) {
      setError('Please enter a query');
      return;
    }

    setIsLoading(true);
    setError(null);
    setResults(null);

    try {
      const result = await databaseApi.executeQuery(query);
      setResults(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to execute query');
    } finally {
      setIsLoading(false);
    }
  };

  const columns =
    results?.columns.map((col) => ({
      key: col,
      header: col,
      render: (row: Record<string, unknown>) => {
        const value = row[col];
        return value !== null && value !== undefined ? String(value) : 'NULL';
      },
    })) || [];

  return (
    <div className={styles.query}>
      <div className={styles.header}>
        <h1>Database Query</h1>
      </div>

      <div className={styles.grid}>
        <Card>
          <div className={styles.section}>
            <h2>Query Editor</h2>
            <textarea
              className={styles.editor}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Enter SQL query..."
              rows={10}
            />
            <div className={styles.actions}>
              <Button variant="primary" onClick={handleExecute} isLoading={isLoading}>
                Execute Query
              </Button>
              <Button variant="secondary" onClick={() => setQuery('')}>
                Clear
              </Button>
            </div>
            {error && <div className={styles.error}>{error}</div>}
          </div>
        </Card>

        {isLoading && (
          <Card>
            <Loading message="Executing query..." />
          </Card>
        )}

        {results && !isLoading && (
          <Card>
            <div className={styles.section}>
              <h2>
                Results ({results.row_count} {results.row_count === 1 ? 'row' : 'rows'})
                {results.execution_time_ms && (
                  <span className={styles.time}>
                    {' '}
                    ({results.execution_time_ms}ms)
                  </span>
                )}
              </h2>
              {results.rows.length > 0 ? (
                <Table<Record<string, unknown>>
                  columns={columns}
                  data={results.rows.map((row) =>
                    results.columns.reduce(
                      (acc, col, idx) => ({ ...acc, [col]: row[idx] }),
                      {} as Record<string, unknown>
                    )
                  )}
                />
              ) : (
                <p className={styles.empty}>No results</p>
              )}
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}

