import { useState } from "react";
import svgPaths from "../imports/svg-pj3tus7kw0";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { Switch } from "./ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { Separator } from "./ui/separator";
import { Slider } from "./ui/slider";

type ManageTabType =
  | "general"
  | "workHistory"
  | "aiAgents"
  | "taskSettings";

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
            activeTab === "general"
              ? "bg-[#2d2d2d]"
              : "hover:bg-[#252525]"
          }`}
        >
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-[rgba(0,0,0,0)] border-solid inset-0 pointer-events-none rounded-[14px]"
          />
          <p
            className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic relative shrink-0 text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre ${
              activeTab === "general"
                ? "text-white"
                : "text-[#cccccc]"
            }`}
          >
            General
          </p>
        </button>

        <button
          onClick={() => onTabChange("workHistory")}
          className={`absolute box-border content-stretch flex gap-[6px] h-[25.178px] items-center justify-center left-[74.01px] px-[8.909px] py-[4.909px] rounded-[14px] top-[5.41px] w-[103.587px] transition-colors ${
            activeTab === "workHistory"
              ? "bg-[#2d2d2d]"
              : "hover:bg-[#252525]"
          }`}
        >
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-[rgba(0,0,0,0)] border-solid inset-0 pointer-events-none rounded-[14px]"
          />
          <p
            className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic relative shrink-0 text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre ${
              activeTab === "workHistory"
                ? "text-white"
                : "text-[#cccccc]"
            }`}
          >
            Work History
          </p>
        </button>

        <button
          onClick={() => onTabChange("aiAgents")}
          className={`absolute box-border content-stretch flex gap-[6px] h-[25.178px] items-center justify-center left-[177.59px] px-[8.909px] py-[4.909px] rounded-[14px] top-[5.41px] w-[81.222px] transition-colors ${
            activeTab === "aiAgents"
              ? "bg-[#2d2d2d]"
              : "hover:bg-[#252525]"
          }`}
        >
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-[rgba(0,0,0,0)] border-solid inset-0 pointer-events-none rounded-[14px]"
          />
          <p
            className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic relative shrink-0 text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre ${
              activeTab === "aiAgents"
                ? "text-white"
                : "text-[#cccccc]"
            }`}
          >
            AI Agents
          </p>
        </button>

        <button
          onClick={() => onTabChange("taskSettings")}
          className={`absolute box-border content-stretch flex gap-[6px] h-[25.178px] items-center justify-center left-[258.81px] px-[8.909px] py-[4.909px] rounded-[14px] top-[5.41px] w-[106.74px] transition-colors ${
            activeTab === "taskSettings"
              ? "bg-[#2d2d2d]"
              : "hover:bg-[#252525]"
          }`}
        >
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-[rgba(0,0,0,0)] border-solid inset-0 pointer-events-none rounded-[14px]"
          />
          <p
            className={`font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic relative shrink-0 text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre ${
              activeTab === "taskSettings"
                ? "text-white"
                : "text-[#cccccc]"
            }`}
          >
            Task Settings
          </p>
        </button>
      </div>
    </div>
  );
}

function GeneralTabContent() {
  const [collaboration, setCollaboration] = useState(true);
  const [requireApproval, setRequireApproval] = useState(false);
  const [assignmentNotifs, setAssignmentNotifs] =
    useState(true);
  const [commentNotifs, setCommentNotifs] = useState(true);
  const [statusNotifs, setStatusNotifs] = useState(false);
  const [projectName, setProjectName] = useState(
    "My Kanban Project",
  );
  const [description, setDescription] = useState(
    "A project management tool with kanban boards and timeline tracking.",
  );

  return (
    <div
      className="h-[1352.6px] relative shrink-0 w-[1216.01px]"
      data-name="ProjectSettings"
    >
      <div className="bg-clip-padding border-0 border-[transparent] border-solid box-border h-[1352.6px] relative w-[1216.01px]">
        {/* Project Details Section */}
        <div
          className="absolute bg-[#1a1a1a] h-[381.754px] left-0 rounded-[10px] top-0 w-[1216.01px]"
          data-name="Container"
        >
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]"
          />
          <div
            className="absolute h-[27.997px] left-[24.91px] top-[24.91px] w-[1166.19px]"
            data-name="Heading 2"
          >
            <p className="absolute font-['Inter:Regular',sans-serif] leading-[28px] left-0 not-italic text-[20px] text-nowrap text-white top-[-0.18px] tracking-[-0.4492px] whitespace-pre">
              Project Details
            </p>
          </div>

          <div className="absolute content-stretch flex flex-col gap-[15.994px] h-[227.955px] items-start left-[24.91px] top-[68.9px] w-[1166.19px]">
            {/* Project Name */}
            <div className="content-stretch flex flex-col gap-[5.994px] h-[55.987px] items-start relative shrink-0 w-full">
              <div className="content-stretch flex gap-[8px] h-[13.999px] items-center relative shrink-0 w-full">
                <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] not-italic relative shrink-0 text-[#d1d5dc] text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre">
                  Project Name
                </p>
              </div>
              <input
                type="text"
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
                className="bg-[#0d0d0d] h-[35.994px] relative rounded-[8px] shrink-0 w-full border-[0.909px] border-neutral-800 px-[12px] py-[4px] font-['Inter:Regular',sans-serif] leading-[20px] text-[14px] text-white tracking-[-0.1504px] focus:outline-none focus:border-[#2d7ff9]"
              />
            </div>

            {/* Description */}
            <div className="content-stretch flex flex-col gap-[5.994px] h-[83.992px] items-start relative shrink-0 w-full">
              <div className="content-stretch flex gap-[8px] h-[13.999px] items-center relative shrink-0 w-full">
                <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] not-italic relative shrink-0 text-[#d1d5dc] text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre">
                  Description
                </p>
              </div>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="bg-[#0d0d0d] h-[63.999px] relative rounded-[8px] shrink-0 w-full border-[0.909px] border-neutral-800 px-[12.909px] py-[8.909px] font-['Inter:Regular',sans-serif] leading-[20px] text-[14px] text-white tracking-[-0.1504px] resize-none focus:outline-none focus:border-[#2d7ff9]"
              />
            </div>

            {/* Project ID and Created */}
            <div className="h-[55.987px] relative shrink-0 w-full">
              <div className="absolute content-stretch flex flex-col gap-[5.994px] h-[55.987px] items-start left-0 top-0 w-[575.099px]">
                <div className="content-stretch flex gap-[8px] h-[13.999px] items-center relative shrink-0 w-full">
                  <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] not-italic relative shrink-0 text-[#d1d5dc] text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre">
                    Project ID
                  </p>
                </div>
                <div className="bg-[#0d0d0d] h-[35.994px] opacity-50 relative rounded-[8px] shrink-0 w-full border-[0.909px] border-neutral-800 px-[12px] py-[4px] flex items-center">
                  <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[#888888] text-[14px] tracking-[-0.1504px]">
                    proj_8k2m9n4p
                  </p>
                </div>
              </div>

              <div className="absolute content-stretch flex flex-col gap-[5.994px] h-[55.987px] items-start left-[591.09px] top-0 w-[575.099px]">
                <div className="content-stretch flex gap-[8px] h-[13.999px] items-center relative shrink-0 w-full">
                  <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] not-italic relative shrink-0 text-[#d1d5dc] text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre">
                    Created
                  </p>
                </div>
                <div className="bg-[#0d0d0d] h-[35.994px] opacity-50 relative rounded-[8px] shrink-0 w-full border-[0.909px] border-neutral-800 px-[12px] py-[4px] flex items-center">
                  <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[#888888] text-[14px] tracking-[-0.1504px]">
                    November 1, 2024
                  </p>
                </div>
              </div>
            </div>
          </div>

          <button className="absolute bg-[#2d7ff9] box-border content-stretch flex gap-[8px] h-[35.994px] items-center justify-center left-[1065.52px] px-[16px] py-[8px] rounded-[8px] top-[320.85px] w-[125.582px] hover:bg-[#2570d9] transition-colors">
            <p className="font-['Inter:Medium',sans-serif] font-medium leading-[20px] not-italic relative shrink-0 text-[14px] text-nowrap text-white tracking-[-0.1504px] whitespace-pre">
              Save Changes
            </p>
          </button>
        </div>

        {/* Team Settings Section */}
        <div className="absolute bg-[#1a1a1a] box-border content-stretch flex flex-col gap-[15.994px] h-[301.74px] items-start left-0 pb-[0.909px] pt-[24.908px] px-[24.908px] rounded-[10px] top-[405.75px] w-[1216.01px]">
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]"
          />
          <div className="h-[27.997px] relative shrink-0 w-full">
            <p className="absolute font-['Inter:Regular',sans-serif] leading-[28px] left-0 not-italic text-[20px] text-nowrap text-white top-[-0.18px] tracking-[-0.4492px] whitespace-pre">
              Team Settings
            </p>
          </div>

          <div className="content-stretch flex flex-col gap-[15.994px] h-[207.933px] items-start relative shrink-0 w-full">
            {/* Default Assignee */}
            <div className="content-stretch flex flex-col gap-[5.994px] h-[55.987px] items-start relative shrink-0 w-full">
              <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                Default Assignee
              </p>
              <button className="bg-[#0d0d0d] h-[35.994px] relative rounded-[8px] shrink-0 w-full border-[0.909px] border-neutral-800 flex items-center justify-between px-[12.905px] hover:bg-[#1a1a1a] transition-colors">
                <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[14px] text-white tracking-[-0.1504px]">
                  Auto-assign
                </p>
                <div className="relative shrink-0 size-[15.994px]">
                  <svg
                    className="block size-full"
                    fill="none"
                    preserveAspectRatio="none"
                    viewBox="0 0 16 16"
                  >
                    <path
                      d={svgPaths.p10a02b40}
                      stroke="#717182"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                      opacity="0.5"
                    />
                  </svg>
                </div>
              </button>
            </div>

            {/* Team Collaboration Toggle */}
            <div className="content-stretch flex h-[59.979px] items-center justify-between relative shrink-0 w-full">
              <div className="h-[35.987px] relative shrink-0 w-[268.615px]">
                <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                  Allow team collaboration
                </p>
                <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[#888888] text-[14px] tracking-[-0.1504px] mt-[1.996px]">
                  Team members can edit tasks and boards
                </p>
              </div>
              <button
                onClick={() => setCollaboration(!collaboration)}
                className={`h-[18.395px] relative rounded-[1.5252e+07px] shrink-0 w-[31.996px] transition-colors ${
                  collaboration
                    ? "bg-[#030213]"
                    : "bg-[#cbced4]"
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    collaboration
                      ? "translate-x-[14.903px]"
                      : "translate-x-[0.909px]"
                  } mt-[0.909px] ${collaboration ? "ml-[0.909px]" : "ml-[0.909px]"}`}
                />
              </button>
            </div>

            {/* Require Approval Toggle */}
            <div className="content-stretch flex h-[59.979px] items-center justify-between relative shrink-0 w-full">
              <div className="h-[35.987px] relative shrink-0 w-[310.277px]">
                <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                  Require approval for done tasks
                </p>
                <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[#888888] text-[14px] tracking-[-0.1504px] mt-[1.996px]">
                  Tasks must be reviewed before marking as done
                </p>
              </div>
              <button
                onClick={() =>
                  setRequireApproval(!requireApproval)
                }
                className={`h-[18.395px] relative rounded-[1.5252e+07px] shrink-0 w-[31.996px] transition-colors ${
                  requireApproval
                    ? "bg-[#030213]"
                    : "bg-[#cbced4]"
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    requireApproval
                      ? "translate-x-[14.903px]"
                      : "translate-x-[0.909px]"
                  } mt-[0.909px] ${requireApproval ? "ml-[0.909px]" : "ml-[0.909px]"}`}
                />
              </button>
            </div>
          </div>
        </div>

        {/* Notifications Section */}
        <div className="absolute bg-[#1a1a1a] box-border content-stretch flex flex-col gap-[15.994px] h-[323.714px] items-start left-0 pb-[0.909px] pt-[24.908px] px-[24.908px] rounded-[10px] top-[731.49px] w-[1216.01px]">
          <div
            aria-hidden="true"
            className="absolute border-[0.909px] border-neutral-800 border-solid inset-0 pointer-events-none rounded-[10px]"
          />
          <div className="h-[27.997px] relative shrink-0 w-full">
            <p className="absolute font-['Inter:Regular',sans-serif] leading-[28px] left-0 not-italic text-[20px] text-nowrap text-white top-[-0.18px] tracking-[-0.4492px] whitespace-pre">
              Notifications
            </p>
          </div>

          <div className="h-[229.908px] relative shrink-0 w-full">
            {/* Task Assignments */}
            <div className="absolute content-stretch flex h-[59.979px] items-center justify-between left-0 top-0 w-[1166.19px]">
              <div className="h-[35.987px] relative shrink-0 w-[234.822px]">
                <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                  Task assignments
                </p>
                <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[#888888] text-[14px] tracking-[-0.1504px] mt-[1.996px]">
                  Get notified when assigned to a task
                </p>
              </div>
              <button
                onClick={() =>
                  setAssignmentNotifs(!assignmentNotifs)
                }
                className={`h-[18.395px] relative rounded-[1.5252e+07px] shrink-0 w-[31.996px] transition-colors ${
                  assignmentNotifs
                    ? "bg-[#030213]"
                    : "bg-[#cbced4]"
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    assignmentNotifs
                      ? "translate-x-[14.903px]"
                      : "translate-x-[0.909px]"
                  } mt-[0.909px] ${assignmentNotifs ? "ml-[0.909px]" : "ml-[0.909px]"}`}
                />
              </button>
            </div>

            <div className="absolute bg-neutral-800 h-[0.994px] left-0 top-[71.97px] w-[1166.19px]" />

            {/* Task Comments */}
            <div className="absolute content-stretch flex h-[59.979px] items-center justify-between left-0 top-[84.96px] w-[1166.19px]">
              <div className="h-[35.987px] relative shrink-0 w-[284.489px]">
                <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                  Task comments
                </p>
                <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[#888888] text-[14px] tracking-[-0.1504px] mt-[1.996px]">
                  Get notified of new comments on your tasks
                </p>
              </div>
              <button
                onClick={() => setCommentNotifs(!commentNotifs)}
                className={`h-[18.395px] relative rounded-[1.5252e+07px] shrink-0 w-[31.996px] transition-colors ${
                  commentNotifs
                    ? "bg-[#030213]"
                    : "bg-[#cbced4]"
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    commentNotifs
                      ? "translate-x-[14.903px]"
                      : "translate-x-[0.909px]"
                  } mt-[0.909px] ${commentNotifs ? "ml-[0.909px]" : "ml-[0.909px]"}`}
                />
              </button>
            </div>

            <div className="absolute bg-neutral-800 h-[0.994px] left-0 top-[156.94px] w-[1166.19px]" />

            {/* Status Changes */}
            <div className="absolute content-stretch flex h-[59.979px] items-center justify-between left-0 top-[169.93px] w-[1166.19px]">
              <div className="h-[35.987px] relative shrink-0 w-[247.23px]">
                <p className="font-['Inter:Medium',sans-serif] font-medium leading-[14px] text-[#d1d5dc] text-[14px] tracking-[-0.1504px]">
                  Status changes
                </p>
                <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[#888888] text-[14px] tracking-[-0.1504px] mt-[1.996px]">
                  Get notified when task status changes
                </p>
              </div>
              <button
                onClick={() => setStatusNotifs(!statusNotifs)}
                className={`h-[18.395px] relative rounded-[1.5252e+07px] shrink-0 w-[31.996px] transition-colors ${
                  statusNotifs ? "bg-[#030213]" : "bg-[#cbced4]"
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    statusNotifs
                      ? "translate-x-[14.903px]"
                      : "translate-x-[0.909px]"
                  } mt-[0.909px] ${statusNotifs ? "ml-[0.909px]" : "ml-[0.909px]"}`}
                />
              </button>
            </div>
          </div>
        </div>

        {/* Danger Zone Section */}
        <div className="absolute bg-[#1a1a1a] box-border content-stretch flex flex-col gap-[15.994px] h-[273.395px] items-start left-0 pb-[0.909px] pt-[24.908px] px-[24.908px] rounded-[10px] top-[1079.2px] w-[1216.01px]">
          <div
            aria-hidden="true"
            className="absolute border-[#5c1515] border-[0.909px] border-solid inset-0 pointer-events-none rounded-[10px]"
          />
          <div className="h-[27.997px] relative shrink-0 w-full">
            <div className="absolute left-0 size-[20px] top-[4px]">
              <svg
                className="block size-full"
                fill="none"
                preserveAspectRatio="none"
                viewBox="0 0 20 20"
              >
                <path
                  d={svgPaths.p14d24500}
                  stroke="#FF6B6B"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="1.66667"
                />
                <path
                  d="M10 6.66667V10"
                  stroke="#FF6B6B"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="1.66667"
                />
                <path
                  d="M10 13.3333H10.0083"
                  stroke="#FF6B6B"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="1.66667"
                />
              </svg>
            </div>
            <p className="absolute font-['Inter:Regular',sans-serif] leading-[28px] left-[28px] not-italic text-[#ff6b6b] text-[20px] text-nowrap top-[-0.18px] tracking-[-0.4492px] whitespace-pre">
              Danger Zone
            </p>
          </div>

          <div className="content-stretch flex flex-col gap-[15.994px] h-[179.588px] items-start relative shrink-0 w-full">
            {/* Archive Project */}
            <div className="h-[81.797px] relative rounded-[10px] shrink-0 w-full border-[0.909px] border-neutral-800 flex items-center justify-between px-[16.903px]">
              <div className="h-[47.99px] relative shrink-0 w-[384.631px]">
                <p className="font-['Inter:Regular',sans-serif] leading-[24px] text-[16px] text-white tracking-[-0.3125px]">
                  Archive this project
                </p>
                <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[#888888] text-[14px] tracking-[-0.1504px] mt-[3.999px]">
                  Make the project read-only and hide it from
                  your dashboard
                </p>
              </div>
              <button className="bg-zinc-900 h-[35.994px] rounded-[8px] w-[83.537px] border-[#5c1515] border-[0.909px] flex items-center justify-center hover:bg-gray-100 transition-colors">
                <p className="font-['Inter:Medium',sans-serif] font-medium leading-[20px] text-[#ff6b6b] text-[14px] tracking-[-0.1504px]">
                  Archive
                </p>
              </button>
            </div>

            {/* Delete Project */}
            <div className="h-[81.797px] relative rounded-[10px] shrink-0 w-full border-[#5c1515] border-[0.909px] flex items-center justify-between px-[16.903px]">
              <div className="h-[47.99px] relative shrink-0 w-[315.646px]">
                <p className="font-['Inter:Regular',sans-serif] leading-[24px] text-[16px] text-white tracking-[-0.3125px]">
                  Delete this project
                </p>
                <p className="font-['Inter:Regular',sans-serif] leading-[20px] text-[#888888] text-[14px] tracking-[-0.1504px] mt-[3.999px]">
                  Permanently delete this project and all of its
                  data
                </p>
              </div>
              <button className="bg-white h-[35.994px] rounded-[8px] w-[100.547px] border-[#5c1515] border-[0.909px] flex items-center justify-center gap-[8px] hover:bg-gray-100 transition-colors">
                <div className="relative size-[15.994px]">
                  <svg
                    className="block size-full"
                    fill="none"
                    preserveAspectRatio="none"
                    viewBox="0 0 16 16"
                  >
                    <path
                      d="M6.6643 7.33073V11.3293"
                      stroke="#FF6B6B"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d="M9.33002 7.33073V11.3293"
                      stroke="#FF6B6B"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d={svgPaths.p1c811700}
                      stroke="#FF6B6B"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d="M1.99929 3.99858H13.995"
                      stroke="#FF6B6B"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                    <path
                      d={svgPaths.p346ee160}
                      stroke="#FF6B6B"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="1.33286"
                    />
                  </svg>
                </div>
                <p className="font-['Inter:Medium',sans-serif] font-medium leading-[20px] text-[#ff6b6b] text-[14px] tracking-[-0.1504px]">
                  Delete
                </p>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function WorkHistoryTabContent() {
  return (
    <div className="min-h-[600px] p-[31.996px]">
      <div className="bg-[#1a1a1a] rounded-[10px] p-[24.908px] border-[0.909px] border-neutral-800">
        <h2 className="font-['Inter:Regular',sans-serif] text-[20px] leading-[28px] text-white tracking-[-0.4492px] mb-[15.994px]">
          Work History
        </h2>
        <p className="font-['Inter:Regular',sans-serif] text-[14px] leading-[20px] text-[#888888] tracking-[-0.1504px]">
          View and analyze your team's work history, time
          tracking, and productivity metrics.
        </p>
        <div className="mt-[31.996px] grid grid-cols-3 gap-[15.994px]">
          {[
            "Total Tasks",
            "Completed This Week",
            "Average Completion Time",
          ].map((metric, i) => (
            <div
              key={i}
              className="bg-[#0d0d0d] rounded-[8px] p-[16px] border-[0.909px] border-neutral-800"
            >
              <p className="font-['Inter:Medium',sans-serif] text-[14px] text-[#d1d5dc] tracking-[-0.1504px] mb-[8px]">
                {metric}
              </p>
              <p className="font-['Inter:Regular',sans-serif] text-[24px] text-white">
                {i === 0 ? "127" : i === 1 ? "23" : "2.3 days"}
              </p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function AIAgentsTabContent() {
  return (
    <div className="min-h-[600px] p-[31.996px]">
      <div className="bg-[#1a1a1a] rounded-[10px] p-[24.908px] border-[0.909px] border-neutral-800">
        <h2 className="font-['Inter:Regular',sans-serif] text-[20px] leading-[28px] text-white tracking-[-0.4492px] mb-[15.994px]">
          AI Agents
        </h2>
        <p className="font-['Inter:Regular',sans-serif] text-[14px] leading-[20px] text-[#888888] tracking-[-0.1504px] mb-[31.996px]">
          Configure AI agents to automate tasks and provide
          intelligent assistance.
        </p>

        <div className="space-y-[15.994px]">
          {[
            {
              name: "Task Suggester",
              description:
                "Automatically suggests task breakdowns and subtasks",
              enabled: true,
            },
            {
              name: "Priority Optimizer",
              description:
                "Analyzes and recommends task prioritization",
              enabled: true,
            },
            {
              name: "Deadline Predictor",
              description:
                "Estimates realistic completion dates based on history",
              enabled: false,
            },
          ].map((agent, i) => (
            <div
              key={i}
              className="bg-[#0d0d0d] rounded-[8px] p-[16px] border-[0.909px] border-neutral-800 flex items-center justify-between"
            >
              <div>
                <p className="font-['Inter:Medium',sans-serif] text-[14px] text-white tracking-[-0.1504px] mb-[4px]">
                  {agent.name}
                </p>
                <p className="font-['Inter:Regular',sans-serif] text-[14px] text-[#888888] tracking-[-0.1504px]">
                  {agent.description}
                </p>
              </div>
              <div
                className={`h-[18.395px] w-[31.996px] rounded-[1.5252e+07px] ${agent.enabled ? "bg-[#030213]" : "bg-[#cbced4]"}`}
              >
                <div
                  className={`bg-white rounded-[1.5252e+07px] size-[15.994px] transition-transform ${
                    agent.enabled
                      ? "translate-x-[14.903px]"
                      : "translate-x-[0.909px]"
                  } mt-[0.909px]`}
                />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function TaskSettingsTabContent() {
  return (
    <div className="space-y-6">
      {/* Task Workflow */}
      <div className="bg-[#1a1a1a] rounded-lg border border-[#262626] p-6">
        <h2 className="text-white text-xl mb-4">
          Task Workflow
        </h2>

        <div className="space-y-4">
          <div>
            <Label
              htmlFor="default-status"
              className="text-[#d1d5dc]"
            >
              Default Status for New Tasks
            </Label>
            <Select defaultValue="todo">
              <SelectTrigger className="mt-1.5 bg-[#0d0d0d] border-[#262626] text-white">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="bg-[#1a1a1a] border-[#262626]">
                <SelectItem value="todo">To Do</SelectItem>
                <SelectItem value="backlog">Backlog</SelectItem>
                <SelectItem value="draft">Draft</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">
                Auto-archive completed tasks
              </Label>
              <p className="text-sm text-[#888888]">
                Archive tasks 30 days after completion
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className="bg-[#262626]" />

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">
                Enable task dependencies
              </Label>
              <p className="text-sm text-[#888888]">
                Tasks can block other tasks from starting
              </p>
            </div>
            <Switch />
          </div>

          <Separator className="bg-[#262626]" />

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">
                Require task descriptions
              </Label>
              <p className="text-sm text-[#888888]">
                Force users to add descriptions to new tasks
              </p>
            </div>
            <Switch />
          </div>
        </div>
      </div>

      {/* Priority Settings */}
      <div className="bg-[#1a1a1a] rounded-lg border border-[#262626] p-6">
        <h2 className="text-white text-xl mb-4">
          Priority & Labels
        </h2>

        <div className="space-y-4">
          <div>
            <Label
              htmlFor="priority-levels"
              className="text-[#d1d5dc]"
            >
              Priority Levels
            </Label>
            <Select defaultValue="4">
              <SelectTrigger className="mt-1.5 bg-[#0d0d0d] border-[#262626] text-white">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="bg-[#1a1a1a] border-[#262626]">
                <SelectItem value="3">
                  3 levels (Low, Medium, High)
                </SelectItem>
                <SelectItem value="4">
                  4 levels (Low, Medium, High, Critical)
                </SelectItem>
                <SelectItem value="5">
                  5 levels (Very Low to Critical)
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">
                Auto-assign priority
              </Label>
              <p className="text-sm text-[#888888]">
                AI suggests priority based on task content
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className="bg-[#262626]" />

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">
                Limit tags per task
              </Label>
              <p className="text-sm text-[#888888]">
                Maximum number of tags allowed
              </p>
            </div>
            <div className="flex items-center gap-3">
              <Input
                type="number"
                defaultValue="5"
                className="w-20 bg-[#0d0d0d] border-[#262626] text-white text-center"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Time Tracking */}
      <div className="bg-[#1a1a1a] rounded-lg border border-[#262626] p-6">
        <h2 className="text-white text-xl mb-4">
          Time Tracking
        </h2>

        <div className="space-y-4">
          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">
                Enable time tracking
              </Label>
              <p className="text-sm text-[#888888]">
                Track time spent on tasks
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className="bg-[#262626]" />

          <div>
            <Label className="text-[#d1d5dc] mb-3 block">
              Estimated time alerts
            </Label>
            <p className="text-sm text-[#888888] mb-3">
              Alert when task exceeds estimated time by:
            </p>
            <div className="flex items-center gap-4">
              <Slider
                defaultValue={[50]}
                max={100}
                step={10}
                className="flex-1"
              />
              <span className="text-white w-12 text-right">
                50%
              </span>
            </div>
          </div>

          <Separator className="bg-[#262626]" />

          <div>
            <Label
              htmlFor="work-hours"
              className="text-[#d1d5dc]"
            >
              Standard Work Hours
            </Label>
            <Select defaultValue="8">
              <SelectTrigger className="mt-1.5 bg-[#0d0d0d] border-[#262626] text-white">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="bg-[#1a1a1a] border-[#262626]">
                <SelectItem value="6">6 hours/day</SelectItem>
                <SelectItem value="8">8 hours/day</SelectItem>
                <SelectItem value="10">10 hours/day</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      {/* Automation */}
      <div className="bg-[#1a1a1a] rounded-lg border border-[#262626] p-6">
        <h2 className="text-white text-xl mb-4">Automation</h2>

        <div className="space-y-4">
          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">
                Auto-move stale tasks
              </Label>
              <p className="text-sm text-[#888888]">
                Move tasks stuck in "In Progress" for 7+ days
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className="bg-[#262626]" />

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">
                Smart task distribution
              </Label>
              <p className="text-sm text-[#888888]">
                AI distributes tasks based on team capacity
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className="bg-[#262626]" />

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">
                Deadline reminders
              </Label>
              <p className="text-sm text-[#888888]">
                Send reminders 24h before deadline
              </p>
            </div>
            <Switch defaultChecked />
          </div>
        </div>
      </div>
    </div>
  );
}

export function ManageTab() {
  const [activeTab, setActiveTab] =
    useState<ManageTabType>("general");

  return (
    <div className="h-full w-full overflow-hidden bg-[#0d0d0d]">
      <div className="h-full overflow-auto">
        <div className="box-border flex flex-col gap-[31.996px] items-start pb-0 pt-[31.996px] px-[31.996px] w-full">
          <Container />
          <div className="content-stretch flex flex-col gap-[31.996px] items-start relative shrink-0 w-full">
            <TabList
              activeTab={activeTab}
              onTabChange={setActiveTab}
            />

            {activeTab === "general" && <GeneralTabContent />}
            {activeTab === "workHistory" && (
              <WorkHistoryTabContent />
            )}
            {activeTab === "aiAgents" && <AIAgentsTabContent />}
            {activeTab === "taskSettings" && (
              <TaskSettingsTabContent />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}