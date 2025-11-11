"use client";

/**
 * Edit Task Modal
 *
 * Modal for editing existing tasks with full CRUD support.
 *
 * @author @darianrosebrook
 */

import { useState, useEffect } from "react";
import {
  X,
  ChevronDown,
  MoreHorizontal,
  Circle,
  CheckCircle2,
} from "lucide-react";
import {
  StatusBadge,
  PriorityIndicator,
  MetadataRow,
  TagChip,
  taskStatusConfig,
  priorityConfig,
  type TaskStatus,
  type Priority,
} from "../compounds";
import { cn } from "../primitives/utils";
import styles from "./TaskModal.module.scss";

interface EditTaskModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onUpdateTask: (data: {
    title: string;
    description?: string;
    status: "backlog" | "todo" | "in-progress" | "done";
    priority?: string;
  }) => void;
  task: {
    id: string;
    title: string;
    description?: string;
    status: "backlog" | "todo" | "in-progress" | "done";
    priority?: "low" | "medium" | "high";
  } | null;
}

type Status = TaskStatus;

export function EditTaskModal({
  open,
  onOpenChange,
  onUpdateTask,
  task,
}: EditTaskModalProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState<Status>("backlog");
  const [priority, setPriority] = useState<Priority>("medium");

  const [showStatusMenu, setShowStatusMenu] = useState(false);
  const [showPriorityMenu, setShowPriorityMenu] = useState(false);

  // Initialize form when task changes
  useEffect(() => {
    if (task) {
      setTitle(task.title);
      setDescription(task.description ?? "");
      setStatus(task.status);
      setPriority(task.priority ?? "medium");
    }
  }, [task]);

  const handleUpdate = () => {
    if (title.trim() && task) {
      onUpdateTask({
        title: title.trim(),
        description: description.trim() || undefined,
        status,
        priority,
      });
      onOpenChange(false);
    }
  };

  if (!open || !task) return null;

  return (
    <div className={styles.modalOverlay} onClick={() => onOpenChange(false)}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className={styles.modalHeader}>
          <button
            onClick={() => onOpenChange(false)}
            className={styles.closeButton}
          >
            <X className={styles.closeButtonIcon} />
          </button>
          <div className={styles.modalHeaderTitle}>
            <span>Edit Task</span>
          </div>
        </div>

        {/* Content */}
        <div className={styles.modalContent}>
          {/* Title */}
          <div>
            <input
              type="text"
              placeholder="Task title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className={styles.titleInput}
              autoFocus
            />
            <textarea
              placeholder="Add a description for this task..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className={styles.descriptionTextarea}
              rows={3}
            />
          </div>

          {/* Metadata Grid */}
          <div className={styles.metadataGrid}>
            {/* Status */}
            <MetadataRow label="Status">
              <div className={styles.metadataDropdown}>
                <StatusBadge
                  status={status}
                  config={taskStatusConfig[status]}
                  onClick={() => setShowStatusMenu(!showStatusMenu)}
                />
                {showStatusMenu && (
                  <div className={styles.dropdownMenu}>
                    {(Object.keys(taskStatusConfig) as TaskStatus[]).map(
                      (key) => (
                        <button
                          key={key}
                          onClick={() => {
                            setStatus(key);
                            setShowStatusMenu(false);
                          }}
                          className={styles.dropdownMenuItem}
                        >
                          <StatusBadge
                            status={key}
                            config={taskStatusConfig[key]}
                          />
                        </button>
                      )
                    )}
                  </div>
                )}
              </div>
            </MetadataRow>

            {/* Priority */}
            <MetadataRow label="Priority">
              <div className={styles.metadataDropdown}>
                <PriorityIndicator
                  priority={priority}
                  config={priorityConfig[priority]}
                  onClick={() => setShowPriorityMenu(!showPriorityMenu)}
                />
                {showPriorityMenu && (
                  <div
                    className={cn(
                      styles.dropdownMenu,
                      styles.dropdownMenuPriority
                    )}
                  >
                    {(Object.keys(priorityConfig) as Priority[]).map((key) => (
                      <button
                        key={key}
                        onClick={() => {
                          setPriority(key);
                          setShowPriorityMenu(false);
                        }}
                        className={cn(
                          styles.dropdownMenuItem,
                          styles.dropdownMenuItemPriority
                        )}
                      >
                        <PriorityIndicator
                          priority={key}
                          config={priorityConfig[key]}
                        />
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </MetadataRow>
          </div>
        </div>

        {/* Footer */}
        <div className={styles.modalFooter}>
          <button
            onClick={() => onOpenChange(false)}
            className={styles.cancelButton}
          >
            Cancel
          </button>
          <button
            onClick={handleUpdate}
            disabled={!title.trim()}
            className={styles.createButton}
          >
            Update Task
          </button>
        </div>
      </div>
    </div>
  );
}


