"use client";

import { useState, useMemo, useEffect, useRef } from "react";
import { gsap } from "gsap";
import {
  CheckCircle2,
  MoreVertical,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";
import { useGSAPNumberAnimation } from "../hooks/useGSAPAnimation";

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

// TODO: Replace hardcoded project data with data from v3 database with the following requirements:
// 1. Recent projects fetching: Load recent projects sorted by last accessed
//    - Data source: GET /api/projects?limit=6&sort=last_accessed endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
//    - Database table: PostgreSQL `projects` table
//    - Include project metadata: id, title, description, progress, status, dates
// 2. Progress calculation: Calculate project progress from task completion
//    - Aggregate completed tasks vs total tasks per project
//    - Calculate progress percentage for display
// 3. Data transformation: Format API response for component
//    - Map API response to Project array with required fields
//    - Handle date formatting and status mapping
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
  const svgRef = useRef<SVGSVGElement>(null);
  const segmentsRef = useRef<SVGPathElement[]>([]);

  // Use GSAP for smooth number animation
  const animatedProgress = useGSAPNumberAnimation(
    currentProject.progress,
    0.8,
    "power2.out"
  );

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
    (animatedProgress / 100) * totalSegments
  );

  // Generate radial segments
  // Format numbers to fixed decimal places to prevent hydration mismatches
  const formatNumber = (value: number, decimals: number = 2): string => {
    return value.toFixed(decimals);
  };

  // Memoize segments to ensure consistent generation between server and client
  const segments = useMemo(() => {
    const segmentList = [];
    const segmentAngle = 360 / totalSegments;
    const gapAngle = 2; // Gap between segments
    const radius = 100;
    const innerRadius = 70;
    const centerX = 120;
    const centerY = 120;

    for (let i = 0; i < totalSegments; i++) {
      const startAngle = i * segmentAngle - 90; // Start from top
      const endAngle = startAngle + segmentAngle - gapAngle;

      // Calculate coordinates and format to fixed decimal places for consistent rendering
      const x1 = formatNumber(
        centerX + radius * Math.cos((startAngle * Math.PI) / 180)
      );
      const y1 = formatNumber(
        centerY + radius * Math.sin((startAngle * Math.PI) / 180)
      );
      const x2 = formatNumber(
        centerX + radius * Math.cos((endAngle * Math.PI) / 180)
      );
      const y2 = formatNumber(
        centerY + radius * Math.sin((endAngle * Math.PI) / 180)
      );
      const x3 = formatNumber(
        centerX + innerRadius * Math.cos((endAngle * Math.PI) / 180)
      );
      const y3 = formatNumber(
        centerY + innerRadius * Math.sin((endAngle * Math.PI) / 180)
      );
      const x4 = formatNumber(
        centerX + innerRadius * Math.cos((startAngle * Math.PI) / 180)
      );
      const y4 = formatNumber(
        centerY + innerRadius * Math.sin((startAngle * Math.PI) / 180)
      );

      // Build path data string with formatted numbers
      const pathData = `M ${x1} ${y1} A ${radius} ${radius} 0 0 1 ${x2} ${y2} L ${x3} ${y3} A ${innerRadius} ${innerRadius} 0 0 0 ${x4} ${y4} Z`;

      segmentList.push(
        <path
          key={i}
          ref={(el) => {
            if (el) segmentsRef.current[i] = el;
          }}
          d={pathData}
          fill={i < completedSegments ? "#fafafa" : "#454545"}
        />
      );
    }

    return segmentList;
  }, [totalSegments, completedSegments]);

  // Animate segments with GSAP when progress changes
  useEffect(() => {
    if (segmentsRef.current.length === 0) return;

    const completedCount = completedSegments;

    // Animate segments with stagger effect
    segmentsRef.current.forEach((segment, index) => {
      const isCompleted = index < completedCount;
      const targetColor = isCompleted ? "#fafafa" : "#454545";

      gsap.to(segment, {
        fill: targetColor,
        duration: 0.3,
        delay: index * 0.02, // Stagger delay
        ease: "power2.out",
      });
    });
  }, [completedSegments]);

  // Initial animation on mount
  useEffect(() => {
    if (segmentsRef.current.length === 0 || !svgRef.current) return;

    // Animate SVG entrance
    gsap.fromTo(
      svgRef.current,
      { opacity: 0, scale: 0.9 },
      {
        opacity: 1,
        scale: 1,
        duration: 0.6,
        ease: "back.out(1.7)",
      }
    );

    // Animate segments entrance with stagger
    gsap.fromTo(
      segmentsRef.current,
      { opacity: 0, scale: 0.8 },
      {
        opacity: 1,
        scale: 1,
        duration: 0.4,
        stagger: 0.03,
        delay: 0.2,
        ease: "back.out(1.7)",
      }
    );
  }, []);

  return (
    <div className="bg-[#111] relative rounded-[12px] size-full border border-[#cacaca]">
      <div className="size-full">
        <div className="box-border flex flex-col p-6 relative size-full">
          {/* Main content area */}
          <div className="flex items-center gap-6 flex-1 min-h-0">
            {/* Left side - Radial chart */}
            <div className="flex-shrink-0">
              <svg
                ref={svgRef}
                width="240"
                height="240"
                viewBox="0 0 240 240"
                xmlns="http://www.w3.org/2000/svg"
                className="transform -rotate-0"
              >
                {segments}
                {/* Center circle with percentage */}
                <circle cx="120" cy="120" r="65" fill="#0a0a0a" />
                <text
                  x="120"
                  y="130"
                  textAnchor="middle"
                  className="fill-neutral-50 transition-none"
                  style={{
                    fontSize: "56px",
                    fontWeight: "300",
                    letterSpacing: "-2.8px",
                  }}
                >
                  {animatedProgress}
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
