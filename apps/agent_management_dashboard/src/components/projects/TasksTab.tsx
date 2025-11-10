"use client";

import { useState, useMemo, useEffect } from "react";
import { KanbanBoard } from "../composers/kanban/KanbanBoard";
import { NewTaskModal } from "./TaskModal";
import { useProjectStore } from "../../lib/stores";
import { useProjectContext } from "./ProjectContext";
import { getProjectTasks } from "../../lib/api/projects";
import { getAgents, type Agent } from "../../lib/api/agents";
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

interface Task {
  id: string;
  title: string;
  description?: string;
  status: "backlog" | "todo" | "in-progress" | "done";
  priority?: "low" | "medium" | "high";
  assigned_worker_id?: string | null;
}

export function TasksTab() {
  const [isNewTaskModalOpen, setIsNewTaskModalOpen] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState<"backlog" | "todo" | "in-progress" | "done">("backlog");
  const { currentProjectId } = useProjectContext();
  const { addTask } = useProjectStore();
  const [tasks, setTasks] = useState<Task[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    async function fetchTasks() {
      if (!currentProjectId) {
        setTasks([]);
        return;
      }

      setIsLoading(true);
      try {
        // Fetch tasks and agents in parallel
        const [tasksResponse, agentsData] = await Promise.all([
          getProjectTasks(currentProjectId),
          getAgents().catch(() => []), // Gracefully handle agent fetch failure
        ]);

        setAgents(agentsData);

        // Create a map of agent IDs to agent names
        const agentMap = new Map<string, string>();
        agentsData.forEach((agent) => {
          agentMap.set(agent.id, agent.name);
        });

        // Transform API tasks to local Task format
        const transformedTasks: Task[] = tasksResponse.tasks.map((task) => {
          // Map API status to Kanban status
          let status: "backlog" | "todo" | "in-progress" | "done" = "backlog";
          if (task.status === "completed") {
            status = "done";
          } else if (task.status === "in_progress" || task.status === "running") {
            status = "in-progress";
          } else if (task.status === "pending") {
            status = "todo";
          }

          // Map priority
          let priority: "low" | "medium" | "high" | undefined = undefined;
          if (task.priority !== null && task.priority !== undefined) {
            if (task.priority >= 3) {
              priority = "high";
            } else if (task.priority >= 2) {
              priority = "medium";
            } else {
              priority = "low";
            }
          }

          return {
            id: task.task_id,
            title: task.title,
            description: task.description ?? undefined,
            status,
            priority,
            assigned_worker_id: task.assigned_worker_id ?? null,
          };
        });

        setTasks(transformedTasks);
      } catch (err) {
        console.error("Failed to fetch project tasks:", err);
        setTasks([]);
      } finally {
        setIsLoading(false);
      }
    }

    fetchTasks();
  }, [currentProjectId]);

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
          
          // Show assigned agent if available
          if (task.assigned_worker_id) {
            const agent = agents.find((a) => a.id === task.assigned_worker_id);
            if (agent) {
              cardMetadata.push({
                icon: { path: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z", size: 16 },
                text: agent.name,
              });
            }
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
  }, [tasks, agents]);

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
