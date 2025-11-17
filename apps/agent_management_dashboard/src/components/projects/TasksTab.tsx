"use client";

/**
 * Tasks Tab - Kanban Board with Full CRUD Operations
 *
 * Features:
 * - Drag and drop tasks between columns
 * - Create, Read, Update, Delete tasks
 * - Comments on tasks (visible to agents as context)
 *
 * @author @darianrosebrook
 */

import { useState, useMemo, useEffect, useCallback } from "react";
import { KanbanBoard } from "../composers/kanban/KanbanBoard";
import { NewTaskModal } from "./TaskModal";
import { EditTaskModal } from "./EditTaskModal";
import { CommentsModal } from "./CommentsModal";
import { DeleteTaskDialog } from "./DeleteTaskDialog";
import { useProjectContext } from "./ProjectContext";
import {
  getProjectTasks,
  createProjectTask,
  updateProjectTask,
  deleteProjectTask,
  type ProjectTask,
} from "../../lib/api/projects";
import { getTaskComments } from "../../lib/api/comments";
import { getAgents, type Agent } from "../../lib/api/agents";
import {
  getStatusLabel,
  type BackendTaskStatus,
  mapLegacyStatusToBackend,
} from "../../lib/utils/taskStatus";
import type {
  KanbanColumnConfig,
  KanbanStatusTag,
  KanbanMetadata,
  KanbanStatus,
} from "../composers/kanban/types";
import styles from "./TasksTab.module.scss";

const STATUS_CONFIG: Record<string, { title: string }> = {
  backlog: { title: "Backlog" },
  todo: { title: "To Do" },
  "in-progress": { title: "In Progress" },
  done: { title: "Done" },
};

const PRIORITY_COLORS: Record<string, { bg: string; text: string }> = {
  high: { bg: "#3a2f1f", text: "#ff9f43" },
  medium: { bg: "#1f2d3a", text: "#54a0ff" },
};

/**
 * Map Kanban status (UI) to backend API status
 * 
 * Kanban uses simplified statuses for UI, maps to backend enum values
 */
const mapKanbanStatusToApi = (status: KanbanStatus): BackendTaskStatus => {
  const statusMap: Record<KanbanStatus, BackendTaskStatus> = {
    backlog: "pending",
    todo: "pending",
    "in-progress": "in_progress",
    done: "completed",
  };
  return statusMap[status];
};

/**
 * Map backend API status to Kanban status (UI)
 * 
 * Handles all backend status values: pending, in_progress, paused, completed, cancelled, failed
 */
const mapApiStatusToKanban = (apiStatus: string): KanbanStatus => {
  // Normalize to backend status first
  const backendStatus = mapLegacyStatusToBackend(apiStatus);
  
  // Map backend statuses to Kanban statuses
  switch (backendStatus) {
    case "completed":
      return "done";
    case "in_progress":
    case "paused": // Paused tasks show as in-progress in Kanban
      return "in-progress";
    case "pending":
      return "todo";
    case "cancelled":
    case "failed":
      // Failed/cancelled tasks show as done (completed column) for now
      // Could add a separate column later if needed
      return "done";
    default:
      return "todo";
  }
};

// Map priority string to number
const mapPriorityToNumber = (priority?: string): number | undefined => {
  if (!priority) return undefined;
  if (priority === "high") return 3;
  if (priority === "medium") return 2;
  return 1;
};

// Map priority number to string
const mapPriorityToString = (
  priority?: number | null
): "low" | "medium" | "high" | undefined => {
  if (priority === null || priority === undefined) return undefined;
  if (priority >= 3) return "high";
  if (priority >= 2) return "medium";
  return "low";
};

interface Task {
  id: string;
  title: string;
  description?: string;
  status: KanbanStatus;
  priority?: "low" | "medium" | "high";
  assigned_worker_id?: string | null;
  commentCount?: number;
}

export function TasksTab() {
  const [isNewTaskModalOpen, setIsNewTaskModalOpen] = useState(false);
  const [isEditTaskModalOpen, setIsEditTaskModalOpen] = useState(false);
  const [isCommentsModalOpen, setIsCommentsModalOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState<KanbanStatus>("backlog");
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [taskToDelete, setTaskToDelete] = useState<Task | null>(null);
  const { currentProjectId } = useProjectContext();
  const [tasks, setTasks] = useState<Task[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  // Fetch tasks and agents
  const fetchTasks = useCallback(async () => {
    if (!currentProjectId) {
      setTasks([]);
      return;
    }

    setIsLoading(true);
    try {
      const [tasksResponse, agentsData] = await Promise.all([
        getProjectTasks(currentProjectId),
        getAgents().catch(() => []),
      ]);

      // Ensure agents is an array before setting
      const agentsArray = Array.isArray(agentsData) 
        ? agentsData 
        : (agentsData?.agents || []);
      setAgents(agentsArray);

      // Fetch comment counts for each task
      const tasksWithComments = await Promise.all(
        tasksResponse.tasks.map(async (task) => {
          try {
            // Use id field (fallback to task_id for backward compatibility)
            const taskId = task.id || task.task_id || '';
            const commentsResponse = await getTaskComments(taskId);
            return {
              ...task,
              commentCount: commentsResponse.comments.length,
            };
          } catch {
            return { ...task, commentCount: 0 };
          }
        })
      );

      // Transform API tasks to local Task format
      const transformedTasks: Task[] = tasksWithComments.map((task) => {
        const status = mapApiStatusToKanban(task.status);
        const priority = mapPriorityToString(task.priority);
        const taskId = task.id || task.task_id || '';

        return {
          id: taskId, // Use normalized id field
          title: task.title,
          description: task.description ?? undefined,
          status,
          priority,
          assigned_worker_id: task.assigned_worker_id ?? null,
          commentCount: task.commentCount ?? 0,
        };
      });

      setTasks(transformedTasks);
    } catch (err) {
      console.error("Failed to fetch project tasks:", err);
      setTasks([]);
    } finally {
      setIsLoading(false);
    }
  }, [currentProjectId]);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  // Handle task creation
  const handleCreateTask = useCallback(
    async (data: {
      title: string;
      description?: string;
      status: KanbanStatus;
      priority?: string;
    }) => {
      if (!currentProjectId) return;

      setIsSaving(true);
      try {
        const apiStatus = mapKanbanStatusToApi(data.status);
        const priorityNumber = mapPriorityToNumber(data.priority);

        await createProjectTask(currentProjectId, {
          title: data.title,
          description: data.description,
          status: apiStatus,
          priority: priorityNumber,
        });

        // Refresh tasks
        await fetchTasks();
        setIsNewTaskModalOpen(false);
      } catch (err) {
        console.error("Failed to create task:", err);
        alert(
          `Failed to create task: ${
            err instanceof Error ? err.message : "Unknown error"
          }`
        );
      } finally {
        setIsSaving(false);
      }
    },
    [currentProjectId, fetchTasks]
  );

  // Handle task update
  const handleUpdateTask = useCallback(
    async (data: {
      title: string;
      description?: string;
      status: KanbanStatus;
      priority?: string;
    }) => {
      if (!currentProjectId || !selectedTask) return;

      setIsSaving(true);
      try {
        const apiStatus = mapKanbanStatusToApi(data.status);
        const priorityNumber = mapPriorityToNumber(data.priority);

        await updateProjectTask(currentProjectId, selectedTask.id, {
          title: data.title,
          description: data.description,
          status: apiStatus,
          priority: priorityNumber,
        });

        // Refresh tasks
        await fetchTasks();
        setIsEditTaskModalOpen(false);
        setSelectedTask(null);
      } catch (err) {
        console.error("Failed to update task:", err);
        alert(
          `Failed to update task: ${
            err instanceof Error ? err.message : "Unknown error"
          }`
        );
      } finally {
        setIsSaving(false);
      }
    },
    [currentProjectId, selectedTask, fetchTasks]
  );

  // Handle task deletion
  const handleDeleteTask = useCallback(async () => {
    if (!currentProjectId || !taskToDelete) return;

    setIsSaving(true);
    try {
      await deleteProjectTask(currentProjectId, taskToDelete.id);

      // Refresh tasks
      await fetchTasks();
      setIsDeleteDialogOpen(false);
      setTaskToDelete(null);
    } catch (err) {
      console.error("Failed to delete task:", err);
      alert(
        `Failed to delete task: ${
          err instanceof Error ? err.message : "Unknown error"
        }`
      );
    } finally {
      setIsSaving(false);
    }
  }, [currentProjectId, taskToDelete, fetchTasks]);

  // Handle task move (drag and drop)
  const handleTaskMove = useCallback(
    async (taskId: string, newStatus: KanbanStatus) => {
      if (!currentProjectId) return;

      const task = tasks.find((t) => t.id === taskId);
      if (!task || task.status === newStatus) {
        return; // No change needed
      }

      // Optimistic update
      setTasks((prev) =>
        prev.map((t) => (t.id === taskId ? { ...t, status: newStatus } : t))
      );

      try {
        const apiStatus = mapKanbanStatusToApi(newStatus);
        await updateProjectTask(currentProjectId, taskId, {
          status: apiStatus,
        });

        // Refresh to ensure consistency
        await fetchTasks();
      } catch (err) {
        console.error("Failed to move task:", err);
        // Revert optimistic update
        await fetchTasks();
        alert(
          `Failed to move task: ${
            err instanceof Error ? err.message : "Unknown error"
          }`
        );
      }
    },
    [currentProjectId, tasks, fetchTasks]
  );

  // Handle edit task
  const handleEditTask = useCallback(
    (taskId: string) => {
      const task = tasks.find((t) => t.id === taskId);
      if (task) {
        setSelectedTask(task);
        setIsEditTaskModalOpen(true);
      }
    },
    [tasks]
  );

  // Handle delete task
  const handleDeleteTaskClick = useCallback(
    (taskId: string) => {
      const task = tasks.find((t) => t.id === taskId);
      if (task) {
        setTaskToDelete(task);
        setIsDeleteDialogOpen(true);
      }
    },
    [tasks]
  );

  // Handle view comments
  const handleViewComments = useCallback(
    (taskId: string) => {
      const task = tasks.find((t) => t.id === taskId);
      if (task) {
        setSelectedTask(task);
        setIsCommentsModalOpen(true);
      }
    },
    [tasks]
  );

  const columns = useMemo<KanbanColumnConfig[]>(() => {
    const statuses: KanbanStatus[] = ["backlog", "todo", "in-progress", "done"];

    return statuses.map((status) => {
      const statusTasks = tasks.filter((task) => task.status === status);
      const config = STATUS_CONFIG[status];

      return {
        status,
        title: config.title,
        cardCount: statusTasks.length,
        cards: statusTasks.map((task) => {
          const cardStatusTags: KanbanStatusTag[] = [];

          if (task.priority) {
            const priorityColors = PRIORITY_COLORS[task.priority] || {};
            cardStatusTags.push({
              label:
                task.priority.charAt(0).toUpperCase() + task.priority.slice(1),
              bgColor: priorityColors.bg,
              textColor: priorityColors.text,
            });
          }

          const cardMetadata: KanbanMetadata[] = [];

          // Show assigned agent or orchestrator if unassigned
          if (task.assigned_worker_id) {
            const agent = agents.find((a) => a.id === task.assigned_worker_id);
            if (agent) {
              cardMetadata.push({
                icon: {
                  path: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z",
                  size: 16,
                },
                text: agent.name,
              });
            }
          } else {
            // Show orchestrator for unassigned tasks
            cardMetadata.push({
              icon: {
                path: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z",
                size: 16,
              },
              text: "Orchestrator",
            });
          }

          return {
            id: task.id,
            title: task.title,
            description: task.description,
            priority: task.priority,
            statusTags: cardStatusTags,
            metadata: cardMetadata,
            commentCount: task.commentCount ?? 0,
          };
        }),
        onAddTask: () => {
          setSelectedStatus(status);
          setIsNewTaskModalOpen(true);
        },
      };
    });
  }, [tasks, agents]);

  if (isLoading) {
    return (
      <div className={styles.tasksTab}>
        <div className={styles.content}>
          <div className={styles.loadingMessage}>Loading tasks...</div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.tasksTab}>
      <div className={styles.content}>
        <KanbanBoard
          columns={columns}
          onAddTask={(status) => {
            setSelectedStatus(status);
            setIsNewTaskModalOpen(true);
          }}
          onTaskMove={handleTaskMove}
          onTaskEdit={handleEditTask}
          onTaskDelete={handleDeleteTaskClick}
          onTaskViewComments={handleViewComments}
        />
      </div>

      {/* Modals */}
      <NewTaskModal
        open={isNewTaskModalOpen}
        onOpenChange={setIsNewTaskModalOpen}
        onCreateTask={handleCreateTask}
        defaultStatus={selectedStatus}
      />

      <EditTaskModal
        open={isEditTaskModalOpen}
        onOpenChange={(open) => {
          setIsEditTaskModalOpen(open);
          if (!open) setSelectedTask(null);
        }}
        onUpdateTask={handleUpdateTask}
        task={selectedTask}
      />

      <CommentsModal
        open={isCommentsModalOpen}
        onOpenChange={(open) => {
          setIsCommentsModalOpen(open);
          if (!open) {
            setSelectedTask(null);
            // Refresh tasks to update comment counts
            fetchTasks();
          }
        }}
        taskId={selectedTask?.id ?? null}
        taskTitle={selectedTask?.title}
      />

      <DeleteTaskDialog
        open={isDeleteDialogOpen}
        onOpenChange={(open) => {
          setIsDeleteDialogOpen(open);
          if (!open) setTaskToDelete(null);
        }}
        onConfirm={handleDeleteTask}
        taskTitle={taskToDelete?.title}
      />
    </div>
  );
}
