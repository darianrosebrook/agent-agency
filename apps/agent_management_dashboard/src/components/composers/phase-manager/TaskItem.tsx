'use client';

import { Circle, CircleDashed } from 'lucide-react';
import {
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '../../ui/accordion';
import { Button } from '../../ui/button';
import { ContextChip } from './ContextChip';
import { SubtaskItem } from './SubtaskItem';
import { ContextMenu } from './ContextMenu';
import { calculateTaskProgress } from './utils';
import type { Task } from './types';
import { cn } from '../../ui/utils';
import styles from './TaskItem.module.scss';

interface TaskItemProps {
  task: Task;
  phaseId: string;
  onUpdateTitle: (newTitle: string) => void;
  onUpdateDescription: (newDescription: string) => void;
  onAddSubtask: () => void;
  onToggleSubtask: (subtaskId: string) => void;
  onDeleteSubtask: (subtaskId: string) => void;
  onAddContextChip: (
    type: 'file' | 'reference' | 'tool',
    label: string
  ) => void;
  onRemoveContextChip: (chipId: string) => void;
}

export function TaskItem({
  task,
  phaseId: _phaseId, // eslint-disable-line @typescript-eslint/no-unused-vars
  onUpdateTitle,
  onUpdateDescription,
  onAddSubtask,
  onToggleSubtask,
  onDeleteSubtask,
  onAddContextChip,
  onRemoveContextChip,
}: TaskItemProps) {
  const progress = calculateTaskProgress(task);

  return (
    <AccordionItem
      value={task.id}
      className={styles.taskItem}
    >
      <AccordionTrigger className={styles.taskTrigger}>
        <div className={styles.taskTriggerContent}>
          {task.subtasks.length > 0 ? (
            <div className={styles.taskProgress}>
              <Circle className={styles.taskProgressIcon} />
              <span className={styles.taskProgressText}>{progress}%</span>
            </div>
          ) : (
            <CircleDashed className={styles.taskProgressIcon} />
          )}
          <input
            type="text"
            value={task.title}
            onChange={(e) => onUpdateTitle(e.target.value)}
            onClick={(e) => e.stopPropagation()}
            className={styles.taskTitleInput}
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

        {task.subtasks.length > 0 && (
          <div className={styles.subtasksContainer}>
            {task.subtasks.map((subtask) => (
              <SubtaskItem
                key={subtask.id}
                subtask={subtask}
                onToggle={() => onToggleSubtask(subtask.id)}
                onDelete={() => onDeleteSubtask(subtask.id)}
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
            onAddFile={() => onAddContextChip('file', 'Uploaded file')}
            onAddReference={(type) =>
              onAddContextChip('reference', type)
            }
            onAddTool={(tool) => onAddContextChip('tool', tool)}
          />
        </div>
      </AccordionContent>
    </AccordionItem>
  );
}

