"use client";

import type { ReactNode } from "react";
import { cn } from "../../primitives/utils";
import styles from "./MarkdownCodeBlock.module.scss";

interface MarkdownCodeBlockProps {
  children: ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export function MarkdownCodeBlock({
  children,
  className = "",
  style,
}: MarkdownCodeBlockProps) {
  return (
    <div
      className={cn(styles.editorCodeBlock, className)}
      style={style}
      data-name="Container"
    >
      <div aria-hidden="true" className={styles.editorCodeBlockBorder} />
      <div className={styles.editorCodeTextContent}>{children}</div>
    </div>
  );
}

