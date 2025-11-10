interface Task {
  name: string;
  progress: number;
  color: string;
}

interface MultiRingProgressProps {
  tasks?: Task[];
  projectedTimeline?: string;
}

export function MultiRingProgress({
  tasks = [
    { name: "Progress 1", progress: 85, color: "#e0e7ff" },
    { name: "Progress 2", progress: 75, color: "#818cf8" },
    { name: "Progress 3", progress: 60, color: "#6366f1" },
  ],
  projectedTimeline = "15 business days",
}: MultiRingProgressProps) {
  // Calculate total progress (average of all tasks)
  const totalProgress = (
    tasks.reduce((sum, task) => sum + task.progress, 0) / tasks.length
  ).toFixed(2);

  const centerX = 200;
  const centerY = 200;
  const totalSegments = 40;

  // Define ring parameters (outer to inner)
  const rings = [
    { radius: 160, innerRadius: 140, strokeWidth: 20 },
    { radius: 135, innerRadius: 115, strokeWidth: 20 },
    { radius: 110, innerRadius: 90, strokeWidth: 20 },
  ];

  // Generate segments for a ring
  const generateRingSegments = (
    progress: number,
    radius: number,
    innerRadius: number,
    color: string
  ) => {
    const segments = [];
    const segmentAngle = 360 / totalSegments;
    const gapAngle = 2;
    const completedSegments = Math.round((progress / 100) * totalSegments);

    for (let i = 0; i < totalSegments; i++) {
      const startAngle = i * segmentAngle - 90;
      const endAngle = startAngle + segmentAngle - gapAngle;

      const x1 = centerX + radius * Math.cos((startAngle * Math.PI) / 180);
      const y1 = centerY + radius * Math.sin((startAngle * Math.PI) / 180);
      const x2 = centerX + radius * Math.cos((endAngle * Math.PI) / 180);
      const y2 = centerY + radius * Math.sin((endAngle * Math.PI) / 180);
      const x3 = centerX + innerRadius * Math.cos((endAngle * Math.PI) / 180);
      const y3 = centerY + innerRadius * Math.sin((endAngle * Math.PI) / 180);
      const x4 = centerX + innerRadius * Math.cos((startAngle * Math.PI) / 180);
      const y4 = centerY + innerRadius * Math.sin((startAngle * Math.PI) / 180);

      const pathData = `
        M ${x1} ${y1}
        A ${radius} ${radius} 0 0 1 ${x2} ${y2}
        L ${x3} ${y3}
        A ${innerRadius} ${innerRadius} 0 0 0 ${x4} ${y4}
        Z
      `;

      segments.push(
        <path
          key={`${radius}-${i}`}
          d={pathData}
          fill={i < completedSegments ? color : "#27272a"}
          className="transition-colors duration-300"
        />
      );
    }

    return segments;
  };

  // Generate radial grid lines
  const generateGridLines = () => {
    const lines = [];
    const angles = [0, 45, 90, 135, 180, 225, 270, 315];
    
    for (const angle of angles) {
      const x1 = centerX + 70 * Math.cos(((angle - 90) * Math.PI) / 180);
      const y1 = centerY + 70 * Math.sin(((angle - 90) * Math.PI) / 180);
      const x2 = centerX + 175 * Math.cos(((angle - 90) * Math.PI) / 180);
      const y2 = centerY + 175 * Math.sin(((angle - 90) * Math.PI) / 180);

      lines.push(
        <line
          key={`line-${angle}`}
          x1={x1}
          y1={y1}
          x2={x2}
          y2={y2}
          stroke="#3f3f46"
          strokeWidth="1"
          opacity="0.3"
        />
      );
    }

    return lines;
  };

  // Generate percentage labels around the chart
  const generatePercentageLabels = () => {
    const labels = [
      { angle: 0, text: "50%" },
      { angle: 45, text: "33.3%" },
      { angle: 90, text: "16.7%" },
      { angle: 180, text: "83.3%" },
      { angle: 225, text: "100%" },
      { angle: 270, text: "0%" },
      { angle: 315, text: "66.7%" },
    ];

    return labels.map(({ angle, text }) => {
      const x = centerX + 185 * Math.cos(((angle - 90) * Math.PI) / 180);
      const y = centerY + 185 * Math.sin(((angle - 90) * Math.PI) / 180);

      return (
        <text
          key={`label-${angle}`}
          x={x}
          y={y}
          textAnchor="middle"
          dominantBaseline="middle"
          className="fill-zinc-600"
          style={{
            fontSize: '12px',
            fontWeight: '400',
          }}
        >
          {text}
        </text>
      );
    });
  };

  // Generate progress labels on the left
  const generateProgressLabels = () => {
    const yPositions = [
      { y: 145, progress: tasks[0]?.progress || 85 },
      { y: 200, progress: tasks[1]?.progress || 75 },
      { y: 255, progress: tasks[2]?.progress || 60 },
    ];

    return yPositions.map(({ y, progress }, index) => (
      <text
        key={`progress-label-${index}`}
        x={20}
        y={y}
        textAnchor="start"
        dominantBaseline="middle"
        className="fill-neutral-50"
        style={{
          fontSize: '14px',
          fontWeight: '500',
        }}
      >
        {progress}%
      </text>
    ));
  };

  return (
    <div className="bg-neutral-950 relative rounded-[12px] size-full border border-[#cacaca]">
      <div className="size-full">
        <div className="box-border flex flex-col gap-4 p-6 relative size-full">
          {/* Header */}
          <div>
            <h3 className="text-neutral-50 text-[20px] tracking-[-0.2px] mb-3">
              Progress
            </h3>
            <div className="flex items-baseline gap-2 mb-1">
              <span className="text-neutral-50 text-[32px] tracking-[-1.6px]">
                {totalProgress}%
              </span>
              <span className="text-zinc-500 text-[14px]">· Total progress</span>
            </div>
            <p className="text-zinc-500 text-[12px]">
              Projected timeline: {projectedTimeline}
            </p>
          </div>

          {/* Chart */}
          <div className="flex-1 flex items-center justify-center">
            <svg
              width="400"
              height="400"
              viewBox="0 0 400 400"
              xmlns="http://www.w3.org/2000/svg"
              className="max-w-full max-h-full"
            >
              {/* Grid lines */}
              {generateGridLines()}

              {/* Concentric circles (background) */}
              <circle
                cx={centerX}
                cy={centerY}
                r={70}
                fill="none"
                stroke="#3f3f46"
                strokeWidth="1"
                opacity="0.2"
              />
              <circle
                cx={centerX}
                cy={centerY}
                r={100}
                fill="none"
                stroke="#3f3f46"
                strokeWidth="1"
                opacity="0.2"
              />
              <circle
                cx={centerX}
                cy={centerY}
                r={125}
                fill="none"
                stroke="#3f3f46"
                strokeWidth="1"
                opacity="0.2"
              />
              <circle
                cx={centerX}
                cy={centerY}
                r={150}
                fill="none"
                stroke="#3f3f46"
                strokeWidth="1"
                opacity="0.2"
              />
              <circle
                cx={centerX}
                cy={centerY}
                r={175}
                fill="none"
                stroke="#3f3f46"
                strokeWidth="1"
                opacity="0.2"
              />

              {/* Progress rings (outermost to innermost) */}
              {tasks.map((task, index) => (
                <g key={task.name}>
                  {generateRingSegments(
                    task.progress,
                    rings[index].radius,
                    rings[index].innerRadius,
                    task.color
                  )}
                </g>
              ))}

              {/* Percentage labels around chart */}
              {generatePercentageLabels()}

              {/* Progress labels on the left */}
              {generateProgressLabels()}
            </svg>
          </div>

          {/* Divider */}
          <div className="w-full h-px bg-[#cacaca]" />

          {/* Legend */}
          <div className="flex items-center justify-center gap-6">
            {tasks.map((task) => (
              <div key={task.name} className="flex items-center gap-2">
                <div
                  className="w-6 h-3 rounded-sm"
                  style={{ backgroundColor: task.color }}
                />
                <span className="text-zinc-500 text-[14px]">{task.name}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
