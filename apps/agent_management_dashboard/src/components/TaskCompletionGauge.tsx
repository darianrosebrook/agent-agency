interface TaskCompletionGaugeProps {
  title?: string;
  subtitle?: string;
}

export function TaskCompletionGauge({
  title = "Task Balance",
  subtitle = "Completion vs Creation Rate",
}: TaskCompletionGaugeProps) {
  // Mock data - showing we're completing 18% more tasks than creating
  const completionRate = 18; // Percentage improvement over last month

  // Task distribution
  const created = 35;
  const inProgress = 42;
  const completed = 68;
  const total = created + inProgress + completed;

  // Calculate percentages for each section
  const createdPercent = (created / total) * 100;
  const inProgressPercent = (inProgress / total) * 100;
  const completedPercent = (completed / total) * 100;

  // Total number of dashes in the semi-circle
  const totalDashes = 60;

  // Calculate number of dashes for each section
  const createdDashes = Math.round(
    (createdPercent / 100) * totalDashes,
  );
  const inProgressDashes = Math.round(
    (inProgressPercent / 100) * totalDashes,
  );
  const completedDashes =
    totalDashes - createdDashes - inProgressDashes;

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
    const endAngle = 0; // End at right (0 degrees)
    const totalAngle = 180; // Semi-circle
    const anglePerDash = totalAngle / totalDashes;

    let currentDash = 0;

    for (let i = 0; i < totalDashes; i++) {
      const angle =
        startAngle - i * anglePerDash - gapAngle / 2;
      const angleRad = (angle * Math.PI) / 180;

      // Determine color based on section
      let color;
      if (currentDash < createdDashes) {
        color = "#27272a"; // zinc-400 - created 
      } else if (
        currentDash <
        createdDashes + inProgressDashes
      ) {
        color = "#6366f1"; // zinc-500 - in progress
      } else {
        color = "#e0e7ff"; // zinc-600 - completed
      }

      // Calculate start and end points of the dash
      const innerRadius = radius - dashLength;
      const x1 = centerX + innerRadius * Math.cos(angleRad);
      const y1 = centerY - innerRadius * Math.sin(angleRad);
      const x2 = centerX + radius * Math.cos(angleRad);
      const y2 = centerY - radius * Math.sin(angleRad);

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
        />,
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
            <h3 className="text-white text-[15px] mb-1">
              {title}
            </h3>
            <p className="text-[#9e9ea0] text-[10px]">
              {subtitle}
            </p>
          </div>

          {/* Gauge */}
          <div className="flex-1 flex flex-col items-center justify-center -mt-2">
            <div className="relative scale-200">
              <svg
                width="240"
                height="140"
                viewBox="0 0 240 140"
              >
                {renderDashes()}
              </svg>

              {/* Center percentage */}
              <div className="absolute inset-0 flex items-center justify-center pt-8">
                <div className="text-center">
                  <div className="text-white text-[32px] leading-none">
                    +{completionRate}%
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
                <div className="w-2 h-2 rounded-full bg-[#a1a1aa]" />
                <span className="text-[#9e9ea0] text-[9px]">
                  Created
                </span>
              </div>
              <div className="flex items-center gap-1.5">
                <div className="w-2 h-2 rounded-full bg-[#71717a]" />
                <span className="text-[#9e9ea0] text-[9px]">
                  In Progress
                </span>
              </div>
              <div className="flex items-center gap-1.5">
                <div className="w-2 h-2 rounded-full bg-[#52525b]" />
                <span className="text-[#9e9ea0] text-[9px]">
                  Completed
                </span>
              </div>
            </div>

            {/* Stats */}
            <div className="grid grid-cols-3 gap-4 mt-3 w-full max-w-[220px]">
              <div className="text-center">
                <div className="text-[#a1a1aa] text-[11px]">
                  {created}
                </div>
                <div className="text-[#9e9ea0] text-[8px]">
                  created
                </div>
              </div>
              <div className="text-center">
                <div className="text-[#71717a] text-[11px]">
                  {inProgress}
                </div>
                <div className="text-[#9e9ea0] text-[8px]">
                  in progress
                </div>
              </div>
              <div className="text-center">
                <div className="text-[#52525b] text-[11px]">
                  {completed}
                </div>
                <div className="text-[#9e9ea0] text-[8px]">
                  completed
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}