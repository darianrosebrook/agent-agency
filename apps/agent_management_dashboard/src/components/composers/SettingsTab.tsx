"use client";

import { useState } from "react";
import {
  GeneralTabContent,
  WorkHistoryTabContent,
  AIAgentsTabContent,
  TaskSettingsTabContent,
} from "./settings";

type ManageTabType = "general" | "workHistory" | "aiAgents" | "taskSettings";

function Heading() {
  return (
    <div
      className="content-stretch flex h-[35.994px] items-start relative shrink-0 w-full"
      data-name="Heading 1"
    >
      <p className="basis-0 font-['Inter:Regular',sans-serif] grow leading-[36px] min-h-px min-w-px not-italic relative shrink-0 text-[30px] text-white tracking-[0.3955px]">
        Project Settings
      </p>
    </div>
  );
}

function Paragraph() {
  return (
    <div
      className="h-[23.999px] relative shrink-0 w-full"
      data-name="Paragraph"
    >
      <p className="absolute font-['Inter:Regular',sans-serif] leading-[24px] left-0 not-italic text-[#888888] text-[16px] text-nowrap top-[-0.73px] tracking-[-0.3125px] whitespace-pre">
        Manage your project configuration and team
      </p>
    </div>
  );
}

function Container() {
  return (
    <div
      className="content-stretch flex flex-col gap-[7.997px] h-[67.99px] items-start relative shrink-0 w-full"
      data-name="Container"
    >
      <Heading />
      <Paragraph />
    </div>
  );
}

interface TabListProps {
  activeTab: ManageTabType;
  onTabChange: (tab: ManageTabType) => void;
}

function TabList({ activeTab, onTabChange }: TabListProps) {
  return (
    <div
      className="bg-[#1a1a1a] h-[35.994px] relative rounded-[14px] shrink-0 w-[370.462px]"
      data-name="Tab List"
    >
      <div
        aria-hidden="true"
        className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[14px]"
      />
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[35.994px] relative w-[370.462px]">
        <button
          onClick={() => onTabChange("general")}
          className={`absolute box-border content-stretch flex gap-[6px] h-[25.178px] items-center justify-center left-[4.91px] px-[8.909px] py-[4.909px] rounded-[14px] top-[5.41px] w-[69.098px] transition-colors ${
            activeTab === "general" ? "bg-[#2d2d2d]" : "hover:bg-[#252525]"
          }`}
        >
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-[rgba(0,0,0,0)] border-solid inset-0 pointer-events-none rounded-[14px]"
          />
          <p
            className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic relative shrink-0 text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre ${
              activeTab === "general" ? "text-white" : "text-[#cccccc]"
            }`}
          >
            General
          </p>
        </button>

        <button
          onClick={() => onTabChange("workHistory")}
          className={`absolute box-border content-stretch flex gap-[6px] h-[25.178px] items-center justify-center left-[74.01px] px-[8.909px] py-[4.909px] rounded-[14px] top-[5.41px] w-[103.587px] transition-colors ${
            activeTab === "workHistory" ? "bg-[#2d2d2d]" : "hover:bg-[#252525]"
          }`}
        >
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-[rgba(0,0,0,0)] border-solid inset-0 pointer-events-none rounded-[14px]"
          />
          <p
            className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic relative shrink-0 text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre ${
              activeTab === "workHistory" ? "text-white" : "text-[#cccccc]"
            }`}
          >
            Work History
          </p>
        </button>

        <button
          onClick={() => onTabChange("aiAgents")}
          className={`absolute box-border content-stretch flex gap-[6px] h-[25.178px] items-center justify-center left-[177.59px] px-[8.909px] py-[4.909px] rounded-[14px] top-[5.41px] w-[81.222px] transition-colors ${
            activeTab === "aiAgents" ? "bg-[#2d2d2d]" : "hover:bg-[#252525]"
          }`}
        >
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-[rgba(0,0,0,0)] border-solid inset-0 pointer-events-none rounded-[14px]"
          />
          <p
            className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic relative shrink-0 text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre ${
              activeTab === "aiAgents" ? "text-white" : "text-[#cccccc]"
            }`}
          >
            AI Agents
          </p>
        </button>

        <button
          onClick={() => onTabChange("taskSettings")}
          className={`absolute box-border content-stretch flex gap-[6px] h-[25.178px] items-center justify-center left-[258.81px] px-[8.909px] py-[4.909px] rounded-[14px] top-[5.41px] w-[106.74px] transition-colors ${
            activeTab === "taskSettings" ? "bg-[#2d2d2d]" : "hover:bg-[#252525]"
          }`}
        >
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-[rgba(0,0,0,0)] border-solid inset-0 pointer-events-none rounded-[14px]"
          />
          <p
            className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic relative shrink-0 text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre ${
              activeTab === "taskSettings" ? "text-white" : "text-[#cccccc]"
            }`}
          >
            Task Settings
          </p>
        </button>
      </div>
    </div>
  );
}

export function ManageTab() {
  const [activeTab, setActiveTab] = useState<ManageTabType>("general");

  return (
    <div className="h-full w-full overflow-hidden bg-[#0d0d0d]">
      <div className="h-full overflow-auto">
        <div className="box-border flex flex-col gap-[31.996px] items-start pb-0 pt-[31.996px] px-[31.996px] w-full">
          <Container />
          <div className="content-stretch flex flex-col gap-[31.996px] items-start relative shrink-0 w-full">
            <TabList activeTab={activeTab} onTabChange={setActiveTab} />

            {activeTab === "general" && <GeneralTabContent />}
            {activeTab === "workHistory" && <WorkHistoryTabContent />}
            {activeTab === "aiAgents" && <AIAgentsTabContent />}
            {activeTab === "taskSettings" && <TaskSettingsTabContent />}
          </div>
        </div>
      </div>
    </div>
  );
}
