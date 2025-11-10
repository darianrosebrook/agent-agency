"use client";

/**
 * Project Layout - Shared layout for all project tabs
 *
 * This layout provides the shared header, breadcrumbs, and tab navigation
 * for all project detail pages. Individual tab pages render their content
 * in the {children} slot.
 *
 * @author @darianrosebrook
 */

import { Suspense, useEffect } from "react";
import { useParams, usePathname, useRouter } from "next/navigation";
import { ChevronRight } from "lucide-react";
import Link from "next/link";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/primitives/breadcrumb";
import { useProjectContext } from "@/components/projects/ProjectContext";
import svgPaths from "@/imports/svg-ustevohwso";
import { cn } from "@/components/primitives/utils";
import styles from "./layout.module.scss";

type TabType = "overview" | "workspace" | "tasks" | "timeline" | "manage";

const TABS: Array<{ id: TabType; label: string; path: string }> = [
  { id: "overview", label: "Overview", path: "overview" },
  { id: "workspace", label: "Workspace", path: "workspace" },
  { id: "tasks", label: "Tasks", path: "tasks" },
  { id: "timeline", label: "Timeline", path: "timeline" },
  { id: "manage", label: "Manage Project", path: "manage" },
];

export default function ProjectLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const params = useParams();
  const pathname = usePathname();
  const router = useRouter();
  const { getProjectById, selectProject } = useProjectContext();

  const projectId =
    typeof params?.projectId === "string"
      ? params.projectId
      : Array.isArray(params?.projectId)
      ? params.projectId[0]
      : null;

  // Initialize project context when projectId changes
  useEffect(() => {
    if (projectId) {
      const project = getProjectById(projectId);
      if (project) {
        selectProject(projectId);
      }
    }
  }, [projectId, getProjectById, selectProject]);

  const project = projectId ? getProjectById(projectId) : null;

  // Determine active tab from pathname
  const getActiveTab = (): TabType => {
    if (!pathname || !projectId) return "overview";
    
    // Extract tab from pathname: /projects/[projectId]/[tab]
    const pathParts = pathname.split("/").filter(Boolean);
    const projectIndex = pathParts.indexOf(projectId);
    
    // If projectId is found and there's a segment after it, that's the tab
    if (projectIndex >= 0 && pathParts[projectIndex + 1]) {
      const tab = pathParts[projectIndex + 1];
      
      // Map path to tab type
      const tabMap: Record<string, TabType> = {
        overview: "overview",
        workspace: "workspace",
        tasks: "tasks",
        timeline: "timeline",
        manage: "manage",
        settings: "manage", // Support both "manage" and "settings"
      };
      
      return tabMap[tab] ?? "overview";
    }
    
    // Default to overview if at root project path
    return "overview";
  };

  const activeTab = getActiveTab();
  const basePath = projectId ? `/projects/${projectId}` : "/projects";

  const handleBackToProjects = () => {
    router.push("/projects");
  };

  if (!projectId || !project) {
    return (
      <div className={styles.loadingContainer}>
        <div className={styles.loadingText}>Loading project...</div>
      </div>
    );
  }

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
                      onClick={handleBackToProjects}
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
                      {project.name}
                    </BreadcrumbPage>
                  </BreadcrumbItem>
                </BreadcrumbList>
              </Breadcrumb>
            </div>

            <div className={styles.headingContainer}>
              <h1 className={styles.heading}>{project.name}</h1>
            </div>
          </div>

          {/* Tabs and Controls */}
          <div className={styles.tabsControlsContainer}>
            {/* Tabs */}
            <div className={styles.tabsContainer}>
              <div className={styles.tabsList}>
                {TABS.map((tab) => {
                  const tabPath = tab.path === "overview" 
                    ? basePath 
                    : `${basePath}/${tab.path}`;
                  const isActive = activeTab === tab.id;
                  
                  return (
                    <Link
                      key={tab.id}
                      href={tabPath}
                      className={styles.tabButton}
                    >
                      <span
                        className={cn(
                          styles.tabLabel,
                          isActive
                            ? styles.tabLabelActive
                            : styles.tabLabelInactive
                        )}
                      >
                        {tab.label}
                      </span>
                      {isActive && (
                        <div className={styles.tabIndicator} />
                      )}
                    </Link>
                  );
                })}
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
        <Suspense
          fallback={
            <div className={styles.loadingContainer}>
              <div className={styles.loadingText}>Loading tab content...</div>
            </div>
          }
        >
          {children}
        </Suspense>
      </div>
    </div>
  );
}

