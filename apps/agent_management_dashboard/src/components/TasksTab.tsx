"use client";

import { useState, useMemo, useEffect, useCallback } from "react";
import { KanbanBoard } from "./composers/kanban/KanbanBoard";
import { NewTaskModal } from "./NewTaskModal";
import { useProjectStore } from "../lib/stores";
import {
  getProjectTasks,
  createProjectTask,
  updateProjectTask,
  type ProjectTask,
} from "../lib/api/projects";
import { toastError, toastSuccess } from "../lib/utils/toast";
import styles from "./TasksTab.module.scss";
import type { KanbanStatus } from "./composers/kanban/types";

/**
 * Convert API priority number to UI priority string
 */
function mapPriorityToUI(priority: number | null | undefined): "low" | "medium" | "high" | undefined {
  if (priority === null || priority === undefined) return undefined;
  if (priority >= 3) return "high";
  if (priority >= 2) return "medium";
  return "low";
}

/**
 * Convert UI priority string to API priority number
 */
function mapPriorityToAPI(priority?: string): number | undefined {
  if (!priority) return undefined;
  if (priority === "high") return 3;
  if (priority === "medium") return 2;
  if (priority === "low") return 1;
  return undefined;
}

/**
 * Convert API task format to UI format
 */
function mapTaskToUI(task: ProjectTask): {
  id: string;
  title: string;
  description?: string;
  status: KanbanStatus;
  priority?: "low" | "medium" | "high";
  assignee?: string; // UI display field (shows assigned_worker_id UUID for now)
} {
  // Use id field (fallback to task_id for backward compatibility)
  const taskId = task.id || task.task_id || '';
  
  return {
    id: taskId,
    title: task.title,
    description: task.description ?? undefined,
    status: task.status as KanbanStatus,
    priority: mapPriorityToUI(task.priority),
    assignee: task.assigned_worker_id ?? undefined, // Display assigned_worker_id UUID
  };
}

export function TasksTab() {
  const [isNewTaskModalOpen, setIsNewTaskModalOpen] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState<KanbanStatus>("backlog");
  const [tasks, setTasks] = useState<ProjectTask[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const { currentProjectId } = useProjectStore();

  /**
   * Fetch tasks from API
   */
  const fetchTasks = useCallback(async () => {
    if (!currentProjectId) {
      setTasks([]);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const response = await getProjectTasks(currentProjectId);
      setTasks(response.tasks || []);
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Failed to fetch tasks");
      setError(error);
      toastError(error.message || "Failed to load tasks");
      console.error("Failed to fetch tasks:", err);
    } finally {
      setIsLoading(false);
    }
  }, [currentProjectId]);

  // Fetch tasks when project changes
  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  const columns = useMemo(() => {
    const statuses: KanbanStatus[] = ["backlog", "todo", "in-progress", "done"];

    const uiTasks = tasks.map(mapTaskToUI);

    return statuses.map((status) => {
      const statusTasks = uiTasks.filter((task) => task.status === status);

      return {
        status,
        title:
          status === "backlog"
            ? "Backlog"
            : status === "todo"
              ? "To Do"
              : status === "in-progress"
                ? "In Progress"
                : "Done",
        cardCount: statusTasks.length,
        cards: statusTasks.map((task) => ({
          id: task.id,
          title: task.title,
          description: task.description,
          priority: task.priority,
          statusTags: task.priority
            ? [
                {
                  label: task.priority.charAt(0).toUpperCase() + task.priority.slice(1),
                  bgColor:
                    task.priority === "high"
                      ? "#3a2f1f"
                      : task.priority === "medium"
                        ? "#1f2d3a"
                        : undefined,
                  textColor:
                    task.priority === "high"
                      ? "#ff9f43"
                      : task.priority === "medium"
                        ? "#54a0ff"
                        : undefined,
                },
              ]
            : [],
          metadata: task.assignee
            ? [
                {
                  icon: {
                    path: "M7 7h.01M7 3h5a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",
                    size: 13.999,
                  },
                  text: task.assignee,
                },
              ]
            : [],
        })),
        onAddTask: () => {
          setSelectedStatus(status);
          setIsNewTaskModalOpen(true);
        },
      };
    });
  }, [tasks]);

  /**
   * Handle task creation via API
   */
  const handleCreateTask = useCallback(
    async (data: {
      title: string;
      description?: string;
      status: KanbanStatus;
      priority?: string;
    }) => {
      if (!currentProjectId) {
        toastError("No project selected");
        return;
      }

      try {
        await createProjectTask(currentProjectId, {
          title: data.title,
          description: data.description,
          status: data.status,
          priority: mapPriorityToAPI(data.priority),
        });
        toastSuccess("Task created successfully");
        setIsNewTaskModalOpen(false);
        // Refresh tasks from API
        await fetchTasks();
      } catch (err) {
        const error = err instanceof Error ? err : new Error("Failed to create task");
        toastError(error.message || "Failed to create task");
        console.error("Failed to create task:", err);
      }
    },
    [currentProjectId, fetchTasks]
  );

  /**
   * Handle task status update (drag and drop)
   */
  const handleTaskMove = useCallback(
    async (taskId: string, newStatus: KanbanStatus) => {
      if (!currentProjectId) {
        toastError("No project selected");
        return;
      }

      try {
        await updateProjectTask(currentProjectId, taskId, {
          status: newStatus,
        });
        // Refresh tasks from API
        await fetchTasks();
      } catch (err) {
        const error = err instanceof Error ? err : new Error("Failed to update task");
        toastError(error.message || "Failed to update task");
        console.error("Failed to update task:", err);
        // Refresh to revert optimistic update
        await fetchTasks();
      }
    },
    [currentProjectId, fetchTasks]
  );

  /**
   * Handle task edit (placeholder - can be expanded later)
   */
  const handleTaskEdit = useCallback((taskId: string) => {
    // TODO: Open edit modal
    console.log("Edit task:", taskId);
  }, []);

  /**
   * Handle task delete (placeholder - can be expanded later)
   */
  const handleTaskDelete = useCallback(
    async (taskId: string) => {
      if (!currentProjectId) {
        toastError("No project selected");
        return;
      }

      // TODO: Implement delete confirmation dialog
      // For now, just log
      console.log("Delete task:", taskId);
    },
    [currentProjectId]
  );

  if (isLoading && tasks.length === 0) {
    return (
      <div className={styles.tasksTab}>
        <div className={styles.tasksTabContent}>
          <div style={{ padding: "2rem", textAlign: "center" }}>Loading tasks...</div>
        </div>
      </div>
    );
  }

  if (error && tasks.length === 0) {
    return (
      <div className={styles.tasksTab}>
        <div className={styles.tasksTabContent}>
          <div style={{ padding: "2rem", textAlign: "center", color: "red" }}>
            Error loading tasks: {error.message}
            <button
              onClick={fetchTasks}
              style={{ marginLeft: "1rem", padding: "0.5rem 1rem" }}
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.tasksTab}>
      <div className={styles.tasksTabContent}>
        <KanbanBoard
          columns={columns}
          onAddTask={(status) => {
            setSelectedStatus(status);
            setIsNewTaskModalOpen(true);
          }}
          onTaskMove={handleTaskMove}
          onTaskEdit={handleTaskEdit}
          onTaskDelete={handleTaskDelete}
        />
      </div>

      <NewTaskModal
        open={isNewTaskModalOpen}
        onOpenChange={setIsNewTaskModalOpen}
        onCreateTask={handleCreateTask}
        defaultStatus={selectedStatus}
      />
    </div>
  );
}
