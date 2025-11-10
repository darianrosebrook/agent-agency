"use client";

import { KanbanCardHeader } from "../../compounds/kanban/KanbanCardHeader";
import type { KanbanColumnHeaderProps } from "./types";
import styles from "./KanbanColumnHeader.module.scss";

export function KanbanColumnHeader({
  title,
  cardCount,
  onAddTask,
  className,
}: KanbanColumnHeaderProps) {
  return (
    <div className={`${styles.header} ${className ?? ""}`}>
      <KanbanCardHeader title={title} cardCount={cardCount} />
      <button className={styles.menuButton} type="button" onClick={onAddTask}>
        <svg className={styles.icon} fill="none" viewBox="0 0 16 16">
          <g>
            <path
              d="M3.33215 7.99716H12.6622"
              stroke="#888888"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="1.33286"
            />
            <path
              d="M3.33215 3.99858H12.6622"
              stroke="#888888"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="1.33286"
            />
            <path
              d="M3.33215 11.9957H12.6622"
              stroke="#888888"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="1.33286"
            />
          </g>
        </svg>
      </button>
    </div>
  );
}
