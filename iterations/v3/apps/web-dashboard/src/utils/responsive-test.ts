/**
 * Responsive Design Testing Utilities
 * Client-side utilities for detecting layout issues
 * 
 * @author @darianrosebrook
 */

'use client';

/**
 * Detects horizontal overflow (horizontal scroll)
 * Returns elements that are causing overflow
 */
export function detectHorizontalOverflow(): HTMLElement[] {
  const overflowingElements: HTMLElement[] = [];
  
  document.querySelectorAll('*').forEach((el) => {
    const element = el as HTMLElement;
    if (element.scrollWidth > element.clientWidth) {
      overflowingElements.push(element);
    }
  });
  
  return overflowingElements;
}

/**
 * Measures cumulative layout shift (CLS)
 * Logs all layout shifts to console
 */
export function measureCLS(): Promise<number> {
  return new Promise((resolve) => {
    let cls = 0;
    
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        const layoutShift = entry as any;
        if (!layoutShift.hadRecentInput) {
          cls += layoutShift.value;
          console.log('[CLS] Layout shift detected:', {
            value: layoutShift.value,
            totalCLS: cls,
            sources: layoutShift.sources,
          });
        }
      }
    });
    
    observer.observe({ type: 'layout-shift', buffered: true });
    
    // Stop observing after 10 seconds
    setTimeout(() => {
      observer.disconnect();
      resolve(cls);
    }, 10000);
  });
}

/**
 * Checks if all interactive elements meet touch target minimum (44x44px)
 * Returns elements that are too small
 */
export function checkTouchTargets(minSize: number = 44): HTMLElement[] {
  const tooSmall: HTMLElement[] = [];
  
  const selectors = 'button, a, input, select, textarea, [role="button"], [onclick]';
  
  document.querySelectorAll(selectors).forEach((el) => {
    const element = el as HTMLElement;
    const rect = element.getBoundingClientRect();
    
    if (rect.width < minSize || rect.height < minSize) {
      tooSmall.push(element);
      console.warn('[Touch Target] Element too small:', {
        element,
        width: rect.width,
        height: rect.height,
        minimum: minSize,
      });
    }
  });
  
  return tooSmall;
}

/**
 * Gets current viewport dimensions
 */
export function getViewportSize(): { width: number; height: number; breakpoint: string } {
  const width = window.innerWidth;
  const height = window.innerHeight;
  
  let breakpoint = 'xs';
  if (width >= 1200) breakpoint = 'xl';
  else if (width >= 1024) breakpoint = 'lg';
  else if (width >= 768) breakpoint = 'md';
  else if (width >= 640) breakpoint = 'sm';
  
  return { width, height, breakpoint };
}

/**
 * Logs all CSS containment properties
 * Helps verify layout isolation
 */
export function auditContainment(): void {
  const contained: Array<{ element: HTMLElement; containValue: string }> = [];
  
  document.querySelectorAll('*').forEach((el) => {
    const element = el as HTMLElement;
    const contain = getComputedStyle(element).contain;
    
    if (contain && contain !== 'none') {
      contained.push({ element, containValue: contain });
    }
  });
  
  console.log('[Containment Audit] Elements with CSS containment:', contained.length);
  console.table(contained.map(({ element, containValue }) => ({
    tag: element.tagName,
    class: element.className,
    contain: containValue,
  })));
}

/**
 * Finds elements without proper dimensions that might cause CLS
 */
export function findPotentialCLSElements(): HTMLElement[] {
  const risky: HTMLElement[] = [];
  
  document.querySelectorAll('*').forEach((el) => {
    const element = el as HTMLElement;
    const style = getComputedStyle(element);
    
    // Elements with content but no height constraint
    const hasContent = element.children.length > 0 || element.textContent?.trim();
    const hasHeightConstraint = style.minHeight !== 'auto' || 
                                 style.height !== 'auto' || 
                                 style.maxHeight !== 'none';
    
    const isDynamic = element.hasAttribute('data-dynamic') || 
                     element.classList.contains('dynamic') ||
                     element.querySelector('[data-dynamic]');
    
    if (hasContent && !hasHeightConstraint && isDynamic) {
      risky.push(element);
    }
  });
  
  console.log('[CLS Risk] Elements without height constraints:', risky.length);
  risky.forEach((el) => {
    console.warn('[CLS Risk]', {
      element: el,
      tag: el.tagName,
      class: el.className,
    });
  });
  
  return risky;
}

/**
 * Performance metrics snapshot
 */
export function getPerformanceMetrics() {
  const paint = performance.getEntriesByType('paint');
  const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
  
  return {
    firstPaint: paint.find(p => p.name === 'first-paint')?.startTime,
    firstContentfulPaint: paint.find(p => p.name === 'first-contentful-paint')?.startTime,
    domContentLoaded: navigation?.domContentLoadedEventEnd - navigation?.domContentLoadedEventStart,
    loadComplete: navigation?.loadEventEnd - navigation?.loadEventStart,
  };
}

/**
 * Debug helper - highlights all layout shifts visually
 * Call this in dev mode to see shifts in real-time
 */
export function enableLayoutShiftVisualization(): () => void {
  const style = document.createElement('style');
  style.innerHTML = `
    .layout-shift-highlight {
      outline: 3px solid red !important;
      outline-offset: -3px;
      background: rgba(255, 0, 0, 0.1) !important;
    }
  `;
  document.head.appendChild(style);
  
  const observer = new PerformanceObserver((list) => {
    for (const entry of list.getEntries()) {
      const shift = entry as any;
      if (!shift.hadRecentInput && shift.sources) {
        shift.sources.forEach((source: any) => {
          if (source.node) {
            source.node.classList.add('layout-shift-highlight');
            setTimeout(() => {
              source.node.classList.remove('layout-shift-highlight');
            }, 1000);
          }
        });
      }
    }
  });
  
  observer.observe({ type: 'layout-shift', buffered: true });
  
  return () => {
    observer.disconnect();
    style.remove();
  };
}

/**
 * Quick test - run all diagnostics
 */
export function runLayoutDiagnostics() {
  console.group('📐 Layout Diagnostics');
  
  console.log('Viewport:', getViewportSize());
  
  const overflow = detectHorizontalOverflow();
  console.log('Horizontal Overflow:', overflow.length, 'elements');
  
  const touchTargets = checkTouchTargets();
  console.log('Touch Targets < 44px:', touchTargets.length, 'elements');
  
  auditContainment();
  
  const clsRisk = findPotentialCLSElements();
  console.log('CLS Risk:', clsRisk.length, 'elements');
  
  console.log('Performance:', getPerformanceMetrics());
  
  measureCLS().then((cls) => {
    console.log('Final CLS Score:', cls);
  });
  
  console.groupEnd();
}

// Expose to window for easy console access in development
if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
  (window as any).layoutTest = {
    detectOverflow: detectHorizontalOverflow,
    measureCLS,
    checkTouchTargets,
    getViewport: getViewportSize,
    auditContainment,
    findCLSRisk: findPotentialCLSElements,
    runDiagnostics: runLayoutDiagnostics,
    visualizeShifts: enableLayoutShiftVisualization,
  };
  
  console.log('💡 Layout testing utilities available at: window.layoutTest');
  console.log('   Run: window.layoutTest.runDiagnostics()');
}

