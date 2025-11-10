"use client";

import type { ReactNode } from "react";
import { cn } from "../utils";
import styles from "./EditorIcon.module.scss";

interface EditorIconProps {
  children: ReactNode;
  className?: string;
  opacity?: number;
  style?: React.CSSProperties;
}

export function EditorIcon({
  children,
  className = "",
  opacity,
  style,
}: EditorIconProps) {
  return (
    <div
      className={cn(styles.editorIcon, className)}
      data-name="Icon"
      style={style}
    >
      <svg
        className={styles.svgIcon}
        fill="none"
        preserveAspectRatio="none"
        viewBox="0 0 16 16"
        style={opacity !== undefined ? { opacity } : undefined}
      >
        <g id="Icon">{children}</g>
      </svg>
    </div>
  );
}

