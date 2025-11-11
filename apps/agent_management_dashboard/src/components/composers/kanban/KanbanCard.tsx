"use client";

/**
 * Kanban Card Component
 *
 * Displays a task card with edit, delete, and comments actions.
 *
 * @author @darianrosebrook
 */

import { useState, useEffect, useRef } from "react";
import { MoreVertical, MessageSquare, Edit2, Trash2 } from "lucide-react";
import { KanbanStatusTag } from "../../compounds/kanban/KanbanStatusTag";
import { KanbanCardMeta } from "../../compounds/kanban/KanbanCardMeta";
import type { KanbanCardProps } from "./types";
import styles from "./KanbanCard.module.scss";

export function KanbanCard({
  id,
  title,
  description,
  statusTags = [],
  metadata = [],
  commentCount = 0,
  height,
  className,
  onEdit,
  onDelete,
  onViewComments,
}: KanbanCardProps) {
  const [showMenu, setShowMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        showMenu &&
        menuRef.current &&
        buttonRef.current &&
        !menuRef.current.contains(event.target as Node) &&
        !buttonRef.current.contains(event.target as Node)
      ) {
        setShowMenu(false);
      }
    };

    if (showMenu) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => {
        document.removeEventListener("mousedown", handleClickOutside);
      };
    }
  }, [showMenu]);

  return (
    <div
      className={`${styles.card} ${className || ""}`}
      style={height ? { height: `${height}px` } : undefined}
      data-name="KanbanCard"
      data-task-id={id}
    >
      <div aria-hidden="true" className={styles.border} />

      {/* Card Header with Actions */}
      <div className={styles.cardHeader}>
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

        <div className={styles.cardActions}>
          <button
            ref={buttonRef}
            className={styles.cardActionButton}
            onClick={(e) => {
              e.stopPropagation();
              setShowMenu(!showMenu);
            }}
            title="Task actions"
          >
            <MoreVertical className={styles.cardActionIcon} />
          </button>

          {showMenu && (
            <div ref={menuRef} className={styles.cardMenu}>
              {onEdit && (
                <button
                  className={styles.cardMenuItem}
                  onClick={(e) => {
                    e.stopPropagation();
                    onEdit(id);
                    setShowMenu(false);
                  }}
                >
                  <Edit2 className={styles.cardMenuIcon} />
                  Edit Task
                </button>
              )}
              {onViewComments && (
                <button
                  className={styles.cardMenuItem}
                  onClick={(e) => {
                    e.stopPropagation();
                    onViewComments(id);
                    setShowMenu(false);
                  }}
                >
                  <MessageSquare className={styles.cardMenuIcon} />
                  Comments {commentCount > 0 && `(${commentCount})`}
                </button>
              )}
              {onDelete && (
                <button
                  className={styles.cardMenuItem}
                  onClick={(e) => {
                    e.stopPropagation();
                    onDelete(id);
                    setShowMenu(false);
                  }}
                >
                  <Trash2 className={styles.cardMenuIcon} />
                  Delete Task
                </button>
              )}
            </div>
          )}
        </div>
      </div>

      <h3 className={styles.title}>{title}</h3>

      {description && <p className={styles.description}>{description}</p>}

      {/* Footer with Metadata and Comment Count */}
      <div className={styles.cardFooter}>
        {metadata.length > 0 && (
          <div className={styles.metadata}>
            {metadata.map((meta, index) => (
              <KanbanCardMeta key={index} icon={meta.icon} text={meta.text} />
            ))}
          </div>
        )}

        {commentCount > 0 && (
          <button
            className={styles.commentBadge}
            onClick={(e) => {
              e.stopPropagation();
              onViewComments?.(id);
            }}
            title={`${commentCount} comment${commentCount !== 1 ? "s" : ""}`}
          >
            <MessageSquare className={styles.commentIcon} />
            <span className={styles.commentCount}>{commentCount}</span>
          </button>
        )}
      </div>
    </div>
  );
}
