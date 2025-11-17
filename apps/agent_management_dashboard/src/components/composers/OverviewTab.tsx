"use client";

import { PanelRightClose, PanelRightOpen } from "lucide-react";
import { useState } from "react";
import svgPaths from "../../imports/svg-8d8l4g1ml9";
import { useProjectStore } from "../../lib/stores";
import styles from "./OverviewTab.module.scss";
import { OverviewEditor } from "./editor";

export function OverviewTab() {
  const [showMetadata, setShowMetadata] = useState(true);
  const { getCurrentProject } = useProjectStore();
  const currentProject = getCurrentProject();

  return (
    <div className={styles.overviewTab}>
      {/* Toggle Button */}
      <button
        onClick={() => setShowMetadata(!showMetadata)}
        className={styles.toggleButton}
        title={showMetadata ? "Hide metadata panel" : "Show metadata panel"}
      >
        {showMetadata ? (
          <PanelRightClose className={styles.toggleIcon} />
        ) : (
          <PanelRightOpen className={styles.toggleIcon} />
        )}
      </button>

      {/* Editor Content */}
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
          <EditorOnly description={currentProject?.description} />
        )}
      </div>
    </div>
  );
}

// Editor without metadata panel
function EditorOnly({ description }: { description?: string }) {
  return (
    <div className={styles.editorOnly}>
      <div className={styles.editorOnlyInner}>
        {/* Editor Toolbar */}
        <div className={styles.editorToolbar}>
          <div aria-hidden="true" className={styles.editorToolbarBorder} />
          <EditorToolbar />
        </div>

        {/* Editor Content Area */}
        <div className={styles.editorContentArea}>
          <div className={styles.editorContentInner}>
            <MarkdownEditorPlaceholder description={description} />
          </div>
        </div>
      </div>
    </div>
  );
}

function EditorToolbar() {
  return (
    <div className={styles.toolbarContainer}>
      {/* Bold Button */}
      <button className={`${styles.toolbarButton} ${styles.toolbarButtonBold}`}>
        <div className={styles.toolbarDropdownIcon}>
          <svg
            className={styles.svgIcon}
            fill="none"
            preserveAspectRatio="none"
            viewBox="0 0 16 16"
          >
            <path
              d={svgPaths.p1b11cb00}
              stroke="#888888"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="1.33286"
            />
            <path
              d={svgPaths.p8cc4400}
              stroke="#888888"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="1.33286"
            />
          </svg>
        </div>
      </button>

      {/* Italic Button */}
      <button
        className={`${styles.toolbarButton} ${styles.toolbarButtonItalic}`}
      >
        <div className={styles.toolbarDropdownIcon}>
          <svg
            className={styles.svgIcon}
            fill="none"
            preserveAspectRatio="none"
            viewBox="0 0 16 16"
          >
            <path
              d={svgPaths.p271f800}
              stroke="#888888"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="1.33286"
            />
            <path
              d={svgPaths.p7307940}
              stroke="#888888"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="1.33286"
            />
          </svg>
        </div>
      </button>

      {/* Divider */}
      <div className={styles.toolbarDivider} />

      {/* Text Style Dropdown */}
      <button className={styles.toolbarDropdown}>
        <span className={styles.toolbarDropdownText}>Text</span>
        <div className={styles.toolbarDropdownIcon}>
          <svg
            className={styles.svgIcon}
            fill="none"
            preserveAspectRatio="none"
            viewBox="0 0 16 16"
          >
            <path
              d={svgPaths.p10a02b40}
              stroke="#717182"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="1.33286"
              opacity="0.5"
            />
          </svg>
        </div>
      </button>

      {/* More toolbar buttons would go here */}
      <div className={styles.toolbarHint}>Rich text editor toolbar</div>
    </div>
  );
}

function MarkdownEditorPlaceholder({ description }: { description?: string }) {
  if (description) {
    return (
      <div className={styles.editorPlaceholder}>
        <div className={styles.editorPlaceholderContent}>
          <p className={styles.editorPlaceholderText}>{description}</p>
        </div>

        {/* Edit hint */}
        <div className={styles.editorPlaceholderHint}>
          Click to start editing...
        </div>
      </div>
    );
  }

  return (
    <div className={styles.editorPlaceholder}>
      {/* Heading */}
      <div className={styles.editorPlaceholderHeading}>
        <h1 className={styles.editorPlaceholderHeadingText}>Project Vision</h1>
      </div>

      {/* Paragraph */}
      <div className={styles.editorPlaceholderSection}>
        <p className={styles.editorPlaceholderText}>
          Start writing your project vision here. This is where you define the
          goals, objectives, and overall direction for your project.
        </p>
      </div>

      {/* Another section */}
      <div className={styles.editorPlaceholderSection}>
        <h2 className={styles.editorPlaceholderSectionHeading}>
          Key Objectives
        </h2>
        <p className={styles.editorPlaceholderText}>
          Define what success looks like for this project. What are the main
          deliverables and milestones?
        </p>
      </div>

      {/* Placeholder for more content */}
      <div className={styles.editorPlaceholderHint}>
        Click to start editing...
      </div>
    </div>
  );
}
