import { KanbanColumnHeader } from "./KanbanColumnHeader";
import { KanbanCard } from "./KanbanCard";
import styles from "./KanbanColumn.module.scss";

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

interface KanbanColumnProps {
  status: "backlog" | "todo" | "in-progress" | "done";
  title: string;
  cardCount: number;
  cards: KanbanCardData[];
  onAddTask?: () => void;
  left?: number;
  className?: string;
}

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
        status={status}
        onAddTask={onAddTask}
      />
      <div className={styles.cardsContainer}>
        {cards.map((card, index) => (
          <KanbanCard
            key={index}
            title={card.title}
            description={card.description}
            statusTags={card.statusTags}
            metadata={card.metadata}
            priority={card.priority}
            height={card.height}
          />
        ))}
      </div>
    </div>
  );
}

