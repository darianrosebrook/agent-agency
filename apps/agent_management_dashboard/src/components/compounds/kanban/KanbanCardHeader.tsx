import { KanbanHeading } from "../../primitives/kanban/KanbanHeading";
import { KanbanText } from "../../primitives/kanban/KanbanText";
import styles from "./KanbanCardHeader.module.scss";

interface KanbanCardHeaderProps {
  title: string;
  cardCount: number;
  className?: string;
}

export function KanbanCardHeader({
  title,
  cardCount,
  className,
}: KanbanCardHeaderProps) {
  return (
    <div className={`${styles.header} ${className ?? ""}`}>
      <KanbanHeading width="auto">{title}</KanbanHeading>
      <KanbanText>{cardCount} Cards</KanbanText>
    </div>
  );
}
