"use client";

import { cn } from "../../primitives/utils";
import styles from "./MarkdownImagePlaceholder.module.scss";

interface MarkdownImagePlaceholderProps {
  src: string;
  alt?: string;
  className?: string;
  style?: React.CSSProperties;
}

export function MarkdownImagePlaceholder({
  src,
  alt = "",
  className = "",
  style,
}: MarkdownImagePlaceholderProps) {
  return (
    <div
      className={cn(styles.editorImage, className)}
      style={style}
      data-name="Image (Placeholder Image)"
    >
      <img
        alt={alt}
        className={styles.editorImageImg}
        src={src}
      />
    </div>
  );
}

