import { KanbanCardHeader } from "../../compounds/kanban/KanbanCardHeader";
import { KanbanCardMeta } from "../../compounds/kanban/KanbanCardMeta";
import { KanbanStatusTag } from "../../compounds/kanban/KanbanStatusTag";
import styles from "./KanbanCard.module.scss";

interface StatusTag {
  label: string;
  icon?: React.ReactNode;
  bgColor?: string;
  textColor?: string;
}

interface Metadata {
  icon: React.ReactNode | { path: string | string[]; size?: number };
  text: string;
}

interface KanbanCardProps {
  title: string;
  description?: string;
  statusTags?: StatusTag[];
  metadata?: Metadata[];
  priority?: "low" | "medium" | "high";
  height?: number;
  className?: string;
}

export function KanbanCard({
  title,
  description,
  statusTags = [],
  metadata = [],
  priority,
  height = 205.753,
  className,
}: KanbanCardProps) {
  return (
    <div
      className={`${styles.card} ${className || ""}`}
      style={{ height: `${height}px` }}
      data-name="KanbanCard"
    >
      <div aria-hidden="true" className={styles.borderOverlay} />
      <div className={styles.cardContent}>
        {statusTags.length > 0 && (
          <div className={styles.statusTagsContainer}>
            {statusTags.map((tag, index) => (
              <KanbanStatusTag
                key={index}
                label={tag.label}
                icon={tag.icon}
                bgColor={tag.bgColor}
                textColor={tag.textColor}
                left={index > 0 ? undefined : 0}
              />
            ))}
          </div>
        )}
        <h3 className={styles.title}>{title}</h3>
        {description && <p className={styles.description}>{description}</p>}
        {metadata.length > 0 && (
          <div className={styles.metadataContainer}>
            {metadata.map((meta, index) => (
              <KanbanCardMeta key={index} icon={meta.icon} text={meta.text} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

