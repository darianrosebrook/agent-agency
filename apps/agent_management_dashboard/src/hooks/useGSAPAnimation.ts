"use client";

import React, { useEffect, useRef, useState } from "react";
import { gsap } from "gsap";

/**
 * Hook to animate a numeric value using GSAP with React state
 * @param targetValue - The target value to animate to
 * @param duration - Animation duration in seconds (default: 0.8)
 * @param ease - GSAP easing function (default: "power2.out")
 * @returns The current animated value (for display)
 */
export function useGSAPNumberAnimation(
  targetValue: number,
  duration: number = 0.8,
  ease: string = "power2.out"
): number {
  const [animatedValue, setAnimatedValue] = useState(targetValue);
  const tweenRef = useRef<gsap.core.Tween | null>(null);
  const objRef = useRef({ value: targetValue });

  useEffect(() => {
    // Kill any existing animation
    if (tweenRef.current) {
      tweenRef.current.kill();
    }

    // Set initial value
    objRef.current.value = animatedValue;

    // Create new animation
    tweenRef.current = gsap.to(objRef.current, {
      value: targetValue,
      duration,
      ease,
      onUpdate: () => {
        setAnimatedValue(Math.round(objRef.current.value));
      },
    });

    return () => {
      if (tweenRef.current) {
        tweenRef.current.kill();
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [targetValue, duration, ease]);

  return animatedValue;
}

/**
 * Hook to animate SVG path stroke-dasharray for drawing animations
 * @param pathRef - Ref to the SVG path element
 * @param duration - Animation duration in seconds (default: 1.5)
 * @param delay - Delay before animation starts in seconds (default: 0)
 */
export function useGSAPPathDraw(
  pathRef: React.RefObject<SVGPathElement>,
  duration: number = 1.5,
  delay: number = 0
) {
  useEffect(() => {
    if (!pathRef.current) return;

    const path = pathRef.current;
    const length = path.getTotalLength();

    // Set up initial state
    gsap.set(path, {
      strokeDasharray: length,
      strokeDashoffset: length,
    });

    // Animate drawing
    gsap.to(path, {
      strokeDashoffset: 0,
      duration,
      delay,
      ease: "power2.out",
    });
  }, [pathRef, duration, delay]);
}

/**
 * Hook to animate SVG elements with stagger for sequential animations
 * @param selector - CSS selector for SVG elements to animate
 * @param animationProps - GSAP animation properties
 * @param stagger - Stagger delay between elements in seconds (default: 0.05)
 * @param delay - Delay before animation starts in seconds (default: 0)
 */
export function useGSAPStagger(
  selector: string,
  animationProps: gsap.TweenVars,
  stagger: number = 0.05,
  delay: number = 0
) {
  useEffect(() => {
    const elements = document.querySelectorAll(selector);
    if (elements.length === 0) return;

    gsap.fromTo(
      elements,
      { opacity: 0, scale: 0.8 },
      {
        ...animationProps,
        opacity: 1,
        scale: 1,
        stagger,
        delay,
        ease: "back.out(1.7)",
      }
    );
  }, [selector, stagger, delay, animationProps]);
}

/**
 * Hook to animate color transitions for SVG elements
 * @param elementRef - Ref to the SVG element
 * @param targetColor - Target color to animate to
 * @param duration - Animation duration in seconds (default: 0.5)
 */
export function useGSAPColorTransition(
  elementRef: React.RefObject<SVGElement>,
  targetColor: string,
  duration: number = 0.5
) {
  useEffect(() => {
    if (!elementRef.current) return;

    gsap.to(elementRef.current, {
      fill: targetColor,
      duration,
      ease: "power2.out",
    });
  }, [elementRef, targetColor, duration]);
}

