"use client";

import { KanbanColumnHeader } from "./KanbanColumnHeader";
import { KanbanCard } from "./KanbanCard";
import { KanbanAddButton } from "../../primitives/kanban/KanbanAddButton";
import type { KanbanColumnProps } from "./types";
import styles from "./KanbanColumn.module.scss";

export function KanbanColumn({
  status,
  title,
  cardCount,
  cards,
  onAddTask,
  left = 0,
  className,
}: KanbanColumnProps) {
  return (
    <div
      className={`${styles.column} ${className || ""}`}
      style={{ left: `${left}px` }}
      data-name="KanbanColumn"
    >
      <KanbanColumnHeader
        title={title}
        cardCount={cardCount}
        onAddTask={onAddTask}
      />
      
      <KanbanAddButton
        status={status}
        icon={<span>+</span>}
        onClick={onAddTask}
      />
      
      <div className={styles.cards}>
        {cards.map((card, index) => (
          <KanbanCard key={index} {...card} />
        ))}
      </div>
    </div>
  );
}
