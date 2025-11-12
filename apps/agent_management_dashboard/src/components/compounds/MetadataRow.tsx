"use client";

import type { ReactNode } from "react";
import { cn } from "../primitives/utils";
import styles from "./MetadataRow.module.scss";

interface MetadataRowProps {
  label: string;
  children: ReactNode;
  className?: string;
}

export function MetadataRow({
  label,
  children,
  className = "",
}: MetadataRowProps) {
  return (
    <div className={cn(styles.metadataRow, className)}>
      <div className={styles.metadataLabel}>{label}</div>
      <div className={styles.metadataValue}>{children}</div>
    </div>
  );
}










