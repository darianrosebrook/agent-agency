"use client";

import type { ReactNode } from "react";
import { cn } from "../../primitives/utils";
import { MetadataRow } from "../../compounds/MetadataRow";
import { EditorIcon } from "../../primitives/editor/EditorIcon";
import svgPaths from "../../../imports/svg-8d8l4g1ml9";
import styles from "./MetadataPanel.module.scss";

export interface MetadataField {
  label: string;
  value: ReactNode;
  icon?: ReactNode;
  style?: React.CSSProperties;
}

interface MetadataPanelProps {
  title: string;
  fields?: MetadataField[];
  onClose?: () => void;
  className?: string;
}

function renderCloseIcon() {
  return (
    <EditorIcon>
      <path
        d={svgPaths.p139f1200}
        id="Vector"
        stroke="var(--stroke-0, #888888)"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.33286"
      />
      <path
        d={svgPaths.pc092300}
        id="Vector_2"
        stroke="var(--stroke-0, #888888)"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.33286"
      />
    </EditorIcon>
  );
}

export function MetadataPanel({
  title,
  fields = [],
  onClose,
  className = "",
}: MetadataPanelProps) {
  return (
    <div className={cn(styles.metadataPanel, className)} data-name="MetadataPanel">
      <div className={styles.metadataPanelInner}>
        {/* Header */}
        <div className={styles.metadataPanelHeader}>
          <div aria-hidden="true" className={styles.metadataPanelHeaderBorder} />
          <div className={styles.metadataPanelTitleContainer}>
            <div className={styles.metadataPanelTitleWrapper}>
              <div className={styles.metadataPanelTitleRow}>
                <div className={styles.metadataPanelTitle}>
                  <div className={styles.metadataPanelTitleText}>
                    <p className={styles.metadataPanelTitleParagraph}>{title}</p>
                  </div>
                </div>
                {onClose && (
                  <div
                    className={styles.metadataPanelCloseButton}
                    onClick={onClose}
                    role="button"
                    tabIndex={0}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        onClose();
                      }
                    }}
                    data-name="Button"
                  >
                    <div className={styles.metadataPanelCloseButtonInner}>
                      {renderCloseIcon()}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Fields */}
        <div className={styles.metadataPanelFields}>
          {fields.map((field, index) => (
            <div
              key={index}
              className={styles.metadataFieldGroup}
              style={field.style}
              data-name="Container"
            >
              <div className={styles.metadataFieldLabel}>
                {field.icon && <div className={styles.metadataFieldIcon}>{field.icon}</div>}
                <div className={styles.metadataFieldLabelText}>
                  <p className={styles.metadataFieldLabelParagraph}>{field.label}</p>
                </div>
              </div>
              <div className={styles.metadataFieldValue}>{field.value}</div>
            </div>
          ))}
        </div>
      </div>
      <div aria-hidden="true" className={styles.metadataPanelBorder} />
    </div>
  );
}

