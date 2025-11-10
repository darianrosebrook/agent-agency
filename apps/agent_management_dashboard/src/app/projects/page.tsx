import { Suspense } from "react";
import dynamic from "next/dynamic";
import styles from "./page.module.scss";

const Projects = dynamic(
  () =>
    import("@/components/projects/Projects").then((mod) => ({
      default: mod.Projects,
    })),
  {
    loading: () => (
      <div className={styles.loadingContainer}>
        <div className={styles.loadingText}>Loading projects...</div>
      </div>
    ),
  }
);

export default function ProjectsPage() {
  return (
    <Suspense
      fallback={
        <div className={styles.loadingContainer}>
          <div className={styles.loadingText}>Loading projects...</div>
        </div>
      }
    >
      <Projects />
    </Suspense>
  );
}
