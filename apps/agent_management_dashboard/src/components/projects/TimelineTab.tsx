"use client";

import { Calendar, ZoomIn, ZoomOut } from "lucide-react";
import { useEffect, useState } from "react";
import { getAgents, type Agent } from "../../lib/api/agents";
import { getProjectTasks } from "../../lib/api/projects";
import { Button } from "../primitives/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../primitives/select";
import { GanttChart } from "./GanttChart";
import { useProjectContext } from "./ProjectContext";
import styles from "./TimelineTab.module.scss";

export type ZoomLevel = "day" | "week" | "month" | "quarter";

export interface TimelineTask {
  id: string;
  title: string;
  worker: string;
  workerId: string;
  startDate: Date;
  endDate: Date;
  status: "completed" | "in-progress" | "pending";
  tags: string[];
  description?: string;
}

export function TimelineTab() {
  const { currentProjectId } = useProjectContext();
  const [zoomLevel, setZoomLevel] = useState<ZoomLevel>("week");
  const [selectedWorker, setSelectedWorker] = useState<string>("all");
  const [tasks, setTasks] = useState<TimelineTask[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchData() {
      if (!currentProjectId) {
        setTasks([]);
        return;
      }

      setIsLoading(true);
      setError(null);

      try {
        // Fetch tasks and agents in parallel
        const [tasksResponse, agentsData] = await Promise.all([
          getProjectTasks(currentProjectId),
          getAgents().catch(() => []), // Gracefully handle agent fetch failure
        ]);

        // Ensure agents is an array before setting
        const agentsArray = Array.isArray(agentsData) 
          ? agentsData 
          : (agentsData?.agents || []);
        setAgents(agentsArray);

        // Create a map of agent IDs to agent names
        const agentMap = new Map<string, string>();
        agentsArray.forEach((agent) => {
          agentMap.set(agent.id, agent.name);
        });

        // Transform API tasks to TimelineTask format
        const timelineTasks: TimelineTask[] = tasksResponse.tasks
          .map((task) => {
            // Parse dates with validation
            const startDate = new Date(task.created_at);
            let endDate: Date;

            if (task.completed_at) {
              endDate = new Date(task.completed_at);
            } else if (task.updated_at) {
              endDate = new Date(task.updated_at);
            } else {
              // Fallback: use start date + 1 day if no end date available
              endDate = new Date(startDate);
              endDate.setDate(endDate.getDate() + 1);
            }

            // Ensure end date is not before start date
            if (endDate < startDate) {
              endDate = new Date(startDate);
              endDate.setDate(endDate.getDate() + 1);
            }

            // Validate dates are not invalid
            if (isNaN(startDate.getTime()) || isNaN(endDate.getTime())) {
              console.warn(`Invalid dates for task ${task.task_id}:`, {
                created_at: task.created_at,
                updated_at: task.updated_at,
                completed_at: task.completed_at,
              });
              // Skip invalid dates
              return null;
            }

            // Map status to TimelineTask status
            let status: "completed" | "in-progress" | "pending" = "pending";
            if (task.status === "completed") {
              status = "completed";
            } else if (
              task.status === "in_progress" ||
              task.status === "running"
            ) {
              status = "in-progress";
            }

            // Extract tags from risk_tier or metadata if available
            const tags: string[] = [];
            if (task.risk_tier) {
              tags.push(task.risk_tier);
            }
            if (task.priority !== null && task.priority !== undefined) {
              tags.push(`Priority ${task.priority}`);
            }

            // Get agent name from assignment
            // If unassigned, treat as orchestrator-managed (planning/coordination phase)
            const workerId = task.assigned_worker_id ?? null;
            const workerName =
              workerId && agentMap.has(workerId)
                ? agentMap.get(workerId)!
                : "Orchestrator";

            return {
              id: task.task_id,
              title: task.title,
              worker: workerName,
              workerId: workerId ?? "orchestrator",
              startDate,
              endDate,
              status,
              tags,
              description: task.description ?? undefined,
            };
          })
          .filter((task): task is TimelineTask => task !== null);

        setTasks(timelineTasks);
      } catch (err) {
        console.error("Failed to fetch project tasks:", err);
        setError(
          err instanceof Error ? err : new Error("Failed to load tasks")
        );
        setTasks([]);
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
  }, [currentProjectId]);

  const workers = Array.from(new Set(tasks.map((t) => t.worker)));
  const filteredTasks =
    selectedWorker === "all"
      ? tasks
      : tasks.filter((t) => t.worker === selectedWorker);

  const handleZoomIn = () => {
    const levels: ZoomLevel[] = ["quarter", "month", "week", "day"];
    const currentIndex = levels.indexOf(zoomLevel);
    if (currentIndex < levels.length - 1) {
      setZoomLevel(levels[currentIndex + 1]);
    }
  };

  const handleZoomOut = () => {
    const levels: ZoomLevel[] = ["quarter", "month", "week", "day"];
    const currentIndex = levels.indexOf(zoomLevel);
    if (currentIndex > 0) {
      setZoomLevel(levels[currentIndex - 1]);
    }
  };

  return (
    <div className={styles.timelineTab}>
      {/* Controls */}
      <div className={styles.controls}>
        <div className={styles.controlsContent}>
          <div className={styles.controlsLeft}>
            <Calendar className={styles.controlsIcon} />
            <h2 className={styles.controlsTitle}>Project Timeline</h2>
          </div>

          <div className={styles.controlsRight}>
            <Select value={selectedWorker} onValueChange={setSelectedWorker}>
              <SelectTrigger className={styles.workerSelect}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent className={styles.workerSelectContent}>
                <SelectItem value="all">All Workers</SelectItem>
                {workers.map((worker) => (
                  <SelectItem key={worker} value={worker}>
                    {worker}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <div className={styles.zoomControls}>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleZoomOut}
                disabled={zoomLevel === "quarter"}
                className={styles.zoomButton}
              >
                <ZoomOut className={styles.zoomIcon} />
              </Button>
              <span className={styles.zoomLevel}>{zoomLevel}</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleZoomIn}
                disabled={zoomLevel === "day"}
                className={styles.zoomButton}
              >
                <ZoomIn className={styles.zoomIcon} />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Gantt Chart */}
      <div className={styles.ganttContainer}>
        {isLoading ? (
          <div className={styles.loadingMessage}>Loading timeline...</div>
        ) : error ? (
          <div className={styles.errorMessage}>
            Error loading timeline: {error.message}
          </div>
        ) : filteredTasks.length === 0 ? (
          <div className={styles.emptyMessage}>
            No tasks found for this project.
          </div>
        ) : (
          <GanttChart tasks={filteredTasks} zoomLevel={zoomLevel} />
        )}
      </div>
    </div>
  );
}
