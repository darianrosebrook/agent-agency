"use client";

import type { ReactNode } from "react";
import { cn } from "../../primitives/utils";
import styles from "./MarkdownListItem.module.scss";

interface MarkdownListItemProps {
  children: ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export function MarkdownListItem({
  children,
  className = "",
  style,
}: MarkdownListItemProps) {
  return (
    <div
      className={cn(styles.editorListItem, className)}
      style={style}
      data-name="List Item"
    >
      <p className={styles.editorListItemText}>{children}</p>
    </div>
  );
}

