"use client";

import { getUnreadCount } from "@/lib/stores/notificationStore";
import {
  Bell,
  ChevronDown,
  FileCode,
  FileSignature,
  FolderPlus,
  HeartPulse,
  LayoutGrid,
  MessageSquare,
  Moon,
  Search,
  Settings,
  TestTube,
  TrendingUp,
  Workflow,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { listProjects, type ProjectListItem } from "../../lib/api/projects";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../primitives/tooltip";
import { cn } from "../primitives/utils";
import styles from "./NavigationSidebar.module.scss";

// Helper function to map project state to status dot color
function getStatusDotClass(state?: string | null): string {
  switch (state?.toLowerCase()) {
    case "active":
    case "in_progress":
      return styles.folderStatusDotBlue;
    case "completed":
    case "done":
      return styles.folderStatusDotYellow;
    case "paused":
    case "on_hold":
      return styles.folderStatusDotGray;
    default:
      return styles.folderStatusDotBlue;
  }
}

export function Sidebar() {
  const location = useLocation();
  const pathname = location.pathname;
  const navigate = useNavigate();
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [unreadCount, setUnreadCount] = useState(0);
  const [recentProjects, setRecentProjects] = useState<ProjectListItem[]>([]);
  const [isLoadingProjects, setIsLoadingProjects] = useState(true);

  const isActive = useCallback(
    (path: string) => {
      return pathname === path;
    },
    [pathname]
  );

  const toggleCollapse = useCallback(() => {
    setIsCollapsed((prev) => !prev);
  }, []);

  useEffect(() => {
    async function fetchRecentProjects() {
      try {
        const response = await listProjects();
        // Sort by updated_at (most recently updated first) and take top 3
        const sorted = response.projects
          .sort((a, b) => {
            const aTime = new Date(a.updated_at || a.created_at).getTime();
            const bTime = new Date(b.updated_at || b.created_at).getTime();
            return bTime - aTime;
          })
          .slice(0, 3);
        setRecentProjects(sorted);
      } catch (error) {
        console.error("Failed to fetch recent projects:", error);
        setRecentProjects([]);
      } finally {
        setIsLoadingProjects(false);
      }
    }

    fetchRecentProjects();
  }, []);

  useEffect(() => {
    // Update unread count
    const updateUnreadCount = () => {
      setUnreadCount(getUnreadCount());
    };

    updateUnreadCount();
    // Check for new notifications every 5 seconds
    const interval = setInterval(updateUnreadCount, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <aside
      className={cn(styles.sidebar, isCollapsed && styles.sidebarCollapsed)}
    >
      {/* Header */}
      <div
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
                <Moon className={styles.iconGray} />
              </div>
              <h4 className={styles.logoText}>Agent Agency</h4>
            </div>
          )}
          {isCollapsed && (
            <div className={cn(styles.logoIcon, styles.logoIconCollapsed)}>
              <Moon className={styles.iconGray} />
            </div>
          )}
          <button onClick={toggleCollapse} className={styles.collapseButton}>
            <LayoutGrid className={styles.icon} />
          </button>
        </div>

        {/* Search */}
        {!isCollapsed && (
          <div
            className={styles.searchContainer}
            onClick={() => navigate("/search")}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                navigate("/search");
              }
            }}
          >
            <Search className={styles.searchIcon} />
            <input
              type="text"
              placeholder="Search"
              className={styles.searchInput}
              readOnly
              onClick={(e) => {
                e.stopPropagation();
                navigate("/search");
              }}
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
                  to="/chat"
                  className={cn(
                    styles.quickLink,
                    isCollapsed
                      ? styles.quickLinkCollapsed
                      : styles.quickLinkExpanded
                  )}
                >
                  <MessageSquare className={styles.icon} />
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
                  to="/projects"
                  className={cn(
                    styles.quickLink,
                    isCollapsed
                      ? styles.quickLinkCollapsed
                      : styles.quickLinkExpanded
                  )}
                >
                  <FileSignature className={styles.icon} />
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
                  to="/"
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
                  <LayoutGrid className={styles.icon} />
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
                  to="/agent-stats"
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
                  <TrendingUp className={styles.icon} />
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
                  to="/rules-governance"
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
                  <FileCode className={styles.icon} />
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
                  to="/agent-health"
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
                  <HeartPulse className={styles.icon} />
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
                  to="/phase-planner"
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
                  <Workflow className={styles.icon} />
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
                  to="/testing"
                  className={cn(
                    styles.navLink,
                    isCollapsed
                      ? styles.navLinkCollapsed
                      : styles.navLinkExpanded,
                    isActive("/testing")
                      ? styles.navLinkActive
                      : styles.navLinkInactive
                  )}
                >
                  <TestTube className={styles.icon} />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Testing</span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Testing</p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  to="/notifications"
                  className={cn(
                    styles.navLink,
                    isCollapsed
                      ? styles.navLinkCollapsed
                      : styles.navLinkExpanded,
                    isActive("/notifications")
                      ? styles.navLinkActive
                      : styles.navLinkInactive
                  )}
                >
                  <div className={styles.iconContainer}>
                    <Bell className={styles.icon} />
                    {unreadCount > 0 && (
                      <span className={styles.unreadBadge}>
                        {unreadCount > 99 ? "99+" : unreadCount}
                      </span>
                    )}
                  </div>
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Notifications</span>
                  )}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>
                    Notifications{unreadCount > 0 ? ` (${unreadCount})` : ""}
                  </p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  to="/settings"
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
                  <Settings className={styles.icon} />
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
              {isLoadingProjects ? (
                <div className={styles.loadingProjects}>
                  Loading projects...
                </div>
              ) : recentProjects.length > 0 ? (
                recentProjects.map((project) => (
                  <Link
                    key={project.project_id}
                    to={`/projects/${project.project_id}`}
                    className={cn(
                      styles.folderButton,
                      styles.folderButtonGroup
                    )}
                  >
                    <div
                      className={cn(
                        styles.folderStatusDot,
                        getStatusDotClass(project.state)
                      )}
                    ></div>
                    <span className={styles.folderName}>{project.name}</span>
                    <ChevronDown className={styles.folderChevron} />
                  </Link>
                ))
              ) : (
                <div className={styles.noProjects}>No recent projects</div>
              )}
              <button className={styles.newProjectButton}>
                <FolderPlus className={styles.icon} />
                <span className={styles.newProjectText}>New Project</span>
              </button>
            </div>
          )}
        </TooltipProvider>
      </nav>
    </aside>
  );
}
