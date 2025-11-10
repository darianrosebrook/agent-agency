"use client";

import { KanbanColumn } from "./KanbanColumn";
import type { KanbanBoardProps } from "./types";
import styles from "./KanbanBoard.module.scss";

// Design token values (matching _kanban.scss)
const COLUMN_WIDTH = 639.41;
const COLUMN_GAP = 15.994;

export function KanbanBoard({
  columns,
  onAddTask,
  className,
}: KanbanBoardProps) {
  return (
    <div className={`${styles.board} ${className || ""}`} data-name="KanbanBoard">
      <div className={styles.content}>
        <div className={styles.columns}>
          {columns.map((column, index) => {
            const left = index * (COLUMN_WIDTH + COLUMN_GAP);
            return (
              <KanbanColumn
                key={column.status}
                {...column}
                onAddTask={() => onAddTask?.(column.status)}
                left={left}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
}
