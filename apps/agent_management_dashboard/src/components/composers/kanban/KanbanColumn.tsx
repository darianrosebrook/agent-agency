"use client";

/**
 * Kanban Column Component with Sortable Cards
 *
 * @author @darianrosebrook
 */

import { useDroppable } from "@dnd-kit/core";
import { SortableContext, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { KanbanColumnHeader } from "./KanbanColumnHeader";
import { KanbanCard } from "./KanbanCard";
import { KanbanAddButton } from "../../primitives/kanban/KanbanAddButton";
import type { KanbanColumnProps } from "./types";
import styles from "./KanbanColumn.module.scss";

function SortableCard({
  card,
  onEdit,
  onDelete,
  onViewComments,
}: {
  card: KanbanColumnProps["cards"][0];
  onEdit?: (taskId: string) => void;
  onDelete?: (taskId: string) => void;
  onViewComments?: (taskId: string) => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: card.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div ref={setNodeRef} style={style} {...attributes} {...listeners}>
      <KanbanCard
        {...card}
        onEdit={onEdit}
        onDelete={onDelete}
        onViewComments={onViewComments}
      />
    </div>
  );
}

export function KanbanColumn({
  status,
  title,
  cardCount,
  cards,
  onAddTask,
  onTaskMove,
  onTaskEdit,
  onTaskDelete,
  onTaskViewComments,
  left = 0,
  className,
}: KanbanColumnProps) {
  const { setNodeRef } = useDroppable({
    id: status,
  });

  return (
    <div
      ref={setNodeRef}
      className={`${styles.column} ${className || ""}`}
      style={{ left: `${left}px` }}
      data-name="KanbanColumn"
      data-status={status}
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
        <SortableContext
          items={cards.map((card) => card.id)}
          strategy={verticalListSortingStrategy}
        >
          {cards.map((card) => (
            <SortableCard
              key={card.id}
              card={card}
              onEdit={onTaskEdit}
              onDelete={onTaskDelete}
              onViewComments={onTaskViewComments}
            />
          ))}
        </SortableContext>
      </div>
    </div>
  );
}
