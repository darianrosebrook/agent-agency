"use client";

import { useState } from "react";
import Container66 from "../imports/Container-16-2951";
import { NewTaskModal } from "./NewTaskModal";
import { useProjectStore } from "../lib/stores";
import styles from "./TasksTab.module.scss";

export function TasksTab() {
  const [isNewTaskModalOpen, setIsNewTaskModalOpen] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState<
    "backlog" | "todo" | "in-progress" | "done"
  >("backlog");
  const { currentProjectId, addTask } = useProjectStore();

  const handleOpenModal = (
    status: "backlog" | "todo" | "in-progress" | "done"
  ) => {
    setSelectedStatus(status);
    setIsNewTaskModalOpen(true);
  };

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
        <div
          className={styles.tasksTabInner}
          onClick={(e) => {
            // Check if clicked element is an "add" button based on class or parent
            const target = e.target as HTMLElement;
            const button = target.closest("[data-add-task]");
            if (button) {
              const status = button.getAttribute("data-status") as
                | "backlog"
                | "todo"
                | "in-progress"
                | "done";
              handleOpenModal(status || "backlog");
            }
          }}
        >
          <Container66 />
        </div>
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
