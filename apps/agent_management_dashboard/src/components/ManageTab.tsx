"use client";

import { useState } from "react";
import svgPaths from "../imports/svg-pj3tus7kw0";
import { Input } from "./primitives/input";
import { Label } from "./primitives/label";
import { Switch } from "./primitives/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./primitives/select";
import { Separator } from "./primitives/separator";
import { Slider } from "./primitives/slider";
import { cn } from "./primitives/utils";
import { KanbanHeading } from "./primitives/kanban/KanbanHeading";
import { KanbanText } from "./primitives/kanban/KanbanText";
import styles from "./ManageTab.module.scss";

type ManageTabType = "general" | "workHistory" | "aiAgents" | "taskSettings";

interface TabItem {
  id: ManageTabType;
  label: string;
}

const TAB_ITEMS: TabItem[] = [
  { id: "general", label: "General" },
  { id: "workHistory", label: "Work History" },
  { id: "aiAgents", label: "AI Agents" },
  { id: "taskSettings", label: "Task Settings" },
];

function GeneralTabContent() {
  const [collaboration, setCollaboration] = useState(true);
  const [requireApproval, setRequireApproval] = useState(false);
  const [assignmentNotifs, setAssignmentNotifs] = useState(true);
  const [commentNotifs, setCommentNotifs] = useState(true);
  const [statusNotifs, setStatusNotifs] = useState(false);
  const [projectName, setProjectName] = useState("My Kanban Project");
  const [description, setDescription] = useState(
    "A project management tool with kanban boards and timeline tracking."
  );

  return (
    <div className={styles.generalTabContent}>
      {/* Project Details Section */}
      <div className={styles.settingsSection}>
        <div aria-hidden="true" className={styles.settingsSectionBorder} />
        <KanbanHeading className={styles.sectionTitle}>
          Project Details
        </KanbanHeading>

        <div className={styles.sectionContent}>
          {/* Project Name */}
          <div className={styles.formGroup}>
            <Label className={styles.formLabel}>
              <KanbanText size="14" className={styles.formLabelText}>
                Project Name
              </KanbanText>
            </Label>
            <input
              type="text"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value)}
              className={styles.formInput}
            />
          </div>

          {/* Description */}
          <div className={styles.formGroup}>
            <Label className={styles.formLabel}>
              <KanbanText size="14" className={styles.formLabelText}>
                Description
              </KanbanText>
            </Label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className={styles.formTextarea}
            />
          </div>

          {/* Project ID and Created */}
          <div className={styles.formRow}>
            <div className={styles.formGroup}>
              <Label className={styles.formLabel}>
                <KanbanText size="14" className={styles.formLabelText}>
                  Project ID
                </KanbanText>
              </Label>
              <div className={cn(styles.formInputDisabled, styles.formInput)}>
                <KanbanText size="14" className={styles.formInputDisabledText}>
                  proj_8k2m9n4p
                </KanbanText>
              </div>
            </div>

            <div className={styles.formGroup}>
              <Label className={styles.formLabel}>
                <KanbanText size="14" className={styles.formLabelText}>
                  Created
                </KanbanText>
              </Label>
              <div className={cn(styles.formInputDisabled, styles.formInput)}>
                <KanbanText size="14" className={styles.formInputDisabledText}>
                  November 1, 2024
                </KanbanText>
              </div>
            </div>
          </div>
        </div>

        <button className={styles.saveButton} type="button">
          <KanbanText size="14" className={styles.saveButtonText}>
            Save Changes
          </KanbanText>
        </button>
      </div>

      {/* Team Settings Section */}
      <div className={styles.settingsSection}>
        <div aria-hidden="true" className={styles.settingsSectionBorder} />
        <KanbanHeading size="lg" className={styles.sectionTitle}>
          Team Settings
        </KanbanHeading>

        <div className={styles.sectionContent}>
          {/* Default Assignee */}
          <div className={styles.formGroup}>
            <KanbanText size="14" className={styles.formLabelText}>
              Default Assignee
            </KanbanText>
            <button className={styles.dropdownButton} type="button">
              <KanbanText size="14" className={styles.dropdownButtonText}>
                Auto-assign
              </KanbanText>
              <div className={styles.dropdownButtonIcon}>
                <svg className={styles.svgIcon} fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
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
          <div className={styles.toggleRow}>
            <div className={styles.toggleRowContent}>
              <KanbanText size="14" className={styles.toggleRowTitle}>
                Allow team collaboration
              </KanbanText>
              <KanbanText size="xs" className={styles.toggleRowDescription}>
                Team members can edit tasks and boards
              </KanbanText>
            </div>
            <button
              onClick={() => setCollaboration(!collaboration)}
              className={cn(
                styles.toggleSwitch,
                collaboration ? styles.toggleSwitchActive : styles.toggleSwitchInactive
              )}
              type="button"
            >
              <div
                className={cn(
                  styles.toggleSwitchThumb,
                  collaboration ? styles.toggleSwitchThumbActive : styles.toggleSwitchThumbInactive
                )}
              />
            </button>
          </div>

          {/* Require Approval Toggle */}
          <div className={styles.toggleRow}>
            <div className={styles.toggleRowContent}>
              <KanbanText size="14" className={styles.toggleRowTitle}>
                Require approval for done tasks
              </KanbanText>
              <KanbanText size="xs" className={styles.toggleRowDescription}>
                Tasks must be reviewed before marking as done
              </KanbanText>
            </div>
            <button
              onClick={() => setRequireApproval(!requireApproval)}
              className={cn(
                styles.toggleSwitch,
                requireApproval ? styles.toggleSwitchActive : styles.toggleSwitchInactive
              )}
              type="button"
            >
              <div
                className={cn(
                  styles.toggleSwitchThumb,
                  requireApproval ? styles.toggleSwitchThumbActive : styles.toggleSwitchThumbInactive
                )}
              />
            </button>
          </div>
        </div>
      </div>

      {/* Notifications Section */}
      <div className={styles.settingsSection}>
        <div aria-hidden="true" className={styles.settingsSectionBorder} />
        <KanbanHeading size="lg" className={styles.sectionTitle}>
          Notifications
        </KanbanHeading>

        <div className={styles.sectionContent}>
          {/* Task Assignments */}
          <div className={styles.toggleRow}>
            <div className={styles.toggleRowContent}>
              <KanbanText size="14" className={styles.toggleRowTitle}>
                Task assignments
              </KanbanText>
              <KanbanText size="xs" className={styles.toggleRowDescription}>
                Get notified when assigned to a task
              </KanbanText>
            </div>
            <button
              onClick={() => setAssignmentNotifs(!assignmentNotifs)}
              className={cn(
                styles.toggleSwitch,
                assignmentNotifs ? styles.toggleSwitchActive : styles.toggleSwitchInactive
              )}
              type="button"
            >
              <div
                className={cn(
                  styles.toggleSwitchThumb,
                  assignmentNotifs ? styles.toggleSwitchThumbActive : styles.toggleSwitchThumbInactive
                )}
              />
            </button>
          </div>

          <Separator className={styles.divider} />

          {/* Task Comments */}
          <div className={styles.toggleRow}>
            <div className={styles.toggleRowContent}>
              <KanbanText size="14" className={styles.toggleRowTitle}>
                Task comments
              </KanbanText>
              <KanbanText size="xs" className={styles.toggleRowDescription}>
                Get notified of new comments on your tasks
              </KanbanText>
            </div>
            <button
              onClick={() => setCommentNotifs(!commentNotifs)}
              className={cn(
                styles.toggleSwitch,
                commentNotifs ? styles.toggleSwitchActive : styles.toggleSwitchInactive
              )}
              type="button"
            >
              <div
                className={cn(
                  styles.toggleSwitchThumb,
                  commentNotifs ? styles.toggleSwitchThumbActive : styles.toggleSwitchThumbInactive
                )}
              />
            </button>
          </div>

          <Separator className={styles.divider} />

          {/* Status Changes */}
          <div className={styles.toggleRow}>
            <div className={styles.toggleRowContent}>
              <KanbanText size="14" className={styles.toggleRowTitle}>
                Status changes
              </KanbanText>
              <KanbanText size="xs" className={styles.toggleRowDescription}>
                Get notified when task status changes
              </KanbanText>
            </div>
            <button
              onClick={() => setStatusNotifs(!statusNotifs)}
              className={cn(
                styles.toggleSwitch,
                statusNotifs ? styles.toggleSwitchActive : styles.toggleSwitchInactive
              )}
              type="button"
            >
              <div
                className={cn(
                  styles.toggleSwitchThumb,
                  statusNotifs ? styles.toggleSwitchThumbActive : styles.toggleSwitchThumbInactive
                )}
              />
            </button>
          </div>
        </div>
      </div>

      {/* Danger Zone Section */}
      <div className={styles.dangerZone}>
        <div aria-hidden="true" className={styles.dangerZoneBorder} />
        <div className={styles.dangerZoneTitle}>
          <div className={styles.dangerZoneIcon}>
            <svg className={styles.svgIcon} fill="none" preserveAspectRatio="none" viewBox="0 0 20 20">
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
          <KanbanHeading size="lg" className={styles.dangerZoneTitleText}>
            Danger Zone
          </KanbanHeading>
        </div>

        <div className={styles.dangerZoneActions}>
          {/* Archive Project */}
          <div className={styles.dangerZoneActionCard}>
            <div className={styles.dangerZoneActionInfo}>
              <KanbanText size="14" className={styles.dangerZoneActionTitle}>
                Archive this project
              </KanbanText>
              <KanbanText size="xs" className={styles.dangerZoneActionDescription}>
                Make the project read-only and hide it from your dashboard
              </KanbanText>
            </div>
            <button className={cn(styles.dangerZoneButton, styles.dangerZoneButtonArchive)} type="button">
              <KanbanText size="14" className={styles.dangerZoneButtonText}>
                Archive
              </KanbanText>
            </button>
          </div>

          {/* Delete Project */}
          <div className={cn(styles.dangerZoneActionCard, styles.dangerZoneActionCardDanger)}>
            <div className={styles.dangerZoneActionInfo}>
              <KanbanText size="14" className={styles.dangerZoneActionTitle}>
                Delete this project
              </KanbanText>
              <KanbanText size="xs" className={styles.dangerZoneActionDescription}>
                Permanently delete this project and all of its data
              </KanbanText>
            </div>
            <button className={cn(styles.dangerZoneButton, styles.dangerZoneButtonDelete)} type="button">
              <div className={styles.dangerZoneButtonIcon}>
                <svg className={styles.svgIcon} fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
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
              <KanbanText size="14" className={styles.dangerZoneButtonText}>
                Delete
              </KanbanText>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function WorkHistoryTabContent() {
  return (
    <div className={styles.workHistoryTab}>
      <div className={styles.workHistoryCard}>
        <KanbanHeading size="lg" className={styles.workHistoryTitle}>
          Work History
        </KanbanHeading>
        <KanbanText size="14" className={styles.workHistoryDescription}>
          View and analyze your team&apos;s work history, time tracking, and productivity metrics.
        </KanbanText>
        <div className={styles.workHistoryMetrics}>
          {[
            { label: "Total Tasks", value: "127" },
            { label: "Completed This Week", value: "23" },
            { label: "Average Completion Time", value: "2.3 days" },
          ].map((metric, i) => (
            <div key={i} className={styles.workHistoryMetricCard}>
              <KanbanText size="xs" className={styles.workHistoryMetricLabel}>
                {metric.label}
              </KanbanText>
              <KanbanText size="lg" className={styles.workHistoryMetricValue}>
                {metric.value}
              </KanbanText>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function AIAgentsTabContent() {
  return (
    <div className={styles.aiAgentsTab}>
      <div className={styles.aiAgentsCard}>
        <KanbanHeading size="lg" className={styles.aiAgentsTitle}>
          AI Agents
        </KanbanHeading>
        <KanbanText size="14" className={styles.aiAgentsDescription}>
          Configure AI agents to automate tasks and provide intelligent assistance.
        </KanbanText>

        <div className={styles.aiAgentsList}>
          {[
            {
              name: "Task Suggester",
              description: "Automatically suggests task breakdowns and subtasks",
              enabled: true,
            },
            {
              name: "Priority Optimizer",
              description: "Analyzes and recommends task prioritization",
              enabled: true,
            },
            {
              name: "Deadline Predictor",
              description: "Estimates realistic completion dates based on history",
              enabled: false,
            },
          ].map((agent, i) => (
            <div key={i} className={styles.aiAgentCard}>
              <div className={styles.aiAgentInfo}>
                <KanbanText size="14" className={styles.aiAgentName}>
                  {agent.name}
                </KanbanText>
                <KanbanText size="xs" className={styles.aiAgentDescription}>
                  {agent.description}
                </KanbanText>
              </div>
              <button
                className={cn(
                  styles.toggleSwitch,
                  agent.enabled ? styles.toggleSwitchActive : styles.toggleSwitchInactive
                )}
                type="button"
              >
                <div
                  className={cn(
                    styles.toggleSwitchThumb,
                    agent.enabled ? styles.toggleSwitchThumbActive : styles.toggleSwitchThumbInactive
                  )}
                />
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function TaskSettingsTabContent() {
  return (
    <div className={styles.taskSettingsTab}>
      {/* Task Workflow */}
      <div className={styles.settingsCard}>
        <KanbanHeading size="lg" className={styles.cardTitle}>Task Workflow</KanbanHeading>

        <div className={styles.settingsGroup}>
          <div>
            <Label htmlFor="default-status" className={styles.label}>
              <KanbanText size="14">Default Status for New Tasks</KanbanText>
            </Label>
            <Select defaultValue="todo">
              <SelectTrigger className={styles.selectTrigger}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent className={styles.selectContent}>
                <SelectItem value="todo">To Do</SelectItem>
                <SelectItem value="backlog">Backlog</SelectItem>
                <SelectItem value="draft">Draft</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                <KanbanText size="14">Auto-archive completed tasks</KanbanText>
              </Label>
              <KanbanText size="xs" className={styles.settingDescription}>
                Archive tasks 30 days after completion
              </KanbanText>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                <KanbanText size="14">Enable task dependencies</KanbanText>
              </Label>
              <KanbanText size="xs" className={styles.settingDescription}>
                Tasks can block other tasks from starting
              </KanbanText>
            </div>
            <Switch />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                <KanbanText size="14">Require task descriptions</KanbanText>
              </Label>
              <KanbanText size="xs" className={styles.settingDescription}>
                Force users to add descriptions to new tasks
              </KanbanText>
            </div>
            <Switch />
          </div>
        </div>
      </div>

      {/* Priority Settings */}
      <div className={styles.settingsCard}>
        <KanbanHeading size="lg" className={styles.cardTitle}>Priority & Labels</KanbanHeading>

        <div className={styles.settingsGroup}>
          <div>
            <Label htmlFor="priority-levels" className={styles.label}>
              <KanbanText size="14">Priority Levels</KanbanText>
            </Label>
            <Select defaultValue="4">
              <SelectTrigger className={styles.selectTrigger}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent className={styles.selectContent}>
                <SelectItem value="3">3 levels (Low, Medium, High)</SelectItem>
                <SelectItem value="4">4 levels (Low, Medium, High, Critical)</SelectItem>
                <SelectItem value="5">5 levels (Very Low to Critical)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                <KanbanText size="14">Auto-assign priority</KanbanText>
              </Label>
              <KanbanText size="xs" className={styles.settingDescription}>
                AI suggests priority based on task content
              </KanbanText>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                <KanbanText size="14">Limit tags per task</KanbanText>
              </Label>
              <KanbanText size="xs" className={styles.settingDescription}>
                Maximum number of tags allowed
              </KanbanText>
            </div>
            <div className={styles.sliderContainer}>
              <Input type="number" defaultValue="5" className={styles.numberInput} />
            </div>
          </div>
        </div>
      </div>

      {/* Time Tracking */}
      <div className={styles.settingsCard}>
        <KanbanHeading size="lg" className={styles.cardTitle}>Time Tracking</KanbanHeading>

        <div className={styles.settingsGroup}>
          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                <KanbanText size="14">Enable time tracking</KanbanText>
              </Label>
              <KanbanText size="xs" className={styles.settingDescription}>
                Track time spent on tasks
              </KanbanText>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div>
            <Label className={cn(styles.label, styles.labelWithMargin)}>
              <KanbanText size="14">Estimated time alerts</KanbanText>
            </Label>
            <KanbanText size="xs" className={styles.settingDescription}>
              Alert when task exceeds estimated time by:
            </KanbanText>
            <div className={styles.sliderContainer}>
              <Slider defaultValue={[50]} max={100} step={10} className={styles.slider} />
              <KanbanText size="14" className={styles.sliderValue}>50%</KanbanText>
            </div>
          </div>

          <Separator className={styles.separator} />

          <div>
            <Label htmlFor="work-hours" className={styles.label}>
              <KanbanText size="14">Standard Work Hours</KanbanText>
            </Label>
            <Select defaultValue="8">
              <SelectTrigger className={styles.selectTrigger}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent className={styles.selectContent}>
                <SelectItem value="6">6 hours/day</SelectItem>
                <SelectItem value="8">8 hours/day</SelectItem>
                <SelectItem value="10">10 hours/day</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      {/* Automation */}
      <div className={styles.settingsCard}>
        <KanbanHeading size="lg" className={styles.cardTitle}>Automation</KanbanHeading>

        <div className={styles.settingsGroup}>
          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                <KanbanText size="14">Auto-move stale tasks</KanbanText>
              </Label>
              <KanbanText size="xs" className={styles.settingDescription}>
                Move tasks stuck in &quot;In Progress&quot; for 7+ days
              </KanbanText>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                <KanbanText size="14">Smart task distribution</KanbanText>
              </Label>
              <KanbanText size="xs" className={styles.settingDescription}>
                AI distributes tasks based on team capacity
              </KanbanText>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                <KanbanText size="14">Deadline reminders</KanbanText>
              </Label>
              <KanbanText size="xs" className={styles.settingDescription}>
                Send reminders 24h before deadline
              </KanbanText>
            </div>
            <Switch defaultChecked />
          </div>
        </div>
      </div>
    </div>
  );
}

export function ManageTab() {
  const [activeTab, setActiveTab] = useState<ManageTabType>("general");

  return (
    <div className={styles.manageTab}>
      <div className={styles.manageTabContent}>
        <div className={styles.manageTabContainer}>
          {/* Header Section */}
          <div className={styles.headerSection}>
            <KanbanHeading size="xl">Project Settings</KanbanHeading>
            <KanbanText size="14" color="secondary">
              Manage your project configuration and team
            </KanbanText>
          </div>

          {/* Content Container */}
          <div className={styles.contentContainer}>
            {/* Tab List */}
            <div className={styles.tabList}>
              {TAB_ITEMS.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={cn(
                    styles.tabButton,
                    activeTab === tab.id ? styles.tabButtonActive : styles.tabButtonInactive
                  )}
                  type="button"
                >
                  <KanbanText
                    size="14"
                    className={cn(
                      activeTab === tab.id ? styles.tabButtonTextActive : styles.tabButtonTextInactive
                    )}
                  >
                    {tab.label}
                  </KanbanText>
                </button>
              ))}
            </div>

            {/* Tab Content */}
            <div className={styles.tabContent}>
              {activeTab === "general" && <GeneralTabContent />}
              {activeTab === "workHistory" && <WorkHistoryTabContent />}
              {activeTab === "aiAgents" && <AIAgentsTabContent />}
              {activeTab === "taskSettings" && <TaskSettingsTabContent />}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
