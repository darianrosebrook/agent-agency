import React from "react";
import type { ContextItem } from "./types";

type Props = {
  items: ContextItem[];
  onRemove: (id: string) => void;
  onToggleExpand?: (id: string) => void;
  expandedId?: string | null;
};

export function ContextTray({ items, onRemove, onToggleExpand, expandedId }: Props) {
  if (items.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-2 pb-2">
      {items.map((item, i) => (
        <div
          key={item.id}
          className={`group relative bg-muted/50 border border-border rounded-lg transition-all duration-200 ${
            expandedId === item.id ? "w-full p-3" : "p-2 pr-8 hover:bg-muted cursor-pointer"
          }`}
          style={{ animationDelay: `${i * 50}ms` }}
          onClick={() => onToggleExpand?.(item.id)}
        >
          <div className="text-sm font-medium truncate">{item.title}</div>
          {expandedId === item.id && item.preview && (
            <div className="text-xs text-muted-foreground mt-1">{item.preview}</div>
          )}
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onRemove(item.id);
            }}
            className="absolute top-2 right-2 p-1 rounded hover:bg-background"
            aria-label={`Remove ${item.title}`}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
