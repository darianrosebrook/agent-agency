import React from 'react';
import Link from 'next/link';
import { Card, Table } from '@/components/ui';
import { databaseApi } from '@/lib/api';
import { formatBytes, formatNumber } from '@/lib/utils';
import type { DatabaseTable } from '@/types';
import styles from './page.module.scss';

export default async function DatabasePage() {
  let tables: DatabaseTable[] = [];

  try {
    tables = await databaseApi.listTables();
  } catch (error) {
    console.error('Failed to fetch database tables:', error);
  }

  const columns = [
    {
      key: 'name',
      header: 'Table Name',
      render: (table: DatabaseTable) => (
        <Link href={`/database/tables/${table.name}`} className={styles.link}>
          {table.name}
        </Link>
      ),
    },
    {
      key: 'row_count',
      header: 'Row Count',
      render: (table: DatabaseTable) =>
        table.row_count !== undefined ? formatNumber(table.row_count) : 'N/A',
    },
    {
      key: 'size_bytes',
      header: 'Size',
      render: (table: DatabaseTable) =>
        table.size_bytes !== undefined ? formatBytes(table.size_bytes) : 'N/A',
    },
  ];

  return (
    <div className={styles.database}>
      <div className={styles.header}>
        <h1>Database</h1>
        <Link href="/database/query" className={styles.queryLink}>
          Query Interface
        </Link>
      </div>

      <Card>
        <div className={styles.section}>
          <h2>Tables</h2>
          {tables.length > 0 ? (
            <Table<DatabaseTable> columns={columns} data={tables} />
          ) : (
            <p className={styles.empty}>No tables found</p>
          )}
        </div>
      </Card>
    </div>
  );
}

