"use client";

import { KanbanStatusTag } from "../../compounds/kanban/KanbanStatusTag";
import { KanbanCardMeta } from "../../compounds/kanban/KanbanCardMeta";
import type { KanbanCardProps } from "./types";
import styles from "./KanbanCard.module.scss";

export function KanbanCard({
  title,
  description,
  statusTags = [],
  metadata = [],
  height,
  className,
}: KanbanCardProps) {
  return (
    <div
      className={`${styles.card} ${className || ""}`}
      style={height ? { height: `${height}px` } : undefined}
      data-name="KanbanCard"
    >
      <div aria-hidden="true" className={styles.border} />
      
      {statusTags.length > 0 && (
        <div className={styles.statusTags}>
          {statusTags.map((tag, index) => (
            <KanbanStatusTag
              key={index}
              label={tag.label}
              icon={tag.icon}
              bgColor={tag.bgColor}
              textColor={tag.textColor}
            />
          ))}
        </div>
      )}
      
      <h3 className={styles.title}>{title}</h3>
      
      {description && (
        <p className={styles.description}>{description}</p>
      )}
      
      {metadata.length > 0 && (
        <div className={styles.metadata}>
          {metadata.map((meta, index) => (
            <KanbanCardMeta key={index} icon={meta.icon} text={meta.text} />
          ))}
        </div>
      )}
    </div>
  );
}
