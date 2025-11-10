"use client";

import { useState } from "react";
import { PanelRightOpen, PanelRightClose } from "lucide-react";
import { OverviewEditor, NotionEditor } from "../composers/editor";
import { useProjectStore } from "../../lib/stores";
import styles from "./OverviewTab.module.scss";

export function OverviewTab() {
  const [showMetadata, setShowMetadata] = useState(true);
  const { getCurrentProject } = useProjectStore();
  const currentProject = getCurrentProject();

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
              onChange={(content) => {
                // TODO: Save content to project store
                console.log("Content changed:", content);
              }}
              editable={true}
            />
          </div>
        )}
      </div>
    </div>
  );
}
