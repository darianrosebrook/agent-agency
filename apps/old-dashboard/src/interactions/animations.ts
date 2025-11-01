/**
 * Animation utilities for Agent Agency Dashboard
 * Based on FlowPress animations with GSAP integration
 *
 * @author @darianrosebrook
 */

import { gsap } from "gsap";
// Tree-shake specific GSAP plugins for smaller bundle
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { TextPlugin } from "gsap/TextPlugin";

// Register plugins to reduce bundle size
gsap.registerPlugin(ScrollTrigger, TextPlugin);

/**
 * Easing functions for smooth animations
 */
export const easings = {
  ease: 'ease',
  'ease-in': 'ease-in',
  'ease-out': 'ease-out',
  'ease-in-out': 'ease-in-out',
  // GSAP easings
  'power1.in': 'power1.in',
  'power1.out': 'power1.out',
  'power1.inOut': 'power1.inOut',
  'power2.in': 'power2.in',
  'power2.out': 'power2.out',
  'power2.inOut': 'power2.inOut',
  'power3.in': 'power3.in',
  'power3.out': 'power3.out',
  'power3.inOut': 'power3.inOut',
  'back.out': 'back.out',
  'elastic.out': 'elastic.out',
} as const;

export type EasingType = keyof typeof easings;

/**
 * Animation duration presets
 */
export const durations = {
  instant: 0,
  fast: 150,
  normal: 300,
  slow: 500,
  slower: 800,
} as const;

export type DurationType = keyof typeof durations;

/**
 * Creates a CSS transition string
 */
export function createTransition(
  property: string,
  duration: number | DurationType = 'normal',
  easing: EasingType = 'ease',
  delay: number = 0
): string {
  const durationValue = typeof duration === 'number' ? duration : durations[duration];
  const easingValue = easings[easing];

  return `${property} ${durationValue}ms ${easingValue}${delay > 0 ? ` ${delay}ms` : ''}`;
}

/**
 * GSAP Animation Presets for Dashboard Components
 */

/**
 * Fade in animation using GSAP
 */
export function animateFadeIn(
  element: HTMLElement | null,
  options: { duration?: number; delay?: number; onComplete?: () => void } = {}
) {
  if (!element) return;

  const { duration = 0.4, delay = 0, onComplete } = options;

  return gsap.fromTo(
    element,
    { opacity: 0 },
    {
      opacity: 1,
      duration,
      delay,
      ease: 'power2.out',
      onComplete: onComplete || (() => {}),
    }
  );
}

/**
 * Slide up and fade in animation using GSAP
 */
export function animateSlideUp(
  element: HTMLElement | null,
  options: { duration?: number; delay?: number; distance?: number; onComplete?: () => void } = {}
) {
  if (!element) return;

  const { duration = 0.5, delay = 0, distance = 20, onComplete } = options;

  return gsap.fromTo(
    element,
    {
      opacity: 0,
      y: distance,
    },
    {
      opacity: 1,
      y: 0,
      duration,
      delay,
      ease: 'power3.out',
      onComplete: onComplete || (() => {}),
    }
  );
}

/**
 * Scale in animation using GSAP
 */
export function animateScaleIn(
  element: HTMLElement | null,
  options: { duration?: number; delay?: number; from?: number; onComplete?: () => void } = {}
) {
  if (!element) return;

  const { duration = 0.4, delay = 0, from = 0.95, onComplete } = options;

  return gsap.fromTo(
    element,
    {
      opacity: 0,
      scale: from,
    },
    {
      opacity: 1,
      scale: 1,
      duration,
      delay,
      ease: 'back.out(1.4)',
      onComplete: onComplete || (() => {}),
    }
  );
}

/**
 * Stagger animation for lists/grids using GSAP
 */
export function animateStagger(
  elements: HTMLElement[] | NodeListOf<HTMLElement>,
  options: { 
    duration?: number; 
    stagger?: number; 
    delay?: number;
    direction?: 'up' | 'down' | 'fade';
    onComplete?: () => void;
  } = {}
) {
  if (!elements || elements.length === 0) return;

  const { duration = 0.4, stagger = 0.05, delay = 0, direction = 'up', onComplete } = options;

  const fromVars: gsap.TweenVars = { opacity: 0 };
  const toVars: gsap.TweenVars = { opacity: 1, duration, ease: 'power2.out', onComplete: onComplete || (() => {}) };

  if (direction === 'up') {
    fromVars.y = 20;
    toVars.y = 0;
  } else if (direction === 'down') {
    fromVars.y = -20;
    toVars.y = 0;
  }

  return gsap.fromTo(elements, fromVars, {
    ...toVars,
    stagger,
    delay,
  });
}

/**
 * Card hover animation using GSAP
 */
export function animateCardHover(element: HTMLElement | null, isHovering: boolean) {
  if (!element) return;

  return gsap.to(element, {
    y: isHovering ? -4 : 0,
    boxShadow: isHovering
      ? '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)'
      : '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
    duration: 0.3,
    ease: 'power2.out',
  });
}

/**
 * Number counter animation using GSAP
 */
export function animateCounter(
  element: HTMLElement | null,
  from: number,
  to: number,
  options: { duration?: number; decimals?: number; onUpdate?: (value: number) => void } = {}
) {
  if (!element) return;

  const { duration = 1, decimals = 0, onUpdate } = options;
  const counter = { value: from };

  return gsap.to(counter, {
    value: to,
    duration,
    ease: 'power2.out',
    onUpdate: () => {
      const value = decimals > 0 ? counter.value.toFixed(decimals) : Math.round(counter.value);
      if (element) {
        element.textContent = String(value);
      }
      onUpdate?.(counter.value);
    },
  });
}

/**
 * Slide in from direction using GSAP
 */
export function animateSlideIn(
  element: HTMLElement | null,
  direction: 'left' | 'right' | 'up' | 'down' = 'up',
  options: { duration?: number; delay?: number; distance?: number } = {}
) {
  if (!element) return;

  const { duration = 0.5, delay = 0, distance = 30 } = options;

  const fromVars: gsap.TweenVars = { opacity: 0 };
  
  switch (direction) {
    case 'left':
      fromVars.x = -distance;
      break;
    case 'right':
      fromVars.x = distance;
      break;
    case 'up':
      fromVars.y = distance;
      break;
    case 'down':
      fromVars.y = -distance;
      break;
  }

  return gsap.fromTo(
    element,
    fromVars,
    {
      opacity: 1,
      x: 0,
      y: 0,
      duration,
      delay,
      ease: 'power3.out',
    }
  );
}

/**
 * Pulse animation using GSAP
 */
export function animatePulse(element: HTMLElement | null) {
  if (!element) return;

  return gsap.to(element, {
    scale: 1.05,
    duration: 0.6,
    ease: 'power1.inOut',
    yoyo: true,
    repeat: -1,
  });
}

/**
 * Rotate animation using GSAP (for loading spinners)
 */
export function animateRotate(element: HTMLElement | null, options: { duration?: number } = {}) {
  if (!element) return;

  const { duration = 1 } = options;

  return gsap.to(element, {
    rotation: 360,
    duration,
    ease: 'linear',
    repeat: -1,
  });
}

/**
 * Creates a CSS properties object for animations
 */
export function fadeIn(
  duration: number | DurationType = 'normal',
  easing: EasingType = 'ease'
): React.CSSProperties {
  return {
    opacity: 1,
    transition: createTransition('opacity', duration, easing),
  };
}

export function fadeOut(
  duration: number | DurationType = 'normal',
  easing: EasingType = 'ease'
): React.CSSProperties {
  return {
    opacity: 0,
    transition: createTransition('opacity', duration, easing),
  };
}

export function slideInRight(
  duration: number | DurationType = 'normal',
  easing: EasingType = 'ease'
): React.CSSProperties {
  return {
    transform: 'translateX(0)',
    transition: createTransition('transform', duration, easing),
  };
}

export function slideOutRight(
  duration: number | DurationType = 'normal',
  easing: EasingType = 'ease'
): React.CSSProperties {
  return {
    transform: 'translateX(100%)',
    transition: createTransition('transform', duration, easing),
  };
}

export function slideDown(
  duration: number | DurationType = 'normal',
  easing: EasingType = 'ease'
): React.CSSProperties {
  return {
    transform: 'translateY(0)',
    transition: createTransition('transform', duration, easing),
  };
}

export function slideUp(
  duration: number | DurationType = 'normal',
  easing: EasingType = 'ease'
): React.CSSProperties {
  return {
    transform: 'translateY(-10px)',
    transition: createTransition('transform', duration, easing),
  };
}


