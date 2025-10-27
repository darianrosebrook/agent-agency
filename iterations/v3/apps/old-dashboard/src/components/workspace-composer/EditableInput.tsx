import React, { useEffect, useRef, useState } from "react";
import { SlashChip } from "./SlashChip";
import type { MessageToken } from "./types";
import { usePlaceholderRotator } from "./hooks/usePlaceholderRotator";
import { useMessageAssembler } from "./hooks/useMessageAssembler";

type Props = {
  value: MessageToken[];                            // controlled AST
  onChange: (tokens: MessageToken[], text: string) => void;
  onEnterSend: () => void;
  onSlashBoundary: (rect: DOMRect) => void;         // for palette positioning
};

export function EditableInput({ value, onChange, onEnterSend, onSlashBoundary }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const [isEmpty, setIsEmpty] = useState(true);
  const placeholder = usePlaceholderRotator([
    "What would you like to create today?",
    "Try /doc to start a new document...",
    "Use /agent to automate a task...",
    "Ask me anything about your workspace...",
  ]);
  const { renderToDOM, extractTokensAndText, findSlashBoundaryRect } = useMessageAssembler();

  // Render AST into the DOM on value changes
  useEffect(() => {
    if (!ref.current) return;
    renderToDOM(ref.current, value);
    setIsEmpty(ref.current.textContent?.trim() === "" && ref.current.querySelectorAll("[data-chip]").length === 0);
  }, [value, renderToDOM]);

  const handleInput = () => {
    if (!ref.current) return;
    const { tokens, text } = extractTokensAndText(ref.current);
    onChange(tokens, text);

    const rect = findSlashBoundaryRect(ref.current);
    if (rect) onSlashBoundary(rect);

    setIsEmpty(text.trim() === "" && ref.current.querySelectorAll("[data-chip]").length === 0);
  };

  const handleKeyDown: React.KeyboardEventHandler = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      onEnterSend();
    }
    if (e.key === "/" && ref.current) {
      const rect = findSlashBoundaryRect(ref.current, true);
      if (rect) onSlashBoundary(rect);
    }
  };

  return (
    <div className="relative">
      {isEmpty && (
        <div className="absolute inset-0 px-4 pt-4 pointer-events-none text-muted-foreground">
          {placeholder}
        </div>
      )}
      <div
        ref={ref}
        contentEditable
        role="textbox"
        aria-multiline="true"
        aria-label="Message composer"
        className="w-full px-4 pt-4 pb-16 bg-transparent text-body min-h-[140px] outline-none"
        onInput={handleInput}
        onKeyDown={handleKeyDown}
        suppressContentEditableWarning
      />
    </div>
  );
}
