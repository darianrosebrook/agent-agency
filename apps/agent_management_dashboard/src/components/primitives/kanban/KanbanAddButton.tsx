import styles from "./KanbanAddButton.module.scss";

interface KanbanAddButtonProps {
  status?: "backlog" | "todo" | "in-progress" | "done";
  icon: React.ReactNode;
  onClick?: () => void;
  className?: string;
}

export function KanbanAddButton({
  status,
  icon,
  onClick,
  className,
}: KanbanAddButtonProps) {
  return (
    <button
      className={`${styles.addButton} ${className || ""}`}
      data-name="Button"
      data-add-task="true"
      data-status={status}
      onClick={onClick}
    >
      <div
        aria-hidden="true"
        className={styles.borderOverlay}
      />
      <div className={styles.buttonContent}>
        {icon}
      </div>
    </button>
  );
}

