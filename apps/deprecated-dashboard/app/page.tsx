import React from 'react';
import { Card } from '@/components/ui';
import { systemApi, tasksApi, projectsApi } from '@/lib/api';
import { formatRelativeTime } from '@/lib/utils';
import styles from './page.module.scss';

export default async function DashboardPage() {
  // Fetch data server-side
  let systemHealth;
  let recentTasks;
  let projects;

  try {
    [systemHealth, recentTasks, projects] = await Promise.allSettled([
      systemApi.getSystemHealth(),
      tasksApi.listTasks().then((tasks) => tasks.slice(0, 5)),
      projectsApi.listProjects().then((projs) => projs.slice(0, 5)),
    ]);
  } catch (error) {
    console.error('Failed to fetch dashboard data:', error);
  }

  const health = systemHealth?.status === 'fulfilled' ? systemHealth.value : null;
  const tasks = recentTasks?.status === 'fulfilled' ? recentTasks.value : [];
  const projs = projects?.status === 'fulfilled' ? projects.value : [];

  return (
    <div className={styles.dashboard}>
      <h1 className={styles.title}>Dashboard</h1>

      <div className={styles.grid}>
        <Card className={styles.card}>
          <div className={styles.cardHeader}>
            <h2>System Health</h2>
          </div>
          <div className={styles.cardBody}>
            {health ? (
              <div className={styles.healthStatus}>
                <span className={`${styles.status} ${styles[`status--${health.status}`]}`}>
                  {health.status}
                </span>
                <p className={styles.timestamp}>
                  Last checked: {formatRelativeTime(health.timestamp)}
                </p>
              </div>
            ) : (
              <p>Unable to fetch system health</p>
            )}
          </div>
        </Card>

        <Card className={styles.card}>
          <div className={styles.cardHeader}>
            <h2>Recent Tasks</h2>
          </div>
          <div className={styles.cardBody}>
            {tasks.length > 0 ? (
              <ul className={styles.list}>
                {tasks.map((task) => (
                  <li key={task.id} className={styles.listItem}>
                    <span className={styles.taskTitle}>{task.title}</span>
                    <span className={styles.taskStatus}>{task.status}</span>
                    <span className={styles.taskTime}>
                      {formatRelativeTime(task.created_at)}
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <p>No recent tasks</p>
            )}
          </div>
        </Card>

        <Card className={styles.card}>
          <div className={styles.cardHeader}>
            <h2>Active Projects</h2>
          </div>
          <div className={styles.cardBody}>
            {projs.length > 0 ? (
              <ul className={styles.list}>
                {projs.map((project) => (
                  <li key={project.id} className={styles.listItem}>
                    <span className={styles.projectName}>{project.name}</span>
                    <span className={styles.projectTime}>
                      {formatRelativeTime(project.created_at)}
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <p>No active projects</p>
            )}
          </div>
        </Card>
      </div>
    </div>
  );
}
