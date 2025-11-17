/**
 * Zustand store for project state management
 *
 * Uses Zod schemas to validate API responses before updating state.
 * Implements optimistic updates with rollback on failure.
 *
 * Adapted from open-webui patterns for agent-agency.
 *
 * @author @darianrosebrook
 */

import { z } from "zod";
import { create } from "zustand";
import { devtools } from "zustand/middleware";
import {
  createProject as createProjectApiClient,
  deleteProject as deleteProjectApiClient,
  getProjectHandler,
  listProjects as listProjectsApiClient,
} from "../api/projects";
import { parseApiError } from "../errors";
import {
  CreateProjectRequestSchema,
  CreateTaskRequestSchema,
  ProjectResponseSchema,
  ProjectSchema,
  UpdateProjectRequestSchema,
  UpdateTaskRequestSchema,
  type CreateProjectRequest,
  type CreateTaskRequest,
  type Project,
  type ProjectTask,
  type UpdateProjectRequest,
  type UpdateTaskRequest,
} from "../schemas/project";
import { apiPatch, apiPost } from "../utils/api";
import { isCacheEnabled } from "../utils/cacheSettings";
import { env } from "../utils/env";
import { toastError, toastLoading, toastSuccess } from "../utils/toast";

interface ProjectState {
  // State
  projects: Project[];
  cacheMetadata: {
    lastFetched: number;
    ttl: number; // Time-to-live in milliseconds (60 seconds for projects)
  };
  currentProjectId: string | null;
  isLoading: boolean;
  error: Error | null;

  // Computed getters
  getCurrentProject: () => Project | null;
  getProjectById: (projectId: string) => Project | undefined;
  getTasks: (projectId: string) => ProjectTask[];

  // Actions
  setProjects: (projects: Project[]) => void;
  setCurrentProjectId: (projectId: string | null) => void;
  createProject: (data: CreateProjectRequest) => string;
  selectProject: (projectId: string) => void;
  clearCurrentProject: () => void;
  deleteProject: (projectId: string) => Promise<void>;
  addTask: (projectId: string, task: CreateTaskRequest) => void;
  updateTask: (
    projectId: string,
    taskId: string,
    updates: UpdateTaskRequest
  ) => void;

  // API actions (with Zod validation)
  fetchProjects: (forceRefresh?: boolean) => Promise<void>;
  createProjectApi: (request: CreateProjectRequest) => Promise<string>;
  updateProject: (
    projectId: string,
    request: UpdateProjectRequest
  ) => Promise<void>;
  addTaskApi: (projectId: string, task: CreateTaskRequest) => Promise<void>;
  updateTaskApi: (
    projectId: string,
    taskId: string,
    updates: UpdateTaskRequest
  ) => Promise<void>;

  // Cache management
  refreshProjects: () => Promise<void>; // Force refresh from database
  isCacheStale: () => boolean; // Check if cache needs refresh

  // Optimistic updates
  optimisticAddTask: (projectId: string, task: ProjectTask) => void;
  optimisticUpdateTask: (
    projectId: string,
    taskId: string,
    updates: Partial<ProjectTask>
  ) => void;
  rollbackOptimisticTask: (projectId: string, taskId: string) => void;

  // Error handling
  setError: (error: Error | null) => void;
  clearError: () => void;
}

/**
 * Helper to validate and transform API response
 */
function validateApiResponse<T>(
  schema: z.ZodSchema<T>,
  data: unknown,
  context: string
): T {
  try {
    return schema.parse(data);
  } catch (error) {
    if (error instanceof z.ZodError) {
      const issues = (error as z.ZodError).issues;
      console.error(`Validation error in ${context}:`, issues);
      throw new Error(
        `Invalid API response format in ${context}: ${issues
          .map((e) => e.message)
          .join(", ")}`
      );
    }
    throw error;
  }
}

export const useProjectStore = create<ProjectState>()(
  env.DEV
    ? devtools(
        (set, get) => ({
          // Initial state
          projects: [],
          cacheMetadata: {
            lastFetched: 0,
            ttl: 60000, // 60 seconds for projects
          },
          currentProjectId: null,
          isLoading: false,
          error: null,

          // Computed getters
          getCurrentProject: () => {
            const { currentProjectId, projects } = get();
            if (!currentProjectId) return null;
            return projects.find((p) => p.id === currentProjectId) ?? null;
          },

          getProjectById: (projectId: string) => {
            return get().projects.find((p) => p.id === projectId);
          },

          getTasks: (projectId: string) => {
            const project = get().projects.find((p) => p.id === projectId);
            return project?.tasks ?? [];
          },

          // Basic actions
          setProjects: (projects) => set({ projects }),
          setCurrentProjectId: (projectId) =>
            set({ currentProjectId: projectId }),

          createProject: (data) => {
            // Validate request
            const validatedRequest = CreateProjectRequestSchema.parse(data);

            const newProjectId = `project-${Date.now()}`;
            const newProject: Project = {
              id: newProjectId,
              name: validatedRequest.name,
              summary: validatedRequest.summary,
              description: validatedRequest.description,
              milestones: (validatedRequest.milestones ?? []).map(
                (title, index) => ({
                  id: `milestone-${Date.now()}-${index}`,
                  title,
                  completed: false,
                })
              ),
              tasks: [],
              createdAt: new Date(),
              lastAccessed: new Date(),
            };

            set((state) => ({
              projects: [newProject, ...state.projects],
              currentProjectId: newProjectId,
            }));

            return newProjectId;
          },

          selectProject: (projectId: string) => {
            set({ currentProjectId: projectId });

            // Update last accessed time optimistically
            set((state) => ({
              projects: state.projects.map((p) =>
                p.id === projectId ? { ...p, lastAccessed: new Date() } : p
              ),
            }));
          },

          clearCurrentProject: () => {
            set({ currentProjectId: null });
          },

          deleteProject: async (projectId: string) => {
            set({ isLoading: true, error: null });

            try {
              await deleteProjectApiClient(projectId);

              // Remove project from state
              set((state) => ({
                projects: state.projects.filter((p) => p.id !== projectId),
                currentProjectId:
                  state.currentProjectId === projectId
                    ? null
                    : state.currentProjectId,
                isLoading: false,
              }));

              toastSuccess("Project deleted successfully");
            } catch (error) {
              const appError = parseApiError(error);
              set({ error: appError, isLoading: false });
              toastError(error);
              throw appError;
            }
          },

          addTask: (projectId: string, task: CreateTaskRequest) => {
            const validatedTask = CreateTaskRequestSchema.parse(task);
            const newTask: ProjectTask = {
              ...validatedTask,
              id: `task-${Date.now()}-${Math.random()
                .toString(36)
                .substr(2, 9)}`,
              createdAt: new Date(),
            };

            set((state) => ({
              projects: state.projects.map((p) =>
                p.id === projectId ? { ...p, tasks: [...p.tasks, newTask] } : p
              ),
            }));
          },

          updateTask: (
            projectId: string,
            taskId: string,
            updates: UpdateTaskRequest
          ) => {
            set((state) => ({
              projects: state.projects.map((p) =>
                p.id === projectId
                  ? {
                      ...p,
                      tasks: p.tasks.map((t) =>
                        t.id === taskId ? { ...t, ...updates } : t
                      ),
                    }
                  : p
              ),
            }));
          },

          // Cache management
          isCacheStale: () => {
            const { cacheMetadata } = get();
            return Date.now() - cacheMetadata.lastFetched > cacheMetadata.ttl;
          },

          refreshProjects: async () => {
            // Force refresh from database
            await get().fetchProjects(true);
          },

          // API actions with Zod validation
          fetchProjects: async (forceRefresh = false) => {
            const { projects, isCacheStale } = get();

            // Check if cache is enabled and use cache if fresh and not forcing refresh
            if (
              isCacheEnabled() &&
              !forceRefresh &&
              projects.length > 0 &&
              !isCacheStale()
            ) {
              return; // Use cached data
            }

            set({ isLoading: true, error: null });
            const loadingToast = toastLoading("Loading projects...");

            try {
              const response = await listProjectsApiClient();

              // Safely extract projects array
              const projectsArray = Array.isArray(response?.projects)
                ? response.projects
                : [];

              // Fetch full details for each project in parallel
              const projectPromises = projectsArray.map(
                async (projectListItem) => {
                  try {
                    const projectDetails = await getProjectHandler(
                      projectListItem.project_id
                    );
                    const validatedProject = validateApiResponse(
                      ProjectResponseSchema,
                      projectDetails,
                      "fetchProjects"
                    );

                    // Safely extract tasks array
                    const tasksArray = Array.isArray(validatedProject?.tasks)
                      ? validatedProject.tasks
                      : [];

                    // Transform API response to Project format
                    return ProjectSchema.parse({
                      id: validatedProject.id,
                      name: validatedProject.name,
                      summary: validatedProject.summary ?? undefined,
                      description: validatedProject.description ?? undefined,
                      milestones: Array.isArray(validatedProject.milestones)
                        ? validatedProject.milestones
                        : [],
                      tasks: tasksArray.map((task) => ({
                        id: task.id,
                        title: task.title,
                        description: task.description ?? undefined,
                        status: task.status,
                        priority: task.priority ?? undefined,
                        assigned_worker_id: task.assigned_worker_id ?? undefined,
                        createdAt: task.created_at,
                      })),
                      createdAt: validatedProject.created_at,
                      lastAccessed: validatedProject.last_accessed,
                    });
                  } catch (err) {
                    console.error(
                      `Failed to fetch project ${projectListItem.project_id}:`,
                      err
                    );
                    return null;
                  }
                }
              );

              const projectsData = (await Promise.all(projectPromises)).filter(
                (project): project is Project => project !== null
              );

              // Update cache metadata
              set({
                projects: projectsData,
                isLoading: false,
                cacheMetadata: {
                  lastFetched: Date.now(),
                  ttl: 60000, // 60 seconds
                },
              });
              loadingToast();
            } catch (error) {
              const appError = parseApiError(error);
              set({ error: appError, isLoading: false });
              loadingToast();
              toastError(error);
              throw appError;
            }
          },

          createProjectApi: async (request) => {
            // Validate request
            const validatedRequest = CreateProjectRequestSchema.parse(request);

            set({ isLoading: true, error: null });

            try {
              // Use API client function
              const apiResponse = await createProjectApiClient({
                name: validatedRequest.name,
                summary: validatedRequest.summary,
                description: validatedRequest.description,
              });

              const validatedProject = validateApiResponse(
                ProjectResponseSchema,
                apiResponse,
                "createProjectApi"
              );

              // Transform to Project format
              const newProject: Project = ProjectSchema.parse({
                id: validatedProject.id,
                name: validatedProject.name,
                summary: validatedProject.summary ?? undefined,
                description: validatedProject.description ?? undefined,
                milestones: Array.isArray(validatedProject.milestones)
                  ? validatedProject.milestones
                  : [],
                tasks: Array.isArray(validatedProject.tasks)
                  ? validatedProject.tasks.map((task) => ({
                      id: task.id,
                      title: task.title,
                      description: task.description ?? undefined,
                      status: task.status,
                      priority: task.priority ?? undefined,
                      assigned_worker_id: task.assigned_worker_id ?? undefined,
                      createdAt: task.created_at,
                    }))
                  : [],
                createdAt: validatedProject.created_at,
                lastAccessed: validatedProject.last_accessed,
              });

              set((state) => ({
                projects: [newProject, ...state.projects],
                currentProjectId: validatedProject.id,
                isLoading: false,
                // Update cache metadata
                cacheMetadata: {
                  lastFetched: Date.now(),
                  ttl: 60000,
                },
              }));

              toastSuccess("Project created successfully");
              return validatedProject.id;
            } catch (error) {
              const appError = parseApiError(error);
              set({ error: appError, isLoading: false });
              toastError(error);
              throw appError;
            }
          },

          updateProject: async (
            projectId: string,
            request: UpdateProjectRequest
          ) => {
            // Validate request
            const validatedRequest = UpdateProjectRequestSchema.parse(request);

            set({ isLoading: true, error: null });

            try {
              const apiUrl = env.API_URL;
              const data = await apiPatch<unknown>(
                `${apiUrl}/api/v1/projects/${projectId}`,
                validatedRequest
              );

              const validatedProject = validateApiResponse(
                ProjectResponseSchema,
                data,
                "updateProject"
              );

              // Update project in state
              set((state) => ({
                projects: state.projects.map((p) =>
                  p.id === projectId
                    ? ProjectSchema.parse({
                        ...p,
                        ...validatedProject,
                        id: p.id, // Preserve ID
                        createdAt: p.createdAt, // Preserve original created date
                        lastAccessed: validatedProject.last_accessed,
                      })
                    : p
                ),
                isLoading: false,
                // Update cache metadata
                cacheMetadata: {
                  lastFetched: Date.now(),
                  ttl: 60000,
                },
              }));
            } catch (error) {
              const appError = parseApiError(error);
              set({ error: appError, isLoading: false });
              toastError(error);
              throw appError;
            }
          },

          addTaskApi: async (projectId: string, task: CreateTaskRequest) => {
            // Validate request
            const validatedTask = CreateTaskRequestSchema.parse(task);

            // Optimistic update
            const optimisticTask: ProjectTask = {
              ...validatedTask,
              id: `temp-${Date.now()}`,
              createdAt: new Date(),
            };
            get().optimisticAddTask(projectId, optimisticTask);

            try {
              const apiUrl = env.API_URL;
              const data = await apiPost<unknown>(
                `${apiUrl}/api/v1/projects/${projectId}/tasks`,
                validatedTask
              );

              const validatedResponse = z
                .object({
                  id: z.string(),
                  title: z.string(),
                  description: z.string().nullable().optional(),
                  status: z.enum(["backlog", "todo", "in-progress", "done"]),
                  priority: z.string().nullable().optional(),
                  assigned_worker_id: z.string().nullable().optional(),
                  created_at: z.string().transform((str) => new Date(str)),
                })
                .parse(data);

              // Replace optimistic task with validated one
              const finalTask: ProjectTask = {
                id: validatedResponse.id,
                title: validatedResponse.title,
                description: validatedResponse.description ?? undefined,
                status: validatedResponse.status,
                priority: typeof validatedResponse.priority === 'number' ? validatedResponse.priority : null,
                assigned_worker_id: validatedResponse.assigned_worker_id ?? undefined,
                createdAt: validatedResponse.created_at,
              };

              set((state) => ({
                projects: state.projects.map((p) =>
                  p.id === projectId
                    ? {
                        ...p,
                        tasks: p.tasks.map((t) =>
                          t.id === optimisticTask.id ? finalTask : t
                        ),
                      }
                    : p
                ),
              }));
            } catch (error) {
              // Rollback optimistic update
              get().rollbackOptimisticTask(projectId, optimisticTask.id);
              const appError = parseApiError(error);
              set({ error: appError });
              toastError(error);
              throw appError;
            }
          },

          updateTaskApi: async (
            projectId: string,
            taskId: string,
            updates: UpdateTaskRequest
          ) => {
            // Validate request
            const validatedUpdates = UpdateTaskRequestSchema.parse(updates);

            // Store original task for rollback
            const project = get().projects.find((p) => p.id === projectId);
            const originalTask = project?.tasks.find((t) => t.id === taskId);
            if (!originalTask) {
              throw new Error(`Task ${taskId} not found`);
            }

            // Optimistic update
            get().optimisticUpdateTask(projectId, taskId, validatedUpdates);

            try {
              const apiUrl = env.API_URL;
              const data = await apiPatch<unknown>(
                `${apiUrl}/api/v1/projects/${projectId}/tasks/${taskId}`,
                validatedUpdates
              );

              const validatedResponse = z
                .object({
                  id: z.string(),
                  title: z.string(),
                  description: z.string().nullable().optional(),
                  status: z.enum(["backlog", "todo", "in-progress", "done"]),
                  priority: z.string().nullable().optional(),
                  assigned_worker_id: z.string().nullable().optional(),
                  created_at: z.string().transform((str) => new Date(str)),
                })
                .parse(data);

              // Update with validated response
              set((state) => ({
                projects: state.projects.map((p) =>
                  p.id === projectId
                    ? {
                        ...p,
                        tasks: p.tasks.map((t) =>
                          t.id === taskId
                            ? {
                                id: validatedResponse.id,
                                title: validatedResponse.title,
                                description:
                                  validatedResponse.description ?? undefined,
                                status: validatedResponse.status,
                                priority:
                                  validatedResponse.priority ?? undefined,
                                assigned_worker_id:
                                  validatedResponse.assigned_worker_id ?? undefined,
                                createdAt: validatedResponse.created_at,
                              }
                            : t
                        ),
                      }
                    : p
                ),
              }) as any);
            } catch (error) {
              // Rollback optimistic update
              if (originalTask) {
                get().optimisticUpdateTask(projectId, taskId, originalTask);
              }
              const appError = parseApiError(error);
              set({ error: appError });
              toastError(error);
              throw appError;
            }
          },

          // Optimistic updates
          optimisticAddTask: (projectId: string, task: ProjectTask) => {
            set((state) => ({
              projects: state.projects.map((p) =>
                p.id === projectId ? { ...p, tasks: [...p.tasks, task] } : p
              ),
            }));
          },

          optimisticUpdateTask: (
            projectId: string,
            taskId: string,
            updates: Partial<ProjectTask>
          ) => {
            set((state) => ({
              projects: state.projects.map((p) =>
                p.id === projectId
                  ? {
                      ...p,
                      tasks: p.tasks.map((t) =>
                        t.id === taskId ? { ...t, ...updates } : t
                      ),
                    }
                  : p
              ),
            }));
          },

          rollbackOptimisticTask: (projectId: string, taskId: string) => {
            set((state) => ({
              projects: state.projects.map((p) =>
                p.id === projectId
                  ? {
                      ...p,
                      tasks: p.tasks.filter((t) => t.id !== taskId),
                    }
                  : p
              ),
            }));
          },

          // Error handling
          setError: (error) => set({ error }),
          clearError: () => set({ error: null }),
        }),
        { name: "ProjectStore" }
      )
    : (set, get) => ({
        // Initial state
        projects: [],
        cacheMetadata: {
          lastFetched: 0,
          ttl: 60000, // 60 seconds for projects
        },
        currentProjectId: null,
        isLoading: false,
        error: null,

        // Computed getters
        getCurrentProject: () => {
          const { currentProjectId, projects } = get();
          if (!currentProjectId) return null;
          return projects.find((p) => p.id === currentProjectId) ?? null;
        },

        getProjectById: (projectId: string) => {
          return get().projects.find((p) => p.id === projectId);
        },

        getTasks: (projectId: string) => {
          const project = get().projects.find((p) => p.id === projectId);
          return project?.tasks ?? [];
        },

        // Basic actions
        setProjects: (projects) => set({ projects }),
        setCurrentProjectId: (projectId) =>
          set({ currentProjectId: projectId }),

        createProject: (data) => {
          // Validate request
          const validatedRequest = CreateProjectRequestSchema.parse(data);

          const newProjectId = `project-${Date.now()}`;
          const newProject: Project = {
            id: newProjectId,
            name: validatedRequest.name,
            summary: validatedRequest.summary,
            description: validatedRequest.description,
            milestones: (validatedRequest.milestones ?? []).map(
              (title, index) => ({
                id: `milestone-${Date.now()}-${index}`,
                title,
                completed: false,
              })
            ),
            tasks: [],
            createdAt: new Date(),
            lastAccessed: new Date(),
          };

          set((state) => ({
            projects: [newProject, ...state.projects],
            currentProjectId: newProjectId,
          }));

          return newProjectId;
        },

        selectProject: (projectId: string) => {
          set({ currentProjectId: projectId });

          // Update last accessed time optimistically
          set((state) => ({
            projects: state.projects.map((p) =>
              p.id === projectId ? { ...p, lastAccessed: new Date() } : p
            ),
          }));
        },

        clearCurrentProject: () => {
          set({ currentProjectId: null });
        },

        deleteProject: async (projectId: string) => {
          set({ isLoading: true, error: null });

          try {
            await deleteProjectApiClient(projectId);

            // Remove project from state
            set((state) => ({
              projects: state.projects.filter((p) => p.id !== projectId),
              currentProjectId:
                state.currentProjectId === projectId
                  ? null
                  : state.currentProjectId,
              isLoading: false,
            }));

            toastSuccess("Project deleted successfully");
          } catch (error) {
            const appError = parseApiError(error);
            set({ error: appError, isLoading: false });
            toastError(error);
            throw appError;
          }
        },

        addTask: (projectId: string, task: CreateTaskRequest) => {
          const validatedTask = CreateTaskRequestSchema.parse(task);
          const newTask: ProjectTask = {
            ...validatedTask,
            id: `task-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
            createdAt: new Date(),
          };

          set((state) => ({
            projects: state.projects.map((p) =>
              p.id === projectId ? { ...p, tasks: [...p.tasks, newTask] } : p
            ),
          }));
        },

        updateTask: (
          projectId: string,
          taskId: string,
          updates: UpdateTaskRequest
        ) => {
          set((state) => ({
            projects: state.projects.map((p) =>
              p.id === projectId
                ? {
                    ...p,
                    tasks: p.tasks.map((t) =>
                      t.id === taskId ? { ...t, ...updates } : t
                    ),
                  }
                : p
            ),
          }));
        },

        // Cache management
        isCacheStale: () => {
          const { cacheMetadata } = get();
          return Date.now() - cacheMetadata.lastFetched > cacheMetadata.ttl;
        },

        refreshProjects: async () => {
          // Force refresh from database
          await get().fetchProjects(true);
        },

        // API actions with Zod validation
        fetchProjects: async (forceRefresh = false) => {
          const { projects, isCacheStale } = get();

          // Check if cache is enabled and use cache if fresh and not forcing refresh
          if (
            isCacheEnabled() &&
            !forceRefresh &&
            projects.length > 0 &&
            !isCacheStale()
          ) {
            return; // Use cached data
          }

          set({ isLoading: true, error: null });
          const loadingToast = toastLoading("Loading projects...");

          try {
            const response = await listProjectsApiClient();

            // Safely extract projects array
            const projectsArray = Array.isArray(response?.projects)
              ? response.projects
              : [];

            // Fetch full details for each project in parallel
            const projectPromises = projectsArray.map(
              async (projectListItem) => {
                try {
                  const projectDetails = await getProjectHandler(
                    projectListItem.project_id
                  );
                  const validatedProject = validateApiResponse(
                    ProjectResponseSchema,
                    projectDetails,
                    "fetchProjects"
                  );

                  // Safely extract tasks array
                  const tasksArray = Array.isArray(validatedProject?.tasks)
                    ? validatedProject.tasks
                    : [];

                  // Transform API response to Project format
                  return ProjectSchema.parse({
                    id: validatedProject.id,
                    name: validatedProject.name,
                    summary: validatedProject.summary ?? undefined,
                    description: validatedProject.description ?? undefined,
                    milestones: Array.isArray(validatedProject.milestones)
                      ? validatedProject.milestones
                      : [],
                    tasks: tasksArray.map((task) => ({
                      id: task.id,
                      title: task.title,
                      description: task.description ?? undefined,
                      status: task.status,
                      priority: task.priority ?? undefined,
                      assigned_worker_id: task.assigned_worker_id ?? undefined,
                      createdAt: task.created_at,
                    })),
                    createdAt: validatedProject.created_at,
                    lastAccessed: validatedProject.last_accessed,
                  });
                } catch (err) {
                  console.error(
                    `Failed to fetch project ${projectListItem.project_id}:`,
                    err
                  );
                  return null;
                }
              }
            );

            const projectsData = (await Promise.all(projectPromises)).filter(
              (project): project is Project => project !== null
            );

            // Update cache metadata
            set({
              projects: projectsData,
              isLoading: false,
              cacheMetadata: {
                lastFetched: Date.now(),
                ttl: 60000, // 60 seconds
              },
            });
            loadingToast();
          } catch (error) {
            const appError = parseApiError(error);
            set({ error: appError, isLoading: false });
            loadingToast();
            toastError(error);
            throw appError;
          }
        },

        createProjectApi: async (request) => {
          // Validate request
          const validatedRequest = CreateProjectRequestSchema.parse(request);

          set({ isLoading: true, error: null });

          try {
            // Use API client function
            const apiResponse = await createProjectApiClient({
              name: validatedRequest.name,
              summary: validatedRequest.summary,
              description: validatedRequest.description,
            });

            const validatedProject = validateApiResponse(
              ProjectResponseSchema,
              apiResponse,
              "createProjectApi"
            );

            // Safely extract tasks array
            const tasksArray = Array.isArray(validatedProject?.tasks)
              ? validatedProject.tasks
              : [];

            // Transform to Project format
            const newProject: Project = ProjectSchema.parse({
              id: validatedProject.id,
              name: validatedProject.name,
              summary: validatedProject.summary ?? undefined,
              description: validatedProject.description ?? undefined,
              milestones: Array.isArray(validatedProject.milestones)
                ? validatedProject.milestones
                : [],
              tasks: tasksArray.map((task) => ({
                id: task.id,
                title: task.title,
                description: task.description ?? undefined,
                status: task.status,
                priority: task.priority ?? null,
                assigned_worker_id: task.assigned_worker_id ?? undefined,
                createdAt: task.created_at,
              })),
              createdAt: validatedProject.created_at,
              lastAccessed: validatedProject.last_accessed,
            });

            set((state) => ({
              projects: [newProject, ...state.projects],
              currentProjectId: validatedProject.id,
              isLoading: false,
              // Update cache metadata
              cacheMetadata: {
                lastFetched: Date.now(),
                ttl: 60000,
              },
            }));

            toastSuccess("Project created successfully");
            return validatedProject.id;
          } catch (error) {
            const appError = parseApiError(error);
            set({ error: appError, isLoading: false });
            toastError(error);
            throw appError;
          }
        },

        updateProject: async (
          projectId: string,
          request: UpdateProjectRequest
        ) => {
          // Validate request
          const validatedRequest = UpdateProjectRequestSchema.parse(request);

          set({ isLoading: true, error: null });

          try {
            const apiUrl =
              process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";
            const data = await apiPatch<unknown>(
              `${apiUrl}/api/v1/projects/${projectId}`,
              validatedRequest
            );

            const validatedProject = validateApiResponse(
              ProjectResponseSchema,
              data,
              "updateProject"
            );

            // Update project in state
            set((state) => ({
              projects: state.projects.map((p) =>
                p.id === projectId
                  ? ProjectSchema.parse({
                      ...p,
                      ...validatedProject,
                      id: p.id, // Preserve ID
                      createdAt: p.createdAt, // Preserve original created date
                      lastAccessed: validatedProject.last_accessed,
                    })
                  : p
              ),
              isLoading: false,
              // Update cache metadata
              cacheMetadata: {
                lastFetched: Date.now(),
                ttl: 60000,
              },
            }));
          } catch (error) {
            const appError = parseApiError(error);
            set({ error: appError, isLoading: false });
            toastError(error);
            throw appError;
          }
        },

        addTaskApi: async (projectId: string, task: CreateTaskRequest) => {
          // Validate request
          const validatedTask = CreateTaskRequestSchema.parse(task);

          // Optimistic update
          const optimisticTask: ProjectTask = {
            ...validatedTask,
            id: `temp-${Date.now()}`,
            createdAt: new Date(),
          };
          get().optimisticAddTask(projectId, optimisticTask);

          try {
            const apiUrl =
              process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";
            const data = await apiPost<unknown>(
              `${apiUrl}/api/v1/projects/${projectId}/tasks`,
              validatedTask
            );

            const validatedResponse = z
              .object({
                id: z.string(),
                title: z.string(),
                description: z.string().nullable().optional(),
                status: z.enum(["backlog", "todo", "in-progress", "done"]),
                priority: z.string().nullable().optional(),
                assigned_worker_id: z.string().nullable().optional(),
                created_at: z.string().transform((str) => new Date(str)),
              })
              .parse(data);

            // Replace optimistic task with validated one
            const finalTask: ProjectTask = {
              id: validatedResponse.id,
              title: validatedResponse.title,
              description: validatedResponse.description ?? undefined,
              status: validatedResponse.status,
              priority: typeof validatedResponse.priority === 'number' ? validatedResponse.priority : undefined,
              assigned_worker_id: validatedResponse.assigned_worker_id ?? undefined,
              createdAt: validatedResponse.created_at,
            };

            set((state) => ({
              projects: state.projects.map((p) =>
                p.id === projectId
                  ? {
                      ...p,
                      tasks: p.tasks.map((t) =>
                        t.id === optimisticTask.id ? finalTask : t
                      ),
                    }
                  : p
              ),
            }));
          } catch (error) {
            // Rollback optimistic update
            get().rollbackOptimisticTask(projectId, optimisticTask.id);
            const appError = parseApiError(error);
            set({ error: appError });
            toastError(error);
            throw appError;
          }
        },

        updateTaskApi: async (
          projectId: string,
          taskId: string,
          updates: UpdateTaskRequest
        ) => {
          // Validate request
          const validatedUpdates = UpdateTaskRequestSchema.parse(updates);

          // Store original task for rollback
          const project = get().projects.find((p) => p.id === projectId);
          const originalTask = project?.tasks.find((t) => t.id === taskId);
          if (!originalTask) {
            throw new Error(`Task ${taskId} not found`);
          }

          // Optimistic update
          get().optimisticUpdateTask(projectId, taskId, validatedUpdates);

          try {
            const apiUrl =
              process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";
            const data = await apiPatch<unknown>(
              `${apiUrl}/api/v1/projects/${projectId}/tasks/${taskId}`,
              validatedUpdates
            );

            const validatedResponse = z
              .object({
                id: z.string(),
                title: z.string(),
                description: z.string().nullable().optional(),
                status: z.enum(["backlog", "todo", "in-progress", "done"]),
                priority: z.string().nullable().optional(),
                assigned_worker_id: z.string().nullable().optional(),
                created_at: z.string().transform((str) => new Date(str)),
              })
              .parse(data);

            // Update with validated response
            set((state) => ({
              projects: state.projects.map((p) =>
                p.id === projectId
                  ? {
                      ...p,
                      tasks: p.tasks.map((t) =>
                        t.id === taskId
                          ? {
                              id: validatedResponse.id,
                              title: validatedResponse.title,
                              description:
                                validatedResponse.description ?? undefined,
                              status: validatedResponse.status,
                              priority: typeof validatedResponse.priority === 'number' ? validatedResponse.priority : undefined,
                              assigned_worker_id: validatedResponse.assigned_worker_id ?? undefined,
                              createdAt: validatedResponse.created_at,
                            }
                          : t
                      ),
                    }
                  : p
              ),
            }));
          } catch (error) {
            // Rollback optimistic update
            if (originalTask) {
              get().optimisticUpdateTask(projectId, taskId, originalTask);
            }
            const appError = parseApiError(error);
            set({ error: appError });
            toastError(error);
            throw appError;
          }
        },

        // Optimistic updates
        optimisticAddTask: (projectId: string, task: ProjectTask) => {
          set((state) => ({
            projects: state.projects.map((p) =>
              p.id === projectId ? { ...p, tasks: [...p.tasks, task] } : p
            ),
          }));
        },

        optimisticUpdateTask: (
          projectId: string,
          taskId: string,
          updates: Partial<ProjectTask>
        ) => {
          set((state) => ({
            projects: state.projects.map((p) =>
              p.id === projectId
                ? {
                    ...p,
                    tasks: p.tasks.map((t) =>
                      t.id === taskId ? { ...t, ...updates } : t
                    ),
                  }
                : p
            ),
          }));
        },

        rollbackOptimisticTask: (projectId: string, taskId: string) => {
          set((state) => ({
            projects: state.projects.map((p) =>
              p.id === projectId
                ? {
                    ...p,
                    tasks: p.tasks.filter((t) => t.id !== taskId),
                  }
                : p
            ),
          }));
        },

        // Error handling
        setError: (error) => set({ error }),
        clearError: () => set({ error: null }),
      })
);
