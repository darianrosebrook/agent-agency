/**
 * Chat List Skeleton Loader
 * 
 * Displays skeleton loading state for chat list items.
 * 
 * @author @darianrosebrook
 */

"use client";

import { Skeleton } from "../ui/skeleton";
import { MessageSquare } from "lucide-react";

export function ChatListSkeleton({ count = 5 }: { count?: number }) {
  return (
    <div className="space-y-1 p-2">
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className="flex items-center gap-2 px-3 py-2 rounded-lg"
        >
          <MessageSquare className="w-3.5 h-3.5 shrink-0 text-gray-700" />
          <Skeleton className="h-4 flex-1 bg-gray-800" />
        </div>
      ))}
    </div>
  );
}

