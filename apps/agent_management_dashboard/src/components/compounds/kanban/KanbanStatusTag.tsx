import styles from "./KanbanStatusTag.module.scss";

interface KanbanStatusTagProps {
  label: string;
  icon?: React.ReactNode;
  bgColor?: string;
  textColor?: string;
  left?: number;
  width?: number;
  className?: string;
}

export function KanbanStatusTag({
  label,
  icon,
  bgColor = "#262626",
  textColor = "#cacaca",
  left = 0,
  width,
  className,
}: KanbanStatusTagProps) {
  const bgStyle = bgColor !== "#262626" ? { backgroundColor: bgColor } : undefined;
  const textStyle = textColor !== "#cacaca" ? { color: textColor } : undefined;
  const widthStyle = width ? { width: `${width}px` } : undefined;
  const leftStyle = left !== 0 ? { left: `${left}px` } : undefined;

  return (
    <div
      className={`${styles.statusTag} ${className || ""}`}
      style={{
        ...bgStyle,
        ...widthStyle,
        ...leftStyle,
      }}
      data-name="StatusTag"
    >
      {icon && <div className={styles.iconWrapper}>{icon}</div>}
      <span className={styles.label} style={textStyle}>
        {label}
      </span>
    </div>
  );
}

