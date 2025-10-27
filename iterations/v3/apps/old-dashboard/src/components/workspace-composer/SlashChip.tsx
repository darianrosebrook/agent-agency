import React from "react";

type Props = {
  command: string;
  onRemove?: () => void;
};

export function SlashChip({ command, onRemove }: Props) {
  return (
    <span
      data-chip
      data-command={command}
      className="inline-flex items-center gap-1.5 px-2.5 py-0.5 mx-0.5 bg-workspace-accent/10
                 border border-workspace-accent/20 rounded-md text-sm align-middle"
      contentEditable={false}
    >
      <span className="font-medium text-workspace-accent">{command}</span>
      {onRemove && (
        <button
          type="button"
          aria-label={`Remove ${command}`}
          className="ml-1 rounded p-0.5 hover:bg-workspace-accent/20"
          onClick={(e) => {
            e.preventDefault();
            onRemove();
          }}
        >
          <svg className="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      )}
    </span>
  );
}
