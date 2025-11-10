"use client";

import type { ReactNode } from "react";
import { cn } from "../../primitives/utils";
import styles from "./MarkdownInline.module.scss";

interface MarkdownInlineProps {
  children: ReactNode;
  className?: string;
}

export function MarkdownBold({ children, className = "" }: MarkdownInlineProps) {
  return <span className={cn(styles.textBold, className)}>{children}</span>;
}

export function MarkdownItalic({ children, className = "" }: MarkdownInlineProps) {
  return <span className={cn(styles.textItalic, className)}>{children}</span>;
}

export function MarkdownLink({
  children,
  href,
  className = "",
}: MarkdownInlineProps & { href?: string }) {
  return (
    <span className={cn(styles.textLink, className)}>
      {href ? <a href={href}>{children}</a> : children}
    </span>
  );
}

export function MarkdownInlineCode({ children, className = "" }: MarkdownInlineProps) {
  return <span className={cn(styles.editorInlineCode, className)}>{children}</span>;
}

