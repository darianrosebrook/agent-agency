import React from 'react';
import { notFound } from 'next/navigation';
import { Card, Table, Badge } from '@/components/ui';
import { projectsApi, tasksApi } from '@/lib/api';
import { formatDate, formatRelativeTime } from '@/lib/utils';
import type { Task } from '@/types';
import styles from './page.module.scss';

export default async function ProjectDetailPage({
  params,
}: {
  params: { id: string };
}) {
  let project;
  let projectTasks: Task[] = [];

  try {
    [project, projectTasks] = await Promise.all([
      projectsApi.getProject(params.id),
      projectsApi.getProjectTasks(params.id).catch(() => []),
    ]);
  } catch (error) {
    console.error('Failed to fetch project:', error);
    notFound();
  }

  if (!project) {
    notFound();
  }

  const taskColumns = [
    {
      key: 'title',
      header: 'Title',
      render: (task: Task) => (
        <a href={`/tasks/${task.id}`} className={styles.link}>
          {task.title}
        </a>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (task: Task) => (
        <Badge
          variant={
            task.status === 'completed'
              ? 'success'
              : task.status === 'failed'
              ? 'error'
              : task.status === 'running'
              ? 'info'
              : 'default'
          }
        >
          {task.status}
        </Badge>
      ),
    },
    {
      key: 'created_at',
      header: 'Created',
      render: (task: Task) => formatDate(task.created_at),
    },
  ];

  return (
    <div className={styles.project}>
      <div className={styles.header}>
        <h1>{project.name}</h1>
      </div>

      <div className={styles.grid}>
        <Card>
          <div className={styles.section}>
            <h2>Project Details</h2>
            <dl className={styles.details}>
              <div>
                <dt>ID</dt>
                <dd>{project.id}</dd>
              </div>
              {project.description && (
                <div>
                  <dt>Description</dt>
                  <dd>{project.description}</dd>
                </div>
              )}
              <div>
                <dt>Created</dt>
                <dd>{formatDate(project.created_at)}</dd>
              </div>
              <div>
                <dt>Updated</dt>
                <dd>{project.updated_at ? formatRelativeTime(project.updated_at) : 'N/A'}</dd>
              </div>
            </dl>
          </div>
        </Card>

        <Card>
          <div className={styles.section}>
            <h2>Tasks ({projectTasks.length})</h2>
            {projectTasks.length > 0 ? (
              <Table<Task> columns={taskColumns} data={projectTasks} />
            ) : (
              <p className={styles.empty}>No tasks for this project</p>
            )}
          </div>
        </Card>
      </div>
    </div>
  );
}

