'use client';

import { Accordion } from '../../ui/accordion';
import { TaskItem } from './TaskItem';
import type { Phase, Task } from './types';

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
    <div className="mb-6 bg-[#1a1a1a] rounded-xl border border-zinc-800 overflow-hidden">
      <div className="px-6 py-5 border-b border-zinc-800">
        <div className="flex items-center gap-3 mb-2">
          <h3 className="text-xl text-white">{phase.title}</h3>
          <span className="px-3 py-1 bg-zinc-800 text-zinc-300 text-sm rounded-full">
            Phase {phase.number}
          </span>
        </div>
        <p className="text-zinc-400 text-sm">{phase.description}</p>
      </div>

      <Accordion type="multiple" className="w-full">
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

