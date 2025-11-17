"use client";

import { useState, useMemo } from "react";
import { KanbanBoard } from "./kanban/KanbanBoard";
import { NewTaskModal } from "./TaskModal";
import { useProjectStore } from "../../lib/stores";
import styles from "./TasksTab.module.scss";

export function TasksTab() {
  const [isNewTaskModalOpen, setIsNewTaskModalOpen] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState<
    "backlog" | "todo" | "in-progress" | "done"
  >("backlog");
  const { currentProjectId, addTask, getTasks } = useProjectStore();

  const tasks = currentProjectId ? getTasks(currentProjectId) : [];

  // Map backend status to kanban status
  const mapBackendStatusToKanban = (backendStatus: string): "backlog" | "todo" | "in-progress" | "done" => {
    switch (backendStatus) {
      case "pending":
        return "backlog";
      case "in_progress":
        return "in-progress";
      case "completed":
        return "done";
      case "paused":
      case "cancelled":
      case "failed":
        return "todo";
      default:
        return "backlog";
    }
  };

  // Convert priority number (0-10) to string label
  const getPriorityLabel = (priority: number | null | undefined): "low" | "medium" | "high" | undefined => {
    if (priority === null || priority === undefined) return undefined;
    if (priority >= 7) return "high";
    if (priority >= 4) return "medium";
    return "low";
  };

  const columns = useMemo(() => {
    const statuses: Array<"backlog" | "todo" | "in-progress" | "done"> = [
      "backlog",
      "todo",
      "in-progress",
      "done",
    ];

    return statuses.map((status) => {
      // Filter tasks by mapping backend status to kanban status
      const statusTasks = tasks.filter((task) => mapBackendStatusToKanban(task.status) === status);
      
      const priorityLabel = getPriorityLabel(statusTasks[0]?.priority);
      
      return {
        status,
        title: status === "backlog" ? "Backlog" : status === "todo" ? "To Do" : status === "in-progress" ? "In Progress" : "Done",
        cardCount: statusTasks.length,
        cards: statusTasks.map((task) => {
          const priority = getPriorityLabel(task.priority);
          return {
            id: task.id,
            title: task.title,
            description: task.description,
            priority,
            statusTags: priority
              ? [
                  {
                    label: priority.charAt(0).toUpperCase() + priority.slice(1),
                    bgColor: priority === "high" ? "#3a2f1f" : priority === "medium" ? "#1f2d3a" : undefined,
                    textColor: priority === "high" ? "#ff9f43" : priority === "medium" ? "#54a0ff" : undefined,
                  },
                ]
              : [],
            metadata: task.assigned_worker_id
              ? [
                  {
                    icon: { path: "M7 7h.01M7 3h5a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z", size: 13.999 },
                    text: task.assigned_worker_id,
                  },
                ]
              : [],
          };
        }),
        onAddTask: () => {
          setSelectedStatus(status);
          setIsNewTaskModalOpen(true);
        },
      };
    });
  }, [tasks]);

  // Map kanban status to backend status
  const mapKanbanStatusToBackend = (kanbanStatus: "backlog" | "todo" | "in-progress" | "done"): "pending" | "in_progress" | "completed" | "paused" => {
    switch (kanbanStatus) {
      case "backlog":
        return "pending";
      case "in-progress":
        return "in_progress";
      case "done":
        return "completed";
      case "todo":
        return "paused";
    }
  };

  // Convert priority string to number (0-10)
  const getPriorityNumber = (priority?: string): number | null | undefined => {
    if (!priority) return undefined;
    switch (priority.toLowerCase()) {
      case "high":
        return 8;
      case "medium":
        return 5;
      case "low":
        return 2;
      default:
        return undefined;
    }
  };

  const handleCreateTask = (data: {
    title: string;
    description?: string;
    status: "backlog" | "todo" | "in-progress" | "done";
    priority?: string;
  }) => {
    if (currentProjectId) {
      addTask(currentProjectId, {
        title: data.title,
        description: data.description,
        status: mapKanbanStatusToBackend(data.status),
        priority: getPriorityNumber(data.priority),
      });
    }
  };

  return (
    <div className={styles.tasksTab}>
      <div className={styles.tasksTabContent}>
        <KanbanBoard
          columns={columns}
          onAddTask={(status) => {
            setSelectedStatus(status);
            setIsNewTaskModalOpen(true);
          }}
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
