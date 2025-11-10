import { Bot } from "lucide-react";
import { Skeleton } from "../ui/skeleton";
import { TaskTimeline } from "../TaskTimeline";
import type { Task } from "../../lib/schemas/chat";
import styles from "./ChatMessageSkeleton.module.scss";

interface ChatMessageSkeletonProps {
  tasks?: Task[];
}

export function ChatMessageSkeleton({ tasks = [] }: ChatMessageSkeletonProps) {
  return (
    <div className={styles.chatMessageSkeleton}>
      {/* Task Timeline - appears above message */}
      {tasks.length > 0 && (
        <div className={styles.taskTimelineContainer}>
          <TaskTimeline tasks={tasks} />
        </div>
      )}

      {/* Loading Message */}
      <div className={styles.messageWrapper}>
        {/* Avatar */}
        <div className={styles.avatar}>
          <Bot className={styles.avatarIcon} />
        </div>

        {/* Content */}
        <div className={styles.content}>
          {/* Loading Card */}
          <div className={styles.loadingCard}>
            {/* Skeleton Content */}
            <div className={styles.loadingCardContent}>
              {/* Content Lines Skeleton */}
              <div className={styles.contentLines}>
                <Skeleton className={styles.contentLine} />
                <Skeleton className={`${styles.contentLine} ${styles.contentLine90}`} />
                <Skeleton className={`${styles.contentLine} ${styles.contentLine95}`} />
                <Skeleton className={`${styles.contentLine} ${styles.contentLine85}`} />
                <Skeleton className={`${styles.contentLine} ${styles.contentLine92}`} />
                <Skeleton className={`${styles.contentLine} ${styles.contentLine88}`} />
                <Skeleton className={`${styles.contentLine} ${styles.contentLine75}`} />
              </div>

              {/* Pulsing Indicator */}
              <div className={styles.pulsingIndicator}>
                <div className={styles.pulsingDots}>
                  <div
                    className={styles.pulsingDot}
                    style={{ animationDelay: "0ms" }}
                  ></div>
                  <div
                    className={styles.pulsingDot}
                    style={{ animationDelay: "150ms" }}
                  ></div>
                  <div
                    className={styles.pulsingDot}
                    style={{ animationDelay: "300ms" }}
                  ></div>
                </div>
                <span className={styles.pulsingText}>
                  Generating response...
                </span>
              </div>
            </div>
          </div>

          {/* Timestamp Skeleton */}
          <Skeleton className={styles.timestampSkeleton} />
        </div>
      </div>
    </div>
  );
}
