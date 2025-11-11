"use client";

import { X } from "lucide-react";
import styles from "./TagChip.module.scss";
import { cn } from "../primitives/utils";

interface TagChipProps {
  tag: string;
  onRemove?: (tag: string) => void;
  className?: string;
}

export function TagChip({ tag, onRemove, className = "" }: TagChipProps) {
  return (
    <span
      className={cn(styles.tagChip, onRemove && styles.removable, className)}
      onClick={() => onRemove?.(tag)}
    >
      {tag}
      {onRemove && <X className={styles.tagChipIcon} />}
    </span>
  );
}




