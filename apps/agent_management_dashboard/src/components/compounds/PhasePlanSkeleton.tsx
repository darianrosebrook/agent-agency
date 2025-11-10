import { Skeleton } from "../ui/skeleton";
import styles from "./PhasePlanSkeleton.module.scss";

export function PhasePlanSkeleton() {
  return (
    <div className={styles.phasePlanSkeleton}>
      {/* Header skeleton */}
      <div className={styles.header}>
        <Skeleton className={styles.headerTitle} />
        <Skeleton className={styles.headerDescription} />
        <div className={styles.headerActions}>
          <Skeleton className={`${styles.headerAction} ${styles.headerActionFirst}`} />
          <Skeleton className={`${styles.headerAction} ${styles.headerActionSecond}`} />
        </div>
      </div>

      {/* Phase cards skeleton */}
      {[1, 2].map((phase) => (
        <div key={phase} className={styles.phaseCard}>
          {/* Phase header */}
          <div className={styles.phaseHeader}>
            <div className={styles.phaseHeaderTop}>
              <Skeleton className={`${styles.phaseTitle} ${styles.phaseTitleFirst}`} />
              <Skeleton className={`${styles.phaseTitle} ${styles.phaseTitleSecond}`} />
            </div>
            <Skeleton className={styles.phaseDescription} />
          </div>

          {/* Task items skeleton */}
          <div className={styles.taskList}>
            {[1, 2, 3].map((task) => (
              <div key={task} className={styles.taskItem}>
                <div className={styles.taskItemContent}>
                  <Skeleton className={styles.taskCheckbox} />
                  <Skeleton className={styles.taskText} />
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
