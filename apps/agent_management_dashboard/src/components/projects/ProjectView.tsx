"use client";

import { useState } from "react";
import { ChevronRight } from "lucide-react";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "../primitives/breadcrumb";
import { OverviewTab } from "./OverviewTab";
import { WorkspaceTab } from "./WorkspaceTab";
import { TasksTab } from "./TasksTab";
import { TimelineTab } from "./TimelineTab";
import { ManageTab } from "./SettingsTab";
import svgPaths from "../../imports/svg-ustevohwso";
import { cn } from "../primitives/utils";
import styles from "./ProjectView.module.scss";

interface ProjectViewProps {
  projectName: string;
  onBackToProjects: () => void;
}

type TabType = "overview" | "workspace" | "tasks" | "timeline" | "manage";

const TABS: Array<{ id: TabType; label: string }> = [
  { id: "overview", label: "Overview" },
  { id: "workspace", label: "Workspace" },
  { id: "tasks", label: "Tasks" },
  { id: "timeline", label: "Timeline" },
  { id: "manage", label: "Manage Project" },
];

export function ProjectView({
  projectName,
  onBackToProjects,
}: ProjectViewProps) {
  const [activeTab, setActiveTab] = useState<TabType>("overview");

  return (
    <div className={styles.projectView}>
      <div className={styles.headerContainer}>
        <div className={styles.headerContent}>
          {/* Breadcrumb and Title */}
          <div className={styles.breadcrumbTitleContainer}>
            <div className={styles.breadcrumbContainer}>
              <Breadcrumb>
                <BreadcrumbList>
                  <BreadcrumbItem>
                    <BreadcrumbLink
                      onClick={onBackToProjects}
                      className={styles.breadcrumbLink}
                    >
                      Projects
                    </BreadcrumbLink>
                  </BreadcrumbItem>
                  <BreadcrumbSeparator>
                    <ChevronRight className={styles.breadcrumbSeparatorIcon} />
                  </BreadcrumbSeparator>
                  <BreadcrumbItem>
                    <BreadcrumbPage className={styles.breadcrumbPage}>
                      {projectName}
                    </BreadcrumbPage>
                  </BreadcrumbItem>
                </BreadcrumbList>
              </Breadcrumb>
            </div>

            <div className={styles.headingContainer}>
              <h1 className={styles.heading}>{projectName}</h1>
            </div>
          </div>

          {/* Tabs and Controls */}
          <div className={styles.tabsControlsContainer}>
            {/* Tabs */}
            <div className={styles.tabsContainer}>
              <div className={styles.tabsList}>
                {TABS.map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={styles.tabButton}
                    type="button"
                  >
                    <span
                      className={cn(
                        styles.tabLabel,
                        activeTab === tab.id
                          ? styles.tabLabelActive
                          : styles.tabLabelInactive
                      )}
                    >
                      {tab.label}
                    </span>
                    {activeTab === tab.id && (
                      <div className={styles.tabIndicator} />
                    )}
                  </button>
                ))}
              </div>
            </div>

            {/* Controls */}
            <div className={styles.controlsContainer}>
              <div className={styles.controlsList}>
                {/* Search Input */}
                <div className={styles.searchContainer}>
                  <div className={styles.searchBox}>
                    <div className={styles.searchInput}>
                      <span className={styles.searchPlaceholder}>Search</span>
                    </div>
                    <div aria-hidden="true" className={styles.searchBorder} />
                  </div>
                  <div className={styles.searchIcon}>
                    <svg
                      className={styles.svgFullSize}
                      fill="none"
                      preserveAspectRatio="none"
                      viewBox="0 0 16 16"
                    >
                      <path
                        d={svgPaths.p24791400}
                        stroke="#888888"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="1.33286"
                      />
                      <path
                        d={svgPaths.p2139fb00}
                        stroke="#888888"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="1.33286"
                      />
                    </svg>
                  </div>
                  <div className={styles.keyboardShortcut}>
                    <span className={styles.keyboardShortcutText}>⌘F</span>
                  </div>
                </div>

                {/* Status Button */}
                <button className={styles.controlButton} type="button">
                  <div
                    aria-hidden="true"
                    className={styles.controlButtonBorder}
                  />
                  <div className={styles.controlButtonContent}>
                    <span className={styles.controlButtonText}>Status: All</span>
                    <div className={styles.controlButtonIcon}>
                      <svg
                        className={styles.svgFullSize}
                        fill="none"
                        preserveAspectRatio="none"
                        viewBox="0 0 16 16"
                      >
                        <path
                          d={svgPaths.p10a02b40}
                          stroke="white"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth="1.33286"
                        />
                      </svg>
                    </div>
                  </div>
                </button>

                {/* Sort Button */}
                <button className={styles.controlButton} type="button">
                  <div
                    aria-hidden="true"
                    className={styles.controlButtonBorder}
                  />
                  <div className={styles.controlButtonContent}>
                    <div className={styles.controlButtonIcon}>
                      <svg
                        className={styles.svgFullSize}
                        fill="none"
                        preserveAspectRatio="none"
                        viewBox="0 0 16 16"
                      >
                        <path
                          d={svgPaths.p26dba700}
                          stroke="white"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth="1.33286"
                        />
                        <path
                          d="M11.3293 13.3286V2.66572"
                          stroke="white"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth="1.33286"
                        />
                        <path
                          d={svgPaths.pea98c00}
                          stroke="white"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth="1.33286"
                        />
                        <path
                          d="M4.66501 2.66572V13.3286"
                          stroke="white"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth="1.33286"
                        />
                      </svg>
                    </div>
                    <span className={styles.controlButtonText}>Sort</span>
                  </div>
                </button>

                {/* Grid View Button */}
                <button className={styles.gridViewButton} type="button">
                  <div
                    aria-hidden="true"
                    className={styles.controlButtonBorder}
                  />
                  <div className={styles.gridViewIcon}>
                    <svg
                      className={styles.svgFullSize}
                      fill="none"
                      preserveAspectRatio="none"
                      viewBox="0 0 16 16"
                    >
                      <path
                        d={svgPaths.p3cc8d400}
                        stroke="white"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="1.33286"
                      />
                      <path
                        d="M1.99929 5.99787H13.995"
                        stroke="white"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="1.33286"
                      />
                      <path
                        d="M1.99929 9.99645H13.995"
                        stroke="white"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="1.33286"
                      />
                      <path
                        d="M5.99787 1.99929V13.995"
                        stroke="white"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="1.33286"
                      />
                      <path
                        d="M9.99645 1.99929V13.995"
                        stroke="white"
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
        </div>
      </div>

      {/* Tab Content Area */}
      <div className={styles.tabContentArea}>
        {activeTab === "overview" && <OverviewTab />}
        {activeTab === "workspace" && <WorkspaceTab />}
        {activeTab === "tasks" && <TasksTab />}
        {activeTab === "timeline" && <TimelineTab />}
        {activeTab === "manage" && <ManageTab />}
      </div>
    </div>
  );
}
