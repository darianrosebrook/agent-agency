'use client';

import { Accordion } from '../../ui/accordion';
import { TaskItem } from './TaskItem';
import type { Phase } from './types';
import styles from './PhaseItem.module.scss';

interface PhaseItemProps {
  phase: Phase;
  onUpdateTaskTitle: (taskId: string, newTitle: string) => void;
  onUpdateTaskDescription: (taskId: string, newDescription: string) => void;
  onAddSubtask: (taskId: string) => void;
  onToggleSubtask: (taskId: string, subtaskId: string) => void;
  onDeleteSubtask: (taskId: string, subtaskId: string) => void;
  onAddContextChip: (
    taskId: string,
    type: 'file' | 'reference' | 'tool',
    label: string
  ) => void;
  onRemoveContextChip: (taskId: string, chipId: string) => void;
}

export function PhaseItem({
  phase,
  onUpdateTaskTitle,
  onUpdateTaskDescription,
  onAddSubtask,
  onToggleSubtask,
  onDeleteSubtask,
  onAddContextChip,
  onRemoveContextChip,
}: PhaseItemProps) {
  return (
    <div className={styles.phaseItem}>
      <div className={styles.phaseHeader}>
        <div className={styles.phaseHeaderTop}>
          <h3 className={styles.phaseTitle}>{phase.title}</h3>
          <span className={styles.phaseBadge}>
            Phase {phase.number}
          </span>
        </div>
        <p className={styles.phaseDescription}>{phase.description}</p>
      </div>

      <Accordion type="multiple" className={styles.phaseAccordion}>
        {phase.tasks.map((task) => (
          <TaskItem
            key={task.id}
            task={task}
            phaseId={phase.id}
            onUpdateTitle={(newTitle) =>
              onUpdateTaskTitle(task.id, newTitle)
            }
            onUpdateDescription={(newDescription) =>
              onUpdateTaskDescription(task.id, newDescription)
            }
            onAddSubtask={() => onAddSubtask(task.id)}
            onToggleSubtask={(subtaskId) =>
              onToggleSubtask(task.id, subtaskId)
            }
            onDeleteSubtask={(subtaskId) =>
              onDeleteSubtask(task.id, subtaskId)
            }
            onAddContextChip={(type, label) =>
              onAddContextChip(task.id, type, label)
            }
            onRemoveContextChip={(chipId) =>
              onRemoveContextChip(task.id, chipId)
            }
          />
        ))}
      </Accordion>
    </div>
  );
}




