/**
 * Progress Indicator Component
 * 
 * Displays progress for long-running operations with percentage and optional message.
 * 
 * @author @darianrosebrook
 */

"use client";

import React from "react";
import { cn } from "../ui/utils";
import { Loader2 } from "lucide-react";

interface ProgressIndicatorProps {
  progress?: number; // 0-100
  message?: string;
  showPercentage?: boolean;
  className?: string;
}

export function ProgressIndicator({
  progress,
  message,
  showPercentage = true,
  className,
}: ProgressIndicatorProps) {
  const displayProgress = Math.min(100, Math.max(0, progress ?? 0));

  return (
    <div className={cn("flex flex-col items-center gap-3", className)}>
      {progress !== undefined ? (
        <>
          {/* Progress Bar */}
          <div className="w-full max-w-xs bg-gray-800 rounded-full h-2 overflow-hidden">
            <div
              className="h-full bg-blue-600 transition-all duration-300 ease-out"
              style={{ width: `${displayProgress}%` }}
            />
          </div>
          
          {/* Percentage and Message */}
          <div className="text-center">
            {showPercentage && (
              <div className="text-sm text-gray-300 mb-1">
                {Math.round(displayProgress)}%
              </div>
            )}
            {message && (
              <div className="text-xs text-gray-500">{message}</div>
            )}
          </div>
        </>
      ) : (
        <>
          {/* Indeterminate Progress */}
          <Loader2 className="w-6 h-6 animate-spin text-blue-600" />
          {message && (
            <div className="text-sm text-gray-400 text-center">{message}</div>
          )}
        </>
      )}
    </div>
  );
}

