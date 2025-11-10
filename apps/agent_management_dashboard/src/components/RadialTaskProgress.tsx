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
import styles from "./RadialTaskProgress.module.scss";

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
    <div className={styles.container}>
      <div className={styles.innerContainer}>
        <div className={styles.content}>
          {/* Main content area */}
          <div className={styles.mainContent}>
            {/* Left side - Radial chart */}
            <div className={styles.chartContainer}>
              <svg
                ref={svgRef}
                width="240"
                height="240"
                viewBox="0 0 240 240"
                xmlns="http://www.w3.org/2000/svg"
                className={styles.svg}
              >
                {segments}
                {/* Center circle with percentage */}
                <circle
                  cx="120"
                  cy="120"
                  r="65"
                  className={styles.centerCircle}
                />
                <text
                  x="120"
                  y="130"
                  textAnchor="middle"
                  className={styles.centerText}
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
            <div className={styles.divider} />

            {/* Right side - Task details */}
            <div className={styles.detailsContainer}>
              {/* Header */}
              <div className={styles.detailsHeader}>
                <div className={styles.detailsTitleContainer}>
                  <h3 className={styles.detailsTitle}>
                    {currentProject.title}
                  </h3>
                  <p className={styles.detailsDescription}>
                    {currentProject.description}
                  </p>
                </div>
                <div className={styles.detailsActions}>
                  {/* Carousel arrows */}
                  <button
                    onClick={handlePrevious}
                    className={styles.actionButton}
                  >
                    <ChevronLeft className={styles.actionButtonIcon} />
                  </button>
                  <button onClick={handleNext} className={styles.actionButton}>
                    <ChevronRight className={styles.actionButtonIcon} />
                  </button>
                  <button
                    className={`${styles.actionButton} ${styles.actionButtonShrink}`}
                  >
                    <MoreVertical className={styles.actionButtonIcon} />
                  </button>
                </div>
              </div>

              {/* Divider */}
              <div className={styles.detailsDivider} />

              {/* Info grid */}
              <div className={styles.infoGrid}>
                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>Status:</span>
                  <div className={styles.infoValueGroup}>
                    <span className={styles.infoValue}>
                      {currentProject.status}
                    </span>
                    <CheckCircle2 className={styles.infoIcon} />
                  </div>
                </div>

                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>Progress:</span>
                  <span className={styles.infoValue}>
                    {currentProject.progress}%
                  </span>
                </div>

                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>Date:</span>
                  <span className={styles.infoValue}>
                    {currentProject.date}
                  </span>
                </div>

                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>ETR:</span>
                  <span className={styles.infoValue}>
                    {currentProject.estimatedDate}
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Carousel dots at the bottom */}
          <div className={styles.carouselDots}>
            {recentProjects.map((_, index) => (
              <button
                key={index}
                onClick={() => handleDotClick(index)}
                className={`${styles.carouselDot} ${
                  index === currentIndex
                    ? styles.carouselDotActive
                    : styles.carouselDotInactive
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
