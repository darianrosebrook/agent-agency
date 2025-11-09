"use client";

import { useAnimatedValue } from "../hooks/useAnimatedValue";

interface TaskCompletionGaugeProps {
  title?: string;
  subtitle?: string;
}

export function TaskCompletionGauge({
  title = "Task Balance",
  subtitle = "Completion vs Creation Rate",
}: TaskCompletionGaugeProps) {
  // TODO: Replace mock data with aggregated task statistics from v3 database with the following requirements:
  // 1. Task statistics fetching: Load aggregated task counts by status
  //    - Data source: GET /api/tasks/stats endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `tasks` table with aggregation queries
  //    - Include counts for created, in-progress, and completed tasks
  // 2. Completion rate calculation: Calculate completion vs creation rate improvement
  //    - Compare current month completion rate to previous month
  //    - Calculate percentage improvement for display
  // 3. Data transformation: Format API response for gauge component
  //    - Map API response to task counts (created, inProgress, completed)
  //    - Calculate percentages for each section of the gauge
  
  // Color constants matching gauge colors
  const COLORS = {
    created: "#27272a",      // zinc-800 - dark gray for created tasks
    inProgress: "#6366f1",   // indigo-500 - indigo for in-progress tasks
    completed: "#e0e7ff",    // indigo-100 - light indigo for completed tasks
  };

  // Mock data - showing we're completing 18% more tasks than creating
  const completionRate = 18; // Percentage improvement over last month

  // Task distribution
  const created = 35;
  const inProgress = 42;
  const completed = 68;

  // Animate task counts
  const animatedCreated = useAnimatedValue(created);
  const animatedInProgress = useAnimatedValue(inProgress);
  const animatedCompleted = useAnimatedValue(completed);
  const animatedCompletionRate = useAnimatedValue(completionRate);

  // Calculate animated totals and percentages
  const animatedTotal = animatedCreated + animatedInProgress + animatedCompleted;
  const createdPercent = animatedTotal > 0 ? (animatedCreated / animatedTotal) * 100 : 0;
  const inProgressPercent = animatedTotal > 0 ? (animatedInProgress / animatedTotal) * 100 : 0;

  // Total number of dashes in the semi-circle
  const totalDashes = 60;

  // Calculate number of dashes for each section using animated values
  const createdDashes = Math.round((createdPercent / 100) * totalDashes);
  const inProgressDashes = Math.round((inProgressPercent / 100) * totalDashes);

  // Format numbers to fixed decimal places to prevent hydration mismatches
  const formatNumber = (value: number, decimals: number = 2): string => {
    return value.toFixed(decimals);
  };

  // Semi-circle parameters
  const centerX = 120;
  const centerY = 120;
  const radius = 85;
  const dashLength = 12;
  const dashWidth = 3;
  const gapAngle = 2; // degrees between dashes

  // Create dashes
  const renderDashes = () => {
    const dashes = [];
    const startAngle = 180; // Start from left (180 degrees)
    const totalAngle = 180; // Semi-circle
    const anglePerDash = totalAngle / totalDashes;

    let currentDash = 0;

    for (let i = 0; i < totalDashes; i++) {
      const angle = startAngle - i * anglePerDash - gapAngle / 2;
      const angleRad = (angle * Math.PI) / 180;

      // Determine color based on section
      let color;
      if (currentDash < createdDashes) {
        color = COLORS.created;
      } else if (currentDash < createdDashes + inProgressDashes) {
        color = COLORS.inProgress;
      } else {
        color = COLORS.completed;
      }

      // Calculate start and end points of the dash and format to fixed decimal places
      const innerRadius = radius - dashLength;
      const x1 = formatNumber(centerX + innerRadius * Math.cos(angleRad));
      const y1 = formatNumber(centerY - innerRadius * Math.sin(angleRad));
      const x2 = formatNumber(centerX + radius * Math.cos(angleRad));
      const y2 = formatNumber(centerY - radius * Math.sin(angleRad));

      dashes.push(
        <line
          key={i}
          x1={x1}
          y1={y1}
          x2={x2}
          y2={y2}
          stroke={color}
          strokeWidth={dashWidth}
          strokeLinecap="round"
          className="transition-colors duration-500 ease-out"
        />
      );

      currentDash++;
    }

    return dashes;
  };

  return (
    <div className="bg-[#111111] relative rounded-[12px] size-full border border-[#cacaca]">
      <div className="size-full">
        <div className="box-border flex flex-col p-6 relative size-full">
          {/* Header */}
          <div className="mb-3">
            <h3 className="text-white text-[15px] mb-1">{title}</h3>
            <p className="text-[#9e9ea0] text-[10px]">{subtitle}</p>
          </div>

          {/* Gauge */}
          <div className="flex-1 flex flex-col items-center justify-center -mt-2">
            <div className="relative scale-200">
              <svg width="240" height="140" viewBox="0 0 240 140">
                {renderDashes()}
              </svg>

              {/* Center percentage */}
              <div className="absolute inset-0 flex items-center justify-center pt-8">
                <div className="text-center">
                  <div className="text-white text-[32px] leading-none transition-none">
                    +{animatedCompletionRate}%
                  </div>
                  <div className="text-[#9e9ea0] text-[10px] mt-1.5">
                    vs last month
                  </div>
                </div>
              </div>
            </div>

            {/* Legend */}
            <div className="flex items-center gap-4 mt-4">
              <div className="flex items-center gap-1.5">
                <div className="w-2 h-2 rounded-full" style={{ backgroundColor: COLORS.created }} />
                <span className="text-[#9e9ea0] text-[9px]">Created</span>
              </div>
              <div className="flex items-center gap-1.5">
                <div className="w-2 h-2 rounded-full" style={{ backgroundColor: COLORS.inProgress }} />
                <span className="text-[#9e9ea0] text-[9px]">In Progress</span>
              </div>
              <div className="flex items-center gap-1.5">
                <div className="w-2 h-2 rounded-full" style={{ backgroundColor: COLORS.completed }} />
                <span className="text-[#9e9ea0] text-[9px]">Completed</span>
              </div>
            </div>

            {/* Stats */}
            <div className="grid grid-cols-3 gap-4 mt-3 w-full max-w-[220px]">
              <div className="text-center">
                <div className="text-[11px] transition-none" style={{ color: COLORS.created }}>{animatedCreated}</div>
                <div className="text-[#9e9ea0] text-[8px]">created</div>
              </div>
              <div className="text-center">
                <div className="text-[11px] transition-none" style={{ color: COLORS.inProgress }}>{animatedInProgress}</div>
                <div className="text-[#9e9ea0] text-[8px]">in progress</div>
              </div>
              <div className="text-center">
                <div className="text-[11px] transition-none" style={{ color: COLORS.completed }}>{animatedCompleted}</div>
                <div className="text-[#9e9ea0] text-[8px]">completed</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
