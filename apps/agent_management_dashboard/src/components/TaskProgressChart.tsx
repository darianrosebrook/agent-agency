"use client";

import { useAnimatedValue } from "../hooks/useAnimatedValue";

interface TaskProgressChartProps {
  completedTasks?: number;
  totalTasks?: number;
  categories?: string[];
}

export function TaskProgressChart({
  // TODO: Replace default props with data fetched from v3 API with the following requirements:
  // 1. Task statistics fetching: Load task completion statistics from API
  //    - Data source: GET /api/projects/:id/tasks/stats endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `tasks` table with aggregation queries
  //    - Include completedTasks count, totalTasks count, and category breakdowns
  // 2. Props handling: Accept projectId prop to fetch project-specific statistics
  //    - Fetch statistics when projectId changes
  //    - Handle loading and error states
  // 3. Data transformation: Format API response for component props
  //    - Map API response to completedTasks, totalTasks, and categories props
  //    - Handle edge cases (zero tasks, all completed, etc.)
  completedTasks = 19,
  totalTasks = 40,
  categories = ["dev", "design"],
}: TaskProgressChartProps) {
  // Animate values when props change
  const animatedCompletedTasks = useAnimatedValue(completedTasks);
  const animatedTotalTasks = useAnimatedValue(totalTasks);
  const animatedPercentage = useAnimatedValue(
    Math.round((completedTasks / totalTasks) * 100)
  );

  return (
    <div className="bg-[#111] relative rounded-[12px] size-full border border-[#cacaca]">
      <div className="size-full">
        <div className="box-border content-stretch flex flex-col gap-[16px] items-start overflow-clip p-[12px] relative size-full">
          {/* Header */}
          <div className="content-stretch flex flex-col gap-[8px] items-start relative shrink-0 w-full">
            <div className="content-stretch flex items-start justify-between relative shrink-0 w-full">
              {/* Category badges */}
              <div className="content-stretch flex gap-[8px] items-start relative shrink-0">
                {categories.map((category) => (
                  <div
                    key={category}
                    className="relative rounded-[2px] shrink-0"
                  >
                    <div className="box-border content-stretch flex items-center overflow-clip px-[4px] py-[2px] relative rounded-[inherit]">
                      <div className="flex flex-col justify-center leading-[0] not-italic relative shrink-0 text-[#cacaca] text-[8px] text-nowrap text-right tracking-[0.12px]">
                        <p className="leading-none whitespace-pre">
                          {category}
                        </p>
                      </div>
                    </div>
                    <div
                      aria-hidden="true"
                      className="absolute border border-[#cacaca] border-solid inset-0 pointer-events-none rounded-[2px]"
                    />
                  </div>
                ))}
              </div>
            </div>
            {/* Title */}
            <div className="flex flex-col justify-center leading-[0] not-italic relative shrink-0 text-[24px] text-center text-neutral-50 text-nowrap tracking-[-0.24px]">
              <p className="leading-[normal] whitespace-pre">
                All active projects completion rate
              </p>
            </div>
          </div>

          {/* Divider */}
          <div className="bg-[#cacaca] h-px shrink-0 w-full" />

          {/* Stats */}
          <div className="content-stretch flex flex-col gap-[8px] items-start relative shrink-0">
            <div className="flex flex-col justify-center leading-[0] not-italic relative shrink-0 text-[#cacaca] text-[10px] text-center text-nowrap tracking-[-0.1px]">
              <p className="leading-[12px] whitespace-pre">
                You have {animatedCompletedTasks} tasks out of{" "}
                {animatedTotalTasks} completed
              </p>
            </div>
            <div className="content-stretch flex gap-[8px] items-center relative shrink-0">
              {/* Percentage */}
              <div className="flex flex-col justify-end leading-[0] not-italic relative shrink-0 text-[48px] text-center text-neutral-50 text-nowrap tracking-[-2.4px]">
                <p className="leading-[48px] whitespace-pre">
                  {animatedPercentage}%
                </p>
              </div>
              {/* Task count badge */}
              <div className="relative rounded-[2px] shrink-0">
                <div className="box-border content-stretch flex items-center overflow-clip px-[4px] py-[2px] relative rounded-[inherit]">
                  <div className="flex flex-col justify-center leading-[0] not-italic relative shrink-0 text-[#cacaca] text-[8px] text-nowrap text-right tracking-[0.12px]">
                    <p className="leading-none whitespace-pre">
                      {animatedCompletedTasks} tasks
                    </p>
                  </div>
                </div>
                <div
                  aria-hidden="true"
                  className="absolute border border-[#cacaca] border-solid inset-0 pointer-events-none rounded-[2px]"
                />
              </div>
              {/* Time reference */}
              <div className="flex flex-col justify-center leading-[0] not-italic relative shrink-0 text-[#cacaca] text-[10px] text-center text-nowrap tracking-[-0.1px]">
                <p className="leading-[12px] whitespace-pre">since last week</p>
              </div>
            </div>
          </div>

          {/* Bar chart */}
          <div className="basis-0 content-stretch flex gap-[2px] grow items-center max-h-[64px] min-h-px min-w-px overflow-clip relative rounded-[4px] shrink-0 w-full">
            {Array.from({ length: animatedTotalTasks }).map((_, index) => (
              <div
                key={index}
                className={`basis-0 grow h-[108px] min-h-px min-w-px shrink-0 transition-colors duration-500 ease-out ${
                  index < animatedCompletedTasks
                    ? "bg-neutral-50"
                    : "bg-[#454545]"
                }`}
              />
            ))}
          </div>
        </div>
      </div>
      <div
        aria-hidden="true"
        className="absolute border border-[#cacaca] border-solid inset-0 pointer-events-none rounded-[12px]"
      />
    </div>
  );
}
