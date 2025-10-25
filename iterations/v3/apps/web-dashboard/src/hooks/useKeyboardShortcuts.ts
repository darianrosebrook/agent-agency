/**
 * Keyboard Shortcuts Hook
 * Provides keyboard shortcuts for power users
 * 
 * @author @darianrosebrook
 */

"use client";

import { useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";

export interface KeyboardShortcut {
  key: string;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  metaKey?: boolean;
  action: () => void;
  description: string;
}

interface UseKeyboardShortcutsOptions {
  enabled?: boolean;
  shortcuts: KeyboardShortcut[];
}

export function useKeyboardShortcuts({ 
  enabled = true, 
  shortcuts 
}: UseKeyboardShortcutsOptions) {
  // const router = useRouter(); // TODO: Implement navigation shortcuts

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (!enabled) return;

    // Don't trigger shortcuts when typing in inputs
    const target = event.target as HTMLElement;
    if (
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.contentEditable === "true"
    ) {
      return;
    }

    shortcuts.forEach((shortcut) => {
      const {
        key,
        ctrlKey = false,
        shiftKey = false,
        altKey = false,
        metaKey = false,
        action,
      } = shortcut;

      if (
        event.key.toLowerCase() === key.toLowerCase() &&
        event.ctrlKey === ctrlKey &&
        event.shiftKey === shiftKey &&
        event.altKey === altKey &&
        event.metaKey === metaKey
      ) {
        event.preventDefault();
        action();
      }
    });
  }, [enabled, shortcuts]);

  useEffect(() => {
    if (!enabled) return;

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [enabled, handleKeyDown]);
}

// Common keyboard shortcuts for the dashboard
export function useDashboardShortcuts(
  onSearchOpen: () => void,
  onSettingsOpen: () => void,
  onTasksOpen: () => void,
  onMetricsOpen: () => void
) {
  const router = useRouter();

  const shortcuts: KeyboardShortcut[] = [
    {
      key: "k",
      ctrlKey: true,
      action: onSearchOpen,
      description: "Open global search",
    },
    {
      key: "s",
      ctrlKey: true,
      action: onSettingsOpen,
      description: "Open settings",
    },
    {
      key: "t",
      ctrlKey: true,
      action: onTasksOpen,
      description: "Go to tasks",
    },
    {
      key: "m",
      ctrlKey: true,
      action: onMetricsOpen,
      description: "Go to metrics",
    },
    {
      key: "h",
      ctrlKey: true,
      action: () => router.push("/"),
      description: "Go to home",
    },
    {
      key: "r",
      ctrlKey: true,
      action: () => window.location.reload(),
      description: "Refresh page",
    },
    {
      key: "Escape",
      action: () => {
        // Close any open modals or dropdowns
        const modals = document.querySelectorAll('[role="dialog"]');
        modals.forEach(modal => {
          const closeButton = modal.querySelector('[aria-label="Close"]');
          if (closeButton) {
            (closeButton as HTMLElement).click();
          }
        });
      },
      description: "Close modals",
    },
  ];

  useKeyboardShortcuts({ shortcuts });
}

// Hook for showing keyboard shortcuts help
export function useKeyboardShortcutsHelp() {
  const shortcuts: KeyboardShortcut[] = [
    {
      key: "K",
      ctrlKey: true,
      action: () => {},
      description: "Open global search",
    },
    {
      key: "S",
      ctrlKey: true,
      action: () => {},
      description: "Open settings",
    },
    {
      key: "T",
      ctrlKey: true,
      action: () => {},
      description: "Go to tasks",
    },
    {
      key: "M",
      ctrlKey: true,
      action: () => {},
      description: "Go to metrics",
    },
    {
      key: "H",
      ctrlKey: true,
      action: () => {},
      description: "Go to home",
    },
    {
      key: "R",
      ctrlKey: true,
      action: () => {},
      description: "Refresh page",
    },
    {
      key: "Escape",
      action: () => {},
      description: "Close modals",
    },
  ];

  return shortcuts;
}
