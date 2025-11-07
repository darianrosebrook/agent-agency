'use client';

import React from 'react';
import styles from './Table.module.scss';

export interface TableColumn<T = Record<string, unknown>> {
  key: string;
  header: string;
  render?: (item: T) => React.ReactNode;
  sortable?: boolean;
}

export interface TableProps<T = Record<string, unknown>> {
  columns: TableColumn<T>[];
  data: T[];
  className?: string;
  onRowClick?: (item: T) => void;
}

export function Table<T = Record<string, unknown>>({
  columns,
  data,
  className = '',
  onRowClick,
}: TableProps<T>) {
  return (
    <div className={`${styles.tableWrapper} ${className}`}>
      <table className={styles.table}>
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key} className={styles.header}>
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.length === 0 ? (
            <tr>
              <td colSpan={columns.length} className={styles.empty}>
                No data available
              </td>
            </tr>
          ) : (
            data.map((item, index) => (
              <tr
                key={index}
                className={onRowClick ? styles.clickableRow : ''}
                onClick={() => onRowClick?.(item)}
              >
                {columns.map((column) => (
                  <td key={column.key} className={styles.cell}>
                    {column.render
                      ? column.render(item as T)
                      : ((item as Record<string, unknown>)[column.key] as React.ReactNode)}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

