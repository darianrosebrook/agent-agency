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
      className="border-b border-zinc-800 last:border-0"
    >
      <AccordionTrigger className="px-6 py-4 hover:bg-[#1f1f1f] hover:no-underline">
        <div className="flex items-center gap-3 flex-1 text-left">
          {task.subtasks.length > 0 ? (
            <div className="flex items-center gap-2">
              <Circle className="w-5 h-5 text-zinc-500" />
              <span className="text-sm text-zinc-400">{progress}%</span>
            </div>
          ) : (
            <CircleDashed className="w-5 h-5 text-zinc-500" />
          )}
          <input
            type="text"
            value={task.title}
            onChange={(e) => onUpdateTitle(e.target.value)}
            onClick={(e) => e.stopPropagation()}
            className="w-full bg-transparent border-none outline-none text-zinc-100 focus:ring-2 focus:ring-blue-500 rounded px-2 py-1 -ml-2"
          />
        </div>
      </AccordionTrigger>
      <AccordionContent className="px-6 pb-6 bg-[#0f0f0f]">
        {task.description && (
          <div className="mb-4 ml-8">
            <textarea
              value={task.description}
              onChange={(e) => onUpdateDescription(e.target.value)}
              placeholder="Add a description..."
              className="w-full bg-transparent border-none outline-none text-zinc-400 text-sm resize-none focus:ring-2 focus:ring-blue-500 rounded px-2 py-1 mt-2 min-h-[60px]"
            />
          </div>
        )}

        {task.contextChips.length > 0 && (
          <div className="flex flex-wrap gap-2 mb-4 ml-8">
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
          <div className="space-y-2 mb-4 ml-8">
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

        <div className="flex items-center gap-2 ml-8">
          <Button
            variant="outline"
            size="sm"
            onClick={onAddSubtask}
            className="text-zinc-300 border-zinc-700 hover:bg-zinc-800 hover:text-zinc-100 bg-zinc-950"
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

