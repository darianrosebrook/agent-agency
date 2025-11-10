import { clsx, type ClassValue } from "clsx";

/**
 * Merge CSS classes for SCSS modules
 * Uses clsx to conditionally join classNames together
 */
export function cn(...inputs: ClassValue[]) {
  return clsx(inputs);
}
