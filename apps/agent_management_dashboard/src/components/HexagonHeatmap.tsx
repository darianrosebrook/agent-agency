import { useMemo } from "react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./primitives/tooltip";
import styles from "./HexagonHeatmap.module.scss";

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
  // TODO: Replace hardcoded agent names and generated completion percentages with real task and agent data from v3 database with the following requirements:
  // 1. Agent data fetching: Load configured AI agents from database
  //    - Data source: GET /api/agents endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `agents` table
  //    - Include agent names, IDs, and status information
  // 2. Task completion data: Fetch task completion statistics by agent
  //    - Data source: GET /api/tasks/stats/by-agent endpoint aggregating task completion from PostgreSQL `tasks` and `worker_assignments` tables
  //    - Calculate completion percentages per agent based on assigned tasks
  //    - Include task names and IDs for tooltip display
  // 3. Hexagon mapping: Map tasks to hexagon grid positions
  //    - Use task IDs or sequential mapping to assign hexagon coordinates
  //    - Preserve visual heatmap pattern while using real completion data
  // 4. Real-time updates: Refresh data when tasks are updated
  //    - Subscribe to task update events or poll for changes
  //    - Update hexagon colors based on latest completion percentages
  // Generate hexagon spiral data with completion percentages
  const hexagons = useMemo(() => {
    const center: Axial = { q: 0, r: 0 };
    const cells = hexSpiral(center, radius);
    const agents = ["Agent Alpha", "Agent Beta", "Agent Gamma", "Agent Delta"];

    const data: HexagonData[] = cells.map((hex, idx) => {
      const { x, y } = axialToPixel(hex, hexSize);

      // Calculate distance from center (0,0) using axial coordinates
      const distance =
        (Math.abs(hex.q) + Math.abs(hex.r) + Math.abs(hex.q + hex.r)) / 2;

      // Create a bias where center hexagons have higher completion
      // Distance ranges from 0 (center) to radius (outer edge)
      const distanceBias = 1 - distance / radius;

      // Generate completion percentage with noise patterns
      const noise = (Math.sin(hex.q * 2.5) * Math.cos(hex.r * 2.5) + 1) / 2;
      const noise2 = (Math.sin(hex.q * 0.8 + hex.r * 1.2) + 1) / 2;
      const baseNoise = noise * 0.6 + noise2 * 0.4;

      // Apply distance bias: center gets 60-100%, edges get 0-60%
      const biasedCompletion = (distanceBias * 0.5 + baseNoise * 0.5) * 100;
      const completion = Math.max(
        0,
        Math.min(100, Math.round(biasedCompletion))
      );

      return {
        id: `hex-${hex.q}-${hex.r}`,
        q: hex.q,
        r: hex.r,
        x,
        y,
        completion,
        taskName: `Task ${idx + 1}`,
        agent: agents[Math.floor(Math.random() * agents.length)],
      };
    });

    return data;
  }, [radius, hexSize]);

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
              {totalTasks} tasks tracked across AI agents
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
