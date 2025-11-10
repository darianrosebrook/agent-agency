'use client';

import { useState } from 'react';
import svgPaths from '../../../imports/svg-pj3tus7kw0';

export function GeneralTabContent() {
  const [collaboration, setCollaboration] = useState(true);
  const [requireApproval, setRequireApproval] = useState(false);
  const [assignmentNotifs, setAssignmentNotifs] = useState(true);
  const [commentNotifs, setCommentNotifs] = useState(true);
  const [statusNotifs, setStatusNotifs] = useState(false);
  // TODO: Replace hardcoded project data with data from v3 database with the following requirements:
  // 1. Project data fetching: Load current project details from database
  //    - Data source: GET /api/projects/:projectId endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `projects` table
  //    - Include project name, description, ID, created date, and settings
  // 2. Project settings persistence: Save project settings updates to database
  //    - Data source: PATCH /api/projects/:projectId endpoint to update project details
  //    - Update project name, description, and notification preferences
  //    - Persist collaboration settings and approval requirements
  // 3. Project metadata display: Show project ID and creation date
  //    - Display project ID from database (read-only)
  //    - Format and display created_at timestamp from database
  //    - Show last updated timestamp if available
  // 4. Settings persistence: Save notification and collaboration preferences
  //    - Data source: PATCH /api/projects/:projectId/settings endpoint
  //    - Store notification preferences (assignment, comment, status)
  //    - Store collaboration and approval settings
  const [projectName, setProjectName] = useState('My Kanban Project');
  const [description, setDescription] = useState(
    'A project management tool with kanban boards and timeline tracking.'
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
                  {/* TODO: Replace hardcoded project ID with project.id from v3 database */}
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
                  {/* TODO: Replace hardcoded created date with project.created_at from v3 database, formatted as readable date */}
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
            {/* TODO: Replace hardcoded "Auto-assign" with user selection dropdown from v3 database with the following requirements:
            // 1. User list fetching: Load project team members from database
            //    - Data source: GET /api/projects/:projectId/members endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
            //    - Database table: PostgreSQL `project_members` or `users` table with project membership
            //    - Include user names, IDs, and avatars
            // 2. Default assignee selection: Allow selecting default assignee for new tasks
            //    - Data source: PATCH /api/projects/:projectId/settings endpoint to update default_assignee_id
            //    - Store selected user ID as default assignee
            //    - Support "Auto-assign" option (round-robin or load-based)
            // 3. User display: Show selected user name and avatar in dropdown
            //    - Display user avatar and name when user is selected
            //    - Show "Auto-assign" option when no specific user is selected
            */}
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
                  collaboration ? 'bg-[#030213]' : 'bg-[#cbced4]'
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    collaboration
                      ? 'translate-x-[14.903px]'
                      : 'translate-x-[0.909px]'
                  } mt-[0.909px] ${
                    collaboration ? 'ml-[0.909px]' : 'ml-[0.909px]'
                  }`}
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
                onClick={() => setRequireApproval(!requireApproval)}
                className={`h-[18.395px] relative rounded-[1.5252e+07px] shrink-0 w-[31.996px] transition-colors ${
                  requireApproval ? 'bg-[#030213]' : 'bg-[#cbced4]'
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    requireApproval
                      ? 'translate-x-[14.903px]'
                      : 'translate-x-[0.909px]'
                  } mt-[0.909px] ${
                    requireApproval ? 'ml-[0.909px]' : 'ml-[0.909px]'
                  }`}
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
                onClick={() => setAssignmentNotifs(!assignmentNotifs)}
                className={`h-[18.395px] relative rounded-[1.5252e+07px] shrink-0 w-[31.996px] transition-colors ${
                  assignmentNotifs ? 'bg-[#030213]' : 'bg-[#cbced4]'
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    assignmentNotifs
                      ? 'translate-x-[14.903px]'
                      : 'translate-x-[0.909px]'
                  } mt-[0.909px] ${
                    assignmentNotifs ? 'ml-[0.909px]' : 'ml-[0.909px]'
                  }`}
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
                  commentNotifs ? 'bg-[#030213]' : 'bg-[#cbced4]'
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    commentNotifs
                      ? 'translate-x-[14.903px]'
                      : 'translate-x-[0.909px]'
                  } mt-[0.909px] ${
                    commentNotifs ? 'ml-[0.909px]' : 'ml-[0.909px]'
                  }`}
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
                  statusNotifs ? 'bg-[#030213]' : 'bg-[#cbced4]'
                }`}
              >
                <div
                  className={`bg-white relative rounded-[1.5252e+07px] shrink-0 size-[15.994px] transition-transform ${
                    statusNotifs
                      ? 'translate-x-[14.903px]'
                      : 'translate-x-[0.909px]'
                  } mt-[0.909px] ${
                    statusNotifs ? 'ml-[0.909px]' : 'ml-[0.909px]'
                  }`}
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
                  Make the project read-only and hide it from your dashboard
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
                  Permanently delete this project and all of its data
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

