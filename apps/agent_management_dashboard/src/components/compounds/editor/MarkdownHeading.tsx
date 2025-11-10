"use client";

import type { ReactNode } from "react";
import { cn } from "../../primitives/utils";
import styles from "./MarkdownHeading.module.scss";

interface MarkdownHeadingProps {
  level?: 1 | 2 | 3 | 4 | 5 | 6;
  children: ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export function MarkdownHeading({
  level = 1,
  children,
  className = "",
  style,
}: MarkdownHeadingProps) {
  const HeadingTag = `h${level}` as keyof JSX.IntrinsicElements;
  const headingClass = level === 1 ? styles.editorHeading : styles.editorHeading2;
  const textClass = level === 1 ? styles.editorHeadingText : styles.editorHeading2Text;

  return (
    <div className={cn(headingClass, className)} style={style} data-name={`Heading ${level}`}>
      <HeadingTag className={textClass}>{children}</HeadingTag>
    </div>
  );
}

