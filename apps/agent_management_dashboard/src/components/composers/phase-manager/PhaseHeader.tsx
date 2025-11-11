"use client";

import { Button } from "../../primitives/button";
import styles from "./PhaseHeader.module.scss";

interface PhaseHeaderProps {
  onSaveToProject?: () => void;
}

export function PhaseHeader({ onSaveToProject }: PhaseHeaderProps) {
  return (
    <div className={styles.phaseHeader}>
      <h2 className={styles.phaseTitle}>Project Plan</h2>
      <p className={styles.phaseDescription}>
        Here&apos;s a comprehensive plan for building your multi-modal RAG
        search UI tool
      </p>

      <div className={styles.phaseActions}>
        <Button onClick={onSaveToProject} className={styles.addToProjectButton}>
          Add to Project
        </Button>
        <Button variant="outline" className={styles.startNewProjectButton}>
          Start New Project
        </Button>
      </div>
    </div>
  );
}




