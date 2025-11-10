"use client";

import { useState, useMemo } from "react";
import { KanbanBoard } from "../composers/kanban/KanbanBoard";
import { NewTaskModal } from "./TaskModal";
import { useProjectStore } from "../../lib/stores";
import type { ProjectTask } from "../../lib/schemas/project";
import styles from "./TasksTab.module.scss";

export function TasksTab() {
  const [isNewTaskModalOpen, setIsNewTaskModalOpen] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState<
    "backlog" | "todo" | "in-progress" | "done"
  >("backlog");
  const { currentProjectId, addTask, getTasks } = useProjectStore();

  const tasks = currentProjectId ? getTasks(currentProjectId) : [];

  const columns = useMemo(() => {
    const statuses: Array<"backlog" | "todo" | "in-progress" | "done"> = [
      "backlog",
      "todo",
      "in-progress",
      "done",
    ];

    return statuses.map((status) => {
      const statusTasks = tasks.filter((task) => task.status === status);
      
      return {
        status,
        title: status === "backlog" ? "Backlog" : status === "todo" ? "To Do" : status === "in-progress" ? "In Progress" : "Done",
        cardCount: statusTasks.length,
        cards: statusTasks.map((task) => ({
          title: task.title,
          description: task.description,
          priority: task.priority as "low" | "medium" | "high" | undefined,
          statusTags: task.priority
            ? [
                {
                  label: task.priority.charAt(0).toUpperCase() + task.priority.slice(1),
                  bgColor: task.priority === "high" ? "#3a2f1f" : task.priority === "medium" ? "#1f2d3a" : undefined,
                  textColor: task.priority === "high" ? "#ff9f43" : task.priority === "medium" ? "#54a0ff" : undefined,
                },
              ]
            : [],
          metadata: task.assignee
            ? [
                {
                  icon: { path: "M7 7h.01M7 3h5a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z", size: 13.999 },
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
