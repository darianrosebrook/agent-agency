"use client";

import { useState } from "react";
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

// TODO: Replace mock timeline data with task and worker assignment data from v3 database with the following requirements:
// 1. Timeline data fetching: Load tasks with worker assignments and dates
//    - Data source: GET /api/projects/:id/tasks/timeline endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
//    - Database tables: PostgreSQL `tasks` and `worker_assignments` tables
//    - Include task metadata: id, title, worker, workerId, startDate, endDate, status, tags, description
// 2. Worker information: Include worker name and ID for each task
//    - Join worker_assignments table to get assigned worker details
//    - Handle unassigned tasks gracefully
// 3. Data transformation: Format API response for GanttChart component
//    - Map API response to TimelineTask array with required fields
//    - Handle date parsing and status mapping
// Mock data for timeline tasks
const mockTasks: TimelineTask[] = [
  {
    id: "1",
    title: "Design system foundation",
    worker: "Sarah Chen",
    workerId: "sarah",
    startDate: new Date(2024, 10, 1),
    endDate: new Date(2024, 10, 5),
    status: "completed",
    tags: ["Design", "High Priority"],
    description: "Created core design tokens and components",
  },
  {
    id: "2",
    title: "Component library setup",
    worker: "Sarah Chen",
    workerId: "sarah",
    startDate: new Date(2024, 10, 6),
    endDate: new Date(2024, 10, 12),
    status: "completed",
    tags: ["Design", "UI"],
    description: "Set up Storybook and base components",
  },
  {
    id: "3",
    title: "Dashboard wireframes",
    worker: "Sarah Chen",
    workerId: "sarah",
    startDate: new Date(2024, 10, 13),
    endDate: new Date(2024, 10, 20),
    status: "in-progress",
    tags: ["Design", "Wireframes"],
    description: "Creating dashboard layout mockups",
  },
  {
    id: "4",
    title: "API endpoint development",
    worker: "Alex Kumar",
    workerId: "alex",
    startDate: new Date(2024, 10, 2),
    endDate: new Date(2024, 10, 8),
    status: "completed",
    tags: ["Backend", "API"],
    description: "Built REST API endpoints for user management",
  },
  {
    id: "5",
    title: "Database schema design",
    worker: "Alex Kumar",
    workerId: "alex",
    startDate: new Date(2024, 10, 9),
    endDate: new Date(2024, 10, 11),
    status: "completed",
    tags: ["Backend", "Database"],
    description: "Designed and implemented database schema",
  },
  {
    id: "6",
    title: "Authentication system",
    worker: "Alex Kumar",
    workerId: "alex",
    startDate: new Date(2024, 10, 12),
    endDate: new Date(2024, 10, 18),
    status: "in-progress",
    tags: ["Backend", "Security"],
    description: "Implementing JWT-based authentication",
  },
  {
    id: "7",
    title: "Frontend routing",
    worker: "Jordan Lee",
    workerId: "jordan",
    startDate: new Date(2024, 10, 1),
    endDate: new Date(2024, 10, 3),
    status: "completed",
    tags: ["Frontend", "Dev"],
    description: "Set up React Router and navigation",
  },
  {
    id: "8",
    title: "Dashboard implementation",
    worker: "Jordan Lee",
    workerId: "jordan",
    startDate: new Date(2024, 10, 4),
    endDate: new Date(2024, 10, 10),
    status: "completed",
    tags: ["Frontend", "UI"],
    description: "Built main dashboard components",
  },
  {
    id: "9",
    title: "User profile pages",
    worker: "Jordan Lee",
    workerId: "jordan",
    startDate: new Date(2024, 10, 11),
    endDate: new Date(2024, 10, 15),
    status: "in-progress",
    tags: ["Frontend", "UI"],
    description: "Creating user profile and settings pages",
  },
  {
    id: "10",
    title: "Responsive design",
    worker: "Jordan Lee",
    workerId: "jordan",
    startDate: new Date(2024, 10, 16),
    endDate: new Date(2024, 10, 22),
    status: "pending",
    tags: ["Frontend", "Mobile"],
    description: "Making all pages mobile responsive",
  },
  {
    id: "11",
    title: "Unit testing",
    worker: "Maria Garcia",
    workerId: "maria",
    startDate: new Date(2024, 10, 3),
    endDate: new Date(2024, 10, 7),
    status: "completed",
    tags: ["QA", "Testing"],
    description: "Writing unit tests for core features",
  },
  {
    id: "12",
    title: "Integration testing",
    worker: "Maria Garcia",
    workerId: "maria",
    startDate: new Date(2024, 10, 8),
    endDate: new Date(2024, 10, 14),
    status: "in-progress",
    tags: ["QA", "Testing"],
    description: "Setting up E2E test suite",
  },
  {
    id: "13",
    title: "Performance optimization",
    worker: "Maria Garcia",
    workerId: "maria",
    startDate: new Date(2024, 10, 15),
    endDate: new Date(2024, 10, 21),
    status: "pending",
    tags: ["QA", "Performance"],
    description: "Optimizing bundle size and load times",
  },
];
export function TimelineTab() {
  const [zoomLevel, setZoomLevel] = useState<ZoomLevel>("week");
  const [selectedWorker, setSelectedWorker] = useState<string>("all");

  const workers = Array.from(new Set(mockTasks.map((t) => t.worker)));
  const filteredTasks =
    selectedWorker === "all"
      ? mockTasks
      : mockTasks.filter((t) => t.worker === selectedWorker);

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
            <Calendar className="w-5 h-5 text-[#888888]" />
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
                <ZoomOut className="w-4 h-4" />
              </Button>
              <span className={styles.zoomLevel}>{zoomLevel}</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleZoomIn}
                disabled={zoomLevel === "day"}
                className={styles.zoomButton}
              >
                <ZoomIn className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Gantt Chart */}
      <div className={styles.ganttContainer}>
        <GanttChart tasks={filteredTasks} zoomLevel={zoomLevel} />
      </div>
    </div>
  );
}
