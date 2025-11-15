import { useEffect, useMemo, useState } from "react";
import { getAgents, type Agent } from "../lib/api/agents";
import { listTasks, type Task } from "../lib/api/tasks";
import styles from "./HexagonHeatmap.module.scss";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./primitives/tooltip";

type Axial = { q: number; r: number };

interface HexagonData {
  id: string;
  q: number;
  r: number;
  x: number;
  y: number;
  completion: number;
  taskName: string;
  agent: string;
}

interface HexagonHeatmapProps {
  radius?: number;
  hexSize?: number;
}

// Hexagonal direction vectors
const DIRS: Axial[] = [
  { q: 1, r: 0 },
  { q: 1, r: -1 },
  { q: 0, r: -1 },
  { q: -1, r: 0 },
  { q: -1, r: 1 },
  { q: 0, r: 1 },
];

// Generate a ring of hexagons at distance k from center
function hexRing(center: Axial, k: number): Axial[] {
  if (k === 0) return [center];
  let cur = {
    q: center.q + DIRS[4].q * k,
    r: center.r + DIRS[4].r * k,
  };
  const out: Axial[] = [];
  for (let side = 0; side < 6; side++) {
    for (let step = 0; step < k; step++) {
      out.push({ ...cur });
      const d = DIRS[side];
      cur = { q: cur.q + d.q, r: cur.r + d.r };
    }
  }
  return out;
}

// Generate a spiral of hexagons from center to radius R
function hexSpiral(center: Axial, R: number): Axial[] {
  const cells: Axial[] = [];
  for (let k = 0; k <= R; k++) cells.push(...hexRing(center, k));
  return cells;
}

// Convert axial coordinates to pixel coordinates (flat-top orientation)
function axialToPixel(hex: Axial, size: number) {
  const x = size * (3 / 2) * hex.q;
  const y = size * Math.sqrt(3) * (hex.r + hex.q / 2);
  return { x, y };
}

// Generate flat-top hexagon path
function hexPath(cx: number, cy: number, size: number) {
  const angles: number[] = [];

  // Flat-top: first vertex at 0° (right), then counterclockwise
  for (let i = 0; i < 6; i++) {
    angles.push((i * 60 * Math.PI) / 180);
  }

  const points = angles.map((angle) => [
    cx + size * Math.cos(angle),
    cy + size * Math.sin(angle),
  ]);

  return `M ${points.map((p) => `${p[0]},${p[1]}`).join(" L ")} Z`;
}

export function HexagonHeatmap({
  radius = 12,
  hexSize = 16,
}: HexagonHeatmapProps) {
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
        // Safely extract tasks array with fallback
        const tasksArray = Array.isArray(tasksData?.tasks)
          ? tasksData.tasks
          : Array.isArray(tasksData)
          ? tasksData
          : [];

        // Safely extract agents array with fallback
        const agentsArray = Array.isArray(agentsData?.agents)
          ? agentsData.agents
          : Array.isArray(agentsData)
          ? agentsData
          : [];

        setTasks(tasksArray);
        setAgents(agentsArray);
      } catch (err) {
        console.error("Failed to fetch tasks and agents:", err);
        setError(err instanceof Error ? err : new Error("Failed to load data"));
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

  // Generate hexagon spiral data with real task completion percentages
  const hexagons = useMemo(() => {
    const center: Axial = { q: 0, r: 0 };
    const cells = hexSpiral(center, radius);

    // Ensure tasks is always an array
    const safeTasks = Array.isArray(tasks) ? tasks : [];

    // Create agent lookup map
    const agentMap = new Map<string, Agent>();
    if (Array.isArray(agents)) {
      agents.forEach((agent) => {
        agentMap.set(agent.id, agent);
      });
    } else {
      console.warn("Agents data is not an array:", agents);
    }

    // Calculate completion percentage for each task
    const getTaskCompletion = (task: Task): number => {
      if (task.status === "completed") return 100;
      if (task.status === "failed" || task.status === "cancelled") return 0;
      if (task.status === "running") return 50; // In progress
      return 0; // Pending
    };

    // Map tasks to hexagons
    const data: HexagonData[] = cells.map((hex, idx) => {
      const { x, y } = axialToPixel(hex, hexSize);

      // Get task for this hexagon (or use empty if we run out of tasks)
      const task = safeTasks[idx] ?? null;
      const completion = task ? getTaskCompletion(task) : 0;
      const agentId = task?.assigned_worker_id || null;
      const agent = agentId ? agentMap.get(agentId) : null;

      return {
        id: task?.id ?? `hex-${hex.q}-${hex.r}`,
        q: hex.q,
        r: hex.r,
        x,
        y,
        completion,
        taskName: task?.title ?? `Task ${idx + 1}`,
        agent: agent?.name ?? "Unassigned",
      };
    });

    return data;
  }, [radius, hexSize, tasks, agents]);

  // Get color based on completion percentage
  const getHexColor = (completion: number): string => {
    if (completion === 0) {
      return "#27272a"; // Grey for not started
    } else if (completion <= 20) {
      return "#1e1b4b"; // Deep purple/blue
    } else if (completion <= 40) {
      return "#312e81"; // Dark blue
    } else if (completion <= 60) {
      return "#4338ca"; // Medium blue
    } else if (completion <= 80) {
      return "#6366f1"; // Bright blue
    } else if (completion <= 95) {
      return "#a5b4fc"; // Light blue
    } else {
      return "#e0e7ff"; // Near white
    }
  };

  // Calculate statistics
  const totalTasks = hexagons.length;
  const completedTasks = hexagons.filter((h) => h.completion === 100).length;
  const averageCompletion = Math.round(
    hexagons.reduce((sum, h) => sum + h.completion, 0) / totalTasks
  );

  // Calculate SVG dimensions
  const maxExtent = radius * hexSize * 2;
  const svgWidth = maxExtent * 2;
  const svgHeight = maxExtent * 2;
  const centerX = svgWidth / 2;
  const centerY = svgHeight / 2;

  // Drawing size with slight gutter effect
  const gutter = 1.5;
  const drawScale = Math.max(0, 1 - gutter / (hexSize * Math.sqrt(3)));
  const drawSize = hexSize * drawScale;

  return (
    <div className={styles.container}>
      <div className={styles.innerContainer}>
        <div className={styles.content}>
          {/* Header */}
          <div className={styles.header}>
            <h3 className={styles.title}>Task Completion Heatmap</h3>
            <p className={styles.subtitle}>
              {isLoading
                ? "Loading tasks and agents..."
                : error
                ? `Error: ${error.message}`
                : Array.isArray(tasks) && tasks.length > 0
                ? `${tasks.length} tasks tracked across ${
                    Array.isArray(agents) ? agents.length : 0
                  } AI agents`
                : "No tasks available"}
            </p>
          </div>

          {/* Hexagon Grid */}
          <div className={styles.hexagonGrid}>
            <TooltipProvider>
              <svg
                viewBox={`0 0 ${svgWidth} ${svgHeight}`}
                className={styles.svg}
                style={{ maxHeight: "100%", maxWidth: "100%" }}
              >
                <g transform={`translate(${centerX}, ${centerY})`}>
                  {hexagons.map((hex) => (
                    <Tooltip key={hex.id} delayDuration={0}>
                      <TooltipTrigger asChild>
                        <path
                          d={hexPath(hex.x, hex.y, drawSize)}
                          fill={getHexColor(hex.completion)}
                          stroke="#000000"
                          strokeWidth="1"
                          strokeOpacity={0.4}
                          className={styles.hexagonPath}
                          style={{
                            filter:
                              hex.completion > 0
                                ? "drop-shadow(0 0 2px rgba(99, 102, 241, 0.2))"
                                : "none",
                          }}
                        />
                      </TooltipTrigger>
                      <TooltipContent
                        side="top"
                        className={styles.tooltipContent}
                      >
                        <div className={styles.tooltipInner}>
                          <div className={styles.tooltipTitle}>
                            {hex.taskName}
                          </div>
                          <div className={styles.tooltipSubtext}>
                            {hex.agent} • {hex.completion}% complete
                          </div>
                        </div>
                      </TooltipContent>
                    </Tooltip>
                  ))}
                </g>
              </svg>
            </TooltipProvider>
          </div>

          {/* Stats and Legend */}
          <div className={styles.statsAndLegend}>
            <div className={styles.stats}>
              <div className={styles.statItem}>
                <div className={styles.statLabel}>Completed</div>
                <div className={styles.statValue}>
                  {completedTasks} / {totalTasks}
                </div>
              </div>
              <div className={styles.statItem}>
                <div className={styles.statLabel}>Avg Completion</div>
                <div className={styles.statValue}>{averageCompletion}%</div>
              </div>
            </div>

            {/* Legend */}
            <div className={styles.legend}>
              <div className={styles.legendItem}>
                <div
                  className={styles.legendSwatch}
                  style={{ backgroundColor: "#27272a" }}
                />
                <span className={styles.legendLabel}>0%</span>
              </div>
              <div className={styles.legendItem}>
                <div
                  className={styles.legendSwatch}
                  style={{ backgroundColor: "#312e81" }}
                />
                <span className={styles.legendLabel}>1-40%</span>
              </div>
              <div className={styles.legendItem}>
                <div
                  className={styles.legendSwatch}
                  style={{ backgroundColor: "#6366f1" }}
                />
                <span className={styles.legendLabel}>41-80%</span>
              </div>
              <div className={styles.legendItem}>
                <div
                  className={styles.legendSwatch}
                  style={{ backgroundColor: "#e0e7ff" }}
                />
                <span className={styles.legendLabel}>81-100%</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
