/**
 * Accessibility Utilities for Agent Agency V3 Dashboard
 * 
 * @author @darianrosebrook
 * 
 * Comprehensive accessibility utilities including keyboard navigation,
 * focus management, screen reader support, and ARIA helpers.
 */

/**
 * Keyboard navigation utilities
 */
export const KEYBOARD_NAVIGATION = {
  // Common keyboard shortcuts
  ESCAPE: 'Escape',
  ENTER: 'Enter',
  SPACE: ' ',
  TAB: 'Tab',
  ARROW_UP: 'ArrowUp',
  ARROW_DOWN: 'ArrowDown',
  ARROW_LEFT: 'ArrowLeft',
  ARROW_RIGHT: 'ArrowRight',
  HOME: 'Home',
  END: 'End',
  PAGE_UP: 'PageUp',
  PAGE_DOWN: 'PageDown',
  
  // Custom shortcuts
  FOCUS_SEARCH: 'Ctrl+k',
  FOCUS_NAVIGATION: 'Alt+n',
  FOCUS_MAIN: 'Alt+m',
  TOGGLE_SIDEBAR: 'Alt+s',
  CLOSE_MODAL: 'Escape',
} as const;

/**
 * ARIA roles and properties
 */
export const ARIA_ROLES = {
  ALERT: 'alert',
  ALERTDIALOG: 'alertdialog',
  APPLICATION: 'application',
  ARTICLE: 'article',
  BANNER: 'banner',
  BUTTON: 'button',
  CELL: 'cell',
  CHECKBOX: 'checkbox',
  COLUMNHEADER: 'columnheader',
  COMBOBOX: 'combobox',
  COMPLEMENTARY: 'complementary',
  CONTENTINFO: 'contentinfo',
  DEFINITION: 'definition',
  DIALOG: 'dialog',
  DIRECTORY: 'directory',
  DOCUMENT: 'document',
  FEED: 'feed',
  FIGURE: 'figure',
  FORM: 'form',
  GRID: 'grid',
  GRIDCELL: 'gridcell',
  GROUP: 'group',
  HEADING: 'heading',
  IMG: 'img',
  LINK: 'link',
  LIST: 'list',
  LISTBOX: 'listbox',
  LISTITEM: 'listitem',
  LOG: 'log',
  MAIN: 'main',
  MARQUEE: 'marquee',
  MATH: 'math',
  MENU: 'menu',
  MENUBAR: 'menubar',
  MENUITEM: 'menuitem',
  MENUITEMCHECKBOX: 'menuitemcheckbox',
  MENUITEMRADIO: 'menuitemradio',
  NAVIGATION: 'navigation',
  NONE: 'none',
  NOTE: 'note',
  OPTION: 'option',
  PRESENTATION: 'presentation',
  PROGRESSBAR: 'progressbar',
  RADIO: 'radio',
  RADIOGROUP: 'radiogroup',
  REGION: 'region',
  ROW: 'row',
  ROWGROUP: 'rowgroup',
  ROWHEADER: 'rowheader',
  SCROLLBAR: 'scrollbar',
  SEARCH: 'search',
  SEPARATOR: 'separator',
  SLIDER: 'slider',
  SPINBUTTON: 'spinbutton',
  STATUS: 'status',
  SWITCH: 'switch',
  TAB: 'tab',
  TABLE: 'table',
  TABLIST: 'tablist',
  TABPANEL: 'tabpanel',
  TEXTBOX: 'textbox',
  TIMER: 'timer',
  TOOLBAR: 'toolbar',
  TOOLTIP: 'tooltip',
  TREE: 'tree',
  TREEGRID: 'treegrid',
  TREEITEM: 'treeitem',
} as const;

/**
 * ARIA states and properties
 */
export const ARIA_STATES = {
  EXPANDED: 'aria-expanded',
  SELECTED: 'aria-selected',
  CHECKED: 'aria-checked',
  DISABLED: 'aria-disabled',
  HIDDEN: 'aria-hidden',
  INVALID: 'aria-invalid',
  REQUIRED: 'aria-required',
  READONLY: 'aria-readonly',
  PRESSED: 'aria-pressed',
  SORT: 'aria-sort',
  LEVEL: 'aria-level',
  POSINSET: 'aria-posinset',
  SIZESET: 'aria-setsize',
  LABELLEDBY: 'aria-labelledby',
  DESCRIBEDBY: 'aria-describedby',
  CONTROLS: 'aria-controls',
  OWNS: 'aria-owns',
  ACTIVE_DESCENDANT: 'aria-activedescendant',
  AUTOMATIC: 'aria-autocomplete',
  BUSY: 'aria-busy',
  LIVE: 'aria-live',
  ATOMIC: 'aria-atomic',
  RELEVANT: 'aria-relevant',
  DROPEFFECT: 'aria-dropeffect',
  GRABBED: 'aria-grabbed',
  FLOWTO: 'aria-flowto',
} as const;

/**
 * Focus management utilities
 */
export class FocusManager {
  private static instance: FocusManager;
  private focusHistory: HTMLElement[] = [];
  private maxHistorySize = 10;

  private constructor() {}

  public static getInstance(): FocusManager {
    if (!FocusManager.instance) {
      FocusManager.instance = new FocusManager();
    }
    return FocusManager.instance;
  }

  /**
   * Set focus to an element with proper ARIA attributes
   */
  public focusElement(element: HTMLElement | null, options: { preventScroll?: boolean } = {}): boolean {
    if (!element) return false;

    try {
      // Store current focus in history
      const currentFocus = document.activeElement as HTMLElement;
      if (currentFocus && currentFocus !== element) {
        this.addToHistory(currentFocus);
      }

      // Focus the element
      element.focus(options);
      
      // Ensure element is visible
      element.scrollIntoView({ 
        behavior: 'smooth', 
        block: 'center',
        inline: 'center'
      });

      return true;
    } catch (error) {
      console.warn('Failed to focus element:', error);
      return false;
    }
  }

  /**
   * Trap focus within a container
   */
  public trapFocus(container: HTMLElement): () => void {
    const focusableElements = this.getFocusableElements(container);
    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Tab') {
        if (event.shiftKey) {
          // Shift + Tab: focus previous element
          if (document.activeElement === firstElement) {
            event.preventDefault();
            lastElement?.focus();
          }
        } else {
          // Tab: focus next element
          if (document.activeElement === lastElement) {
            event.preventDefault();
            firstElement?.focus();
          }
        }
      }
    };

    container.addEventListener('keydown', handleKeyDown);
    
    // Focus first element
    firstElement?.focus();

    // Return cleanup function
    return () => {
      container.removeEventListener('keydown', handleKeyDown);
    };
  }

  /**
   * Get all focusable elements within a container
   */
  public getFocusableElements(container: HTMLElement): HTMLElement[] {
    const focusableSelectors = [
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      'a[href]',
      '[tabindex]:not([tabindex="-1"])',
      '[contenteditable="true"]',
    ].join(', ');

    return Array.from(container.querySelectorAll(focusableSelectors)) as HTMLElement[];
  }

  /**
   * Add element to focus history
   */
  private addToHistory(element: HTMLElement): void {
    this.focusHistory.unshift(element);
    if (this.focusHistory.length > this.maxHistorySize) {
      this.focusHistory = this.focusHistory.slice(0, this.maxHistorySize);
    }
  }

  /**
   * Return focus to previous element
   */
  public returnFocus(): boolean {
    const previousElement = this.focusHistory.shift();
    if (previousElement) {
      return this.focusElement(previousElement);
    }
    return false;
  }

  /**
   * Clear focus history
   */
  public clearHistory(): void {
    this.focusHistory = [];
  }
}

/**
 * Screen reader utilities
 */
export class ScreenReaderManager {
  private static instance: ScreenReaderManager;
  private liveRegion: HTMLElement | null = null;

  private constructor() {
    this.createLiveRegion();
  }

  public static getInstance(): ScreenReaderManager {
    if (!ScreenReaderManager.instance) {
      ScreenReaderManager.instance = new ScreenReaderManager();
    }
    return ScreenReaderManager.instance;
  }

  /**
   * Create live region for announcements
   */
  private createLiveRegion(): void {
    if (typeof document === 'undefined') return;

    this.liveRegion = document.createElement('div');
    this.liveRegion.setAttribute('aria-live', 'polite');
    this.liveRegion.setAttribute('aria-atomic', 'true');
    this.liveRegion.style.position = 'absolute';
    this.liveRegion.style.left = '-10000px';
    this.liveRegion.style.width = '1px';
    this.liveRegion.style.height = '1px';
    this.liveRegion.style.overflow = 'hidden';
    document.body.appendChild(this.liveRegion);
  }

  /**
   * Announce message to screen readers
   */
  public announce(message: string, priority: 'polite' | 'assertive' = 'polite'): void {
    if (!this.liveRegion) return;

    this.liveRegion.setAttribute('aria-live', priority);
    this.liveRegion.textContent = message;
    
    // Clear after announcement
    setTimeout(() => {
      if (this.liveRegion) {
        this.liveRegion.textContent = '';
      }
    }, 1000);
  }

  /**
   * Announce error message
   */
  public announceError(message: string): void {
    this.announce(`Error: ${message}`, 'assertive');
  }

  /**
   * Announce success message
   */
  public announceSuccess(message: string): void {
    this.announce(`Success: ${message}`, 'polite');
  }

  /**
   * Announce status change
   */
  public announceStatus(message: string): void {
    this.announce(`Status: ${message}`, 'polite');
  }
}

/**
 * Keyboard navigation utilities
 */
export class KeyboardNavigation {
  /**
   * Handle arrow key navigation in a list
   */
  public static handleArrowNavigation(
    event: KeyboardEvent,
    items: HTMLElement[],
    currentIndex: number,
    orientation: 'horizontal' | 'vertical' = 'vertical'
  ): number {
    const isVertical = orientation === 'vertical';
    
    let newIndex = currentIndex;

    switch (event.key) {
      case isVertical ? 'ArrowUp' : 'ArrowLeft':
        event.preventDefault();
        newIndex = currentIndex > 0 ? currentIndex - 1 : items.length - 1;
        break;
      case isVertical ? 'ArrowDown' : 'ArrowRight':
        event.preventDefault();
        newIndex = currentIndex < items.length - 1 ? currentIndex + 1 : 0;
        break;
      case 'Home':
        event.preventDefault();
        newIndex = 0;
        break;
      case 'End':
        event.preventDefault();
        newIndex = items.length - 1;
        break;
      default:
        return currentIndex;
    }

    // Focus the new item
    const newItem = items[newIndex];
    if (newItem) {
      newItem.focus();
    }

    return newIndex;
  }

  /**
   * Handle escape key to close modal
   */
  public static handleEscape(
    event: KeyboardEvent,
    onEscape: () => void
  ): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      onEscape();
    }
  }

  /**
   * Handle enter/space key activation
   */
  public static handleActivation(
    event: KeyboardEvent,
    onActivate: () => void
  ): void {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onActivate();
    }
  }
}

/**
 * ARIA helper functions
 */
export const ARIA_HELPERS = {
  /**
   * Generate unique ID for ARIA relationships
   */
  generateId: (prefix: string = 'aria'): string => {
    return `${prefix}-${Math.random().toString(36).substr(2, 9)}`;
  },

  /**
   * Set ARIA attributes on an element
   */
  setAttributes: (element: HTMLElement, attributes: Record<string, string | boolean | number>): void => {
    Object.entries(attributes).forEach(([key, value]) => {
      if (typeof value === 'boolean') {
        if (value) {
          element.setAttribute(key, 'true');
        } else {
          element.removeAttribute(key);
        }
      } else {
        element.setAttribute(key, String(value));
      }
    });
  },

  /**
   * Create accessible button from any element
   */
  makeButton: (element: HTMLElement, onClick: () => void): void => {
    element.setAttribute('role', 'button');
    element.setAttribute('tabindex', '0');
    element.addEventListener('click', onClick);
    element.addEventListener('keydown', (event) => {
      KeyboardNavigation.handleActivation(event, onClick);
    });
  },

  /**
   * Create accessible toggle button
   */
  makeToggleButton: (
    element: HTMLElement, 
    isPressed: boolean, 
    onToggle: (pressed: boolean) => void
  ): void => {
    element.setAttribute('role', 'button');
    element.setAttribute('aria-pressed', String(isPressed));
    element.setAttribute('tabindex', '0');
    
    const handleToggle = () => {
      const newPressed = !isPressed;
      element.setAttribute('aria-pressed', String(newPressed));
      onToggle(newPressed);
    };

    element.addEventListener('click', handleToggle);
    element.addEventListener('keydown', (event) => {
      KeyboardNavigation.handleActivation(event, handleToggle);
    });
  },
};

// Export singleton instances
export const focusManager = FocusManager.getInstance();
export const screenReader = ScreenReaderManager.getInstance();

/**
 * Utility function for quick focus management
 */
export function focusElement(element: HTMLElement | null, options?: { preventScroll?: boolean }): boolean {
  return focusManager.focusElement(element, options);
}

/**
 * Utility function for quick screen reader announcements
 */
export function announce(message: string, priority?: 'polite' | 'assertive'): void {
  screenReader.announce(message, priority);
}

/**
 * Utility function for accessible button creation
 */
export function makeAccessibleButton(
  element: HTMLElement, 
  onClick: () => void
): void {
  ARIA_HELPERS.makeButton(element, onClick);
}

/**
 * Utility function for accessible toggle button creation
 */
export function makeAccessibleToggle(
  element: HTMLElement,
  isPressed: boolean,
  onToggle: (pressed: boolean) => void
): void {
  ARIA_HELPERS.makeToggleButton(element, isPressed, onToggle);
}
