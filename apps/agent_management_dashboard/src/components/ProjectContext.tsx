"use client";

import { createContext, useContext, useState, ReactNode } from "react";

export interface Milestone {
  id: string;
  title: string;
  completed: boolean;
}

export interface Task {
  id: string;
  title: string;
  description?: string;
  status: "backlog" | "todo" | "in-progress" | "done";
  priority?: string;
  assignee?: string;
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
  getCurrentProject: () => Project | null;
  getProjectById: (projectId: string) => Project | undefined;
  createProject: (data: {
    name: string;
    summary?: string;
    description?: string;
    milestones?: string[];
  }) => string;
  selectProject: (projectId: string) => void;
  clearCurrentProject: () => void;
  addTask: (projectId: string, task: Omit<Task, "id" | "createdAt">) => void;
  updateTask: (
    projectId: string,
    taskId: string,
    updates: Partial<Task>
  ) => void;
  getTasks: (projectId: string) => Task[];
}

const ProjectContext = createContext<ProjectContextType | undefined>(undefined);

export function ProjectProvider({ children }: { children: ReactNode }) {
  // TODO: Replace local state with data from v3 PostgreSQL database with the following requirements:
  // 1. Project data fetching: Load projects from PostgreSQL database
  //    - Data source: GET /api/projects endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `projects` table via `iterations/v3/data-infrastructure`
  //    - Include project metadata: name, summary, description, milestones, tasks, timestamps
  // 2. Project creation: Persist new projects to database
  //    - Data source: POST /api/projects endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `projects` table
  //    - Handle validation and error responses from API
  // 3. Project updates: Sync project modifications to database
  //    - Data source: PATCH /api/projects/:id endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Update last_accessed timestamp on project selection
  //    - Handle optimistic updates with rollback on failure
  const [projects, setProjects] = useState<Project[]>([]);
  const [currentProjectId, setCurrentProjectId] = useState<string | null>(null);

  const getCurrentProject = () => {
    if (!currentProjectId) return null;
    return projects.find((p) => p.id === currentProjectId) ?? null;
  };

  const getProjectById = (projectId: string) => {
    return projects.find((p) => p.id === projectId);
  };

  const createProject = (data: {
    name: string;
    summary?: string;
    description?: string;
    milestones?: string[];
  }) => {
    // TODO: Replace local state update with API call to v3 data-infrastructure with the following requirements:
    // 1. API call: POST /api/projects endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
    //    - Database table: PostgreSQL `projects` table
    //    - Request body: { name, summary, description, milestones }
    //    - Handle API response with created project data including server-generated ID
    // 2. Error handling: Handle API errors and validation failures
    //    - Display user-friendly error messages
    //    - Rollback local state if API call fails
    // 3. State synchronization: Update local state only after successful API response
    //    - Use server-returned project ID instead of client-generated ID
    //    - Set created_at and last_accessed timestamps from server response
    const newProjectId = `project-${Date.now()}`;
    const newProject: Project = {
      id: newProjectId,
      name: data.name,
      summary: data.summary,
      description: data.description,
      milestones: (data.milestones ?? []).map((title, index) => ({
        id: `milestone-${Date.now()}-${index}`,
        title,
        completed: false,
      })),
      tasks: [],
      createdAt: new Date(),
      lastAccessed: new Date(),
    };

    setProjects((prev) => [newProject, ...prev]);
    setCurrentProjectId(newProjectId);
    return newProjectId;
  };

  const selectProject = (projectId: string) => {
    setCurrentProjectId(projectId);

    // TODO: Update last accessed time via API call with the following requirements:
    // 1. API call: PATCH /api/projects/:id endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
    //    - Database table: PostgreSQL `projects` table
    //    - Update column: `last_accessed` timestamp
    //    - Use current timestamp from server to ensure consistency
    // 2. Optimistic update: Update local state immediately for better UX
    //    - Rollback if API call fails
    //    - Handle network errors gracefully
    // Update last accessed time
    setProjects((prev) =>
      prev.map((p) =>
        p.id === projectId ? { ...p, lastAccessed: new Date() } : p
      )
    );
  };

  const clearCurrentProject = () => {
    setCurrentProjectId(null);
  };

  const addTask = (projectId: string, task: Omit<Task, "id" | "createdAt">) => {
    // TODO: Replace local state update with API call to v3 data-infrastructure with the following requirements:
    // 1. API call: POST /api/projects/:projectId/tasks endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
    //    - Database table: PostgreSQL `tasks` table
    //    - Request body: { title, description, status, priority, assignee }
    //    - Handle API response with created task data including server-generated ID
    // 2. Error handling: Handle API errors and validation failures
    //    - Display user-friendly error messages
    //    - Rollback local state if API call fails
    // 3. State synchronization: Update local state only after successful API response
    //    - Use server-returned task ID instead of client-generated ID
    //    - Set createdAt timestamp from server response
    const newTask: Task = {
      ...task,
      id: `task-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      createdAt: new Date(),
    };

    setProjects((prev) =>
      prev.map((p) =>
        p.id === projectId ? { ...p, tasks: [...p.tasks, newTask] } : p
      )
    );
  };

  const updateTask = (
    projectId: string,
    taskId: string,
    updates: Partial<Task>
  ) => {
    // TODO: Replace local state update with API call to v3 data-infrastructure with the following requirements:
    // 1. API call: PATCH /api/projects/:projectId/tasks/:taskId endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
    //    - Database table: PostgreSQL `tasks` table
    //    - Request body: Partial task updates (status, priority, assignee, description, etc.)
    //    - Handle API response with updated task data
    // 2. Error handling: Handle API errors and validation failures
    //    - Display user-friendly error messages
    //    - Rollback local state if API call fails
    // 3. Optimistic updates: Update local state immediately for better UX
    //    - Rollback if API call fails
    //    - Handle concurrent update conflicts
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
  };

  const getTasks = (projectId: string) => {
    const project = projects.find((p) => p.id === projectId);
    return project?.tasks ?? [];
  };

  return (
    <ProjectContext.Provider
      value={{
        projects,
        currentProjectId,
        getCurrentProject,
        getProjectById,
        createProject,
        selectProject,
        clearCurrentProject,
        addTask,
        updateTask,
        getTasks,
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
