import React from 'react';
import { notFound } from 'next/navigation';
import { Card, Badge, Button } from '@/components/ui';
import { ChainOfThoughtViewer } from '@/components/dashboard/ChainOfThoughtViewer';
import { CouncilDecisionsViewer } from '@/components/dashboard/CouncilDecisionsViewer';
import { tasksApi } from '@/lib/api';
import { formatDate, formatRelativeTime, formatDuration } from '@/lib/utils';
import styles from './page.module.scss';

export default async function TaskDetailPage({
  params,
}: {
  params: { id: string };
}) {
  let task;
  let chainOfThought;
  let councilDecisions;

  try {
    [task, chainOfThought, councilDecisions] = await Promise.allSettled([
      tasksApi.getTask(params.id),
      tasksApi.getChainOfThought(params.id).catch(() => null),
      tasksApi.getCouncilDecisions(params.id).catch(() => null),
    ]);
  } catch (error) {
    console.error('Failed to fetch task:', error);
    notFound();
  }

  if (task.status !== 'fulfilled' || !task.value) {
    notFound();
  }

  const taskData = task.value;
  const chainData = chainOfThought.status === 'fulfilled' ? chainOfThought.value : null;
  const decisionsData =
    councilDecisions.status === 'fulfilled' ? councilDecisions.value : null;

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

  return (
    <div className={styles.task}>
      <div className={styles.header}>
        <div>
          <h1>{taskData.title}</h1>
          <Badge variant={getStatusVariant(taskData.status)} className={styles.status}>
            {taskData.status}
          </Badge>
        </div>
        <div className={styles.actions}>
          {taskData.status === 'running' && (
            <>
              <Button variant="secondary" size="sm">
                Pause
              </Button>
              <Button variant="danger" size="sm">
                Cancel
              </Button>
            </>
          )}
          {taskData.status === 'paused' && (
            <Button variant="primary" size="sm">
              Resume
            </Button>
          )}
        </div>
      </div>

      <div className={styles.grid}>
        <Card>
          <div className={styles.section}>
            <h2>Task Details</h2>
            <dl className={styles.details}>
              <div>
                <dt>ID</dt>
                <dd className={styles.mono}>{taskData.id}</dd>
              </div>
              <div>
                <dt>Description</dt>
                <dd>{taskData.description}</dd>
              </div>
              <div>
                <dt>Risk Tier</dt>
                <dd>{taskData.risk_tier}</dd>
              </div>
              <div>
                <dt>Status</dt>
                <dd>
                  <Badge variant={getStatusVariant(taskData.status)}>
                    {taskData.status}
                  </Badge>
                </dd>
              </div>
              {taskData.assigned_worker_id && (
                <div>
                  <dt>Assigned Worker</dt>
                  <dd className={styles.mono}>{taskData.assigned_worker_id}</dd>
                </div>
              )}
              <div>
                <dt>Created</dt>
                <dd>{formatDate(taskData.created_at)}</dd>
              </div>
              <div>
                <dt>Updated</dt>
                <dd>{formatRelativeTime(taskData.updated_at)}</dd>
              </div>
              {taskData.completed_at && (
                <div>
                  <dt>Completed</dt>
                  <dd>{formatDate(taskData.completed_at)}</dd>
                </div>
              )}
            </dl>
          </div>
        </Card>

        {chainData && (
          <div className={styles.fullWidth}>
            <ChainOfThoughtViewer chainOfThought={chainData} />
          </div>
        )}

        {decisionsData && (
          <div className={styles.fullWidth}>
            <CouncilDecisionsViewer decisions={decisionsData} />
          </div>
        )}
      </div>
    </div>
  );
}

