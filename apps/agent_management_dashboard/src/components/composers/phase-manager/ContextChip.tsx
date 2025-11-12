'use client';

import { X, Upload, Link as LinkIcon, Wrench } from 'lucide-react';
import type { ContextChip as ContextChipType } from './types';
import styles from './ContextChip.module.scss';

interface ContextChipProps {
  chip: ContextChipType;
  onRemove: () => void;
}

function getChipIcon(type: string) {
  switch (type) {
    case 'file':
      return <Upload className={styles.contextChipIcon} />;
    case 'reference':
      return <LinkIcon className={styles.contextChipIcon} />;
    case 'tool':
      return <Wrench className={styles.contextChipIcon} />;
    default:
      return null;
  }
}

export function ContextChip({ chip, onRemove }: ContextChipProps) {
  return (
    <div className={styles.contextChip}>
      {getChipIcon(chip.type)}
      <span>{chip.label}</span>
      <button
        onClick={onRemove}
        className={styles.contextChipRemove}
      >
        <X className={styles.contextChipRemoveIcon} />
      </button>
    </div>
  );
}














