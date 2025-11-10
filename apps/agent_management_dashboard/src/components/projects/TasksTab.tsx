"use client";

import { useState, useMemo } from "react";
import { KanbanBoard } from "../composers/kanban/KanbanBoard";
import { NewTaskModal } from "./TaskModal";
import { useProjectStore } from "../../lib/stores";
import type { KanbanColumnConfig, KanbanStatusTag, KanbanMetadata } from "../composers/kanban/types";
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

export function TasksTab() {
  const [isNewTaskModalOpen, setIsNewTaskModalOpen] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState<"backlog" | "todo" | "in-progress" | "done">("backlog");
  const { currentProjectId, addTask, getTasks } = useProjectStore();

  const tasks = currentProjectId ? getTasks(currentProjectId) : [];

  const columns = useMemo<KanbanColumnConfig[]>(() => {
    const statuses: Array<"backlog" | "todo" | "in-progress" | "done"> = [
      "backlog",
      "todo",
      "in-progress",
      "done",
    ];

    return statuses.map((status) => {
      const statusTasks = tasks.filter((task) => task.status === status);
      const config = STATUS_CONFIG[status];
      
      const statusTags: KanbanStatusTag[] = [];
      const metadata: KanbanMetadata[] = [];
      
      return {
        status,
        title: config.title,
        cardCount: statusTasks.length,
        cards: statusTasks.map((task) => {
          const cardStatusTags: KanbanStatusTag[] = [];
          
          if (task.priority) {
            const priorityColors = PRIORITY_COLORS[task.priority] || {};
            cardStatusTags.push({
              label: task.priority.charAt(0).toUpperCase() + task.priority.slice(1),
              bgColor: priorityColors.bg,
              textColor: priorityColors.text,
            });
          }
          
          const cardMetadata: KanbanMetadata[] = [];
          if (task.assignee) {
            cardMetadata.push({
              icon: { path: "M7 7h.01M7 3h5a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z", size: 13.999 },
              text: task.assignee,
            });
          }
          
          return {
            title: task.title,
            description: task.description,
            priority: task.priority as "low" | "medium" | "high" | undefined,
            statusTags: cardStatusTags,
            metadata: cardMetadata,
          };
        }),
        onAddTask: () => {
          setSelectedStatus(status);
          setIsNewTaskModalOpen(true);
        },
      };
    });
  }, [tasks]);

  const handleCreateTask = (data: {
    title: string;
    description?: string;
    status: "backlog" | "todo" | "in-progress" | "done";
    priority?: string;
  }) => {
    if (currentProjectId) {
      addTask(currentProjectId, data);
    }
  };

  return (
    <div className={styles.tasksTab}>
      <div className={styles.content}>
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
