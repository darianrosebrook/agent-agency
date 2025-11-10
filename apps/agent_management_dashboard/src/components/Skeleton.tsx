/**
 * Skeleton Loading Components
 * 
 * Provides skeleton loaders for various content types.
 * 
 * @author @darianrosebrook
 */

import React from "react";

"use client";

import { cn } from "./primitives/utils";
import styles from "./Skeleton.module.scss";

/**
 * Base skeleton component
 */
export function Skeleton({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(styles.skeleton, className)}
      {...props}
    />
  );
}

/**
 * Chat message skeleton
 */
export function ChatMessageSkeleton() {
  return (
    <div className={styles.skeletonChatMessage}>
      <Skeleton className={styles.skeletonChatAvatar} />
      <div className={styles.skeletonChatContent}>
        <Skeleton className={cn(styles.skeletonChatLine, styles.skeletonChatLineWide)} />
        <Skeleton className={cn(styles.skeletonChatLine, styles.skeletonChatLineNarrow)} />
      </div>
    </div>
  );
}

/**
 * Project card skeleton
 */
export function ProjectCardSkeleton() {
  return (
    <div className={styles.skeletonProjectCard}>
      <Skeleton className={styles.skeletonProjectTitle} />
      <Skeleton className={styles.skeletonProjectLine} />
      <Skeleton className={cn(styles.skeletonProjectLine, styles.skeletonProjectLineNarrow)} />
      <div className={styles.skeletonProjectTags}>
        <Skeleton className={styles.skeletonTag} />
        <Skeleton className={styles.skeletonTag} />
      </div>
    </div>
  );
}

/**
 * Table row skeleton
 */
export function TableRowSkeleton({ columns = 4 }: { columns?: number }) {
  return (
    <tr>
      {Array.from({ length: columns }).map((_, i) => (
        <td key={i} className={styles.skeletonTableRow}>
          <Skeleton className={styles.skeletonTableCell} />
        </td>
      ))}
    </tr>
  );
}

/**
 * List item skeleton
 */
export function ListItemSkeleton() {
  return (
    <div className={styles.skeletonListItem}>
      <Skeleton className={styles.skeletonListItemAvatar} />
      <div className={styles.skeletonListItemContent}>
        <Skeleton className={styles.skeletonListItemTitle} />
        <Skeleton className={styles.skeletonListItemSubtitle} />
      </div>
    </div>
  );
}

