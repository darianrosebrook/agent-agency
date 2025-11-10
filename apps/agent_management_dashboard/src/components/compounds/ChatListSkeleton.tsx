/**
 * Chat List Skeleton Loader
 *
 * Displays skeleton loading state for chat list items.
 *
 * @author @darianrosebrook
 */

"use client";

import { Skeleton } from "../primitives/skeleton";
import { MessageSquare } from "lucide-react";
import styles from "./ChatListSkeleton.module.scss";

export function ChatListSkeleton({ count = 5 }: { count?: number }) {
  return (
    <div className={styles.chatListSkeleton}>
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className={styles.chatSkeletonItem}>
          <MessageSquare className={styles.chatSkeletonIcon} />
          <Skeleton className={styles.chatSkeletonText} />
        </div>
      ))}
    </div>
  );
}
