import { KanbanIcon } from "../../primitives/kanban/KanbanIcon";
import { KanbanText } from "../../primitives/kanban/KanbanText";
import styles from "./KanbanCardMeta.module.scss";

interface KanbanCardMetaProps {
  icon: React.ReactNode | { path: string | string[]; size?: number };
  text: string;
  className?: string;
}

export function KanbanCardMeta({ icon, text, className }: KanbanCardMetaProps) {
  const iconElement =
    icon && typeof icon === "object" && "path" in icon ? (
      <KanbanIcon
        iconPath={icon.path}
        size={icon.size || 13.999}
        viewBox="0 0 14 14"
      />
    ) : (
      icon
    );

  return (
    <div className={`${styles.meta} ${className || ""}`}>
      {iconElement}
      <KanbanText>{text}</KanbanText>
    </div>
  );
}

