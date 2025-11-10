import { Bot } from "lucide-react";
import { Skeleton } from "../ui/skeleton";
import { TaskTimeline } from "../TaskTimeline";
import type { Task } from "../composers/Chat";
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
          <Bot className="w-4 h-4 text-gray-300" />
        </div>

        {/* Content */}
        <div className={styles.content}>
          {/* Loading Card */}
          <div className={styles.loadingCard}>
            {/* Skeleton Content */}
            <div className={styles.loadingCardContent}>
              {/* Content Lines Skeleton */}
              <div className={styles.contentLines}>
                <Skeleton className="h-3 w-full bg-gray-800" />
                <Skeleton className="h-3 w-[90%] bg-gray-800" />
                <Skeleton className="h-3 w-[95%] bg-gray-800" />
                <Skeleton className="h-3 w-[85%] bg-gray-800" />
                <Skeleton className="h-3 w-[92%] bg-gray-800" />
                <Skeleton className="h-3 w-[88%] bg-gray-800" />
                <Skeleton className="h-3 w-[75%] bg-gray-800" />
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
