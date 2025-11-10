import styles from "./KanbanText.module.scss";

interface KanbanTextProps {
  children: React.ReactNode;
  size?: "14" | "16";
  color?: string;
  weight?: "normal" | "bold";
  width?: number | string;
  className?: string;
}

export function KanbanText({
  children,
  size = "14",
  color = "#cacaca",
  weight = "normal",
  width,
  className,
}: KanbanTextProps) {
  const sizeClass = size === "14" ? styles.text14 : styles.text16;
  const weightClass = weight === "normal" ? styles.fontNormal : styles.fontBold;
  const colorStyle = color !== "#cacaca" ? { color } : undefined;
  const widthStyle = width ? { width: typeof width === "number" ? `${width}px` : width } : undefined;

  return (
    <div className={`${styles.text} ${className || ""}`} style={widthStyle}>
      <div className={styles.textInner}>
        <p
          className={`${styles.textContent} ${sizeClass} ${weightClass} ${styles.leading20} ${styles.left0} ${styles.notItalic} ${styles.trackingNeg1504}`}
          style={{ ...colorStyle, top: "0.36px" }}
        >
          {children}
        </p>
      </div>
    </div>
  );
}

