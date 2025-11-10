/**
 * Project List Skeleton Loader
 * 
 * Displays skeleton loading state for project list items.
 * 
 * @author @darianrosebrook
 */

"use client";

import { Skeleton } from "../ui/skeleton";
import { Folder } from "lucide-react";
import styles from "./ProjectListSkeleton.module.scss";

export function ProjectListSkeleton({ count = 6 }: { count?: number }) {
  return (
    <div className={styles.projectListSkeleton}>
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className={styles.projectSkeletonItem}
        >
          <Folder className={styles.projectSkeletonIcon} />
          <div className={styles.projectSkeletonContent}>
            <Skeleton className={styles.projectSkeletonTitle} />
            <Skeleton className={styles.projectSkeletonSubtitle} />
          </div>
          <Skeleton className={styles.projectSkeletonDate} />
        </div>
      ))}
    </div>
  );
}

