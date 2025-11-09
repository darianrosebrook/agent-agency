/**
 * Utility function for combining class names
 * Works with SCSS modules and conditional classes
 * 
 * @author @darianrosebrook
 */

import { clsx, type ClassValue } from "clsx";

/**
 * Combines class names, handling conditional classes and SCSS module classes
 * @param inputs - Class names, SCSS module objects, or conditional class objects
 * @returns Combined class string
 */
export function cn(...inputs: ClassValue[]) {
  return clsx(inputs);
}
