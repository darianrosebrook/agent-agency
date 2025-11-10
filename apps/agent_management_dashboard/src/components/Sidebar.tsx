"use client";

import { useState } from "react";
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
} from "./primitives/tooltip";
import { cn } from "./primitives/utils";
import styles from "./Sidebar.module.scss";

export function Sidebar() {
  const pathname = usePathname();
  const [isCollapsed, setIsCollapsed] = useState(false);

  const isActive = (path: string) => {
    return pathname === path;
  };

  return (
    <aside
      className={cn(
        styles.sidebar,
        isCollapsed ? styles.sidebarCollapsed : styles.sidebarExpanded
      )}
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
          <button
            onClick={() => setIsCollapsed(!isCollapsed)}
            className={styles.collapseButton}
          >
            <LayoutGrid className={styles.icon} />
          </button>
        </div>

        {/* Search */}
        {!isCollapsed && (
          <div className={styles.searchContainer}>
            <Search className={styles.searchIcon} />
            <input
              type="text"
              placeholder="Search"
              className={styles.searchInput}
            />
            <kbd className={styles.searchKbd}>/</kbd>
          </div>
        )}

        {/* Quick Links */}
        <TooltipProvider>
          <div
            className={cn(
              styles.quickLinks,
              isCollapsed && styles.quickLinksCollapsed
            )}
          >
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/chat"
                  className={cn(
                    styles.quickLink,
                    isCollapsed ? styles.quickLinkCollapsed : styles.quickLinkExpanded
                  )}
                >
                  <MessageSquare className={styles.icon} />
                  {!isCollapsed && (
                    <span className={styles.quickLinkText}>Chat</span>
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
                    isCollapsed ? styles.quickLinkCollapsed : styles.quickLinkExpanded
                  )}
                >
                  <FileSignature className={styles.icon} />
                  {!isCollapsed && (
                    <span className={styles.quickLinkText}>Projects</span>
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
                    isCollapsed ? styles.navLinkCollapsed : styles.navLinkExpanded,
                    isActive("/") ? styles.navLinkActive : styles.navLinkInactive
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
                <button
                  className={cn(
                    styles.navLink,
                    styles.navLinkInactive,
                    isCollapsed ? styles.navLinkCollapsed : styles.navLinkExpanded
                  )}
                >
                  <TrendingUp className={styles.icon} />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Agent Stats</span>
                  )}
                </button>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Agent Stats</p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  className={cn(
                    styles.navLink,
                    styles.navLinkInactive,
                    isCollapsed ? styles.navLinkCollapsed : styles.navLinkExpanded
                  )}
                >
                  <FileCode className={styles.icon} />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Rules & Governance</span>
                  )}
                </button>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Rules & Governance</p>
                </TooltipContent>
              )}
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  className={cn(
                    styles.navLink,
                    styles.navLinkInactive,
                    isCollapsed ? styles.navLinkCollapsed : styles.navLinkExpanded
                  )}
                >
                  <HeartPulse className={styles.icon} />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Agent Health</span>
                  )}
                </button>
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
                    isCollapsed ? styles.navLinkCollapsed : styles.navLinkExpanded,
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
                <button
                  className={cn(
                    styles.navLink,
                    styles.navLinkInactive,
                    isCollapsed ? styles.navLinkCollapsed : styles.navLinkExpanded
                  )}
                >
                  <Settings className={styles.icon} />
                  {!isCollapsed && (
                    <span className={styles.navLinkText}>Settings</span>
                  )}
                </button>
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
              <button className={cn(styles.folderButton)}>
                <div
                  className={cn(
                    styles.folderStatusDot,
                    styles.folderStatusDotBlue
                  )}
                ></div>
                <span className={styles.folderName}>Recent Project</span>
                <ChevronDown className={styles.folderChevron} />
              </button>
              <button className={cn(styles.folderButton)}>
                <div
                  className={cn(
                    styles.folderStatusDot,
                    styles.folderStatusDotGray
                  )}
                ></div>
                <span className={styles.folderName}>Recent Project</span>
                <ChevronDown className={styles.folderChevron} />
              </button>
              <button className={cn(styles.folderButton)}>
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
