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
  const [projects, setProjects] = useState<Project[]>([]);
  const [currentProjectId, setCurrentProjectId] = useState<string | null>(null);

  const getCurrentProject = () => {
    if (!currentProjectId) return null;
    return projects.find((p) => p.id === currentProjectId) || null;
  };

  const createProject = (data: {
    name: string;
    summary?: string;
    description?: string;
    milestones?: string[];
  }) => {
    const newProjectId = `project-${Date.now()}`;
    const newProject: Project = {
      id: newProjectId,
      name: data.name,
      summary: data.summary,
      description: data.description,
      milestones: (data.milestones || []).map((title, index) => ({
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
    return project?.tasks || [];
  };

  return (
    <ProjectContext.Provider
      value={{
        projects,
        currentProjectId,
        getCurrentProject,
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
