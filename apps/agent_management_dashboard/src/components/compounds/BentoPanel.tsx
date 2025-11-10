import { ReactNode } from "react";
import { cn } from "../ui/utils";
import styles from "./BentoPanel.module.scss";

interface BentoPanelProps {
  children?: ReactNode;
  className?: string;
}

export function BentoPanel({
  children,
  className = "",
}: BentoPanelProps) {
  return (
    <div className={cn(styles.bentoPanel, className)}>
      {children}
    </div>
  );
}