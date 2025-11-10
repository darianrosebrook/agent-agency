import { Skeleton } from "../ui/skeleton";
import { cn } from "../ui/utils";
import styles from "./PhasePlanSkeleton.module.scss";

export function PhasePlanSkeleton() {
  return (
    <div className={styles.phasePlanSkeleton}>
      {/* Header skeleton */}
      <div className={styles.header}>
        <Skeleton className={cn(styles.headerTitle, "bg-gray-800")} />
        <Skeleton className={cn(styles.headerDescription, "bg-gray-800")} />
        <div className={styles.headerActions}>
          <Skeleton className={cn(styles.headerAction, styles.headerActionFirst, "bg-gray-800")} />
          <Skeleton className={cn(styles.headerAction, styles.headerActionSecond, "bg-gray-800")} />
        </div>
      </div>

      {/* Phase cards skeleton */}
      {[1, 2].map((phase) => (
        <div key={phase} className={styles.phaseCard}>
          {/* Phase header */}
          <div className={styles.phaseHeader}>
            <div className={styles.phaseHeaderTop}>
              <Skeleton className={cn(styles.phaseTitle, styles.phaseTitleFirst, "bg-gray-800")} />
              <Skeleton className={cn(styles.phaseTitle, styles.phaseTitleSecond, "bg-gray-800")} />
            </div>
            <Skeleton className={cn(styles.phaseDescription, "bg-gray-800")} />
          </div>

          {/* Task items skeleton */}
          <div className={styles.taskList}>
            {[1, 2, 3].map((task) => (
              <div key={task} className={styles.taskItem}>
                <div className={styles.taskItemContent}>
                  <Skeleton className={cn(styles.taskCheckbox, "bg-gray-800")} />
                  <Skeleton className={cn(styles.taskText, "bg-gray-800")} />
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
