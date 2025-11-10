import React from 'react';
import Link from 'next/link';
import { Card, Table, Badge, Button } from '@/components/ui';
import { tasksApi } from '@/lib/api';
import { formatDate, formatRelativeTime } from '@/lib/utils';
import type { Task } from '@/types';
import styles from './page.module.scss';

export default async function TasksPage() {
  let tasks: Task[] = [];

  try {
    tasks = await tasksApi.listTasks();
  } catch (error) {
    console.error('Failed to fetch tasks:', error);
  }

  const getStatusVariant = (status: string) => {
    switch (status) {
      case 'completed':
        return 'success';
      case 'failed':
        return 'error';
      case 'running':
        return 'info';
      case 'paused':
        return 'warning';
      default:
        return 'default';
    }
  };

  const columns = [
    {
      key: 'id',
      header: 'ID',
      render: (task: Task) => (
        <Link href={`/tasks/${task.id}`} className={styles.link}>
          {task.id.slice(0, 8)}...
        </Link>
      ),
    },
    {
      key: 'title',
      header: 'Title',
      render: (task: Task) => task.title,
    },
    {
      key: 'status',
      header: 'Status',
      render: (task: Task) => (
        <Badge variant={getStatusVariant(task.status)}>{task.status}</Badge>
      ),
    },
    {
      key: 'created_at',
      header: 'Created',
      render: (task: Task) => formatDate(task.created_at),
    },
    {
      key: 'updated_at',
      header: 'Updated',
      render: (task: Task) => formatRelativeTime(task.updated_at),
    },
  ];

  return (
    <div className={styles.tasks}>
      <div className={styles.header}>
        <h1>Tasks</h1>
        <Button variant="primary">Create Task</Button>
      </div>

      <Card>
        {tasks.length > 0 ? (
          <Table<Task> columns={columns} data={tasks} />
        ) : (
          <div className={styles.empty}>
            <p>No tasks found</p>
          </div>
        )}
      </Card>
    </div>
  );
}

