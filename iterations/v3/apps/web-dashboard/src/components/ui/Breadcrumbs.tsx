/**
 * Breadcrumb Navigation Component
 * Shows current location and navigation path
 * 
 * @author @darianrosebrook
 */

"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight, Home } from "lucide-react";
import { Text } from "@/design-system/primitives";
import styles from "./Breadcrumbs.module.scss";

interface BreadcrumbItem {
  label: string;
  href: string;
  isActive?: boolean;
}

interface BreadcrumbsProps {
  className?: string;
  customItems?: BreadcrumbItem[];
}

export default function Breadcrumbs({ className, customItems }: BreadcrumbsProps) {
  const pathname = usePathname();

  // Generate breadcrumbs from pathname or use custom items
  const breadcrumbs = customItems || generateBreadcrumbs(pathname);

  if (breadcrumbs.length <= 1) {
    return null; // Don't show breadcrumbs for root pages
  }

  return (
    <nav className={`${styles.breadcrumbs} ${className || ""}`} aria-label="Breadcrumb">
      <ol className={styles.breadcrumbList}>
        {breadcrumbs.map((item, index) => (
          <li key={item.href} className={styles.breadcrumbItem}>
            {index > 0 && (
              <ChevronRight className={styles.separator} size={14} />
            )}
            
            {item.isActive ? (
              <span className={styles.currentItem}>
                {index === 0 && <Home className={styles.homeIcon} size={14} />}
                <Text variant="paragraph-small" weight="medium">
                  {item.label}
                </Text>
              </span>
            ) : (
              <Link href={item.href} className={styles.breadcrumbLink}>
                {index === 0 && <Home className={styles.homeIcon} size={14} />}
                <Text variant="paragraph-small">
                  {item.label}
                </Text>
              </Link>
            )}
          </li>
        ))}
      </ol>
    </nav>
  );
}

function generateBreadcrumbs(pathname: string): BreadcrumbItem[] {
  const segments = pathname.split('/').filter(Boolean);
  const breadcrumbs: BreadcrumbItem[] = [
    {
      label: "Dashboard",
      href: "/",
      isActive: pathname === "/",
    },
  ];

  let currentPath = "";
  
  segments.forEach((segment, index) => {
    currentPath += `/${segment}`;
    const isLast = index === segments.length - 1;
    
    // Convert segment to readable label
    const label = formatSegmentLabel(segment);
    
    breadcrumbs.push({
      label,
      href: currentPath,
      isActive: isLast,
    });
  });

  return breadcrumbs;
}

function formatSegmentLabel(segment: string): string {
  // Handle special cases
  const specialCases: Record<string, string> = {
    "tasks": "Tasks",
    "metrics": "Metrics",
    "settings": "Settings",
    "chat": "Chat",
    "data-quality": "Data Quality",
    "analytics": "Analytics",
    "streaming-dashboard": "Streaming Dashboard",
  };

  if (specialCases[segment]) {
    return specialCases[segment];
  }

  // Handle dynamic segments (like task IDs)
  if (segment.match(/^[a-f0-9-]{36}$/)) {
    return "Task Details";
  }

  // Default: capitalize and replace hyphens
  return segment
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}
