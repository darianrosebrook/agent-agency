"use client";

/**
 * Project Detail Page - Dynamic Route Implementation
 * 
 * This page displays detailed information about a specific project,
 * including overview, workspace, tasks, timeline, and management tabs.
 */

import { useParams, useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { ProjectView } from "@/components/projects/ProjectView";
import { useProjectContext } from "@/components/projects/ProjectContext";
import styles from "./page.module.scss";

export default function ProjectDetailPage() {
  const params = useParams();
  const router = useRouter();
  const { getProjectById, selectProject } = useProjectContext();
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Safely extract projectId from params, ensuring it's a string
  const projectId = typeof params?.projectId === 'string' 
    ? params.projectId 
    : Array.isArray(params?.projectId) 
      ? params.projectId[0] 
      : null;

  useEffect(() => {
    // TODO: Replace local project lookup with API call to fetch project from v3 database with the following requirements:
    // 1. Project data fetching: Load project details from database
    //    - Data source: GET /api/projects/:projectId endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
    //    - Database table: PostgreSQL `projects` table
    //    - Include project metadata: name, summary, description, milestones, tasks, timestamps
    // 2. Project not found handling: Handle 404 errors gracefully
    //    - Display error message if project doesn't exist
    //    - Redirect to projects list if project is not found
    //    - Show loading state while fetching project data
    // 3. Project selection: Update current project context
    //    - Call selectProject with projectId to update context
    //    - Update last_accessed timestamp when project is viewed
    // 4. URL synchronization: Keep URL and project context in sync
    //    - Update URL when project is selected from other pages
    //    - Handle browser back/forward navigation
    if (!projectId) {
      setError("Invalid project ID");
      setIsLoading(false);
      return;
    }

    if (projectId) {
      const project = getProjectById(projectId);
      if (project) {
        selectProject(projectId);
        setIsLoading(false);
      } else {
        // Project not found in local context - would need to fetch from API
        setError("Project not found");
        setIsLoading(false);
      }
    }
  }, [projectId, getProjectById, selectProject]);

  const handleBackToProjects = () => {
    router.push("/projects");
  };

  if (isLoading) {
    return (
      <div className={styles.loadingContainer}>
        <div className={styles.loadingText}>Loading project...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.errorContainer}>
        <div className={styles.errorCard}>
          <h2 className={styles.errorTitle}>Project Not Found</h2>
          <p className={styles.errorMessage}>{error}</p>
          <button
            onClick={handleBackToProjects}
            className={styles.errorButton}
          >
            Back to Projects
          </button>
        </div>
      </div>
    );
  }

  const project = getProjectById(projectId);
  if (!project) {
    return null;
  }

  return (
    <ProjectView
      projectName={project.name}
      onBackToProjects={handleBackToProjects}
    />
  );
}

