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
  bgColor = "#3a2f1f", // Match Figma default: bg-[#3a2f1f]
  textColor = "#cacaca",
  left = 0,
  width,
  className,
}: KanbanStatusTagProps) {
  const bgStyle = bgColor !== "#3a2f1f" ? { backgroundColor: bgColor } : undefined;
  const textStyle = textColor !== "#cacaca" ? { color: textColor } : undefined;
  const widthStyle = width ? { width: `${width}px` } : undefined;
  // Note: left positioning is now handled by the container, not individual tags

  return (
    <div
      className={`${styles.statusTag} ${className || ""}`}
      style={{
        ...bgStyle,
        ...widthStyle,
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

