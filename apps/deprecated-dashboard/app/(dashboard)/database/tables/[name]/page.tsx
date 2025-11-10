import React from 'react';
import { notFound } from 'next/navigation';
import { Card, Table } from '@/components/ui';
import { databaseApi } from '@/lib/api';
import type { DatabaseColumn } from '@/types';
import styles from './page.module.scss';

export default async function TableDetailPage({
  params,
}: {
  params: { name: string };
}) {
  let schema;

  try {
    schema = await databaseApi.getTableSchema(params.name);
  } catch (error) {
    console.error('Failed to fetch table schema:', error);
    notFound();
  }

  if (!schema) {
    notFound();
  }

  const columnColumns = [
    {
      key: 'name',
      header: 'Column Name',
    },
    {
      key: 'type',
      header: 'Type',
    },
    {
      key: 'nullable',
      header: 'Nullable',
      render: (col: DatabaseColumn) => (col.nullable ? 'Yes' : 'No'),
    },
    {
      key: 'default',
      header: 'Default',
      render: (col: DatabaseColumn) => col.default || '—',
    },
  ];

  return (
    <div className={styles.table}>
      <div className={styles.header}>
        <h1>Table: {schema.name}</h1>
      </div>

      <div className={styles.grid}>
        <Card>
          <div className={styles.section}>
            <h2>Schema</h2>
            <Table<DatabaseColumn> columns={columnColumns} data={schema.columns} />
          </div>
        </Card>

        {schema.constraints && schema.constraints.length > 0 && (
          <Card>
            <div className={styles.section}>
              <h2>Constraints</h2>
              <ul className={styles.constraints}>
                {schema.constraints.map((constraint, index) => (
                  <li key={index} className={styles.constraint}>
                    <strong>{constraint.name}</strong> ({constraint.type}):{' '}
                    {constraint.columns.join(', ')}
                  </li>
                ))}
              </ul>
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}

