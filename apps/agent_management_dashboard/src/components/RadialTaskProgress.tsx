"use client";

import { useState } from "react";
import {
  CheckCircle2,
  MoreVertical,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";

interface Project {
  id: number;
  title: string;
  description: string;
  progress: number;
  status: string;
  date: string;
  estimatedDate: string;
}

interface RadialTaskProgressProps {
  totalSegments?: number;
}

// Sample data for 6 recent projects
const recentProjects: Project[] = [
  {
    id: 1,
    title: "E-Commerce Platform",
    description:
      "Building a modern shopping experience with AI recommendations",
    progress: 78,
    status: "On Track",
    date: "March 11, 2025",
    estimatedDate: "March 28, 2025",
  },
  {
    id: 2,
    title: "Mobile App Redesign",
    description: "Complete UI/UX overhaul for iOS and Android applications",
    progress: 45,
    status: "In Progress",
    date: "March 10, 2025",
    estimatedDate: "April 5, 2025",
  },
  {
    id: 3,
    title: "Data Analytics Dashboard",
    description: "Real-time analytics with custom visualizations and reporting",
    progress: 92,
    status: "Nearly Done",
    date: "March 8, 2025",
    estimatedDate: "March 15, 2025",
  },
  {
    id: 4,
    title: "API Integration Suite",
    description: "Connecting third-party services for seamless data flow",
    progress: 62,
    status: "Normal",
    date: "March 7, 2025",
    estimatedDate: "March 22, 2025",
  },
  {
    id: 5,
    title: "Security Audit System",
    description: "Automated vulnerability scanning and compliance checking",
    progress: 34,
    status: "Early Stage",
    date: "March 5, 2025",
    estimatedDate: "April 12, 2025",
  },
  {
    id: 6,
    title: "Cloud Migration",
    description: "Transitioning legacy systems to modern cloud infrastructure",
    progress: 88,
    status: "On Track",
    date: "March 3, 2025",
    estimatedDate: "March 18, 2025",
  },
];

export function RadialTaskProgress({
  totalSegments = 24,
}: RadialTaskProgressProps) {
  const [currentIndex, setCurrentIndex] = useState(0);
  const currentProject = recentProjects[currentIndex];

  const handlePrevious = () => {
    setCurrentIndex((prev) =>
      prev === 0 ? recentProjects.length - 1 : prev - 1
    );
  };

  const handleNext = () => {
    setCurrentIndex((prev) =>
      prev === recentProjects.length - 1 ? 0 : prev + 1
    );
  };

  const handleDotClick = (index: number) => {
    setCurrentIndex(index);
  };

  const completedSegments = Math.round(
    (currentProject.progress / 100) * totalSegments
  );

  // Generate radial segments
  const generateSegments = () => {
    const segments = [];
    const segmentAngle = 360 / totalSegments;
    const gapAngle = 2; // Gap between segments
    const radius = 100;
    const innerRadius = 70;
    const centerX = 120;
    const centerY = 120;

    for (let i = 0; i < totalSegments; i++) {
      const startAngle = i * segmentAngle - 90; // Start from top
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
          key={i}
          d={pathData}
          fill={i < completedSegments ? "#fafafa" : "#454545"}
          className="transition-colors duration-300"
        />
      );
    }

    return segments;
  };

  return (
    <div className="bg-neutral-950 relative rounded-[12px] size-full border border-[#cacaca]">
      <div className="size-full">
        <div className="box-border flex flex-col p-6 relative size-full">
          {/* Main content area */}
          <div className="flex items-center gap-6 flex-1 min-h-0">
            {/* Left side - Radial chart */}
            <div className="flex-shrink-0">
              <svg
                width="240"
                height="240"
                viewBox="0 0 240 240"
                xmlns="http://www.w3.org/2000/svg"
                className="transform -rotate-0"
              >
                {generateSegments()}
                {/* Center circle with percentage */}
                <circle cx="120" cy="120" r="65" fill="#0a0a0a" />
                <text
                  x="120"
                  y="130"
                  textAnchor="middle"
                  className="fill-neutral-50"
                  style={{
                    fontSize: "56px",
                    fontWeight: "300",
                    letterSpacing: "-2.8px",
                  }}
                >
                  {currentProject.progress}
                  <tspan
                    style={{
                      fontSize: "28px",
                      fontWeight: "300",
                    }}
                  >
                    %
                  </tspan>
                </text>
              </svg>
            </div>

            {/* Divider */}
            <div className="h-full w-px bg-[#cacaca] flex-shrink-0" />

            {/* Right side - Task details */}
            <div className="flex-1 flex flex-col gap-4 min-w-0">
              {/* Header */}
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <h3 className="text-neutral-50 text-[20px] tracking-[-0.2px] mb-2">
                    {currentProject.title}
                  </h3>
                  <p className="text-[#cacaca] text-[12px] tracking-[-0.12px] leading-[16px]">
                    {currentProject.description}
                  </p>
                </div>
                <div className="flex items-center gap-2 flex-shrink-0">
                  {/* Carousel arrows */}
                  <button
                    onClick={handlePrevious}
                    className="text-[#cacaca] hover:text-neutral-50 transition-colors p-1 hover:bg-neutral-900 rounded"
                  >
                    <ChevronLeft className="w-5 h-5" />
                  </button>
                  <button
                    onClick={handleNext}
                    className="text-[#cacaca] hover:text-neutral-50 transition-colors p-1 hover:bg-neutral-900 rounded"
                  >
                    <ChevronRight className="w-5 h-5" />
                  </button>
                  <button className="text-[#cacaca] hover:text-neutral-50 transition-colors flex-shrink-0">
                    <MoreVertical className="w-5 h-5" />
                  </button>
                </div>
              </div>

              {/* Divider */}
              <div className="w-full h-px bg-[#cacaca]" />

              {/* Info grid */}
              <div className="grid grid-cols-2 gap-y-3 gap-x-6">
                <div className="flex items-center justify-between">
                  <span className="text-[#cacaca] text-[14px] tracking-[-0.14px]">
                    Status:
                  </span>
                  <div className="flex items-center gap-2">
                    <span className="text-neutral-50 text-[14px] tracking-[-0.14px]">
                      {currentProject.status}
                    </span>
                    <CheckCircle2 className="w-4 h-4 text-neutral-50" />
                  </div>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-[#cacaca] text-[14px] tracking-[-0.14px]">
                    Progress:
                  </span>
                  <span className="text-neutral-50 text-[14px] tracking-[-0.14px]">
                    {currentProject.progress}%
                  </span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-[#cacaca] text-[14px] tracking-[-0.14px]">
                    Date:
                  </span>
                  <span className="text-neutral-50 text-[14px] tracking-[-0.14px]">
                    {currentProject.date}
                  </span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-[#cacaca] text-[14px] tracking-[-0.14px]">
                    ETR:
                  </span>
                  <span className="text-neutral-50 text-[14px] tracking-[-0.14px]">
                    {currentProject.estimatedDate}
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Carousel dots at the bottom */}
          <div className="flex justify-center gap-2 mt-4 pt-4 border-t border-[#cacaca]">
            {recentProjects.map((_, index) => (
              <button
                key={index}
                onClick={() => handleDotClick(index)}
                className={`w-2 h-2 rounded-full transition-all duration-300 ${
                  index === currentIndex
                    ? "bg-neutral-50 w-6"
                    : "bg-[#454545] hover:bg-[#cacaca]"
                }`}
                aria-label={`Go to project ${index + 1}`}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
