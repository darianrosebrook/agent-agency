"use client";

import { EditorToolbar } from "../../compounds/editor/EditorToolbar";
import { MarkdownEditor, type MarkdownContent } from "./MarkdownEditor";
import { MetadataPanel, type MetadataField } from "./MetadataPanel";
import styles from "./OverviewEditor.module.scss";

interface OverviewEditorProps {
  content?: MarkdownContent;
  metadata?: {
    title: string;
    fields?: MetadataField[];
  };
  onContentChange?: (content: MarkdownContent) => void;
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
            <EditorToolbar />
            <div className={styles.editorContentWrapper}>
              <MarkdownEditor content={content} />
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

