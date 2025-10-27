import React, { useEffect, useRef } from "react";
import type { CommandDef } from "./types";

type Props = {
  open: boolean;
  anchor: { top: number; left: number } | null;
  commands: CommandDef[];
  onSelect: (cmd: CommandDef) => void;
  onClose: () => void;
};

export function CommandPalette({ open, anchor, commands, onSelect, onClose }: Props) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (ref.current && !ref.current.contains(event.target as Node)) {
        onClose();
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    if (open) {
      document.addEventListener("mousedown", handleClickOutside);
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [open, onClose]);

  if (!open || !anchor) return null;

  return (
    <div
      ref={ref}
      className="fixed z-50 animate-in fade-in slide-in-from-bottom-2"
      style={{ top: anchor.top, left: anchor.left }}
      role="listbox"
      aria-label="Slash commands"
    >
      <div className="w-[350px] border border-border shadow-lg rounded-lg bg-background">
        <div className="p-2 border-b border-border">
          <div className="text-xs text-muted-foreground">Commands</div>
        </div>
        <div className="max-h-64 overflow-y-auto">
          {commands.length === 0 ? (
            <div className="p-4 text-center text-muted-foreground">No commands found.</div>
          ) : (
            commands.map((command) => {
              const Icon = command.icon;
              return (
                <button
                  key={command.value}
                  onClick={() => onSelect(command)}
                  className="w-full p-3 text-left hover:bg-muted border-b border-border/50 last:border-b-0 flex items-start gap-3"
                  role="option"
                  aria-selected="false"
                >
                  {Icon && <Icon className="h-4 w-4 text-muted-foreground mt-0.5 flex-shrink-0" />}
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-sm">{command.label}</div>
                    {command.description && (
                      <div className="text-xs text-muted-foreground mt-1">{command.description}</div>
                    )}
                  </div>
                  <div className="text-xs text-muted-foreground bg-muted px-2 py-1 rounded flex-shrink-0">
                    {command.value}
                  </div>
                </button>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
