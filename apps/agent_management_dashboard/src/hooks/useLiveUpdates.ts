/**
 * Live updates hook for polling project overview changes
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from "react";
import { getProjectHandler } from "../lib/api/projects";

interface UseLiveUpdatesOptions {
  projectId: string | null;
  enabled?: boolean;
  pollInterval?: number; // milliseconds
  onUpdate?: (overview: string | null) => void;
}

/**
 * Poll for project overview updates
 */
export function useLiveUpdates({
  projectId,
  enabled = true,
  pollInterval = 5000, // 5 seconds default
  onUpdate,
}: UseLiveUpdatesOptions) {
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const [hasUpdates, setHasUpdates] = useState(false);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const lastOverviewRef = useRef<string | null>(null);

  // Store onUpdate in a ref to avoid recreating the effect
  const onUpdateRef = useRef(onUpdate);
  useEffect(() => {
    onUpdateRef.current = onUpdate;
  }, [onUpdate]);

  useEffect(() => {
    if (!enabled || !projectId) {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      setIsPolling(false);
      return;
    }

    setIsPolling(true);

    const poll = async () => {
      try {
        const project = await getProjectHandler(projectId);
        const currentOverview = project.overview ?? null;

        // Check if overview has changed
        if (currentOverview !== lastOverviewRef.current) {
          setHasUpdates(true);
          setLastUpdated(new Date());

          if (
            onUpdateRef.current &&
            currentOverview !== lastOverviewRef.current
          ) {
            onUpdateRef.current(currentOverview);
          }

          lastOverviewRef.current = currentOverview;
        }
      } catch (err) {
        console.error("Failed to poll for project updates:", err);
      }
    };

    // Initial poll
    poll();

    // Set up polling interval
    intervalRef.current = setInterval(poll, pollInterval);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      setIsPolling(false);
    };
  }, [projectId, enabled, pollInterval]);

  /**
   * Acknowledge updates (clear the hasUpdates flag)
   */
  const acknowledgeUpdates = () => {
    setHasUpdates(false);
  };

  return {
    isPolling,
    hasUpdates,
    lastUpdated,
    acknowledgeUpdates,
  };
}
