/**
 * Loading Spinner Component
 * 
 * Reusable loading spinner with different sizes and variants.
 * 
 * @author @darianrosebrook
 */

"use client";

import { Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

interface LoadingSpinnerProps {
  size?: "sm" | "md" | "lg";
  className?: string;
  text?: string;
}

const sizeClasses = {
  sm: "w-4 h-4",
  md: "w-6 h-6",
  lg: "w-8 h-8",
};

export function LoadingSpinner({
  size = "md",
  className,
  text,
}: LoadingSpinnerProps) {
  return (
    <div className={cn("flex items-center gap-2", className)}>
      <Loader2
        className={cn(
          "animate-spin text-gray-400",
          sizeClasses[size]
        )}
      />
      {text && (
        <span className="text-sm text-gray-400">{text}</span>
      )}
    </div>
  );
}

/**
 * Full page loading spinner
 */
export function PageLoading({ text = "Loading..." }: { text?: string }) {
  return (
    <div className="flex items-center justify-center min-h-screen">
      <div className="text-center">
        <LoadingSpinner size="lg" />
        {text && (
          <p className="mt-4 text-gray-400 text-sm">{text}</p>
        )}
      </div>
    </div>
  );
}

/**
 * Button loading spinner (for async actions)
 */
export function ButtonLoading() {
  return <LoadingSpinner size="sm" />;
}

