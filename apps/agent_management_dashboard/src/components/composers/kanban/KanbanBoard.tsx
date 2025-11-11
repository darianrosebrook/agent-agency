"use client";

/**
 * Kanban Board Component with Drag and Drop
 *
 * Provides a draggable Kanban board where tasks can be moved between columns.
 *
 * @author @darianrosebrook
 */

import {
  DndContext,
  DragOverlay,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragStartEvent,
  DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { useState } from "react";
import { KanbanColumn } from "./KanbanColumn";
import type { KanbanBoardProps, KanbanStatus } from "./types";
import styles from "./KanbanBoard.module.scss";

// Design token values (matching _kanban.scss)
const COLUMN_WIDTH = 639.41;
const COLUMN_GAP = 15.994;

export function KanbanBoard({
  columns,
  onAddTask,
  onTaskMove,
  onTaskEdit,
  onTaskDelete,
  onTaskViewComments,
  className,
}: KanbanBoardProps) {
  const [activeId, setActiveId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8, // Require 8px of movement before drag starts
      },
    }),
    useSensor(KeyboardSensor)
  );

  const handleDragStart = (event: DragStartEvent) => {
    setActiveId(event.active.id as string);
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    setActiveId(null);

    if (!over || !onTaskMove) {
      return;
    }

    const taskId = active.id as string;
    const overId = over.id as string;

    // Check if dropped on a column (status) or another card
    let newStatus: KanbanStatus | null = null;

    // If dropped on a column status (droppable)
    if (columns.some((col) => col.status === overId)) {
      newStatus = overId as KanbanStatus;
    } else {
      // If dropped on another card, find which column it's in
      for (const column of columns) {
        if (column.cards.some((card) => card.id === overId)) {
          newStatus = column.status;
          break;
        }
      }
    }

    if (!newStatus) {
      return;
    }

    // Find the current status of the task
    let currentStatus: KanbanStatus | null = null;
    for (const column of columns) {
      if (column.cards.some((card) => card.id === taskId)) {
        currentStatus = column.status;
        break;
      }
    }

    // Only move if status actually changed
    if (currentStatus && currentStatus !== newStatus) {
      onTaskMove(taskId, newStatus);
    }
  };

  // Find the active card for drag overlay
  const activeCard = activeId
    ? columns.flatMap((col) => col.cards).find((card) => card.id === activeId)
    : null;

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      <div
        className={`${styles.board} ${className || ""}`}
        data-name="KanbanBoard"
      >
        <div className={styles.content}>
          <div className={styles.columns}>
            {columns.map((column, index) => {
              const left = index * (COLUMN_WIDTH + COLUMN_GAP);
              return (
                <KanbanColumn
                  key={column.status}
                  {...column}
                  onAddTask={() => onAddTask?.(column.status)}
                  onTaskMove={onTaskMove}
                  onTaskEdit={onTaskEdit}
                  onTaskDelete={onTaskDelete}
                  onTaskViewComments={onTaskViewComments}
                  left={left}
                />
              );
            })}
          </div>
        </div>
      </div>

      <DragOverlay>
        {activeCard ? (
          <div className={styles.dragOverlay}>
            <div className={styles.dragCard}>
              <h3 className={styles.dragCardTitle}>{activeCard.title}</h3>
              {activeCard.description && (
                <p className={styles.dragCardDescription}>
                  {activeCard.description}
                </p>
              )}
            </div>
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
