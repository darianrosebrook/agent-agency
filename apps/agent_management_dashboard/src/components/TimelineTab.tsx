"use client";

import { Calendar, ZoomIn, ZoomOut } from "lucide-react";
import { useEffect, useState } from "react";
import { getAgents } from "../lib/api/agents";
import { listTasks, type Task } from "../lib/api/tasks";
import { GanttChart } from "./GanttChart";
import styles from "./TimelineTab.module.scss";
import { Button } from "./primitives/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./primitives/select";

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

// Helper function to map task status to timeline status
function mapTaskStatus(
  status: string
): "completed" | "in-progress" | "pending" {
  if (status === "completed") return "completed";
  if (status === "in_progress" || status === "running") return "in-progress";
  return "pending";
}

// Helper function to extract tags from metadata
function extractTags(metadata?: Record<string, unknown>): string[] {
  if (!metadata) return [];

  // Check for tags array in metadata
  if (Array.isArray(metadata.tags)) {
    return metadata.tags.map((tag) => String(tag));
  }

  // Check for tags as comma-separated string
  if (typeof metadata.tags === "string") {
    return metadata.tags
      .split(",")
      .map((tag) => tag.trim())
      .filter(Boolean);
  }

  // Extract from other metadata fields if available
  const tags: string[] = [];
  if (metadata.priority) tags.push(`Priority: ${metadata.priority}`);
  if (metadata.risk_tier) tags.push(`Risk: ${metadata.risk_tier}`);

  return tags;
}

// Helper function to calculate end date
function calculateEndDate(task: Task): Date {
  // Use deadline if available
  if (task.metadata?.deadline && typeof task.metadata.deadline === "string") {
    const deadline = new Date(task.metadata.deadline);
    if (!isNaN(deadline.getTime())) return deadline;
  }

  // Use completed_at if task is completed
  if (task.completed_at) {
    return new Date(task.completed_at);
  }

  // Use updated_at as fallback
  if (task.updated_at) {
    return new Date(task.updated_at);
  }

  // Default to 7 days from start date
  const startDate = task.created_at ? new Date(task.created_at) : new Date();
  return new Date(startDate.getTime() + 7 * 24 * 60 * 60 * 1000);
}

export function TimelineTab() {
  const [zoomLevel, setZoomLevel] = useState<ZoomLevel>("week");
  const [selectedWorker, setSelectedWorker] = useState<string>("all");
  const [tasks, setTasks] = useState<TimelineTask[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchData() {
      setIsLoading(true);
      setError(null);

      try {
        // Fetch tasks and agents in parallel
        const [tasksResponse, agentsResponse] = await Promise.all([
          listTasks(),
          getAgents(),
        ]);

        // Create a map of worker_id to agent name
        const workerMap = new Map<string, string>();
        if (Array.isArray(agentsResponse)) {
          agentsResponse.forEach((agent) => {
            workerMap.set(agent.id, agent.name);
          });
        } else {
          console.warn("Agents response is not an array:", agentsResponse);
        }

        // Transform tasks to TimelineTask format
        const timelineTasks: TimelineTask[] = tasksResponse.tasks
          .filter((task) => {
            // Only include tasks with assigned workers and valid dates
            return task.assigned_worker_id && task.created_at;
          })
          .map((task) => {
            const workerId = task.assigned_worker_id || "";
            const workerName = workerMap.get(workerId) || "Unassigned";
            const startDate = task.created_at
              ? new Date(task.created_at)
              : new Date();
            const endDate = calculateEndDate(task);
            const tags = extractTags(task.metadata || undefined);

            return {
              id: task.id,
              title: task.title,
              worker: workerName,
              workerId: workerId,
              startDate,
              endDate,
              status: mapTaskStatus(task.status),
              tags,
              description: task.description || undefined,
            };
          });

        setTasks(timelineTasks);
      } catch (err) {
        console.error("Failed to fetch timeline data:", err);
        setError(
          err instanceof Error ? err : new Error("Failed to load timeline data")
        );
        setTasks([]);
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
  }, []);

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

  if (isLoading) {
    return (
      <div className={styles.timelineTab}>
        <div className={styles.loading}>Loading timeline data...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.timelineTab}>
        <div className={styles.error}>
          Failed to load timeline data: {error.message}
        </div>
      </div>
    );
  }

  return (
    <div className={styles.timelineTab}>
      {/* Controls */}
      <div className={styles.controls}>
        <div className={styles.controlsInner}>
          <div className={styles.controlsLeft}>
            <Calendar className={styles.iconMedium} />
            <h2 className={styles.title}>Project Timeline</h2>
          </div>

          <div className={styles.controlsRight}>
            <Select value={selectedWorker} onValueChange={setSelectedWorker}>
              <SelectTrigger className={styles.selectTrigger}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent className={styles.selectContent}>
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
              <span className={styles.zoomLabel}>{zoomLevel}</span>
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
        {filteredTasks.length > 0 ? (
          <GanttChart tasks={filteredTasks} zoomLevel={zoomLevel} />
        ) : (
          <div className={styles.emptyState}>
            No tasks with assigned workers found. Assign workers to tasks to see
            them on the timeline.
          </div>
        )}
      </div>
    </div>
  );
}
