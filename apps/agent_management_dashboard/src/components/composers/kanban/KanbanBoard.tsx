"use client";

import { KanbanColumn } from "./KanbanColumn";
import styles from "./KanbanBoard.module.scss";

interface KanbanCardData {
  title: string;
  description?: string;
  statusTags?: Array<{
    label: string;
    icon?: React.ReactNode;
    bgColor?: string;
    textColor?: string;
  }>;
  metadata?: Array<{
    icon: React.ReactNode | { path: string | string[]; size?: number };
    text: string;
  }>;
  priority?: "low" | "medium" | "high";
  height?: number;
}

interface KanbanColumnConfig {
  status: "backlog" | "todo" | "in-progress" | "done";
  title: string;
  cardCount: number;
  cards: KanbanCardData[];
  onAddTask?: () => void;
}

interface KanbanBoardProps {
  columns: KanbanColumnConfig[];
  onAddTask?: (status: "backlog" | "todo" | "in-progress" | "done") => void;
  className?: string;
}

export function KanbanBoard({
  columns,
  onAddTask,
  className,
}: KanbanBoardProps) {
  const columnWidth = 639.41;
  const columnGap = 15.994;

  return (
    <div
      className={`${styles.board} ${className || ""}`}
      data-name="KanbanBoard"
    >
      <div className={styles.boardContent}>
        <div className={styles.columnsContainer}>
          {columns.map((column, index) => {
            const left = index * (columnWidth + columnGap);
            return (
              <KanbanColumn
                key={column.status}
                status={column.status}
                title={column.title}
                cardCount={column.cardCount}
                cards={column.cards}
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

