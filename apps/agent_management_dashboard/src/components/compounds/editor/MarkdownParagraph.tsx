"use client";

import type { ReactNode } from "react";
import { cn } from "../../primitives/utils";
import styles from "./MarkdownParagraph.module.scss";

interface MarkdownParagraphProps {
  children: ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export function MarkdownParagraph({
  children,
  className = "",
  style,
}: MarkdownParagraphProps) {
  return (
    <div
      className={cn(styles.editorParagraph, className)}
      style={style}
      data-name="Paragraph"
    >
      <p className={styles.editorParagraphText}>{children}</p>
    </div>
  );
}

