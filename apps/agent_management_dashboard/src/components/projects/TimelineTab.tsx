"use client";

import { useState, useEffect } from "react";
import { GanttChart } from "./GanttChart";
import { ZoomIn, ZoomOut, Calendar } from "lucide-react";
import { Button } from "../primitives/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../primitives/select";
import { useProjectContext } from "./ProjectContext";
import { getProjectTasks } from "../../lib/api/projects";
import { getAgents, type Agent } from "../../lib/api/agents";
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

        setAgents(agentsData);

        // Create a map of agent IDs to agent names
        const agentMap = new Map<string, string>();
        agentsData.forEach((agent) => {
          agentMap.set(agent.id, agent.name);
        });
        
        // Transform API tasks to TimelineTask format
        const timelineTasks: TimelineTask[] = tasksResponse.tasks.map((task) => {
          const startDate = new Date(task.created_at);
          const endDate = task.completed_at 
            ? new Date(task.completed_at)
            : new Date(task.updated_at);
          
          // Map status to TimelineTask status
          let status: "completed" | "in-progress" | "pending" = "pending";
          if (task.status === "completed") {
            status = "completed";
          } else if (task.status === "in_progress" || task.status === "running") {
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
          const workerId = task.assigned_worker_id ?? null;
          const workerName = workerId && agentMap.has(workerId) 
            ? agentMap.get(workerId)! 
            : "Unassigned";

          return {
            id: task.task_id,
            title: task.title,
            worker: workerName,
            workerId: workerId ?? "unassigned",
            startDate,
            endDate,
            status,
            tags,
            description: task.description ?? undefined,
          };
        });

        setTasks(timelineTasks);
      } catch (err) {
        console.error("Failed to fetch project tasks:", err);
        setError(err instanceof Error ? err : new Error("Failed to load tasks"));
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
          <div className={styles.errorMessage}>Error loading timeline: {error.message}</div>
        ) : filteredTasks.length === 0 ? (
          <div className={styles.emptyMessage}>No tasks found for this project.</div>
        ) : (
          <GanttChart tasks={filteredTasks} zoomLevel={zoomLevel} />
        )}
      </div>
    </div>
  );
}
