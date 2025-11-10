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
import styles from "./ManageTab.module.scss";

type ManageTabType = "general" | "workHistory" | "aiAgents" | "taskSettings";

function Heading() {
  return (
    <div
      className={styles.headingContainer}
      data-name="Heading 1"
    >
      <p className={styles.headingText}>
        Project Settings
      </p>
    </div>
  );
}

function Paragraph() {
  return (
    <div
      className={styles.paragraphContainer}
      data-name="Paragraph"
    >
      <p className={styles.paragraphText}>
        Manage your project configuration and team
      </p>
    </div>
  );
}

function Container() {
  return (
    <div
      className={styles.container}
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
      className={styles.tabList}
      data-name="Tab List"
    >
      <div
        aria-hidden="true"
        className={styles.tabListBorder}
      />
      <div className={styles.tabListContent}>
        <button
          onClick={() => onTabChange("general")}
          className={cn(
            styles.tabButton,
            styles.tabButtonGeneral,
            activeTab === "general" ? styles.tabButtonActive : styles.tabButtonInactive
          )}
        >
          <div
            aria-hidden="true"
            className={styles.tabButtonBorder}
          />
          <p
            className={cn(
              styles.tabButtonText,
              activeTab === "general" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
            )}
          >
            General
          </p>
        </button>

        <button
          onClick={() => onTabChange("workHistory")}
          className={cn(
            styles.tabButton,
            styles.tabButtonWorkHistory,
            activeTab === "workHistory" ? styles.tabButtonActive : styles.tabButtonInactive
          )}
        >
          <div
            aria-hidden="true"
            className={styles.tabButtonBorder}
          />
          <p
            className={cn(
              styles.tabButtonText,
              activeTab === "workHistory" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
            )}
          >
            Work History
          </p>
        </button>

        <button
          onClick={() => onTabChange("aiAgents")}
          className={cn(
            styles.tabButton,
            styles.tabButtonAIAgents,
            activeTab === "aiAgents" ? styles.tabButtonActive : styles.tabButtonInactive
          )}
        >
          <div
            aria-hidden="true"
            className={styles.tabButtonBorder}
          />
          <p
            className={cn(
              styles.tabButtonText,
              activeTab === "aiAgents" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
            )}
          >
            AI Agents
          </p>
        </button>

        <button
          onClick={() => onTabChange("taskSettings")}
          className={cn(
            styles.tabButton,
            styles.tabButtonTaskSettings,
            activeTab === "taskSettings" ? styles.tabButtonActive : styles.tabButtonInactive
          )}
        >
          <div
            aria-hidden="true"
            className={styles.tabButtonBorder}
          />
          <p
            className={cn(
              styles.tabButtonText,
              activeTab === "taskSettings" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
            )}
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
  const [assignmentNotifs, setAssignmentNotifs] = useState(true);
  const [commentNotifs, setCommentNotifs] = useState(true);
  const [statusNotifs, setStatusNotifs] = useState(false);
  const [projectName, setProjectName] = useState("My Kanban Project");
  const [description, setDescription] = useState(
    "A project management tool with kanban boards and timeline tracking."
  );

  return (
    <div
      className={styles.generalTabContent}
      data-name="ProjectSettings"
    >
      <div className={styles.generalTabContentInner}>
        {/* Project Details Section */}
        <div
          className={styles.settingsSection}
          style={{ height: '23.859625rem', left: 0, top: 0, width: '76.000625rem' }}
          data-name="Container"
        >
          <div
            aria-hidden="true"
            className={styles.settingsSectionBorder}
          />
          <div
            className={styles.sectionTitle}
            style={{ height: '1.7498125rem', left: '1.556875rem', top: '1.556875rem', width: '72.886875rem' }}
            data-name="Heading 2"
          >
            <p>Project Details</p>
          </div>

          <div className={styles.sectionContent} style={{ height: '14.2471875rem', left: '1.556875rem', top: '4.30625rem', width: '72.886875rem' }}>
            {/* Project Name */}
            <div className={styles.formGroup} style={{ height: '3.4991875rem' }}>
              <div className={styles.formLabel}>
                <p className={styles.formLabelText}>
                  Project Name
                </p>
              </div>
              <input
                type="text"
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
                className={styles.formInput}
              />
            </div>

            {/* Description */}
            <div className={styles.formGroup} style={{ height: '5.2495rem' }}>
              <div className={styles.formLabel}>
                <p className={styles.formLabelText}>
                  Description
                </p>
              </div>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className={styles.formTextarea}
              />
            </div>

            {/* Project ID and Created */}
            <div style={{ height: '3.4991875rem', position: 'relative', flexShrink: 0, width: '100%' }}>
              <div style={{ position: 'absolute', display: 'flex', flexDirection: 'column', gap: '0.374625rem', height: '3.4991875rem', alignItems: 'flex-start', left: 0, top: 0, width: '35.9436875rem' }}>
                <div className={styles.formLabel}>
                  <p className={styles.formLabelText}>
                    Project ID
                  </p>
                </div>
                <div className={cn(styles.formInputDisabled, styles.formInput)}>
                  <p className={styles.formInputDisabledText}>
                    proj_8k2m9n4p
                  </p>
                </div>
              </div>

              <div style={{ position: 'absolute', display: 'flex', flexDirection: 'column', gap: '0.374625rem', height: '3.4991875rem', alignItems: 'flex-start', left: '36.943125rem', top: 0, width: '35.9436875rem' }}>
                <div className={styles.formLabel}>
                  <p className={styles.formLabelText}>
                    Created
                  </p>
                </div>
                <div className={cn(styles.formInputDisabled, styles.formInput)}>
                  <p className={styles.formInputDisabledText}>
                    November 1, 2024
                  </p>
                </div>
              </div>
            </div>
          </div>

          <button className={styles.saveButton}>
            <p className={styles.saveButtonText}>
              Save Changes
            </p>
          </button>
        </div>

        {/* Team Settings Section */}
        <div className={styles.settingsSection} style={{ boxSizing: 'border-box', display: 'flex', flexDirection: 'column', gap: '0.999625rem', height: '18.85875rem', alignItems: 'flex-start', left: 0, paddingBottom: '0.0568125rem', paddingTop: '1.55675rem', paddingInline: '1.55675rem', top: '25.359375rem', width: '76.000625rem' }}>
          <div
            aria-hidden="true"
            className={styles.settingsSectionBorder}
          />
          <div className={styles.sectionTitle} style={{ height: '1.7498125rem' }}>
            <p>Team Settings</p>
          </div>

          <div className={styles.sectionContent} style={{ height: '12.9958125rem', width: '100%' }}>
            {/* Default Assignee */}
            <div className={styles.formGroup} style={{ height: '3.4991875rem' }}>
              <p className={styles.formLabelText}>
                Default Assignee
              </p>
              <button className={styles.dropdownButton}>
                <p className={styles.dropdownButtonText}>
                  Auto-assign
                </p>
                <div className={styles.dropdownButtonIcon}>
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
            <div className={styles.toggleRow}>
              <div className={styles.toggleRowContent} style={{ width: '16.7884375rem' }}>
                <p className={styles.toggleRowTitle}>
                  Allow team collaboration
                </p>
                <p className={styles.toggleRowDescription}>
                  Team members can edit tasks and boards
                </p>
              </div>
              <button
                onClick={() => setCollaboration(!collaboration)}
                className={cn(
                  styles.toggleSwitch,
                  collaboration ? styles.toggleSwitchActive : styles.toggleSwitchInactive
                )}
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
              <div className={styles.toggleRowContent} style={{ width: '19.3923125rem' }}>
                <p className={styles.toggleRowTitle}>
                  Require approval for done tasks
                </p>
                <p className={styles.toggleRowDescription}>
                  Tasks must be reviewed before marking as done
                </p>
              </div>
              <button
                onClick={() => setRequireApproval(!requireApproval)}
                className={cn(
                  styles.toggleSwitch,
                  requireApproval ? styles.toggleSwitchActive : styles.toggleSwitchInactive
                )}
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
        <div className={styles.settingsSection} style={{ boxSizing: 'border-box', display: 'flex', flexDirection: 'column', gap: '0.999625rem', height: '20.232125rem', alignItems: 'flex-start', left: 0, paddingBottom: '0.0568125rem', paddingTop: '1.55675rem', paddingInline: '1.55675rem', top: '45.718125rem', width: '76.000625rem' }}>
          <div
            aria-hidden="true"
            className={styles.settingsSectionBorder}
          />
          <div className={styles.sectionTitle} style={{ height: '1.7498125rem' }}>
            <p>Notifications</p>
          </div>

          <div style={{ height: '14.36925rem', position: 'relative', flexShrink: 0, width: '100%' }}>
            {/* Task Assignments */}
            <div className={styles.toggleRow} style={{ position: 'absolute', left: 0, top: 0, width: '72.886875rem' }}>
              <div className={styles.toggleRowContent} style={{ width: '14.676375rem' }}>
                <p className={styles.toggleRowTitle}>
                  Task assignments
                </p>
                <p className={styles.toggleRowDescription}>
                  Get notified when assigned to a task
                </p>
              </div>
              <button
                onClick={() => setAssignmentNotifs(!assignmentNotifs)}
                className={cn(
                  styles.toggleSwitch,
                  assignmentNotifs ? styles.toggleSwitchActive : styles.toggleSwitchInactive
                )}
              >
                <div
                  className={cn(
                    styles.toggleSwitchThumb,
                    assignmentNotifs ? styles.toggleSwitchThumbActive : styles.toggleSwitchThumbInactive
                  )}
                />
              </button>
            </div>

            <div className={styles.divider} style={{ left: 0, top: '4.498125rem', width: '72.886875rem' }} />

            {/* Task Comments */}
            <div className={styles.toggleRow} style={{ position: 'absolute', left: 0, top: '5.31rem', width: '72.886875rem' }}>
              <div className={styles.toggleRowContent} style={{ width: '17.7805625rem' }}>
                <p className={styles.toggleRowTitle}>
                  Task comments
                </p>
                <p className={styles.toggleRowDescription}>
                  Get notified of new comments on your tasks
                </p>
              </div>
              <button
                onClick={() => setCommentNotifs(!commentNotifs)}
                className={cn(
                  styles.toggleSwitch,
                  commentNotifs ? styles.toggleSwitchActive : styles.toggleSwitchInactive
                )}
              >
                <div
                  className={cn(
                    styles.toggleSwitchThumb,
                    commentNotifs ? styles.toggleSwitchThumbActive : styles.toggleSwitchThumbInactive
                  )}
                />
              </button>
            </div>

            <div className={styles.divider} style={{ left: 0, top: '9.80875rem', width: '72.886875rem' }} />

            {/* Status Changes */}
            <div className={styles.toggleRow} style={{ position: 'absolute', left: 0, top: '10.620625rem', width: '72.886875rem' }}>
              <div className={styles.toggleRowContent} style={{ width: '15.451875rem' }}>
                <p className={styles.toggleRowTitle}>
                  Status changes
                </p>
                <p className={styles.toggleRowDescription}>
                  Get notified when task status changes
                </p>
              </div>
              <button
                onClick={() => setStatusNotifs(!statusNotifs)}
                className={cn(
                  styles.toggleSwitch,
                  statusNotifs ? styles.toggleSwitchActive : styles.toggleSwitchInactive
                )}
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
        <div className={styles.dangerZone} style={{ top: '67.45rem', width: '76.000625rem' }}>
          <div
            aria-hidden="true"
            className={styles.dangerZoneBorder}
          />
          <div className={styles.dangerZoneTitle}>
            <div className={styles.dangerZoneIcon}>
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
            <p className={styles.dangerZoneTitleText}>
              Danger Zone
            </p>
          </div>

          <div className={styles.dangerZoneActions}>
            {/* Archive Project */}
            <div className={styles.dangerZoneActionCard}>
              <div className={styles.dangerZoneActionInfo} style={{ width: '24.0394375rem' }}>
                <p className={styles.dangerZoneActionTitle}>
                  Archive this project
                </p>
                <p className={styles.dangerZoneActionDescription}>
                  Make the project read-only and hide it from your dashboard
                </p>
              </div>
              <button className={cn(styles.dangerZoneButton, styles.dangerZoneButtonArchive)}>
                <p className={styles.dangerZoneButtonText}>
                  Archive
                </p>
              </button>
            </div>

            {/* Delete Project */}
            <div className={cn(styles.dangerZoneActionCard, styles.dangerZoneActionCardDanger)}>
              <div className={styles.dangerZoneActionInfo} style={{ width: '19.727875rem' }}>
                <p className={styles.dangerZoneActionTitle}>
                  Delete this project
                </p>
                <p className={styles.dangerZoneActionDescription}>
                  Permanently delete this project and all of its data
                </p>
              </div>
              <button className={cn(styles.dangerZoneButton, styles.dangerZoneButtonDelete)}>
                <div className={styles.dangerZoneButtonIcon}>
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
                <p className={styles.dangerZoneButtonText}>
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
    <div className={styles.workHistoryTab}>
      <div className={styles.workHistoryCard}>
        <h2 className={styles.workHistoryTitle}>
          Work History
        </h2>
        <p className={styles.workHistoryDescription}>
          View and analyze your team&apos;s work history, time tracking, and
          productivity metrics.
        </p>
        <div className={styles.workHistoryMetrics}>
          {[
            "Total Tasks",
            "Completed This Week",
            "Average Completion Time",
          ].map((metric, i) => (
            <div
              key={i}
              className={styles.workHistoryMetricCard}
            >
              <p className={styles.workHistoryMetricLabel}>
                {metric}
              </p>
              <p className={styles.workHistoryMetricValue}>
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
    <div className={styles.aiAgentsTab}>
      <div className={styles.aiAgentsCard}>
        <h2 className={styles.aiAgentsTitle}>
          AI Agents
        </h2>
        <p className={styles.aiAgentsDescription}>
          Configure AI agents to automate tasks and provide intelligent
          assistance.
        </p>

        <div className={styles.aiAgentsList}>
          {[
            {
              name: "Task Suggester",
              description:
                "Automatically suggests task breakdowns and subtasks",
              enabled: true,
            },
            {
              name: "Priority Optimizer",
              description: "Analyzes and recommends task prioritization",
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
              className={styles.aiAgentCard}
            >
              <div className={styles.aiAgentInfo}>
                <p className={styles.aiAgentName}>
                  {agent.name}
                </p>
                <p className={styles.aiAgentDescription}>
                  {agent.description}
                </p>
              </div>
              <div
                className={cn(
                  styles.toggleSwitch,
                  agent.enabled ? styles.toggleSwitchActive : styles.toggleSwitchInactive
                )}
              >
                <div
                  className={cn(
                    styles.toggleSwitchThumb,
                    agent.enabled ? styles.toggleSwitchThumbActive : styles.toggleSwitchThumbInactive
                  )}
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
    <div className={styles.taskSettingsTab}>
      {/* Task Workflow */}
      <div className={styles.settingsCard}>
        <h2 className={styles.cardTitle}>Task Workflow</h2>

        <div className={styles.settingsGroup}>
          <div>
            <Label htmlFor="default-status" className={styles.label}>
              Default Status for New Tasks
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
                Auto-archive completed tasks
              </Label>
              <p className={styles.settingDescription}>
                Archive tasks 30 days after completion
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Enable task dependencies</Label>
              <p className={styles.settingDescription}>
                Tasks can block other tasks from starting
              </p>
            </div>
            <Switch />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>
                Require task descriptions
              </Label>
              <p className={styles.settingDescription}>
                Force users to add descriptions to new tasks
              </p>
            </div>
            <Switch />
          </div>
        </div>
      </div>

      {/* Priority Settings */}
      <div className={styles.settingsCard}>
        <h2 className={styles.cardTitle}>Priority & Labels</h2>

        <div className={styles.settingsGroup}>
          <div>
            <Label htmlFor="priority-levels" className={styles.label}>
              Priority Levels
            </Label>
            <Select defaultValue="4">
              <SelectTrigger className={styles.selectTrigger}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent className={styles.selectContent}>
                <SelectItem value="3">3 levels (Low, Medium, High)</SelectItem>
                <SelectItem value="4">
                  4 levels (Low, Medium, High, Critical)
                </SelectItem>
                <SelectItem value="5">
                  5 levels (Very Low to Critical)
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Auto-assign priority</Label>
              <p className={styles.settingDescription}>
                AI suggests priority based on task content
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Limit tags per task</Label>
              <p className={styles.settingDescription}>
                Maximum number of tags allowed
              </p>
            </div>
            <div className={styles.sliderContainer}>
              <Input
                type="number"
                defaultValue="5"
                className={styles.numberInput}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Time Tracking */}
      <div className={styles.settingsCard}>
        <h2 className={styles.cardTitle}>Time Tracking</h2>

        <div className={styles.settingsGroup}>
          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Enable time tracking</Label>
              <p className={styles.settingDescription}>
                Track time spent on tasks
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div>
            <Label className={cn(styles.label, styles.labelWithMargin)}>
              Estimated time alerts
            </Label>
            <p className={styles.settingDescription} style={{ marginBottom: '0.75rem' }}>
              Alert when task exceeds estimated time by:
            </p>
            <div className={styles.sliderContainer}>
              <Slider
                defaultValue={[50]}
                max={100}
                step={10}
                className={styles.slider}
              />
              <span className={styles.sliderValue}>50%</span>
            </div>
          </div>

          <Separator className={styles.separator} />

          <div>
            <Label htmlFor="work-hours" className={styles.label}>
              Standard Work Hours
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
        <h2 className={styles.cardTitle}>Automation</h2>

        <div className={styles.settingsGroup}>
          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Auto-move stale tasks</Label>
              <p className={styles.settingDescription}>
                Move tasks stuck in &quot;In Progress&quot; for 7+ days
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Smart task distribution</Label>
              <p className={styles.settingDescription}>
                AI distributes tasks based on team capacity
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Deadline reminders</Label>
              <p className={styles.settingDescription}>
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
  const [activeTab, setActiveTab] = useState<ManageTabType>("general");

  return (
    <div className={styles.manageTab}>
      <div className={styles.manageTabContent}>
        <div className={styles.manageTabContainer}>
          <Container />
          <div className={styles.contentContainer}>
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
