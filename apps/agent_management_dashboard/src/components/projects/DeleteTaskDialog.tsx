"use client";

/**
 * Delete Task Confirmation Dialog
 *
 * @author @darianrosebrook
 */

import { AlertTriangle } from "lucide-react";
import styles from "./DeleteTaskDialog.module.scss";

interface DeleteTaskDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  taskTitle?: string;
}

export function DeleteTaskDialog({
  open,
  onOpenChange,
  onConfirm,
  taskTitle,
}: DeleteTaskDialogProps) {
  if (!open) return null;

  const handleConfirm = () => {
    onConfirm();
    onOpenChange(false);
  };

  return (
    <div className={styles.overlay} onClick={() => onOpenChange(false)}>
      <div className={styles.dialog} onClick={(e) => e.stopPropagation()}>
        <div className={styles.dialogHeader}>
          <AlertTriangle className={styles.warningIcon} />
          <h3 className={styles.dialogTitle}>Delete Task</h3>
        </div>

        <div className={styles.dialogContent}>
          <p className={styles.dialogMessage}>
            Are you sure you want to delete this task?
            {taskTitle && (
              <>
                <br />
                <strong className={styles.taskTitle}>{taskTitle}</strong>
              </>
            )}
          </p>
          <p className={styles.dialogWarning}>This action cannot be undone.</p>
        </div>

        <div className={styles.dialogActions}>
          <button
            onClick={() => onOpenChange(false)}
            className={styles.cancelButton}
          >
            Cancel
          </button>
          <button onClick={handleConfirm} className={styles.deleteButton}>
            Delete Task
          </button>
        </div>
      </div>
    </div>
  );
}









