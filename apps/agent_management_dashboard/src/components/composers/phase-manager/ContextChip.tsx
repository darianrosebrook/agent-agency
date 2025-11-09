'use client';

import { X, Upload, Link as LinkIcon, Wrench } from 'lucide-react';
import type { ContextChip as ContextChipType } from './types';

interface ContextChipProps {
  chip: ContextChipType;
  onRemove: () => void;
}

function getChipIcon(type: string) {
  switch (type) {
    case 'file':
      return <Upload className="w-3 h-3" />;
    case 'reference':
      return <LinkIcon className="w-3 h-3" />;
    case 'tool':
      return <Wrench className="w-3 h-3" />;
    default:
      return null;
  }
}

export function ContextChip({ chip, onRemove }: ContextChipProps) {
  return (
    <div className="inline-flex items-center gap-2 px-3 py-1.5 bg-blue-500/10 text-blue-400 rounded-full text-sm group border border-blue-500/20">
      {getChipIcon(chip.type)}
      <span>{chip.label}</span>
      <button
        onClick={onRemove}
        className="opacity-0 group-hover:opacity-100 transition-opacity hover:text-blue-300"
      >
        <X className="w-3 h-3" />
      </button>
    </div>
  );
}

