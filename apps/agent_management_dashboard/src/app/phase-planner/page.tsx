'use client';

import { PhaseManager } from "@/components/projects/PhaseManager";
import { useProjectContext } from "@/components/projects/ProjectContext";
import styles from "./page.module.scss";

export default function PhasePlannerPage() {
  // Try to get current project ID from context (if user navigated from a project)
  const { currentProjectId } = useProjectContext();

  return (
    <div className={styles.phasePlannerPage}>
      <PhaseManager projectId={currentProjectId || undefined} />
    </div>
  );
}
