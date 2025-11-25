"use client";

import { Circle, CircleDashed, CheckCircle2 } from "lucide-react";
import {
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "../../primitives/accordion";
import { Button } from "../../primitives/button";
import { ContextChip } from "./ContextChip";
import { ContextMenu } from "./ContextMenu";
import { SubtaskItem } from "./SubtaskItem";
import styles from "./TaskItem.module.scss";
import type { Task } from "./types";
import { calculateTaskProgress } from "./utils";
import { cn } from "../../primitives/utils";

interface TaskItemProps {
  task: Task;
  phaseId: string;
  onUpdateTitle: (newTitle: string) => void;
  onUpdateDescription: (newDescription: string) => void;
  onToggleTask: () => void;
  onAddSubtask: () => void;
  onUpdateSubtaskText: (subtaskId: string, newText: string) => void;
  onToggleSubtask: (subtaskId: string) => void;
  onDeleteSubtask: (subtaskId: string) => void;
  onAddContextChip: (
    type: "file" | "reference" | "tool",
    label: string
  ) => void;
  onRemoveContextChip: (chipId: string) => void;
}

export function TaskItem({
  task,
  phaseId: _phaseId, // eslint-disable-line no-unused-vars
  onUpdateTitle,
  onUpdateDescription,
  onToggleTask,
  onAddSubtask,
  onUpdateSubtaskText,
  onToggleSubtask,
  onDeleteSubtask,
  onAddContextChip,
  onRemoveContextChip,
}: TaskItemProps) {
  const progress = calculateTaskProgress(task);
  const isTaskCompleted = task.completed || false;

  return (
    <AccordionItem value={task.id} className={styles.taskItem}>
      <AccordionTrigger className={styles.taskTrigger}>
        <div className={styles.taskTriggerContent}>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onToggleTask();
            }}
            className={styles.taskToggleButton}
          >
            {isTaskCompleted ? (
              <CheckCircle2 className={styles.taskCompletedIcon} />
            ) : task.subtasks.length > 0 ? (
              <div className={styles.taskProgress}>
                <Circle className={styles.taskProgressIcon} />
                <span className={styles.taskProgressText}>{progress}%</span>
              </div>
            ) : (
              <CircleDashed className={styles.taskProgressIcon} />
            )}
          </button>
          <input
            type="text"
            value={task.title}
            onChange={(e) => onUpdateTitle(e.target.value)}
            onClick={(e) => e.stopPropagation()}
            className={cn(
              styles.taskTitleInput,
              isTaskCompleted && styles.taskTitleCompleted
            )}
          />
        </div>
      </AccordionTrigger>
      <AccordionContent className={styles.taskContent}>
        {task.description && (
          <div className={styles.taskDescriptionContainer}>
            <textarea
              value={task.description}
              onChange={(e) => onUpdateDescription(e.target.value)}
              placeholder="Add a description..."
              className={styles.taskDescription}
            />
          </div>
        )}

        {task.contextChips.length > 0 && (
          <div className={styles.contextChipsContainer}>
            {task.contextChips.map((chip) => (
              <ContextChip
                key={chip.id}
                chip={chip}
                onRemove={() => onRemoveContextChip(chip.id)}
              />
            ))}
          </div>
        )}
        <div className={styles.taskActions}>
          <Button
            variant="outline"
            size="sm"
            onClick={onAddSubtask}
            className={styles.addSubtaskButton}
          >
            Add subtask
          </Button>

          <ContextMenu
            onAddFile={() => onAddContextChip("file", "Uploaded file")}
            onAddReference={(type) => onAddContextChip("reference", type)}
            onAddTool={(tool) => onAddContextChip("tool", tool)}
          />
        </div>
        {task.subtasks.length > 0 && (
          <div className={styles.subtasksContainer}>
            {task.subtasks.map((subtask) => (
              <SubtaskItem
                key={subtask.id}
                subtask={subtask}
                onToggle={() => onToggleSubtask(subtask.id)}
                onDelete={() => onDeleteSubtask(subtask.id)}
                onUpdateText={(newText) => onUpdateSubtaskText(subtask.id, newText)}
              />
            ))}
          </div>
        )}
      </AccordionContent>
    </AccordionItem>
  );
}
