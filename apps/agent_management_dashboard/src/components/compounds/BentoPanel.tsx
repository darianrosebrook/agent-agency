import { ReactNode } from "react";

interface BentoPanelProps {
  children?: ReactNode;
  className?: string;
}

export function BentoPanel({
  children,
  className = "",
}: BentoPanelProps) {
  return (
    <div
      className={`bg-[#111111] relative rounded-[12px] size-full border border-[#cacaca] ${className}`}
    >
      {children}
    </div>
  );
}