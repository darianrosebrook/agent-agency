/**
 * Version history hook for tracking document versions
 *
 * @author @darianrosebrook
 */

import { useState, useCallback, useRef, useEffect } from "react";
import type { ProjectOverviewVersion } from "../lib/api/projects";

const MAX_LOCAL_VERSIONS = 50; // Keep last 50 versions in localStorage

/**
 * Version history manager
 */
export function useVersionHistory(projectId: string | null) {
  const [versions, setVersions] = useState<ProjectOverviewVersion[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const lastSavedContentRef = useRef<string>("");
  const versionCounterRef = useRef<number>(0);

  // Load versions from localStorage on mount
  useEffect(() => {
    if (!projectId) {
      setVersions([]);
      return;
    }

    try {
      const stored = localStorage.getItem(
        `project-overview-versions-${projectId}`
      );
      if (stored) {
        const parsed = JSON.parse(stored) as ProjectOverviewVersion[];
        setVersions(parsed);
        // Set counter to highest version number
        if (parsed.length > 0) {
          const maxVersion = Math.max(
            ...parsed.map((v) => {
              const match = v.version_id.match(/v(\d+)/);
              return match ? parseInt(match[1], 10) : 0;
            })
          );
          versionCounterRef.current = maxVersion;
        }
      }
    } catch (err) {
      console.error("Failed to load version history from localStorage:", err);
    }
  }, [projectId]);

  // Save versions to localStorage whenever they change
  useEffect(() => {
    if (!projectId || versions.length === 0) {
      return;
    }

    try {
      // Keep only last MAX_LOCAL_VERSIONS
      const versionsToStore = versions.slice(-MAX_LOCAL_VERSIONS);
      localStorage.setItem(
        `project-overview-versions-${projectId}`,
        JSON.stringify(versionsToStore)
      );
    } catch (err) {
      console.error("Failed to save version history to localStorage:", err);
    }
  }, [projectId, versions]);

  /**
   * Create a new version snapshot
   */
  const createVersion = useCallback(
    (content: string, changeSummary?: string) => {
      if (!projectId) {
        return;
      }

      // Don't create version if content hasn't changed
      if (content === lastSavedContentRef.current) {
        return;
      }

      versionCounterRef.current += 1;
      const versionId = `v${versionCounterRef.current}`;
      const now = new Date().toISOString();

      const newVersion: ProjectOverviewVersion = {
        version_id: versionId,
        project_id: projectId,
        overview: content,
        created_at: now,
        created_by: null, // TODO: Get current user ID
        change_summary: changeSummary ?? null,
      };

      setVersions((prev) => [...prev, newVersion].slice(-MAX_LOCAL_VERSIONS));
      lastSavedContentRef.current = content;
    },
    [projectId]
  );

  /**
   * Get a specific version by ID
   */
  const getVersion = useCallback(
    (versionId: string): ProjectOverviewVersion | undefined => {
      return versions.find((v) => v.version_id === versionId);
    },
    [versions]
  );

  /**
   * Restore content from a version
   */
  const restoreVersion = useCallback(
    (versionId: string): string | null => {
      const version = getVersion(versionId);
      if (version) {
        lastSavedContentRef.current = version.overview;
        return version.overview;
      }
      return null;
    },
    [getVersion]
  );

  /**
   * Clear version history
   */
  const clearHistory = useCallback(() => {
    if (!projectId) {
      return;
    }

    setVersions([]);
    versionCounterRef.current = 0;
    localStorage.removeItem(`project-overview-versions-${projectId}`);
  }, [projectId]);

  return {
    versions,
    isLoading,
    createVersion,
    getVersion,
    restoreVersion,
    clearHistory,
  };
}

