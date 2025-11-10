import styles from "./KanbanHeading.module.scss";

interface KanbanHeadingProps {
  children: React.ReactNode;
  level?: "h3" | "h4";
  color?: string;
  width?: number | string;
  className?: string;
}

export function KanbanHeading({
  children,
  level = "h3",
  color = "#ffffff",
  width,
  className,
}: KanbanHeadingProps) {
  const widthStyle = width ? { width: typeof width === "number" ? `${width}px` : width } : undefined;
  const colorStyle = color !== "#ffffff" ? { color } : undefined;

  return (
    <div
      className={`${styles.heading} ${className || ""}`}
      style={{ height: "23.999px", ...widthStyle }}
      data-name={`Heading ${level === "h3" ? "3" : "4"}`}
    >
      <div className={styles.headingInner} style={{ height: "23.999px", ...widthStyle }}>
        <p
          className={`${styles.headingContent} ${styles.fontNormal} ${styles.leading24} ${styles.left0} ${styles.notItalic} ${styles.text16} ${styles.textNowrap} ${styles.textWhite} ${styles.trackingNeg3125} ${styles.whitespacePre}`}
          style={{ ...colorStyle, top: "-0.73px" }}
        >
          {children}
        </p>
      </div>
    </div>
  );
}

