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
} from "./ui/breadcrumb";
import { OverviewTab } from "./OverviewTab";
import { WorkspaceTab } from "./WorkspaceTab";
import { TasksTab } from "./TasksTab";
import { TimelineTab } from "./TimelineTab";
import { ManageTab } from "./ManageTab";
import svgPaths from "../imports/svg-ustevohwso";

interface ProjectViewProps {
  projectName: string;
  onBackToProjects: () => void;
}

type TabType = "overview" | "workspace" | "tasks" | "timeline" | "manage";

export function ProjectView({
  projectName,
  onBackToProjects,
}: ProjectViewProps) {
  const [activeTab, setActiveTab] = useState<TabType>("overview");

  const tabs = [
    { id: "overview" as TabType, label: "Overview" },
    { id: "workspace" as TabType, label: "Workspace" },
    { id: "tasks" as TabType, label: "Tasks" },
    { id: "timeline" as TabType, label: "Timeline" },
    { id: "manage" as TabType, label: "Manage Project" },
  ];

  return (
    <div className="bg-[#0d0d0d] content-stretch flex flex-col items-start relative size-full">
      {/* Header Container */}
      <div className="relative shrink-0 w-full border-b border-neutral-800">
        <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border content-stretch flex flex-col gap-[15.994px] items-start pb-[0.909px] pl-[23.999px] pr-[23.998px] pt-[15.994px] relative w-full">
          {/* Breadcrumb and Title */}
          <div className="content-stretch flex flex-col gap-[3.999px] items-start relative shrink-0 w-full">
            {/* Breadcrumb */}
            <div className="h-[19.993px] relative shrink-0 w-full">
              <Breadcrumb>
                <BreadcrumbList>
                  <BreadcrumbItem>
                    <BreadcrumbLink
                      onClick={onBackToProjects}
                      className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic text-[#888888] text-[14px] tracking-[-0.1504px] cursor-pointer hover:text-gray-300"
                    >
                      Projects
                    </BreadcrumbLink>
                  </BreadcrumbItem>
                  <BreadcrumbSeparator>
                    <ChevronRight className="w-3 h-3 text-[#888888]" />
                  </BreadcrumbSeparator>
                  <BreadcrumbItem>
                    <BreadcrumbPage className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic text-[#888888] text-[14px] tracking-[-0.1504px]">
                      {projectName}
                    </BreadcrumbPage>
                  </BreadcrumbItem>
                </BreadcrumbList>
              </Breadcrumb>
            </div>

            {/* Heading */}
            <div className="h-[31.996px] relative shrink-0 w-full">
              <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[32px] left-0 not-italic text-[24px] text-nowrap text-white top-[-0.18px] tracking-[0.0703px] whitespace-pre">
                {projectName}
              </p>
            </div>
          </div>

          {/* Tabs and Controls */}
          <div className="content-stretch flex h-[35.994px] items-center justify-between relative shrink-0 w-full">
            {/* Tabs */}
            <div className="h-[35.994px] relative shrink-0">
              <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[35.994px] relative flex gap-[24px] items-start">
                {tabs.map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className="relative h-[35.994px] group"
                  >
                    <p
                      className={`font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic text-[16px] text-nowrap tracking-[-0.3125px] whitespace-pre transition-colors ${
                        activeTab === tab.id
                          ? "text-white"
                          : "text-[#888888] hover:text-gray-300"
                      }`}
                    >
                      {tab.label}
                    </p>
                    {activeTab === tab.id && (
                      <div className="absolute bg-white h-[1.996px] left-0 top-[34px] right-0" />
                    )}
                  </button>
                ))}
              </div>
            </div>

            {/* Controls */}
            <div className="h-[35.994px] relative shrink-0">
              <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[35.994px] relative flex gap-[12px] items-center">
                {/* Search Input */}
                <div className="relative h-[35.994px] w-[255.994px] shrink-0">
                  <div className="bg-[#1a1a1a] h-[35.994px] rounded-[8px] w-[255.994px]">
                    <div className="box-border content-stretch flex h-[35.994px] items-center overflow-clip pl-[36px] pr-[48px] py-[4px] relative rounded-[inherit] w-[255.994px]">
                      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[normal] not-italic relative shrink-0 text-[#888888] text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre">
                        Search
                      </p>
                    </div>
                    <div
                      aria-hidden="true"
                      className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[8px]"
                    />
                  </div>
                  {/* Search Icon */}
                  <div className="absolute left-[12px] size-[15.994px] top-[10px]">
                    <svg
                      className="block size-full"
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
                  {/* Keyboard Shortcut */}
                  <div className="absolute h-[16.001px] left-[226.53px] top-[10px] w-[17.465px]">
                    <p className="absolute font-['Inter:Regular',sans-serif] font-normal leading-[16px] left-0 not-italic text-[#888888] text-[12px] text-nowrap top-[0.46px] whitespace-pre">
                      ⌘F
                    </p>
                  </div>
                </div>

                {/* Status Button */}
                <button className="bg-[#1a1a1a] h-[35.994px] rounded-[8px] px-4 relative shrink-0 hover:bg-[#252525] transition-colors">
                  <div
                    aria-hidden="true"
                    className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[8px]"
                  />
                  <div className="flex items-center gap-2 h-full whitespace-nowrap">
                    <p className="font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic text-[14px] text-white tracking-[-0.1504px]">
                      Status: All
                    </p>
                    <div className="size-[15.994px] shrink-0">
                      <svg
                        className="block size-full"
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
                <button className="bg-[#1a1a1a] h-[35.994px] rounded-[8px] px-4 relative shrink-0 hover:bg-[#252525] transition-colors">
                  <div
                    aria-hidden="true"
                    className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[8px]"
                  />
                  <div className="flex items-center gap-2 h-full whitespace-nowrap">
                    <div className="size-[15.994px] shrink-0">
                      <svg
                        className="block size-full"
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
                    <p className="font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic text-[14px] text-white tracking-[-0.1504px]">
                      Sort
                    </p>
                  </div>
                </button>

                {/* Grid View Button */}
                <button className="bg-[#1a1a1a] box-border content-stretch flex h-[35.994px] items-center justify-center p-[0.909px] rounded-[8px] w-[41.804px] shrink-0 relative hover:bg-[#252525] transition-colors">
                  <div
                    aria-hidden="true"
                    className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[8px]"
                  />
                  <div className="shrink-0 size-[15.994px]">
                    <svg
                      className="block size-full"
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
      <div className="flex-1 w-full overflow-hidden">
        {activeTab === "overview" && <OverviewTab />}
        {activeTab === "workspace" && <WorkspaceTab />}
        {activeTab === "tasks" && <TasksTab />}
        {activeTab === "timeline" && <TimelineTab />}
        {activeTab === "manage" && <ManageTab />}
      </div>
    </div>
  );
}
