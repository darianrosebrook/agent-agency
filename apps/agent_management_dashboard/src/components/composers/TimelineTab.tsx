"use client";

import { useState, useEffect, useMemo } from "react";
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
import { listTasks, type Task } from "../../lib/api/tasks";
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
  const [zoomLevel, setZoomLevel] = useState<ZoomLevel>("week");
  const [selectedWorker, setSelectedWorker] = useState<string>("all");
  const [tasks, setTasks] = useState<Task[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchData() {
      setIsLoading(true);
      setError(null);
      try {
        const [tasksData, agentsData] = await Promise.all([
          listTasks(),
          getAgents(),
        ]);
        setTasks(tasksData.tasks);
        setAgents(agentsData);
      } catch (err) {
        console.error("Failed to fetch tasks and agents:", err);
        setError(err instanceof Error ? err : new Error("Failed to load timeline data"));
        setTasks([]);
        setAgents([]);
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
    // Refresh every 30 seconds
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, []);

  // Transform tasks to TimelineTask format
  const timelineTasks = useMemo(() => {
    // Create agent lookup map
    const agentMap = new Map<string, Agent>();
    agents.forEach((agent) => {
      agentMap.set(agent.id, agent);
    });

    return tasks.map((task): TimelineTask => {
      const agent = task.worker_id ? agentMap.get(task.worker_id) : null;
      const startDate = task.started_at 
        ? new Date(task.started_at) 
        : new Date(task.created_at);
      
      // Calculate end date: use completed_at if available, otherwise estimate
      let endDate: Date;
      if (task.completed_at) {
        endDate = new Date(task.completed_at);
      } else if (task.status === "completed") {
        // If marked completed but no date, use updated_at
        endDate = new Date(task.updated_at);
      } else {
        // Estimate end date: 7 days from start for pending/running tasks
        endDate = new Date(startDate);
        endDate.setDate(endDate.getDate() + 7);
      }

      // Map status
      let status: "completed" | "in-progress" | "pending";
      if (task.status === "completed") {
        status = "completed";
      } else if (task.status === "running" || task.status === "in_progress") {
        status = "in-progress";
      } else {
        status = "pending";
      }

      // Extract tags from metadata or use defaults based on task type/priority
      const tags: string[] = [];
      if (task.priority) tags.push(task.priority);
      if (task.type) tags.push(task.type);
      if (task.metadata && typeof task.metadata === "object") {
        const metadataTags = (task.metadata as any).tags;
        if (Array.isArray(metadataTags)) {
          tags.push(...metadataTags);
        }
      }

      return {
        id: task.id,
        title: task.title,
        worker: agent?.name || "Unassigned",
        workerId: agent?.id || "unassigned",
        startDate,
        endDate,
        status,
        tags: tags.length > 0 ? tags : ["General"],
        description: task.description || undefined,
      };
    });
  }, [tasks, agents]);

  const workers = useMemo(() => {
    return Array.from(new Set(timelineTasks.map((t) => t.worker)));
  }, [timelineTasks]);

  const filteredTasks = useMemo(() => {
    if (selectedWorker === "all") {
      return timelineTasks;
    }
    return timelineTasks.filter((t) => t.worker === selectedWorker);
  }, [timelineTasks, selectedWorker]);

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
            <Calendar className={styles.iconMedium} />
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
                <ZoomOut className={styles.icon} />
              </Button>
              <span className={styles.zoomLevel}>{zoomLevel}</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleZoomIn}
                disabled={zoomLevel === "day"}
                className={styles.zoomButton}
              >
                <ZoomIn className={styles.icon} />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Gantt Chart */}
      <div className={styles.ganttContainer}>
        {isLoading ? (
          <div className={styles.loadingState}>
            <p>Loading timeline data...</p>
          </div>
        ) : error ? (
          <div className={styles.errorState}>
            <p>Error: {error.message}</p>
          </div>
        ) : filteredTasks.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No tasks available for timeline</p>
          </div>
        ) : (
          <GanttChart tasks={filteredTasks} zoomLevel={zoomLevel} />
        )}
      </div>
    </div>
  );
}
