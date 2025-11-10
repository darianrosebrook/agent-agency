import { KanbanCardHeader } from "../../compounds/kanban/KanbanCardHeader";
import { KanbanAddButton } from "../../primitives/kanban/KanbanAddButton";
import styles from "./KanbanColumnHeader.module.scss";

interface KanbanColumnHeaderProps {
  title: string;
  cardCount: number;
  status: "backlog" | "todo" | "in-progress" | "done";
  onAddTask?: () => void;
  className?: string;
}

export function KanbanColumnHeader({
  title,
  cardCount,
  status,
  onAddTask,
  className,
}: KanbanColumnHeaderProps) {
  return (
    <div className={`${styles.header} ${className || ""}`}>
      <KanbanCardHeader title={title} cardCount={cardCount} />
      <KanbanAddButton status={status} icon={<span>+</span>} onClick={onAddTask} />
    </div>
  );
}

