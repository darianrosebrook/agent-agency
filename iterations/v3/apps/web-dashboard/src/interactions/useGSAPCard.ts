/**
 * GSAP Card Animation Hook
 * Professional card interactions for dashboard
 *
 * @author @darianrosebrook
 */

"use client";

import { useRef, useCallback, useEffect } from "react";
import { gsap } from "gsap";

export interface UseGSAPCardOptions {
  hoverY?: number;
  hoverScale?: number;
  duration?: number;
  ease?: string;
}

/**
 * Hook for GSAP-powered card hover animations
 * Provides smooth, professional interactions
 */
export function useGSAPCard(options: UseGSAPCardOptions = {}) {
  const {
    hoverY = -4,
    hoverScale = 1,
    duration = 0.3,
    ease = 'power2.out',
  } = options;

  const cardRef = useRef<HTMLDivElement>(null);
  const tweenRef = useRef<gsap.core.Tween | null>(null);

  const handleMouseEnter = useCallback(() => {
    if (!cardRef.current) return;

    // Kill any existing tween
    if (tweenRef.current) {
      tweenRef.current.kill();
    }

    // Animate card up with shadow
    tweenRef.current = gsap.to(cardRef.current, {
      y: hoverY,
      scale: hoverScale,
      boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
      duration,
      ease,
    });

    // Animate accent border
    const accentBefore = cardRef.current.querySelector('::before');
    if (accentBefore) {
      gsap.to(accentBefore, {
        opacity: 1,
        duration,
        ease,
      });
    }
  }, [hoverY, hoverScale, duration, ease]);

  const handleMouseLeave = useCallback(() => {
    if (!cardRef.current) return;

    if (tweenRef.current) {
      tweenRef.current.kill();
    }

    // Animate back to normal
    tweenRef.current = gsap.to(cardRef.current, {
      y: 0,
      scale: 1,
      boxShadow: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
      duration,
      ease,
    });
  }, [duration, ease]);

  return {
    ref: cardRef,
    handleMouseEnter,
    handleMouseLeave,
  };
}

/**
 * Hook for metric card number animations
 * Animates numbers counting up smoothly
 */
export function useMetricAnimation(
  targetValue: number,
  options: { duration?: number; decimals?: number; enabled?: boolean } = {}
) {
  const { duration = 1.2, decimals = 0, enabled = true } = options;

  const elementRef = useRef<HTMLElement>(null);
  const previousValueRef = useRef(0);

  const animate = useCallback(
    (to: number) => {
      if (!elementRef.current || !enabled) {
        if (elementRef.current) {
          elementRef.current.textContent = decimals > 0 ? to.toFixed(decimals) : String(Math.round(to));
        }
        return;
      }

      const from = previousValueRef.current;
      const counter = { value: from };

      gsap.to(counter, {
        value: to,
        duration,
        ease: 'power2.out',
        onUpdate: () => {
          if (elementRef.current) {
            elementRef.current.textContent = decimals > 0 
              ? counter.value.toFixed(decimals) 
              : String(Math.round(counter.value));
          }
        },
        onComplete: () => {
          previousValueRef.current = to;
        },
      });
    },
    [duration, decimals, enabled]
  );

  // Trigger animation when targetValue changes
  useEffect(() => {
    animate(targetValue);
  }, [targetValue, animate]);

  return {
    ref: elementRef,
  };
}


