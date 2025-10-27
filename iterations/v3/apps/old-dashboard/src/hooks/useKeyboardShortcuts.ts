/**
 * Keyboard Shortcuts Hook for Agent Agency V3 Dashboard
 * 
 * @author @darianrosebrook
 * 
 * Comprehensive keyboard shortcuts system with customizable shortcuts,
 * conflict resolution, and accessibility support.
 */

'use client';

import { useEffect, useCallback, useRef } from 'react';
import { focusManager, screenReader } from '@/lib/accessibility';

export interface KeyboardShortcut {
  key: string;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  metaKey?: boolean;
  description: string;
  action: () => void;
  preventDefault?: boolean;
  stopPropagation?: boolean;
  disabled?: boolean;
}

export interface KeyboardShortcutsConfig {
  shortcuts: KeyboardShortcut[];
  enabled?: boolean;
  global?: boolean;
  preventDefault?: boolean;
  stopPropagation?: boolean;
}

/**
 * Custom hook for managing keyboard shortcuts
 */
export function useKeyboardShortcuts(config: KeyboardShortcutsConfig) {
  const {
    shortcuts,
    enabled = true,
    global = true,
    preventDefault: globalPreventDefault = true,
    stopPropagation: globalStopPropagation = false,
  } = config;

  const shortcutsRef = useRef(shortcuts);
  const enabledRef = useRef(enabled);

  // Update refs when props change
  useEffect(() => {
    shortcutsRef.current = shortcuts;
  }, [shortcuts]);

  useEffect(() => {
    enabledRef.current = enabled;
  }, [enabled]);

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (!enabledRef.current) return;

    // Find matching shortcut
    const matchingShortcut = shortcutsRef.current.find(shortcut => {
      if (shortcut.disabled) return false;
      
      return (
        shortcut.key === event.key &&
        !!shortcut.ctrlKey === event.ctrlKey &&
        !!shortcut.shiftKey === event.shiftKey &&
        !!shortcut.altKey === event.altKey &&
        !!shortcut.metaKey === event.metaKey
      );
    });

    if (matchingShortcut) {
      // Prevent default behavior if specified
      if (matchingShortcut.preventDefault ?? globalPreventDefault) {
        event.preventDefault();
      }

      // Stop propagation if specified
      if (matchingShortcut.stopPropagation ?? globalStopPropagation) {
        event.stopPropagation();
      }

      // Execute the action
      try {
        matchingShortcut.action();
        
        // Announce shortcut execution to screen readers
        screenReader.announce(`Shortcut executed: ${matchingShortcut.description}`, 'polite');
      } catch (error) {
          console.error('Error executing keyboard shortcut:', error);
          screenReader.announceError(`Failed to execute shortcut: ${matchingShortcut.description}`);
        }
    }
  }, [globalPreventDefault, globalStopPropagation]);

  useEffect(() => {
    if (!enabled) return;

    const target = global ? document : document.body;
    target.addEventListener('keydown', handleKeyDown);

    return () => {
      target.removeEventListener('keydown', handleKeyDown);
    };
  }, [enabled, global, handleKeyDown]);

  return {
    shortcuts: shortcutsRef.current,
    enabled: enabledRef.current,
  };
}

/**
 * Predefined dashboard shortcuts
 */
export const DASHBOARD_SHORTCUTS: KeyboardShortcut[] = [
  {
    key: 'k',
    ctrlKey: true,
    description: 'Focus search',
    action: () => {
      const searchInput = document.querySelector('input[type="search"], input[placeholder*="search" i]') as HTMLElement;
      if (searchInput) {
        focusManager.focusElement(searchInput);
      }
    },
  },
  {
    key: 'n',
    altKey: true,
    description: 'Focus navigation',
    action: () => {
      const nav = document.querySelector('nav, [role="navigation"]') as HTMLElement;
      if (nav) {
        const firstFocusable = focusManager.getFocusableElements(nav)[0];
        focusManager.focusElement(firstFocusable);
      }
    },
  },
  {
    key: 'm',
    altKey: true,
    description: 'Focus main content',
    action: () => {
      const main = document.querySelector('main, [role="main"]') as HTMLElement;
      if (main) {
        const firstFocusable = focusManager.getFocusableElements(main)[0];
        focusManager.focusElement(firstFocusable);
      }
    },
  },
  {
    key: 's',
    altKey: true,
    description: 'Toggle sidebar',
    action: () => {
      const sidebar = document.querySelector('[data-sidebar], .sidebar') as HTMLElement;
      if (sidebar) {
        const isHidden = sidebar.getAttribute('aria-hidden') === 'true';
        sidebar.setAttribute('aria-hidden', String(!isHidden));
        screenReader.announce(isHidden ? 'Sidebar opened' : 'Sidebar closed');
      }
    },
  },
  {
    key: 'Escape',
    description: 'Close modal or return focus',
    action: () => {
      const modal = document.querySelector('[role="dialog"][aria-modal="true"]') as HTMLElement;
      if (modal) {
        // Close modal
        const closeButton = modal.querySelector('[aria-label*="close" i], [aria-label*="dismiss" i]') as HTMLElement;
        if (closeButton) {
          closeButton.click();
        }
      } else {
        // Return focus to previous element
        focusManager.returnFocus();
      }
    },
  },
  {
    key: '?',
    shiftKey: true,
    description: 'Show keyboard shortcuts help',
    action: () => {
      // TODO: Implement help modal
      screenReader.announce('Keyboard shortcuts help not yet implemented');
    },
  },
];

/**
 * Navigation shortcuts for data tables
 */
export const TABLE_NAVIGATION_SHORTCUTS: KeyboardShortcut[] = [
  {
    key: 'ArrowUp',
    description: 'Move to previous row',
    action: () => {
      const table = document.querySelector('table[role="grid"]') as HTMLElement;
      if (table) {
        const currentRow = document.activeElement?.closest('tr');
        if (currentRow) {
          const previousRow = currentRow.previousElementSibling as HTMLElement;
          if (previousRow) {
            const firstCell = previousRow.querySelector('td, th') as HTMLElement;
            focusManager.focusElement(firstCell);
          }
        }
      }
    },
  },
  {
    key: 'ArrowDown',
    description: 'Move to next row',
    action: () => {
      const table = document.querySelector('table[role="grid"]') as HTMLElement;
      if (table) {
        const currentRow = document.activeElement?.closest('tr');
        if (currentRow) {
          const nextRow = currentRow.nextElementSibling as HTMLElement;
          if (nextRow) {
            const firstCell = nextRow.querySelector('td, th') as HTMLElement;
            focusManager.focusElement(firstCell);
          }
        }
      }
    },
  },
  {
    key: 'ArrowLeft',
    description: 'Move to previous cell',
    action: () => {
      const currentCell = document.activeElement as HTMLElement;
      if (currentCell && currentCell.tagName === 'TD') {
        const previousCell = currentCell.previousElementSibling as HTMLElement;
        if (previousCell) {
          focusManager.focusElement(previousCell);
        }
      }
    },
  },
  {
    key: 'ArrowRight',
    description: 'Move to next cell',
    action: () => {
      const currentCell = document.activeElement as HTMLElement;
      if (currentCell && currentCell.tagName === 'TD') {
        const nextCell = currentCell.nextElementSibling as HTMLElement;
        if (nextCell) {
          focusManager.focusElement(nextCell);
        }
      }
    },
  },
  {
    key: 'Home',
    description: 'Move to first cell',
    action: () => {
      const table = document.querySelector('table[role="grid"]') as HTMLElement;
      if (table) {
        const firstCell = table.querySelector('td, th') as HTMLElement;
        focusManager.focusElement(firstCell);
      }
    },
  },
  {
    key: 'End',
    description: 'Move to last cell',
    action: () => {
      const table = document.querySelector('table[role="grid"]') as HTMLElement;
      if (table) {
        const cells = table.querySelectorAll('td, th');
        const lastCell = cells[cells.length - 1] as HTMLElement;
        focusManager.focusElement(lastCell);
      }
    },
  },
];

/**
 * Form shortcuts
 */
export const FORM_SHORTCUTS: KeyboardShortcut[] = [
  {
    key: 'Tab',
    description: 'Move to next field',
    action: () => {
      // Default browser behavior
    },
  },
  {
    key: 'Tab',
    shiftKey: true,
    description: 'Move to previous field',
    action: () => {
      // Default browser behavior
    },
  },
  {
    key: 'Enter',
    description: 'Submit form or activate button',
    action: () => {
      const form = document.activeElement?.closest('form');
      if (form) {
        const submitButton = form.querySelector('button[type="submit"]') as HTMLElement;
        if (submitButton) {
          submitButton.click();
        }
      }
    },
  },
];

/**
 * Utility function to create custom shortcuts
 */
export function createShortcut(
  key: string,
  description: string,
  action: () => void,
  modifiers: {
    ctrlKey?: boolean;
    shiftKey?: boolean;
    altKey?: boolean;
    metaKey?: boolean;
  } = {}
): KeyboardShortcut {
  return {
    key,
    description,
    action,
    ...modifiers,
  };
}

/**
 * Utility function to combine multiple shortcut sets
 */
export function combineShortcuts(...shortcutSets: KeyboardShortcut[][]): KeyboardShortcut[] {
  return shortcutSets.flat();
}

/**
 * Hook for dashboard-specific shortcuts
 */
export function useDashboardShortcuts(enabled: boolean = true) {
  return useKeyboardShortcuts({
    shortcuts: DASHBOARD_SHORTCUTS,
    enabled,
    global: true,
  });
}

/**
 * Hook for table navigation shortcuts
 */
export function useTableNavigationShortcuts(enabled: boolean = true) {
  return useKeyboardShortcuts({
    shortcuts: TABLE_NAVIGATION_SHORTCUTS,
    enabled,
    global: false, // Only active when table is focused
  });
}

/**
 * Hook for form shortcuts
 */
export function useFormShortcuts(enabled: boolean = true) {
  return useKeyboardShortcuts({
    shortcuts: FORM_SHORTCUTS,
    enabled,
    global: false, // Only active when form is focused
  });
}