import { useCallback } from "react";
import type { MessageToken } from "../types";

export function useMessageAssembler() {
  const renderToDOM = useCallback((root: HTMLElement, tokens: MessageToken[]) => {
    // Clear existing content
    root.innerHTML = "";

    tokens.forEach((token) => {
      if (token.kind === "text") {
        const textNode = document.createTextNode(token.text);
        root.appendChild(textNode);
      } else if (token.kind === "command") {
        const chip = document.createElement("span");
        chip.setAttribute("data-chip", "");
        chip.setAttribute("data-command", token.command);
        chip.className = "inline-flex items-center gap-1.5 px-2.5 py-0.5 mx-0.5 bg-workspace-accent/10 border border-workspace-accent/20 rounded-md text-sm align-middle";
        chip.contentEditable = "false";

        const commandText = document.createElement("span");
        commandText.className = "font-medium text-workspace-accent";
        commandText.textContent = token.command;

        chip.appendChild(commandText);
        root.appendChild(chip);
      }
    });

    // Add a trailing text node for editing
    const trailingText = document.createTextNode("");
    root.appendChild(trailingText);
  }, []);

  const extractTokensAndText = useCallback((root: HTMLElement): { tokens: MessageToken[]; text: string } => {
    const tokens: MessageToken[] = [];
    const textParts: string[] = [];
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT);

    let node: Node | null;
    while ((node = walker.nextNode())) {
      if (node.nodeType === Node.TEXT_NODE) {
        const text = node.textContent || "";
        if (text) {
          tokens.push({ kind: "text", text });
          textParts.push(text);
        }
      } else if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as HTMLElement;
        if (element.hasAttribute("data-chip")) {
          const command = element.getAttribute("data-command") || "";
          tokens.push({ kind: "command", command });
          textParts.push(command);
        }
      }
    }

    return {
      tokens,
      text: textParts.join(""),
    };
  }, []);

  const findSlashBoundaryRect = useCallback((root: HTMLElement, onKey = false): DOMRect | null => {
    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return null;

    const range = selection.getRangeAt(0);
    const text = root.textContent || "";

    // Find the last "/" before cursor
    const cursorPos = range.startOffset;
    const textBefore = text.substring(0, cursorPos);
    const lastSlashIndex = textBefore.lastIndexOf("/");

    if (lastSlashIndex === -1) return null;

    // Check if it's at word boundary
    const charAfter = text.charAt(lastSlashIndex + 1);
    const isBoundary = lastSlashIndex === 0 || /\s/.test(text.charAt(lastSlashIndex - 1));

    if (!isBoundary && (!onKey || charAfter)) return null;

    // Create a range for the slash character
    const slashRange = document.createRange();
    slashRange.setStart(root.firstChild || root, lastSlashIndex);
    slashRange.setEnd(root.firstChild || root, lastSlashIndex + 1);

    return slashRange.getBoundingClientRect();
  }, []);

  return {
    renderToDOM,
    extractTokensAndText,
    findSlashBoundaryRect,
  };
}
