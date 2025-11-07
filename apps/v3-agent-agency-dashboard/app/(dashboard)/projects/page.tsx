import React from 'react';
import Link from 'next/link';
import { Card, Table, Button, Badge } from '@/components/ui';
import { projectsApi } from '@/lib/api';
import { formatDate, formatRelativeTime } from '@/lib/utils';
import type { Project } from '@/types';
import styles from './page.module.scss';

export default async function ProjectsPage() {
  let projects: Project[] = [];

  try {
    projects = await projectsApi.listProjects();
  } catch (error) {
    console.error('Failed to fetch projects:', error);
  }

  const columns = [
    {
      key: 'name',
      header: 'Name',
      render: (project: Project) => (
        <Link href={`/projects/${project.id}`} className={styles.link}>
          {project.name}
        </Link>
      ),
    },
    {
      key: 'description',
      header: 'Description',
      render: (project: Project) => (
        <span className={styles.description}>
          {project.description || 'No description'}
        </span>
      ),
    },
    {
      key: 'created_at',
      header: 'Created',
      render: (project: Project) => formatDate(project.created_at),
    },
    {
      key: 'updated_at',
      header: 'Updated',
      render: (project: Project) => formatRelativeTime(project.updated_at),
    },
  ];

  return (
    <div className={styles.projects}>
      <div className={styles.header}>
        <h1>Projects</h1>
        <Button variant="primary">Create Project</Button>
      </div>

      <Card>
        {projects.length > 0 ? (
          <Table<Project> columns={columns} data={projects} />
        ) : (
          <div className={styles.empty}>
            <p>No projects found</p>
          </div>
        )}
      </Card>
    </div>
  );
}

