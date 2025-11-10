import styles from "./KanbanIcon.module.scss";

interface KanbanIconProps {
  iconPath: string | string[];
  size?: number | { width: number; height: number };
  color?: string;
  strokeWidth?: number;
  viewBox?: string;
  className?: string;
}

export function KanbanIcon({
  iconPath,
  size = 15.994,
  color = "#888888",
  strokeWidth = 1.33286,
  viewBox = "0 0 16 16",
  className,
}: KanbanIconProps) {
  const sizeStyle =
    typeof size === "number"
      ? { width: `${size}px`, height: `${size}px` }
      : { width: `${size.width}px`, height: `${size.height}px` };

  const paths = Array.isArray(iconPath) ? iconPath : [iconPath];

  return (
    <div
      className={`${styles.icon} ${className || ""}`}
      style={sizeStyle}
      data-name="Icon"
    >
      <svg
        className={styles.svg}
        fill="none"
        preserveAspectRatio="none"
        viewBox={viewBox}
      >
        <g id="Icon">
          {paths.map((path, index) => (
            <path
              key={index}
              d={path}
              id={`Vector${index > 0 ? `_${index + 1}` : ""}`}
              stroke={`var(--stroke-0, ${color})`}
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={strokeWidth}
            />
          ))}
        </g>
      </svg>
    </div>
  );
}

