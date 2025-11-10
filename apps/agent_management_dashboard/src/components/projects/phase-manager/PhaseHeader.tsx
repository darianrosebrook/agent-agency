'use client';

import { Button } from '../../ui/button';
import styles from './PhaseHeader.module.scss';

interface PhaseHeaderProps {
  onSaveToProject?: () => void;
}

export function PhaseHeader({ onSaveToProject }: PhaseHeaderProps) {
  return (
    <div className={styles.phaseHeader}>
      <h2 className={styles.phaseTitle}>Project Plan</h2>
      <p className={styles.phaseDescription}>
        Here&apos;s a comprehensive plan for building your multi-modal RAG search
        UI tool
      </p>

      <div className={styles.phaseActions}>
        <Button
          onClick={onSaveToProject}
          className="bg-blue-600 text-white hover:bg-blue-700"
        >
          Add to Project
        </Button>
        <Button
          variant="outline"
          className="bg-[#1a1a1a] border-zinc-700 text-zinc-300 hover:bg-zinc-800"
        >
          Start New Project
        </Button>
      </div>
    </div>
  );
}

