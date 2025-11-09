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
} from "../ui/tooltip";

export function Sidebar() {
  const pathname = usePathname();
  const [isCollapsed, setIsCollapsed] = useState(false);

  const isActive = (path: string) => {
    return pathname === path;
  };

  return (
    <aside
      className={`${
        isCollapsed ? "w-16" : "w-80"
      } bg-[#1a1a1a] border-r border-gray-800 flex flex-col h-screen transition-all duration-300`}
    >
      {/* Header */}
      <div
        className={`${isCollapsed ? "p-3" : "p-6"} border-b border-gray-800`}
      >
        <div
          className={`flex items-center ${
            isCollapsed ? "justify-center mb-3" : "justify-between mb-6"
          }`}
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
      <nav className={`flex-1 ${isCollapsed ? "p-2" : "p-4"} overflow-y-auto`}>
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
                <button
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg`}
                >
                  <TrendingUp className="w-4 h-4" />
                  {!isCollapsed && <span className="text-sm">Agent Stats</span>}
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
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg`}
                >
                  <FileCode className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className="text-sm">Rules & Governance</span>
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
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg`}
                >
                  <HeartPulse className="w-4 h-4" />
                  {!isCollapsed && (
                    <span className="text-sm">Agent Health</span>
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
                <button
                  className={`w-full flex items-center ${
                    isCollapsed ? "justify-center" : "gap-3"
                  } px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg`}
                >
                  <Settings className="w-4 h-4" />
                  {!isCollapsed && <span className="text-sm">Settings</span>}
                </button>
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
