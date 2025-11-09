"use client";

export interface PriorityConfig {
  label: string;
  color: string;
  icon: string;
}

export interface PriorityIndicatorProps {
  priority: string;
  config: PriorityConfig;
  onClick?: () => void;
  className?: string;
}

export function PriorityIndicator({
  priority: _priority, // eslint-disable-line @typescript-eslint/no-unused-vars, no-unused-vars
  config,
  onClick,
  className = "",
}: PriorityIndicatorProps) {
  const content = (
    <>
      <span className={config.color}>{config.icon}</span>
      <span>{config.label}</span>
    </>
  );

  if (onClick) {
    return (
      <button
        onClick={onClick}
        className={`flex items-center gap-2 hover:opacity-80 transition-opacity ${className}`}
        type="button"
      >
        {content}
      </button>
    );
  }

  return (
    <div className={`flex items-center gap-2 ${className}`}>{content}</div>
  );
}
