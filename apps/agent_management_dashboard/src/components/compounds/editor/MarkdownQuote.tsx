"use client";

import type { ReactNode } from "react";
import { cn } from "../../primitives/utils";
import styles from "./MarkdownQuote.module.scss";

interface MarkdownQuoteProps {
  children: ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export function MarkdownQuote({
  children,
  className = "",
  style,
}: MarkdownQuoteProps) {
  return (
    <div
      className={cn(styles.editorQuote, className)}
      style={style}
      data-name="Quote"
    >
      <div aria-hidden="true" className={styles.editorQuoteBorder} />
      <p className={styles.editorQuoteText}>{children}</p>
    </div>
  );
}

