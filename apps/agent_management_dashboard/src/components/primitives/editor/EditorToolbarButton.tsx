"use client";

import type { ReactNode } from "react";
import { cn } from "../utils";
import styles from "./EditorToolbarButton.module.scss";

interface EditorToolbarButtonProps {
  icon: ReactNode;
  onClick?: () => void;
  active?: boolean;
  className?: string;
  style?: React.CSSProperties;
  "data-name"?: string;
}

export function EditorToolbarButton({
  icon,
  onClick,
  active = false,
  className = "",
  style,
  "data-name": dataName = "Button",
}: EditorToolbarButtonProps) {
  return (
    <div
      className={cn(
        styles.toolbarButton,
        active && styles.toolbarButtonActive,
        className
      )}
      onClick={onClick}
      style={style}
      data-name={dataName}
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={
        onClick
          ? (e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onClick();
              }
            }
          : undefined
      }
    >
      {icon}
    </div>
  );
}

