"use client";

import { CheckCircle2, Circle, Trash2 } from "lucide-react";
import type { Subtask } from "./types";
import { cn } from "../../primitives/utils";
import styles from "./SubtaskItem.module.scss";

interface SubtaskItemProps {
  subtask: Subtask;
  onToggle: () => void;
  onDelete: () => void;
}

export function SubtaskItem({ subtask, onToggle, onDelete }: SubtaskItemProps) {
  return (
    <div className={styles.subtaskItem}>
      <button onClick={onToggle} className={styles.subtaskToggle}>
        {subtask.completed ? (
          <CheckCircle2
            className={cn(styles.subtaskIcon, styles.subtaskIconCompleted)}
          />
        ) : (
          <Circle
            className={cn(styles.subtaskIcon, styles.subtaskIconIncomplete)}
          />
        )}
      </button>
      <span
        className={cn(
          styles.subtaskText,
          subtask.completed
            ? styles.subtaskTextCompleted
            : styles.subtaskTextIncomplete
        )}
      >
        {subtask.text}
      </span>
      <button onClick={onDelete} className={styles.subtaskDelete}>
        <Trash2 className={styles.subtaskDeleteIcon} />
      </button>
    </div>
  );
}



