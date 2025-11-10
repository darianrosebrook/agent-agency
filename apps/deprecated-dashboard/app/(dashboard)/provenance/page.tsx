import React from 'react';
import Link from 'next/link';
import { Card, Table, Badge } from '@/components/ui';
import { provenanceApi } from '@/lib/api';
import { formatDateTime } from '@/lib/utils';
import type { ProvenanceEntry } from '@/types';
import styles from './page.module.scss';

export default async function ProvenancePage() {
  let entries: ProvenanceEntry[] = [];

  try {
    entries = await provenanceApi.listProvenance();
  } catch (error) {
    console.error('Failed to fetch provenance:', error);
  }

  const columns = [
    {
      key: 'id',
      header: 'ID',
      render: (entry: ProvenanceEntry) => (
        <Link href={`/provenance/${entry.id}`} className={styles.link}>
          {entry.id.slice(0, 8)}...
        </Link>
      ),
    },
    {
      key: 'action',
      header: 'Action',
      render: (entry: ProvenanceEntry) => (
        <Badge variant="default">{entry.action}</Badge>
      ),
    },
    {
      key: 'actor',
      header: 'Actor',
      render: (entry: ProvenanceEntry) => entry.actor,
    },
    {
      key: 'task_id',
      header: 'Task ID',
      render: (entry: ProvenanceEntry) => (
        <Link href={`/tasks/${entry.task_id}`} className={styles.link}>
          {entry.task_id.slice(0, 8)}...
        </Link>
      ),
    },
    {
      key: 'timestamp',
      header: 'Timestamp',
      render: (entry: ProvenanceEntry) => formatDateTime(entry.timestamp),
    },
  ];

  return (
    <div className={styles.provenance}>
      <div className={styles.header}>
        <h1>Provenance</h1>
      </div>

      <Card>
        {entries.length > 0 ? (
          <Table<ProvenanceEntry> columns={columns} data={entries} />
        ) : (
          <div className={styles.empty}>
            <p>No provenance entries found</p>
          </div>
        )}
      </Card>
    </div>
  );
}

