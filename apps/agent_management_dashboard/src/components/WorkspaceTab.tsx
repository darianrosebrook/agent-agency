"use client";

import { useState } from "react";
import svgPaths from "../imports/svg-jzcqnicw4t";
import { ChatSidebar } from "./ChatSidebar";
import WorkspacePanel from "../imports/WorkspacePanel";
import { cn } from "./primitives/utils";
import styles from "./WorkspaceTab.module.scss";

export function WorkspaceTab() {
  const [activeTab, setActiveTab] = useState<"context" | "chats">("context");
  const [selectedItem, setSelectedItem] = useState<string | null>(null);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);

  return (
    <div className={styles.workspaceTab}>
      <div className={styles.workspaceTabContent}>
        <div className={styles.workspaceContainer}>
          {/* Main Content */}
          <div className={styles.mainContent}>
            {/* Expand Button - shown when sidebar is collapsed */}
            {isSidebarCollapsed && (
              <button
                onClick={() => setIsSidebarCollapsed(false)}
                className={styles.expandButton}
              >
                <div className={styles.expandButtonIcon}>
                  <svg
                    className="block size-full"
                    fill="none"
                    preserveAspectRatio="none"
                    viewBox="0 0 16 16"
                  >
                    <path
                      d={svgPaths.p24b5a500}
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                  </svg>
                </div>
              </button>
            )}

            {/* Workspace Sidebar */}
            <div
              className={cn(
                styles.workspaceSidebar,
                isSidebarCollapsed
                  ? styles.workspaceSidebarCollapsed
                  : styles.workspaceSidebarExpanded
              )}
            >
              <div
                aria-hidden="true"
                className={styles.workspaceSidebarBorder}
              />

              {/* Header */}
              <div className={styles.sidebarHeader}>
                <div
                  aria-hidden="true"
                  className={styles.sidebarHeaderBorder}
                />
                <div className={styles.sidebarHeaderContent}>
                  <div className={styles.sidebarHeaderTop}>
                    <div className={styles.sidebarHeaderIcon}>
                      <svg
                        className="block size-full"
                        fill="none"
                        preserveAspectRatio="none"
                        viewBox="0 0 16 16"
                      >
                        <g clipPath="url(#clip0_16_2398)">
                          <path
                            d={svgPaths.p14b1d380}
                            stroke="#FECA57"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="1.33286"
                          />
                        </g>
                        <defs>
                          <clipPath id="clip0_16_2398">
                            <rect
                              fill="white"
                              height="15.9943"
                              width="15.9943"
                            />
                          </clipPath>
                        </defs>
                      </svg>
                    </div>
                    <p className={styles.sidebarHeaderTitle}>Workspace</p>
                    <button
                      onClick={() => setIsSidebarCollapsed(!isSidebarCollapsed)}
                      className={styles.sidebarCollapseButton}
                    >
                      <div className={styles.sidebarCollapseButtonIcon}>
                        <svg
                          className={styles.svgIcon}
                          fill="none"
                          preserveAspectRatio="none"
                          viewBox="0 0 16 16"
                        >
                          <path
                            d={svgPaths.pc477740}
                            stroke="#888888"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="1.33286"
                          />
                        </svg>
                      </div>
                    </button>
                  </div>
                </div>
              </div>

              {/* Tab List */}
              <div className={styles.tabList}>
                <div aria-hidden="true" className={styles.tabListBorder} />
                <div className={styles.tabListContent}>
                  <button
                    onClick={() => setActiveTab("context")}
                    className={cn(
                      styles.tabButton,
                      activeTab === "context"
                        ? styles.tabButtonActive
                        : styles.tabButtonInactive
                    )}
                  >
                    <p
                      className={cn(
                        styles.tabButtonText,
                        activeTab === "context"
                          ? styles.tabButtonTextActive
                          : styles.tabButtonTextInactive
                      )}
                    >
                      Context
                    </p>
                  </button>
                  <button
                    onClick={() => setActiveTab("chats")}
                    className={cn(
                      styles.tabButton,
                      activeTab === "chats"
                        ? styles.tabButtonActive
                        : styles.tabButtonInactive
                    )}
                  >
                    <p
                      className={cn(
                        styles.tabButtonText,
                        activeTab === "chats"
                          ? styles.tabButtonTextActive
                          : styles.tabButtonTextInactive
                      )}
                    >
                      Chats
                    </p>
                  </button>
                </div>
              </div>

              {/* Tab Content */}
              <div className={styles.tabContent}>
                {activeTab === "context" ? (
                  <ContextFileTree onSelect={setSelectedItem} />
                ) : (
                  <div className={styles.fullHeight}>
                    <ChatSidebar onSelect={setSelectedItem} />
                  </div>
                )}
              </div>
            </div>

            {/* Workspace Panel - appears when item is selected */}
            {selectedItem && (
              <div className={styles.workspacePanel}>
                <WorkspacePanel
                  title={selectedItem}
                  onClose={() => setSelectedItem(null)}
                />
              </div>
            )}

            {/* Bento Grid - reflows based on sidebar and panel visibility */}
            <div
              className={cn(
                styles.bentoGridContainer,
                selectedItem
                  ? styles.bentoGridContainerWithPanel
                  : styles.bentoGridContainerWithoutPanel
              )}
            >
              <BentoGrid hasPanel={!!selectedItem} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

interface ContextFileTreeProps {
  onSelect: (item: string) => void;
}

function ContextFileTree({ onSelect }: ContextFileTreeProps) {
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(
    new Set(["src", "public"])
  );

  const toggleFolder = (folderId: string) => {
    const newExpanded = new Set(expandedFolders);
    if (newExpanded.has(folderId)) {
      newExpanded.delete(folderId);
    } else {
      newExpanded.add(folderId);
    }
    setExpandedFolders(newExpanded);
  };

  return (
    <div className={styles.fileTree}>
      {/* src folder */}
      <div className={styles.folderContainer}>
        <button
          onClick={() => toggleFolder("src")}
          className={styles.folderButton}
        >
          <div className={styles.folderIcon}>
            <svg
              className="block size-full"
              fill="none"
              preserveAspectRatio="none"
              viewBox="0 0 16 16"
            >
              <path
                d={
                  expandedFolders.has("src")
                    ? svgPaths.p10a02b40
                    : svgPaths.p24b5a500
                }
                stroke="#D1D5DC"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
            </svg>
          </div>
          <div className={styles.folderIcon}>
            <svg
              className="block size-full"
              fill="none"
              preserveAspectRatio="none"
              viewBox="0 0 16 16"
            >
              <g clipPath="url(#clip0_15_2243)">
                <path
                  d={svgPaths.p14b1d380}
                  stroke="#FECA57"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="1.33286"
                />
              </g>
              <defs>
                <clipPath id="clip0_15_2243">
                  <rect fill="white" height="15.9943" width="15.9943" />
                </clipPath>
              </defs>
            </svg>
          </div>
          <p className={styles.folderName}>src</p>
        </button>
        {expandedFolders.has("src") && (
          <div className={styles.folderChildren}>
            <button
              onClick={() => onSelect("src/components")}
              className={styles.fileButton}
            >
              <div className={styles.fileIcon}>
                <svg
                  className="block size-full"
                  fill="none"
                  preserveAspectRatio="none"
                  viewBox="0 0 16 16"
                >
                  <path
                    d={svgPaths.p24b5a500}
                    stroke="#D1D5DC"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth="1.33286"
                  />
                </svg>
              </div>
              <div className={styles.fileIcon}>
                <svg
                  className="block size-full"
                  fill="none"
                  preserveAspectRatio="none"
                  viewBox="0 0 16 16"
                >
                  <g clipPath="url(#clip0_15_2228)">
                    <path
                      d={svgPaths.p8e3b480}
                      stroke="#FECA57"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                  </g>
                  <defs>
                    <clipPath id="clip0_15_2228">
                      <rect fill="white" height="15.9943" width="15.9943" />
                    </clipPath>
                  </defs>
                </svg>
              </div>
              <p className={styles.fileName}>components</p>
            </button>
            <button
              onClick={() => onSelect("src/utils")}
              className={styles.fileButton}
            >
              <div className={styles.fileIcon}>
                <svg
                  className="block size-full"
                  fill="none"
                  preserveAspectRatio="none"
                  viewBox="0 0 16 16"
                >
                  <path
                    d={svgPaths.p24b5a500}
                    stroke="#D1D5DC"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth="1.33286"
                  />
                </svg>
              </div>
              <div className={styles.fileIcon}>
                <svg
                  className="block size-full"
                  fill="none"
                  preserveAspectRatio="none"
                  viewBox="0 0 16 16"
                >
                  <g clipPath="url(#clip0_15_2228)">
                    <path
                      d={svgPaths.p8e3b480}
                      stroke="#FECA57"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                  </g>
                  <defs>
                    <clipPath id="clip0_15_2228">
                      <rect fill="white" height="15.9943" width="15.9943" />
                    </clipPath>
                  </defs>
                </svg>
              </div>
              <p className={styles.fileName}>utils</p>
            </button>
          </div>
        )}
      </div>

      {/* public folder */}
      <div className={styles.folderContainer}>
        <button
          onClick={() => toggleFolder("public")}
          className={styles.folderButton}
        >
          <div className={styles.folderIcon}>
            <svg
              className="block size-full"
              fill="none"
              preserveAspectRatio="none"
              viewBox="0 0 16 16"
            >
              <path
                d={
                  expandedFolders.has("public")
                    ? svgPaths.p10a02b40
                    : svgPaths.p24b5a500
                }
                stroke="#D1D5DC"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
            </svg>
          </div>
          <div className={styles.folderIcon}>
            <svg
              className="block size-full"
              fill="none"
              preserveAspectRatio="none"
              viewBox="0 0 16 16"
            >
              <g clipPath="url(#clip0_15_2243)">
                <path
                  d={svgPaths.p14b1d380}
                  stroke="#FECA57"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="1.33286"
                />
              </g>
              <defs>
                <clipPath id="clip0_15_2243">
                  <rect fill="white" height="15.9943" width="15.9943" />
                </clipPath>
              </defs>
            </svg>
          </div>
          <p className={styles.folderName}>public</p>
        </button>
        {expandedFolders.has("public") && (
          <div className={styles.folderChildren}>
            <button
              onClick={() => onSelect("public/index.html")}
              className={cn(styles.fileButton, styles.fileButtonNested)}
            >
              <div className={styles.fileIcon}>
                <svg
                  className="block size-full"
                  fill="none"
                  preserveAspectRatio="none"
                  viewBox="0 0 16 16"
                >
                  <g clipPath="url(#clip0_15_2221)">
                    <path
                      d={svgPaths.p1aaaa600}
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d={svgPaths.p1bffbec0}
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d="M6.6643 5.99787H5.33144"
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d="M10.6629 8.66359H5.33144"
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d="M10.6629 11.3293H5.33144"
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                  </g>
                  <defs>
                    <clipPath id="clip0_15_2221">
                      <rect fill="white" height="15.9943" width="15.9943" />
                    </clipPath>
                  </defs>
                </svg>
              </div>
              <p className={styles.fileName}>index.html</p>
            </button>
            <button
              onClick={() => onSelect("public/styles.css")}
              className={cn(styles.fileButton, styles.fileButtonNested)}
            >
              <div className={styles.fileIcon}>
                <svg
                  className="block size-full"
                  fill="none"
                  preserveAspectRatio="none"
                  viewBox="0 0 16 16"
                >
                  <g clipPath="url(#clip0_15_2221)">
                    <path
                      d={svgPaths.p1aaaa600}
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d={svgPaths.p1bffbec0}
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d="M6.6643 5.99787H5.33144"
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d="M10.6629 8.66359H5.33144"
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d="M10.6629 11.3293H5.33144"
                      stroke="#888888"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                  </g>
                  <defs>
                    <clipPath id="clip0_15_2221">
                      <rect fill="white" height="15.9943" width="15.9943" />
                    </clipPath>
                  </defs>
                </svg>
              </div>
              <p className={styles.fileName}>styles.css</p>
            </button>
          </div>
        )}
      </div>

      {/* package.json */}
      <button
        onClick={() => onSelect("package.json")}
        className={styles.fileButton}
      >
        <div className={styles.fileIcon}>
          <svg
            className="block size-full"
            fill="none"
            preserveAspectRatio="none"
            viewBox="0 0 16 16"
          >
            <g clipPath="url(#clip0_15_2231)">
              <path
                d={svgPaths.p1aaaa600}
                stroke="#FECA57"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
              <path
                d={svgPaths.p1bffbec0}
                stroke="#FECA57"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
              <path
                d={svgPaths.p89bb2c0}
                stroke="#FECA57"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
              <path
                d={svgPaths.p381b1faa}
                stroke="#FECA57"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
            </g>
            <defs>
              <clipPath id="clip0_15_2231">
                <rect fill="white" height="15.9943" width="15.9943" />
              </clipPath>
            </defs>
          </svg>
        </div>
        <p className={styles.fileName}>package.json</p>
      </button>

      {/* README.md */}
      <button
        onClick={() => onSelect("README.md")}
        className={styles.fileButton}
      >
        <div className={styles.fileIcon}>
          <svg
            className="block size-full"
            fill="none"
            preserveAspectRatio="none"
            viewBox="0 0 16 16"
          >
            <g clipPath="url(#clip0_15_2221)">
              <path
                d={svgPaths.p1aaaa600}
                stroke="#888888"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
              <path
                d={svgPaths.p1bffbec0}
                stroke="#888888"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
              <path
                d="M6.6643 5.99787H5.33144"
                stroke="#888888"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
              <path
                d="M10.6629 8.66359H5.33144"
                stroke="#888888"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
              <path
                d="M10.6629 11.3293H5.33144"
                stroke="#888888"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="1.33286"
              />
            </g>
            <defs>
              <clipPath id="clip0_15_2221">
                <rect fill="white" height="15.9943" width="15.9943" />
              </clipPath>
            </defs>
          </svg>
        </div>
        <p className={styles.fileName}>README.md</p>
      </button>
    </div>
  );
}

interface BentoGridProps {
  hasPanel: boolean;
}

function BentoGrid({ hasPanel }: BentoGridProps) {
  return (
    <div
      className={cn(
        styles.bentoGrid,
        hasPanel ? styles.bentoGridWithPanel : styles.bentoGridWithoutPanel
      )}
    >
      {/* Panel 1 - spans 4 columns, 2 rows */}
      <div
        className={cn(
          styles.bentoPanel,
          hasPanel ? styles.bentoPanelWithPanel : styles.colSpan4,
          styles.rowSpan2
        )}
      >
        <p className={styles.bentoPanelText}>Panel 1</p>
      </div>

      {/* Panel 2 - spans 8 columns, 2 rows */}
      <div
        className={cn(
          styles.bentoPanel,
          hasPanel ? styles.bentoPanelWithPanel : styles.colSpan8,
          styles.rowSpan2
        )}
      >
        <p className={styles.bentoPanelText}>Panel 2</p>
      </div>

      {/* Panel 3 - spans 7 columns, 3 rows */}
      <div
        className={cn(
          styles.bentoPanel,
          hasPanel ? styles.bentoPanelWithPanel : styles.colSpan7,
          styles.rowSpan3
        )}
      >
        <p className={styles.bentoPanelText}>Panel 3</p>
      </div>

      {/* Panel 4 - spans 5 columns, 3 rows */}
      <div
        className={cn(
          styles.bentoPanel,
          hasPanel ? styles.bentoPanelWithPanel : styles.colSpan5,
          styles.rowSpan3
        )}
      >
        <p className={styles.bentoPanelText}>Panel 4</p>
      </div>

      {/* Panel 5 - spans 12 columns, 3 rows */}
      <div
        className={cn(
          styles.bentoPanel,
          hasPanel ? styles.bentoPanelWithPanel : styles.colSpan12,
          styles.rowSpan3
        )}
      >
        <p className={styles.bentoPanelText}>Panel 5</p>
      </div>

      {/* Panel 6 - spans 4 columns, 2 rows */}
      <div
        className={cn(
          styles.bentoPanel,
          hasPanel ? styles.bentoPanelWithPanel : styles.colSpan4,
          styles.rowSpan2
        )}
      >
        <p className={styles.bentoPanelText}>Panel 6</p>
      </div>

      {/* Panel 7 - spans 4 columns, 2 rows */}
      <div
        className={cn(
          styles.bentoPanel,
          hasPanel ? styles.bentoPanelWithPanel : styles.colSpan4,
          styles.rowSpan2
        )}
      >
        <p className={styles.bentoPanelText}>Panel 7</p>
      </div>

      {/* Panel 8 - spans 4 columns, 2 rows */}
      <div
        className={cn(
          styles.bentoPanel,
          hasPanel ? styles.bentoPanelWithPanel : styles.colSpan4,
          styles.rowSpan2
        )}
      >
        <p className={styles.bentoPanelText}>Panel 8</p>
      </div>
    </div>
  );
}
