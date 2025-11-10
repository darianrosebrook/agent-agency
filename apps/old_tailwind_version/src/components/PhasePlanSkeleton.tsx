import { Skeleton } from "./ui/skeleton";

export function PhasePlanSkeleton() {
  return (
    <div className="w-full animate-pulse">
      {/* Header skeleton */}
      <div className="mb-6">
        <Skeleton className="h-8 w-48 mb-2 bg-gray-800" />
        <Skeleton className="h-4 w-full max-w-xl mb-4 bg-gray-800" />
        <div className="flex items-center gap-2">
          <Skeleton className="h-10 w-32 bg-gray-800" />
          <Skeleton className="h-10 w-40 bg-gray-800" />
        </div>
      </div>

      {/* Phase cards skeleton */}
      {[1, 2].map((phase) => (
        <div
          key={phase}
          className="mb-6 bg-[#1a1a1a] rounded-xl border border-gray-800 overflow-hidden"
        >
          {/* Phase header */}
          <div className="px-6 py-5 border-b border-gray-800">
            <div className="flex items-center gap-3 mb-2">
              <Skeleton className="h-6 w-40 bg-gray-800" />
              <Skeleton className="h-6 w-20 rounded-full bg-gray-800" />
            </div>
            <Skeleton className="h-4 w-full max-w-2xl bg-gray-800" />
          </div>

          {/* Task items skeleton */}
          <div className="divide-y divide-gray-800">
            {[1, 2, 3].map((task) => (
              <div key={task} className="px-6 py-4">
                <div className="flex items-center gap-3">
                  <Skeleton className="h-5 w-5 rounded-full bg-gray-800" />
                  <Skeleton className="h-5 w-64 bg-gray-800" />
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
