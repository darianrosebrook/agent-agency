"use client";

import { useState, useEffect, useRef } from "react";

/**
 * Hook to animate a numeric value from one number to another
 * @param targetValue - The target value to animate to
 * @param duration - Animation duration in milliseconds (default: 800)
 * @param easing - Easing function type: 'easeOut' | 'easeIn' | 'linear' (default: 'easeOut')
 * @returns The current animated value
 */
export function useAnimatedValue(
  targetValue: number,
  duration: number = 800,
  easing: "easeOut" | "easeIn" | "linear" = "easeOut"
): number {
  const [animatedValue, setAnimatedValue] = useState(targetValue);
  const animationFrameRef = useRef<number | null>(null);
  const startTimeRef = useRef<number | null>(null);
  const startValueRef = useRef<number>(targetValue);
  const animatedValueRef = useRef<number>(targetValue);

  // Update ref whenever animatedValue changes
  useEffect(() => {
    animatedValueRef.current = animatedValue;
  }, [animatedValue]);

  // Initialize on mount
  useEffect(() => {
    setAnimatedValue(targetValue);
    animatedValueRef.current = targetValue;
    startValueRef.current = targetValue;
  }, []);

  // Animate when targetValue changes
  useEffect(() => {
    // Cancel any ongoing animation
    if (animationFrameRef.current !== null) {
      cancelAnimationFrame(animationFrameRef.current);
    }

    // Use current animated value as start
    startValueRef.current = animatedValueRef.current;
    const startValue = startValueRef.current;
    const target = targetValue;
    startTimeRef.current = null;

    // Only animate if there's a difference
    if (startValue === target) {
      setAnimatedValue(target);
      return;
    }

    const animate = (timestamp: number) => {
      if (startTimeRef.current === null) {
        startTimeRef.current = timestamp;
      }

      const elapsed = timestamp - startTimeRef.current;
      const progress = Math.min(elapsed / duration, 1);

      // Apply easing function
      let easedProgress: number;
      switch (easing) {
        case "easeOut":
          easedProgress = 1 - Math.pow(1 - progress, 3);
          break;
        case "easeIn":
          easedProgress = Math.pow(progress, 3);
          break;
        case "linear":
        default:
          easedProgress = progress;
      }

      const currentValue = Math.round(
        startValue + (target - startValue) * easedProgress
      );

      setAnimatedValue(currentValue);

      if (progress < 1) {
        animationFrameRef.current = requestAnimationFrame(animate);
      } else {
        setAnimatedValue(target);
        animationFrameRef.current = null;
      }
    };

    animationFrameRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationFrameRef.current !== null) {
        cancelAnimationFrame(animationFrameRef.current);
        animationFrameRef.current = null;
      }
    };
  }, [targetValue, duration, easing]);

  return animatedValue;
}

