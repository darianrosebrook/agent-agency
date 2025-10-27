import { Button } from "@/design-system/primitives";
import type { Mode } from "../types";
import { useComposer } from "../ComposerProvider";
import { useState } from "react";

const MODES: { value: Mode; label: string }[] = [
  { value: "chat", label: "Chat" },
  { value: "agent", label: "Agent" },
  { value: "planning", label: "Planning" },
];

export function ModeSelect() {
  const { meta, setMeta } = useComposer();
  const [isOpen, setIsOpen] = useState(false);

  const currentMode = MODES.find(m => m.value === meta.mode);

  return (
    <div className="relative">
      <Button
        size="sm"
        variant="ghost"
        onClick={() => setIsOpen(!isOpen)}
        className="h-8 w-[110px] text-xs justify-start"
      >
        {currentMode?.label || "Chat"}
      </Button>
      {isOpen && (
        <div className="absolute top-full left-0 z-10 mt-1 w-[110px] bg-background border border-border rounded-md shadow-lg">
          {MODES.map((mode) => (
            <button
              key={mode.value}
              onClick={() => {
                setMeta((m) => ({ ...m, mode: mode.value }));
                setIsOpen(false);
              }}
              className="w-full px-3 py-2 text-left text-xs hover:bg-muted first:rounded-t-md last:rounded-b-md"
            >
              {mode.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
