/**
 * Scroll animation hook for Agent Agency Dashboard
 * Uses GSAP for smooth scroll-triggered animations
 *
 * @author @darianrosebrook
 */

"use client";

import { useEffect, useRef, useState } from "react";
import { gsap } from "gsap";

export interface UseScrollAnimationOptions {
  threshold?: number;
  rootMargin?: string;
  triggerOnce?: boolean;
  delay?: number;
  duration?: number;
  type?: 'fade' | 'slideUp' | 'slideDown' | 'slideLeft' | 'slideRight' | 'scale';
  distance?: number;
}

export interface UseScrollAnimationReturn<T extends HTMLElement = HTMLElement> {
  ref: React.RefObject<T | null>;
  isVisible: boolean;
  hasAnimated: boolean;
}

/**
 * Hook for managing scroll-triggered GSAP animations
 * Automatically triggers when element enters viewport
 */
export function useScrollAnimation<T extends HTMLElement = HTMLElement>(
  options: UseScrollAnimationOptions = {}
): UseScrollAnimationReturn<T> {
  const {
    threshold = 0.1,
    rootMargin = "0px 0px -50px 0px",
    triggerOnce = true,
    delay = 0,
    duration = 0.6,
    type = 'fade',
    distance = 30,
  } = options;

  const [isVisible, setIsVisible] = useState(false);
  const [hasAnimated, setHasAnimated] = useState(false);
  const ref = useRef<T | null>(null);
  const tweenRef = useRef<gsap.core.Tween | null>(null);

  useEffect(() => {
    const element = ref.current;
    if (!element) return;

    // Set initial state based on animation type
    const fromVars: gsap.TweenVars = { opacity: 0 };
    
    switch (type) {
      case 'slideUp':
        fromVars.y = distance;
        break;
      case 'slideDown':
        fromVars.y = -distance;
        break;
      case 'slideLeft':
        fromVars.x = distance;
        break;
      case 'slideRight':
        fromVars.x = -distance;
        break;
      case 'scale':
        fromVars.scale = 0.95;
        break;
    }

    // Set initial state
    gsap.set(element, fromVars);

    // Helper to animate element in
    const animateIn = () => {
      if (!triggerOnce || !hasAnimated) {
        setIsVisible(true);
        setHasAnimated(true);

        // Kill any existing animation
        if (tweenRef.current) {
          tweenRef.current.kill();
        }

        // Animate in with GSAP
        tweenRef.current = gsap.to(element, {
          opacity: 1,
          x: 0,
          y: 0,
          scale: 1,
          duration,
          delay: delay / 1000, // Convert ms to seconds for GSAP
          ease: 'power3.out',
        });
      }
    };

    // Helper to animate element out
    const animateOut = () => {
      setIsVisible(false);

      if (tweenRef.current) {
        tweenRef.current.kill();
      }

      tweenRef.current = gsap.to(element, {
        ...fromVars,
        duration: duration * 0.5,
        ease: 'power2.in',
      });
    };

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            animateIn();
          } else if (!triggerOnce) {
            // Animate out if not triggerOnce
            animateOut();
          }
        });
      },
      {
        threshold,
        rootMargin,
      }
    );

    // Use requestAnimationFrame to check viewport after layout is complete
    const rafId = requestAnimationFrame(() => {
      const rect = element.getBoundingClientRect();
      const viewportHeight = window.innerHeight || document.documentElement.clientHeight;
      const isAlreadyVisible = rect.top < viewportHeight && rect.bottom > 0;

      if (isAlreadyVisible) {
        // Trigger animation immediately if already in viewport
        animateIn();
      } else {
        // Only observe if not already visible
        observer.observe(element);
      }
    });

    return () => {
      cancelAnimationFrame(rafId);
      observer.unobserve(element);
      if (tweenRef.current) {
        tweenRef.current.kill();
      }
    };
  }, [threshold, rootMargin, triggerOnce, delay, duration, type, distance, hasAnimated]);

  return {
    ref,
    isVisible,
    hasAnimated,
  };
}

/**
 * Hook for stagger animations on lists/grids
 */
export function useStaggerAnimation<T extends HTMLElement = HTMLElement>(
  options: {
    delay?: number;
    stagger?: number;
    duration?: number;
    type?: 'fade' | 'slideUp';
  } = {}
) {
  const {
    delay = 0,
    stagger = 0.05,
    duration = 0.4,
    type = 'slideUp',
  } = options;

  const containerRef = useRef<T>(null);
  const [hasAnimated, setHasAnimated] = useState(false);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || hasAnimated) return;

    const children = container.children;
    if (!children.length) return;

    const fromVars: gsap.TweenVars = { opacity: 0 };
    
    if (type === 'slideUp') {
      fromVars.y = 20;
    }

    // Set initial state for all children
    gsap.set(children, fromVars);

    // Helper to animate children in
    const animateChildren = () => {
      if (!hasAnimated) {
        setHasAnimated(true);

        // Stagger animation
        gsap.to(children, {
          opacity: 1,
          y: 0,
          duration,
          delay,
          stagger,
          ease: 'power2.out',
        });
      }
    };

    // Observe container
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            animateChildren();
          }
        });
      },
      {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px',
      }
    );

    // Use requestAnimationFrame to check viewport after layout is complete
    const rafId = requestAnimationFrame(() => {
      const rect = container.getBoundingClientRect();
      const viewportHeight = window.innerHeight || document.documentElement.clientHeight;
      const isAlreadyVisible = rect.top < viewportHeight && rect.bottom > 0;

      if (isAlreadyVisible) {
        // Trigger animation immediately if already in viewport
        animateChildren();
      } else {
        // Only observe if not already visible
        observer.observe(container);
      }
    });

    return () => {
      cancelAnimationFrame(rafId);
      observer.unobserve(container);
    };
  }, [delay, stagger, duration, type, hasAnimated]);

  return {
    ref: containerRef,
    hasAnimated,
  };
}

