'use client';

import { CheckCircle2, Circle, Trash2 } from 'lucide-react';
import type { Subtask } from './types';

interface SubtaskItemProps {
  subtask: Subtask;
  onToggle: () => void;
  onDelete: () => void;
}

export function SubtaskItem({ subtask, onToggle, onDelete }: SubtaskItemProps) {
  return (
    <div className="flex items-center gap-3 group py-1">
      <button onClick={onToggle} className="flex-shrink-0">
        {subtask.completed ? (
          <CheckCircle2 className="w-4 h-4 text-green-500" />
        ) : (
          <Circle className="w-4 h-4 text-zinc-500" />
        )}
      </button>
      <span
        className={`flex-1 text-sm ${
          subtask.completed
            ? 'text-zinc-500 line-through'
            : 'text-zinc-300'
        }`}
      >
        {subtask.text}
      </span>
      <button
        onClick={onDelete}
        className="opacity-0 group-hover:opacity-100 transition-opacity text-zinc-500 hover:text-red-500"
      >
        <Trash2 className="w-4 h-4" />
      </button>
    </div>
  );
}

