"use client";

import { useState, useEffect, useRef, useCallback } from "react";

const gsapLoader = async () => {
  const gsap = await import("gsap");
  return gsap.gsap;
};
import {
  Search,
  MessageSquare,
  FileSignature,
  LayoutGrid,
  TrendingUp,
  FileCode,
  HeartPulse,
  Workflow,
  Settings,
  Moon,
  ChevronDown,
  FolderPlus,
} from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../ui/tooltip";
import { cn } from "../ui/utils";
import styles from "./NavigationSidebar.module.scss";

export function Sidebar() {
  const pathname = usePathname();
  const [isCollapsed, setIsCollapsed] = useState(false);
  const sidebarRef = useRef<HTMLElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);

  const isActive = useCallback(
    (path: string) => {
      return pathname === path;
    },
    [pathname]
  );

  const toggleCollapse = useCallback(() => {
    setIsCollapsed((prev) => !prev);
  }, []);

  // Optimize sidebar animation with GSAP - lazy loaded
  useEffect(() => {
    if (!sidebarRef.current) return;

    const sidebar = sidebarRef.current;
    const targetWidth = isCollapsed ? 64 : 320;

    sidebar.style.willChange = "width";

    let tween: gsap.core.Tween | null = null;

    gsapLoader().then((gsap) => {
      tween = gsap.to(sidebar, {
        width: targetWidth,
        duration: 0.3,
        ease: "power2.out",
        force3D: true,
        onComplete: () => {
          sidebar.style.willChange = "auto";
        },
      });
    });

    return () => {
      if (tween) {
        tween.kill();
      }
      sidebar.style.willChange = "auto";
    };
  }, [isCollapsed]);

  // Animate content opacity for smoother transitions - lazy loaded GSAP
  useEffect(() => {
    if (!contentRef.current) return;

    const content = contentRef.current;
    const textElements = Array.from(
      content.querySelectorAll("span, h4, input")
    ) as HTMLElement[];

    if (textElements.length === 0) return;

    let tween: gsap.core.Tween | null = null;

    gsapLoader().then((gsap) => {
      if (isCollapsed) {
        tween = gsap.to(textElements, {
          opacity: 0,
          duration: 0.15,
          stagger: 0.01,
          ease: "power2.in",
          force3D: true,
        });
      } else {
        tween = gsap.fromTo(
          textElements,
          { opacity: 0 },
          {
            opacity: 1,
            duration: 0.2,
            stagger: 0.01,
            delay: 0.1,
            ease: "power2.out",
            force3D: true,
          }
        );
      }
    });

    return () => {
      if (tween) {
        tween.kill();
      }
    };
  }, [isCollapsed]);

  return (
    <aside
      ref={sidebarRef}
      className={styles.sidebar}
      style={{
        width: isCollapsed ? "64px" : "320px", // w-16 = 64px, w-80 = 320px
        // Use contain for layout isolation to prevent layout thrashing
        contain: "layout style paint",
        // Optimize rendering
        backfaceVisibility: "hidden",
        transform: "translateZ(0)", // Force GPU layer
      }}
    >
      {/* Header */}
      <div
        ref={contentRef}
        className={cn(
          styles.header,
          isCollapsed ? styles.headerCollapsed : styles.headerExpanded
        )}
      >
        <div
          className={cn(
            styles.headerTop,
            isCollapsed ? styles.headerTopCollapsed : styles.headerTopExpanded
          )}
        >
          {!isCollapsed && (
            <div className={styles.logoContainer}>
              <div className={styles.logoIcon}>
                <Moon className="w-4 h-4 text-gray-400" />
              </div>
              <h4 className={styles.logoText}>Agent Agency</h4>
            </div>
          )}
          {isCollapsed && (
            <div className={cn(styles.logoIcon, styles.logoIconCollapsed)}>
              <Moon className="w-4 h-4 text-gray-400" />
            </div>
          )}
          <button onClick={toggleCollapse} className={styles.collapseButton}>
            <LayoutGrid className="w-4 h-4" />
          </button>
        </div>

        {/* Search */}
        {!isCollapsed && (
          <div className={styles.searchContainer}>
            {/* TODO: Implement global search functionality with the following requirements:
            // 1. Search API endpoint: Create unified search endpoint
            //    - Data source: GET /api/search?q={query} endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
            //    - Search across projects, tasks, chat messages, and files
            //    - Support filtering by type (projects, tasks, chats, files)
            // 2. Search results display: Show search results in dropdown or modal
            //    - Display matching projects, tasks, chats, and files
            //    - Highlight search terms in results
            //    - Show result type icons and metadata
            // 3. Keyboard shortcuts: Support keyboard navigation
            //    - "/" key to focus search input
            //    - Arrow keys to navigate results
            //    - Enter to select result
            //    - Escape to close search
            // 4. Search suggestions: Provide search suggestions as user types
            //    - Autocomplete for project names, task titles, etc.
            //    - Recent searches history
            //    - Popular searches
            // 5. Search result navigation: Navigate to relevant pages on result click
            //    - Projects -> /projects/:projectId
            //    - Tasks -> /projects/:projectId with task highlighted
            //    - Chats -> /chat with message highlighted
            //    - Files -> /projects/:projectId/workspace with file selected */}
            <Search className={styles.searchIcon} />
            <input
              type="text"
              placeholder="Search"
              className={styles.searchInput}
            />
            <kbd className={styles.searchKeyboard}>/</kbd>
          </div>
        )}

        {/* Quick Links */}
        <TooltipProvider>
          <div
            className={
              isCollapsed ? styles.quickLinksCollapsed : styles.quickLinks
            }
          >
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/chat"
                  className={cn(
                    styles.quickLink,
                    isCollapsed
                      ? styles.quickLinkCollapsed
                      : styles.quickLinkExpanded
                  )}
                >
                  <MessageSquare className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Chat</span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Chat</p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/projects"
                  className={cn(
                    styles.quickLink,
                    isCollapsed
                      ? styles.quickLinkCollapsed
                      : styles.quickLinkExpanded
                  )}
                >
                  <FileSignature className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Projects</span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Projects</p>
                </TooltipContent>
              )}
            </Tooltip>
          </div>
        </TooltipProvider>
      </div>

      {/* Navigation */}
      <nav
        className={cn(
          styles.nav,
          isCollapsed ? styles.navCollapsed : styles.navExpanded
        )}
      >
        <TooltipProvider>
          <div className={styles.navLinks}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/"
                  className={cn(
                    styles.navLink,
                    isCollapsed
                      ? styles.navLinkCollapsed
                      : styles.navLinkExpanded,
                    isActive("/")
                      ? styles.navLinkActive
                      : styles.navLinkInactive
                  )}
                >
                  <LayoutGrid className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Dashboard</span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Dashboard</p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/agent-stats"
                  className={cn(
                    styles.navLink,
                    isCollapsed
                      ? styles.navLinkCollapsed
                      : styles.navLinkExpanded,
                    isActive("/agent-stats")
                      ? styles.navLinkActive
                      : styles.navLinkInactive
                  )}
                >
                  <TrendingUp className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Agent Stats</span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Agent Stats</p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/rules-governance"
                  className={cn(
                    styles.navLink,
                    isCollapsed
                      ? styles.navLinkCollapsed
                      : styles.navLinkExpanded,
                    isActive("/rules-governance")
                      ? styles.navLinkActive
                      : styles.navLinkInactive
                  )}
                >
                  <FileCode className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>
                      Rules & Governance
                    </span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Rules & Governance</p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/agent-health"
                  className={cn(
                    styles.navLink,
                    isCollapsed
                      ? styles.navLinkCollapsed
                      : styles.navLinkExpanded,
                    isActive("/agent-health")
                      ? styles.navLinkActive
                      : styles.navLinkInactive
                  )}
                >
                  <HeartPulse className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Agent Health</span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Agent Health</p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/phase-planner"
                  className={cn(
                    styles.navLink,
                    isCollapsed
                      ? styles.navLinkCollapsed
                      : styles.navLinkExpanded,
                    isActive("/phase-planner")
                      ? styles.navLinkActive
                      : styles.navLinkInactive
                  )}
                >
                  <Workflow className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Phase Planner</span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Phase Planner</p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/settings"
                  className={cn(
                    styles.navLink,
                    isCollapsed
                      ? styles.navLinkCollapsed
                      : styles.navLinkExpanded,
                    isActive("/settings")
                      ? styles.navLinkActive
                      : styles.navLinkInactive
                  )}
                >
                  <Settings className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Settings</span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Settings</p>
                </TooltipContent>
              )}
            </Tooltip>
          </div>
          <hr className={styles.divider} />
          {/* Folders */}
          {!isCollapsed && (
            <div className={styles.folders}>
              {/* TODO: Replace hardcoded recent projects with dynamic project list from v3 database with the following requirements:
              // 1. Recent projects fetching: Load recent projects sorted by last_accessed
              //    - Data source: GET /api/projects?limit=3&sort=last_accessed endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
              //    - Database table: PostgreSQL `projects` table
              //    - Include project name, ID, and status for display
              // 2. Project navigation: Make project buttons link to project detail view
              //    - Link to /projects/:projectId route
              //    - Update last_accessed timestamp when project is clicked
              // 3. Project status indicators: Show project status with color-coded dots
              //    - Use project.status field to determine dot color
              //    - Map status values to colors (active=blue, paused=gray, completed=yellow, etc.)
              // 4. New Project button: Open new project modal
              //    - Trigger NewProjectModal component
              //    - Handle project creation and refresh recent projects list
              // 5. Expandable project items: Show project details on hover/click
              //    - Display project description or summary
              //    - Show project progress or task count
              //    - Allow quick actions (open, archive, delete) */}
              <button
                className={cn(styles.folderButton, styles.folderButtonGroup)}
              >
                <div
                  className={cn(
                    styles.folderStatusDot,
                    styles.folderStatusDotBlue
                  )}
                ></div>
                <span className={styles.folderName}>Recent Project</span>
                <ChevronDown className={styles.folderChevron} />
              </button>
              <button
                className={cn(styles.folderButton, styles.folderButtonGroup)}
              >
                <div
                  className={cn(
                    styles.folderStatusDot,
                    styles.folderStatusDotGray
                  )}
                ></div>
                <span className={styles.folderName}>Recent Project</span>
                <ChevronDown className={styles.folderChevron} />
              </button>
              <button
                className={cn(styles.folderButton, styles.folderButtonGroup)}
              >
                <div
                  className={cn(
                    styles.folderStatusDot,
                    styles.folderStatusDotYellow
                  )}
                ></div>
                <span className={styles.folderName}>Recent Project</span>
                <ChevronDown className={styles.folderChevron} />
              </button>
              <button className={styles.newProjectButton}>
                <FolderPlus className="w-4 h-4" />
                <span className={styles.newProjectText}>New Project</span>
              </button>
            </div>
          )}
        </TooltipProvider>
      </nav>
    </aside>
  );
}
