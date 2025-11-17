"use client";

/**
 * Project Overview Tab
 *
 * Rich text editor for project overview with:
 * - Debounced autosave (saves 2 seconds after user stops typing)
 * - Version history tracking (local storage + future API integration)
 * - Live content updates (polls for changes from other users)
 * - Version rollback UI
 *
 * @author @darianrosebrook
 */

import { useState, useCallback, useRef, useEffect } from "react";
import { PanelRightOpen, PanelRightClose, History, RotateCcw } from "lucide-react";
import { OverviewEditor, NotionEditor } from "../composers/editor";
import { useProjectStore } from "../../lib/stores";
import { updateProjectOverview, restoreProjectOverviewVersion } from "../../lib/api/projects";
import { useDebounce } from "../../hooks/useDebounce";
import { useVersionHistory } from "../../hooks/useVersionHistory";
import { useLiveUpdates } from "../../hooks/useLiveUpdates";
import { Button } from "../primitives/button";
import styles from "./OverviewTab.module.scss";

export function OverviewTab() {
  const [showMetadata, setShowMetadata] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState<Error | null>(null);
  const [showVersionHistory, setShowVersionHistory] = useState(false);
  const [editorContent, setEditorContent] = useState<string>("");
  const [lastSavedContent, setLastSavedContent] = useState<string>("");
  const editorContentRef = useRef<string>("");
  const isRestoringRef = useRef<boolean>(false);
  
  // Use only useProjectStore (removed dual state management with ProjectContext)
  const { getCurrentProject, currentProjectId } = useProjectStore();
  const currentProject = getCurrentProject();

  // Debounce content changes for autosave (2 second delay)
  const debouncedContent = useDebounce(editorContent, 2000);

  // Version history management
  const {
    versions,
    createVersion,
    restoreVersion,
    clearHistory,
  } = useVersionHistory(currentProjectId);

  // Handle content changes from editor
  const handleContentChange = useCallback((content: string) => {
    editorContentRef.current = content;
    setEditorContent(content);
    setSaveError(null);
    lastUserEditRef.current = Date.now(); // Track when user last edited
  }, []);

  // Autosave when debounced content changes
  useEffect(() => {
    if (!currentProjectId || !debouncedContent) {
      return;
    }

    // Don't save if we're restoring a version
    if (isRestoringRef.current) {
      isRestoringRef.current = false;
      return;
    }

    // Don't save if content hasn't changed
    if (debouncedContent === lastSavedContent) {
      return;
    }

    const saveContent = async () => {
      setIsSaving(true);
      setSaveError(null);

      try {
        // Save to API
        await updateProjectOverview(currentProjectId, debouncedContent);
        
        // Create version snapshot after successful save
        createVersion(debouncedContent);
        setLastSavedContent(debouncedContent);
      } catch (error) {
        console.error("Failed to save project overview:", error);
        const err = error instanceof Error ? error : new Error("Failed to save overview");
        setSaveError(err);
      } finally {
        setIsSaving(false);
      }
    };

    saveContent();
  }, [currentProjectId, debouncedContent, lastSavedContent, createVersion]);

  // Track last user edit time to avoid overwriting recent edits
  const lastUserEditRef = useRef<number>(0);
  const DEBOUNCE_DELAY = 2000; // Same as autosave delay

  // Live updates - poll for changes from other users
  const { hasUpdates, lastUpdated, acknowledgeUpdates } = useLiveUpdates({
    projectId: currentProjectId,
    enabled: true,
    pollInterval: 5000, // Poll every 5 seconds
    onUpdate: useCallback((overview: string | null) => {
      // Don't update if user has edited recently (within debounce window)
      const timeSinceLastEdit = Date.now() - lastUserEditRef.current;
      if (timeSinceLastEdit < DEBOUNCE_DELAY + 1000) {
        return; // User is still editing, don't overwrite
      }

      const newContent = overview ? `<p>${overview}</p>` : "";

      // Only update if content is different and not currently saving
      if (newContent !== editorContentRef.current && !isSaving) {
        editorContentRef.current = newContent;
        setEditorContent(newContent);
        setLastSavedContent(overview ?? "");
        acknowledgeUpdates(); // Clear the update indicator after applying
      }
    }, [isSaving]),
  });

  // Initialize content from project
  useEffect(() => {
    if (currentProject?.description) {
      const initialContent = `<p>${currentProject.description}</p>`;
      editorContentRef.current = initialContent;
      setEditorContent(initialContent);
      setLastSavedContent(currentProject.description);
    }
  }, [currentProject?.description]);

  // Handle version restore
  const handleRestoreVersion = useCallback(
    async (versionId: string) => {
      if (!currentProjectId) {
        return;
      }

      const restoredContent = restoreVersion(versionId);
      if (!restoredContent) {
        alert("Failed to restore version");
        return;
      }

      try {
        isRestoringRef.current = true;
        
        // Update editor content
        const htmlContent = `<p>${restoredContent}</p>`;
        editorContentRef.current = htmlContent;
        setEditorContent(htmlContent);
        setLastSavedContent(restoredContent);

        // Save to API
        await updateProjectOverview(currentProjectId, restoredContent);
        
        // Create new version snapshot
        createVersion(restoredContent, `Restored from version ${versionId}`);
        
        setShowVersionHistory(false);
      } catch (error) {
        console.error("Failed to restore version:", error);
        alert(`Failed to restore version: ${error instanceof Error ? error.message : "Unknown error"}`);
      }
    },
    [currentProjectId, restoreVersion, createVersion]
  );

  // Format date for display
  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  };

  return (
    <div className={styles.overviewTab}>
      {/* Header Controls */}
      <div className={styles.headerControls}>
        <button
          onClick={() => setShowMetadata(!showMetadata)}
          className={styles.toggleButton}
          title={showMetadata ? "Hide metadata panel" : "Show metadata panel"}
          type="button"
        >
          {showMetadata ? (
            <PanelRightClose className={styles.toggleIcon} />
          ) : (
            <PanelRightOpen className={styles.toggleIcon} />
          )}
        </button>

        {/* Version History Button */}
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setShowVersionHistory(!showVersionHistory)}
          className={styles.versionHistoryButton}
          title="View version history"
        >
          <History className={styles.versionHistoryIcon} />
          {versions.length > 0 && (
            <span className={styles.versionCount}>{versions.length}</span>
          )}
        </Button>

        {/* Save Status Indicator */}
        <div className={styles.saveStatus}>
          {isSaving && (
            <span className={styles.savingIndicator}>Saving...</span>
          )}
          {saveError && (
            <span className={styles.saveError} title={saveError.message}>
              Save failed
            </span>
          )}
          {!isSaving && !saveError && lastSavedContent === editorContent && (
            <span className={styles.savedIndicator}>Saved</span>
          )}
          {hasUpdates && (
            <span className={styles.updateIndicator} title={`Updated ${lastUpdated?.toLocaleTimeString()}`}>
              Updated
            </span>
          )}
        </div>
      </div>

      {/* Version History Panel */}
      {showVersionHistory && (
        <div className={styles.versionHistoryPanel}>
          <div className={styles.versionHistoryHeader}>
            <h3 className={styles.versionHistoryTitle}>Version History</h3>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowVersionHistory(false)}
              className={styles.closeButton}
            >
              ×
            </Button>
          </div>
          <div className={styles.versionHistoryList}>
            {versions.length === 0 ? (
              <div className={styles.noVersions}>No versions saved yet</div>
            ) : (
              versions
                .slice()
                .reverse()
                .map((version) => (
                  <div key={version.version_id} className={styles.versionItem}>
                    <div className={styles.versionInfo}>
                      <div className={styles.versionId}>{version.version_id}</div>
                      <div className={styles.versionDate}>
                        {formatDate(version.created_at)}
                      </div>
                      {version.change_summary && (
                        <div className={styles.versionSummary}>
                          {version.change_summary}
                        </div>
                      )}
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleRestoreVersion(version.version_id)}
                      className={styles.restoreButton}
                      title="Restore this version"
                    >
                      <RotateCcw className={styles.restoreIcon} />
                      Restore
                    </Button>
                  </div>
                ))
            )}
          </div>
        </div>
      )}

      {/* Editor Content */}
      <div className={styles.editorContent}>
        {showMetadata ? (
          <OverviewEditor
            metadata={{
              title: "UI / Components (light)",
              fields: [
                {
                  label: "Created At",
                  value: currentProject?.createdAt
                    ? new Date(currentProject.createdAt).toLocaleString()
                    : "February 15, 2020 6:08 AM",
                },
                {
                  label: "Created By",
                  value: "Darian Rosebrook",
                },
              ],
            }}
            onMetadataClose={() => setShowMetadata(false)}
          />
        ) : (
          <div className={styles.editorOnly}>
            <NotionEditor
              content={editorContent || undefined}
              placeholder="Type '/' for commands, or start writing your project overview..."
              onChange={handleContentChange}
              editable={true}
            />
          </div>
        )}
      </div>
    </div>
  );
}
