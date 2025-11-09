"use client";

import { useState, useEffect, useRef } from "react";
import { gsap } from "gsap";
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

export function Sidebar() {
  const pathname = usePathname();
  const [isCollapsed, setIsCollapsed] = useState(false);
  const sidebarRef = useRef<HTMLElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);

  const isActive = (path: string) => {
    return pathname === path;
  };

  // Optimize sidebar animation with GSAP
  useEffect(() => {
    if (!sidebarRef.current) return;

    const sidebar = sidebarRef.current;
    const targetWidth = isCollapsed ? 64 : 320; // w-16 = 64px, w-80 = 320px

    // Set will-change before animation starts
    sidebar.style.willChange = "width";

    // Use GSAP for optimized animation with force3D for GPU acceleration
    const tween = gsap.to(sidebar, {
      width: targetWidth,
      duration: 0.3,
      ease: "power2.out",
      force3D: true, // Force GPU acceleration
      onComplete: () => {
        sidebar.style.willChange = "auto";
      },
    });

    return () => {
      tween.kill();
      sidebar.style.willChange = "auto";
    };
  }, [isCollapsed]);

  // Animate content opacity for smoother transitions
  useEffect(() => {
    if (!contentRef.current) return;

    const content = contentRef.current;
    // Only animate text elements that should fade
    const textElements = Array.from(
      content.querySelectorAll("span, h4, input")
    ) as HTMLElement[];

    if (textElements.length === 0) return;

    if (isCollapsed) {
      // Fade out text elements quickly
      const tween = gsap.to(textElements, {
        opacity: 0,
        duration: 0.15,
        stagger: 0.01,
        ease: "power2.in",
        force3D: true, // GPU acceleration
      });

      return () => tween.kill();
    } else {
      // Fade in text elements with slight delay
      const tween = gsap.fromTo(
        textElements,
        { opacity: 0 },
        {
          opacity: 1,
          duration: 0.2,
          stagger: 0.01,
          delay: 0.1,
          ease: "power2.out",
          force3D: true, // GPU acceleration
        }
      );

      return () => tween.kill();
    }
  }, [isCollapsed]);

  return (
    <aside
      ref={sidebarRef}
      className={`${
        isCollapsed ? "w-16" : "w-80"
      } bg-[#1a1a1a] border-r border-gray-800 flex flex-col h-screen overflow-hidden`}
      style={{
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
        className={`${
          isCollapsed ? "p-3" : "p-6"
        } border-b border-gray-800 transition-[padding] duration-300 ease-out`}
      >
        <div
          className={`flex items-center ${
            isCollapsed ? "justify-center mb-3" : "justify-between mb-6"
          } transition-[margin] duration-300 ease-out`}
        >
          {!isCollapsed && (
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 bg-gray-700 rounded flex items-center justify-center">
                <Moon className="w-4 h-4 text-gray-400" />
              </div>
              <h4 className="inline text-white">Agent Agency</h4>
            </div>
          )}
          {isCollapsed && (
            <div className="w-6 h-6 bg-gray-700 rounded flex items-center justify-center mb-0">
              <Moon className="w-4 h-4 text-gray-400" />
            </div>
          )}
          <button
            onClick={() => setIsCollapsed(!isCollapsed)}
            className="text-gray-400 hover:text-gray-200"
          >
            <LayoutGrid className="w-4 h-4" />
          </button>
        </div>

        {/* Search */}
        {!isCollapsed && (
          <div className="relative">
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
            <Search className="w-4 h-4 text-gray-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder="Search"
              className="w-full bg-[#0f0f0f] border border-gray-800 rounded-lg pl-10 pr-8 py-2 text-gray-200 placeholder:text-gray-500"
            />
            <kbd className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 text-xs">
              /
            </kbd>
          </div>
        )}

        {/* Quick Links */}
        <TooltipProvider>
          <div className={`${isCollapsed ? "mt-0" : "mt-4"} space-y-1`}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/chat"
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg`}
                >
                  <MessageSquare className="w-4 h-4" />
                  {!isCollapsed && <span className="text-sm">Chat</span>}
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
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg`}
                >
                  <FileSignature className="w-4 h-4" />
                  {!isCollapsed && <span className="text-sm">Projects</span>}
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
        className={`flex-1 ${
          isCollapsed ? "p-2" : "p-4"
        } overflow-y-auto transition-[padding] duration-300 ease-out`}
      >
        <TooltipProvider>
          <div className="space-y-1">
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href="/"
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 rounded-lg ${
                    isActive("/")
                      ? "text-white bg-gray-800/50"
                      : "text-gray-300 hover:bg-gray-800/50"
                  }`}
                >
                  <LayoutGrid className="w-4 h-4" />
                  {!isCollapsed && <span className="text-sm">Dashboard</span>}
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
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 rounded-lg ${
                    isActive("/agent-stats")
                      ? "text-white bg-gray-800/50"
                      : "text-gray-300 hover:bg-gray-800/50"
                  }`}
                >
                  <TrendingUp className="w-4 h-4" />
                  {!isCollapsed && <span className="text-sm">Agent Stats</span>}
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
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 rounded-lg ${
                    isActive("/rules-governance")
                      ? "text-white bg-gray-800/50"
                      : "text-gray-300 hover:bg-gray-800/50"
                  }`}
                >
                  <FileCode className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className="text-sm">Rules & Governance</span>
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
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 rounded-lg ${
                    isActive("/agent-health")
                      ? "text-white bg-gray-800/50"
                      : "text-gray-300 hover:bg-gray-800/50"
                  }`}
                >
                  <HeartPulse className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className="text-sm">Agent Health</span>
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
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 rounded-lg ${
                    isActive("/phase-planner")
                      ? "text-white bg-gray-800/50"
                      : "text-gray-300 hover:bg-gray-800/50"
                  }`}
                >
                  <Workflow className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className="text-sm">Phase Planner</span>
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
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 rounded-lg ${
                    isActive("/settings")
                      ? "text-white bg-gray-800/50"
                      : "text-gray-300 hover:bg-gray-800/50"
                  }`}
                >
                  <Settings className="w-4 h-4" />
                  {!isCollapsed && <span className="text-sm">Settings</span>}
                </Link>
              </TooltipTrigger>
              {isCollapsed && (
                <TooltipContent side="right">
                  <p>Settings</p>
                </TooltipContent>
              )}
            </Tooltip>
          </div>
          <hr />
          {/* Folders */}
          {!isCollapsed && (
            <div className="mt-6 space-y-1">
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
              <button className="w-full flex items-center gap-3 px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg group">
                <div className="w-2 h-2 bg-blue-500 rounded-sm"></div>
                <span className="text-sm flex-1 text-left">Recent Project</span>
                <ChevronDown className="w-4 h-4 opacity-0 group-hover:opacity-100" />
              </button>
              <button className="w-full flex items-center gap-3 px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg group">
                <div className="w-2 h-2 bg-gray-500 rounded-sm"></div>
                <span className="text-sm flex-1 text-left">Recent Project</span>
                <ChevronDown className="w-4 h-4 opacity-0 group-hover:opacity-100" />
              </button>
              <button className="w-full flex items-center gap-3 px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg group">
                <div className="w-2 h-2 bg-yellow-500 rounded-sm"></div>
                <span className="text-sm flex-1 text-left">Recent Project</span>
                <ChevronDown className="w-4 h-4 opacity-0 group-hover:opacity-100" />
              </button>
              <button className="w-full flex items-center gap-3 px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg">
                <FolderPlus className="w-4 h-4" />
                <span className="text-sm">New Project</span>
              </button>
            </div>
          )}
        </TooltipProvider>
      </nav>
    </aside>
  );
}
