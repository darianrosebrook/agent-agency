"use client";

import { useState, useCallback } from "react";
import { PanelRightOpen, PanelRightClose } from "lucide-react";
import { OverviewEditor, NotionEditor } from "../composers/editor";
import { useProjectStore } from "../../lib/stores";
import { useProjectContext } from "./ProjectContext";
import { updateProjectOverview } from "../../lib/api/projects";
import styles from "./OverviewTab.module.scss";

export function OverviewTab() {
  const [showMetadata, setShowMetadata] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const { getCurrentProject } = useProjectStore();
  const { currentProjectId } = useProjectContext();
  const currentProject = getCurrentProject();

  const handleContentChange = useCallback(async (content: string) => {
    if (!currentProjectId) {
      console.warn("Cannot save overview: no project ID");
      return;
    }

    setIsSaving(true);
    try {
      // Extract text content from HTML
      const textContent = content.replace(/<[^>]*>/g, '').trim();
      await updateProjectOverview(currentProjectId, textContent);
    } catch (error) {
      console.error("Failed to save project overview:", error);
      alert(`Failed to save overview: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsSaving(false);
    }
  }, [currentProjectId]);

  return (
    <div className={styles.overviewTab}>
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

      <div className={styles.editorContent}>
        {showMetadata ? (
          <OverviewEditor
            metadata={{
              title: "UI / Components (light)",
              fields: [
                {
                  label: "Created At",
                  value: "February 15, 2020 6:08 AM",
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
              content={currentProject?.description ? `<p>${currentProject.description}</p>` : undefined}
              placeholder="Type '/' for commands, or start writing your project overview..."
              onChange={handleContentChange}
              editable={true}
            />
            {isSaving && (
              <div className={styles.savingIndicator}>Saving...</div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
