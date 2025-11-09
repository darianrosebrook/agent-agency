'use client';

import { X } from 'lucide-react';

interface TagChipProps {
  tag: string;
  onRemove?: (tag: string) => void;
  className?: string;
}

export function TagChip({ tag, onRemove, className = '' }: TagChipProps) {
  return (
    <span
      className={`inline-flex items-center gap-1 px-2 py-1 bg-zinc-700 rounded text-xs ${
        onRemove ? 'cursor-pointer hover:bg-zinc-600' : ''
      } transition-colors ${className}`}
      onClick={() => onRemove?.(tag)}
    >
      {tag}
      {onRemove && <X className="w-3 h-3 text-gray-400 hover:text-white" />}
    </span>
  );
}

