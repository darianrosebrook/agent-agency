"use client";

import { NotionEditor } from "./NotionEditor";
import { MetadataPanel, type MetadataField } from "./MetadataPanel";
import styles from "./OverviewEditor.module.scss";

interface OverviewEditorProps {
  content?: string;
  metadata?: {
    title: string;
    fields?: MetadataField[];
  };
  onContentChange?: (content: string) => void;
  onMetadataChange?: (metadata: { title: string; fields?: MetadataField[] }) => void;
  onMetadataClose?: () => void;
  className?: string;
}

export function OverviewEditor({
  content,
  metadata,
  onContentChange,
  onMetadataChange,
  onMetadataClose,
  className = "",
}: OverviewEditorProps) {
  return (
    <div className={`${styles.containerRoot} ${className}`} data-name="Container">
      <div className={styles.overviewEditor}>
        {/* Editor Container */}
        <div className={styles.editorContainer}>
          <div className={styles.editorInner}>
            <div className={styles.editorContentWrapper}>
              <NotionEditor
                content={content}
                placeholder="Type '/' for commands, or start writing your project overview..."
                onChange={onContentChange}
                editable={true}
              />
            </div>
          </div>
        </div>

        {/* Metadata Panel */}
        {metadata && (
          <MetadataPanel
            title={metadata.title}
            fields={metadata.fields}
            onClose={onMetadataClose}
          />
        )}
      </div>
    </div>
  );
}

