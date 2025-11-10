/**
 * Project List Skeleton Loader
 * 
 * Displays skeleton loading state for project list items.
 * 
 * @author @darianrosebrook
 */

"use client";

import { Skeleton } from "../ui/skeleton";
import { Folder } from "lucide-react";

export function ProjectListSkeleton({ count = 6 }: { count?: number }) {
  return (
    <div className="space-y-2">
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className="flex items-center gap-3 p-4 bg-[#1a1a1a] border border-gray-800 rounded-lg"
        >
          <Folder className="w-5 h-5 shrink-0 text-gray-700" />
          <div className="flex-1 space-y-2">
            <Skeleton className="h-5 w-3/4 bg-gray-800" />
            <Skeleton className="h-3 w-1/2 bg-gray-800" />
          </div>
          <Skeleton className="h-4 w-20 bg-gray-800" />
        </div>
      ))}
    </div>
  );
}

