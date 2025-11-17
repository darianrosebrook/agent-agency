"use client";

import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
import {
  listProjects,
  createProject as createProjectApi,
  updateProjectHandler,
  getProjectHandler,
  deleteProject as deleteProjectApi,
  createProjectMilestone,
  createProjectTask,
  updateProjectTask,
  type ProjectApiResponse,
} from "../../lib/api/projects";
import { retryWithBackoff } from "../../lib/utils/retry";

export interface Milestone {
  id: string;
  title: string;
  completed: boolean;
}

import type { TaskWithOptionalDescription } from '../../lib/types/task';

/**
 * Task interface for ProjectContext (UI-specific)
 * 
 * Simplified version of canonical Task for UI display.
 * Uses Date object for createdAt instead of RFC3339 string.
 * Only includes fields actually used in the UI.
 */
export interface Task extends Pick<TaskWithOptionalDescription, 'id' | 'title' | 'description' | 'status' | 'priority' | 'assigned_worker_id'> {
  // UI-specific: createdAt as Date object for easier manipulation
  createdAt: Date;
}

export interface Project {
  id: string;
  name: string;
  summary?: string;
  description?: string;
  milestones: Milestone[];
  tasks: Task[];
  createdAt: Date;
  lastAccessed: Date;
}

interface ProjectContextType {
  projects: Project[];
  currentProjectId: string | null;
  isLoading: boolean;
  error: Error | null;
  getCurrentProject: () => Project | null;
  getProjectById: (projectId: string) => Project | undefined;
  createProject: (data: {
    name: string;
    summary?: string;
    description?: string;
    milestones?: string[];
  }) => Promise<string>;
  selectProject: (projectId: string) => void;
  clearCurrentProject: () => void;
  deleteProject: (projectId: string) => Promise<void>;
  addTask: (
    projectId: string,
    task: Omit<Task, "id" | "createdAt">
  ) => Promise<void>;
  updateTask: (
    projectId: string,
    taskId: string,
    updates: Partial<Task>
  ) => Promise<void>;
  getTasks: (projectId: string) => Task[];
  refreshProjects: () => Promise<void>;
}

const ProjectContext = createContext<ProjectContextType | undefined>(undefined);

export function ProjectProvider({ children }: { children: ReactNode }) {
  const [projects, setProjects] = useState<Project[]>([]);
  const [currentProjectId, setCurrentProjectId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  // Transform API response to Project format
  const transformApiProject = (apiProject: ProjectApiResponse): Project => {
    // Use project_id as id, with fallback to id field
    const projectId = apiProject.project_id ?? apiProject.id ?? "";

    // Transform milestones
    const milestones = (apiProject.milestones ?? []).map((m) => ({
      id: m.id ?? m.milestone_id ?? "",
      title: m.title,
      completed: m.completed ?? m.state === "completed",
    }));

    // Transform tasks
    const tasks = (apiProject.tasks ?? []).map((t) => {
      // Normalize task ID
      const taskId = t.id ?? t.task_id ?? "";
      // Normalize status to backend enum (handle legacy values)
      const normalizedStatus = t.status === "backlog" || t.status === "todo" 
        ? "pending" 
        : t.status === "in-progress" 
        ? "in_progress" 
        : t.status === "done" 
        ? "completed" 
        : t.status as Task["status"];
      
      return {
        id: taskId,
        title: t.title,
        description: t.description ?? undefined,
        status: normalizedStatus,
        priority: t.priority ?? undefined, // Keep as number
        assigned_worker_id: t.assigned_worker_id ?? null, // Use assigned_worker_id (UUID)
        createdAt: new Date(t.created_at),
      };
    });

    return {
      id: projectId,
      name: apiProject.name,
      summary: apiProject.summary ?? apiProject.overview ?? undefined,
      description: apiProject.description ?? undefined,
      milestones,
      tasks,
      createdAt: new Date(apiProject.created_at),
      lastAccessed: apiProject.last_accessed
        ? new Date(apiProject.last_accessed)
        : new Date(apiProject.updated_at), // Fallback to updated_at if last_accessed doesn't exist
    };
  };

  // Fetch projects from API on mount
  useEffect(() => {
    async function fetchProjects() {
      setIsLoading(true);
      setError(null);
      try {
        // Retry listProjects call with exponential backoff
        const response = await retryWithBackoff(() => listProjects(), {
          maxAttempts: 3,
          initialDelay: 1000,
        });

        // Fetch full details for each project in parallel with retry
        const projectPromises = response.projects.map(
          async (projectListItem) => {
            try {
              const projectDetails = await retryWithBackoff(
                () => getProjectHandler(projectListItem.project_id),
                { maxAttempts: 2, initialDelay: 500 }
              );
              return transformApiProject(projectDetails);
            } catch (err) {
              console.error(
                `Failed to fetch project ${projectListItem.project_id} after retries:`,
                err
              );
              // Return null for failed projects - we'll filter them out
              return null;
            }
          }
        );

        const projectsData = (await Promise.all(projectPromises)).filter(
          (project): project is Project => project !== null
        );

        if (projectsData.length === 0 && response.projects.length > 0) {
          // All projects failed to load
          setError(
            new Error("Failed to load any projects. Please try refreshing.")
          );
        }

        setProjects(projectsData);
      } catch (err) {
        console.error("Failed to fetch projects after retries:", err);
        const errorMessage =
          err instanceof Error
            ? err.message
            : "Failed to load projects. Please check your connection and try again.";
        setError(new Error(errorMessage));
      } finally {
        setIsLoading(false);
      }
    }

    fetchProjects();
  }, []);

  const getCurrentProject = () => {
    if (!currentProjectId) return null;
    return projects.find((p) => p.id === currentProjectId) ?? null;
  };

  const getProjectById = (projectId: string) => {
    return projects.find((p) => p.id === projectId);
  };

  const createProject = async (data: {
    name: string;
    summary?: string;
    description?: string;
    milestones?: string[];
  }): Promise<string> => {
    try {
      // Call API to create project with retry
      const apiResponse = await retryWithBackoff(
        () =>
          createProjectApi({
            name: data.name,
            summary: data.summary,
            description: data.description,
          }),
        { maxAttempts: 3, initialDelay: 1000 }
      );

      // Transform API response to Project format
      const newProject = transformApiProject(apiResponse);

      // Create milestones if provided (with retry for each)
      if (data.milestones && data.milestones.length > 0) {
        const createdMilestones = [];
        const failedMilestones: string[] = [];

        for (const title of data.milestones) {
          try {
            const milestone = await retryWithBackoff(
              () => createProjectMilestone(newProject.id, { title }),
              { maxAttempts: 2, initialDelay: 500 }
            );
            createdMilestones.push({
              id: milestone.milestone_id ?? "",
              title: milestone.title ?? "",
              completed: milestone.completed ?? false,
            });
          } catch (err) {
            console.error(
              `Failed to create milestone "${title}" after retries:`,
              err
            );
            failedMilestones.push(title);
            // Continue with other milestones even if one fails
          }
        }

        newProject.milestones = createdMilestones;

        // Warn if some milestones failed
        if (failedMilestones.length > 0) {
          console.warn(
            `Failed to create ${failedMilestones.length} milestone(s):`,
            failedMilestones
          );
        }
      }

      // Update state with new project
      setProjects((prev) => [newProject, ...prev]);
      setCurrentProjectId(newProject.id);
      return newProject.id;
    } catch (err) {
      console.error("Failed to create project after retries:", err);
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to create project. Please check your connection and try again.";
      const error = new Error(errorMessage);
      setError(error);
      throw error;
    }
  };

  const selectProject = async (projectId: string) => {
    // Optimistic update: Update local state immediately
    const previousProjects = [...projects];
    setCurrentProjectId(projectId);
    setProjects((prev) =>
      prev.map((p) =>
        p.id === projectId ? { ...p, lastAccessed: new Date() } : p
      )
    );

    // Update last_accessed timestamp via API
    try {
      // Note: The API might not have a specific endpoint for updating last_accessed
      // We'll try to update it, but if it fails, we'll keep the optimistic update
      // In a real implementation, this might be handled by the backend automatically
      await updateProjectHandler(projectId, {
        // Empty update - backend should update last_accessed automatically
        // If backend doesn't support this, we might need a separate endpoint
      });
    } catch (err) {
      console.error("Failed to update last accessed time:", err);
      // Rollback optimistic update on failure
      setProjects(previousProjects);
      // Don't throw - selection should still work even if timestamp update fails
    }
  };

  const clearCurrentProject = () => {
    setCurrentProjectId(null);
  };

  const deleteProject = async (projectId: string) => {
    try {
      // Call API to delete project with retry
      await retryWithBackoff(() => deleteProjectApi(projectId), {
        maxAttempts: 3,
        initialDelay: 1000,
      });

      // Remove project from state
      setProjects((prev) => prev.filter((p) => p.id !== projectId));

      // Clear current project if it was deleted
      if (currentProjectId === projectId) {
        setCurrentProjectId(null);
      }
    } catch (err) {
      console.error("Failed to delete project after retries:", err);
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to delete project. Please check your connection and try again.";
      const error = new Error(errorMessage);
      setError(error);
      throw error;
    }
  };

  const addTask = async (
    projectId: string,
    task: Omit<Task, "id" | "createdAt">
  ) => {
    try {
      // Call API to create task with retry
      const apiTask = await retryWithBackoff(
        () =>
          createProjectTask(projectId, {
            title: task.title,
            description: task.description,
            status: task.status,
            priority: task.priority ?? undefined,
            assigned_worker_id: task.assigned_worker_id ?? undefined,
          }),
        { maxAttempts: 3, initialDelay: 1000 }
      );

      // Transform API response to Task format
      const taskId = apiTask.id || apiTask.task_id || '';
      const newTask: Task = {
        id: taskId, // Use id field (fallback to task_id for backward compatibility)
        title: apiTask.title,
        description: apiTask.description ?? undefined,
        status: apiTask.status as Task["status"],
        priority: apiTask.priority ?? undefined, // Keep as number
        assigned_worker_id: apiTask.assigned_worker_id ?? null,
        createdAt: new Date(apiTask.created_at),
      };

      // Update state with new task
      setProjects((prev) =>
        prev.map((p) =>
          p.id === projectId ? { ...p, tasks: [...p.tasks, newTask] } : p
        )
      );
    } catch (err) {
      console.error("Failed to create task after retries:", err);
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to create task. Please check your connection and try again.";
      const error = new Error(errorMessage);
      setError(error);
      throw error;
    }
  };

  const updateTask = async (
    projectId: string,
    taskId: string,
    updates: Partial<Task>
  ) => {
    // Optimistic update: Update local state immediately
    const previousProjects = [...projects];
    setProjects((prev) =>
      prev.map((p) =>
        p.id === projectId
          ? {
              ...p,
              tasks: p.tasks.map((t) =>
                t.id === taskId ? { ...t, ...updates } : t
              ),
            }
          : p
      )
    );

    try {
      // Call API to update task with retry
      await retryWithBackoff(
        () =>
          updateProjectTask(projectId, taskId, {
            title: updates.title,
            description: updates.description,
            status: updates.status,
            priority: updates.priority ?? undefined, // Keep as number
            assigned_worker_id: updates.assigned_worker_id ?? undefined,
          }),
        { maxAttempts: 3, initialDelay: 1000 }
      );

      // Refresh project to get latest task data with retry
      const projectDetails = await retryWithBackoff(
        () => getProjectHandler(projectId),
        { maxAttempts: 2, initialDelay: 500 }
      );
      const updatedProject = transformApiProject(projectDetails);

      // Update state with server response
      setProjects((prev) =>
        prev.map((p) => (p.id === projectId ? updatedProject : p))
      );
    } catch (err) {
      console.error("Failed to update task after retries:", err);
      // Rollback optimistic update on failure
      setProjects(previousProjects);
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to update task. Changes have been reverted.";
      const error = new Error(errorMessage);
      setError(error);
      throw error;
    }
  };

  const getTasks = (projectId: string) => {
    const project = projects.find((p) => p.id === projectId);
    return project?.tasks ?? [];
  };

  const refreshProjects = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await listProjects();

      // Fetch full details for each project in parallel
      const projectPromises = response.projects.map(async (projectListItem) => {
        try {
          const projectDetails = await getProjectHandler(
            projectListItem.project_id
          );
          return transformApiProject(projectDetails);
        } catch (err) {
          console.error(
            `Failed to fetch project ${projectListItem.project_id}:`,
            err
          );
          return null;
        }
      });

      const projectsData = (await Promise.all(projectPromises)).filter(
        (project): project is Project => project !== null
      );

      setProjects(projectsData);
    } catch (err) {
      console.error("Failed to refresh projects:", err);
      setError(
        err instanceof Error ? err : new Error("Failed to refresh projects")
      );
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <ProjectContext.Provider
      value={{
        projects,
        currentProjectId,
        isLoading,
        error,
        getCurrentProject,
        getProjectById,
        createProject,
        selectProject,
        clearCurrentProject,
        deleteProject,
        addTask,
        updateTask,
        getTasks,
        refreshProjects,
      }}
    >
      {children}
    </ProjectContext.Provider>
  );
}

export function useProjectContext() {
  const context = useContext(ProjectContext);
  if (context === undefined) {
    throw new Error("useProjectContext must be used within a ProjectProvider");
  }
  return context;
}
