"use client";

import { useState } from "react";
import svgPaths from "../../../imports/svg-pj3tus7kw0";
import { KanbanHeading } from "../../primitives/kanban/KanbanHeading";
import { KanbanText } from "../../primitives/kanban/KanbanText";
import { Separator } from "../../primitives/separator";
import { cn } from "../../primitives/utils";
import styles from "./GeneralTab.module.scss";

export function GeneralTabContent() {
  const [collaboration, setCollaboration] = useState(true);
  const [requireApproval, setRequireApproval] = useState(false);
  const [assignmentNotifs, setAssignmentNotifs] = useState(true);
  const [commentNotifs, setCommentNotifs] = useState(true);
  const [statusNotifs, setStatusNotifs] = useState(false);
  // TODO: Replace hardcoded project data with data from v3 database
  const [projectName, setProjectName] = useState("My Kanban Project");
  const [description, setDescription] = useState(
    "A project management tool with kanban boards and timeline tracking."
  );

  return (
    <div className={styles.generalTab}>
      <div className={styles.generalTabInner}>
        {/* Project Details Section */}
        <div className={styles.settingsSection}>
          <div aria-hidden="true" className={styles.settingsSectionBorder} />
          <KanbanHeading className={styles.sectionTitle}>
            Project Details
          </KanbanHeading>

          <div className={styles.sectionContent}>
            {/* Project Name */}
            <div className={styles.formField}>
              <label className={styles.formLabel}>
                <KanbanText size="14" className={styles.formLabelText}>
                  Project Name
                </KanbanText>
              </label>
              <input
                type="text"
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
                className={styles.formInput}
              />
            </div>

            {/* Description */}
            <div className={styles.formField}>
              <label className={styles.formLabel}>
                <KanbanText size="14" className={styles.formLabelText}>
                  Description
                </KanbanText>
              </label>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className={styles.formTextarea}
              />
            </div>

            {/* Project ID and Created */}
            <div className={styles.formRow}>
              <div className={styles.formField}>
                <label className={styles.formLabel}>
                  <KanbanText size="14" className={styles.formLabelText}>
                    Project ID
                  </KanbanText>
                </label>
                <div className={styles.readOnlyField}>
                  <KanbanText size="14" className={styles.readOnlyFieldText}>
                    proj_8k2m9n4p
                  </KanbanText>
                </div>
              </div>

              <div className={styles.formField}>
                <label className={styles.formLabel}>
                  <KanbanText size="14" className={styles.formLabelText}>
                    Created
                  </KanbanText>
                </label>
                <div className={styles.readOnlyField}>
                  <KanbanText size="14" className={styles.readOnlyFieldText}>
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
          <KanbanHeading className={styles.sectionTitle}>
            Team Settings
          </KanbanHeading>

          <div className={styles.sectionContent}>
            {/* Default Assignee */}
            <div className={styles.formField}>
              <KanbanText size="14" className={styles.formLabelText}>
                Default Assignee
              </KanbanText>
              <button className={styles.defaultAssigneeButton} type="button">
                <KanbanText
                  size="14"
                  className={styles.defaultAssigneeButtonText}
                >
                  Auto-assign
                </KanbanText>
                <div className={styles.defaultAssigneeIcon}>
                  <svg
                    className={styles.svgIcon}
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
              <div className={styles.toggleRowContent}>
                <KanbanText size="14" className={styles.toggleRowTitle}>
                  Allow team collaboration
                </KanbanText>
                <KanbanText size="14" className={styles.toggleRowDescription}>
                  Team members can edit tasks and boards
                </KanbanText>
              </div>
              <button
                onClick={() => setCollaboration(!collaboration)}
                className={cn(
                  styles.toggleSwitch,
                  collaboration
                    ? styles.toggleSwitchActive
                    : styles.toggleSwitchInactive
                )}
                type="button"
              >
                <div
                  className={cn(
                    styles.toggleThumb,
                    collaboration
                      ? styles.toggleThumbActive
                      : styles.toggleThumbInactive
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
                <KanbanText size="14" className={styles.toggleRowDescription}>
                  Tasks must be reviewed before marking as done
                </KanbanText>
              </div>
              <button
                onClick={() => setRequireApproval(!requireApproval)}
                className={cn(
                  styles.toggleSwitch,
                  requireApproval
                    ? styles.toggleSwitchActive
                    : styles.toggleSwitchInactive
                )}
                type="button"
              >
                <div
                  className={cn(
                    styles.toggleThumb,
                    requireApproval
                      ? styles.toggleThumbActive
                      : styles.toggleThumbInactive
                  )}
                />
              </button>
            </div>
          </div>
        </div>

        {/* Notifications Section */}
        <div className={styles.settingsSection}>
          <div aria-hidden="true" className={styles.settingsSectionBorder} />
          <KanbanHeading className={styles.sectionTitle}>
            Notifications
          </KanbanHeading>

          <div className={styles.sectionContent}>
            {/* Task Assignments */}
            <div className={styles.toggleRow}>
              <div className={styles.toggleRowContent}>
                <KanbanText size="14" className={styles.toggleRowTitle}>
                  Task assignments
                </KanbanText>
                <KanbanText size="14" className={styles.toggleRowDescription}>
                  Get notified when assigned to a task
                </KanbanText>
              </div>
              <button
                onClick={() => setAssignmentNotifs(!assignmentNotifs)}
                className={cn(
                  styles.toggleSwitch,
                  assignmentNotifs
                    ? styles.toggleSwitchActive
                    : styles.toggleSwitchInactive
                )}
                type="button"
              >
                <div
                  className={cn(
                    styles.toggleThumb,
                    assignmentNotifs
                      ? styles.toggleThumbActive
                      : styles.toggleThumbInactive
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
                <KanbanText size="14" className={styles.toggleRowDescription}>
                  Get notified of new comments on your tasks
                </KanbanText>
              </div>
              <button
                onClick={() => setCommentNotifs(!commentNotifs)}
                className={cn(
                  styles.toggleSwitch,
                  commentNotifs
                    ? styles.toggleSwitchActive
                    : styles.toggleSwitchInactive
                )}
                type="button"
              >
                <div
                  className={cn(
                    styles.toggleThumb,
                    commentNotifs
                      ? styles.toggleThumbActive
                      : styles.toggleThumbInactive
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
                <KanbanText size="14" className={styles.toggleRowDescription}>
                  Get notified when task status changes
                </KanbanText>
              </div>
              <button
                onClick={() => setStatusNotifs(!statusNotifs)}
                className={cn(
                  styles.toggleSwitch,
                  statusNotifs
                    ? styles.toggleSwitchActive
                    : styles.toggleSwitchInactive
                )}
                type="button"
              >
                <div
                  className={cn(
                    styles.toggleThumb,
                    statusNotifs
                      ? styles.toggleThumbActive
                      : styles.toggleThumbInactive
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
              <svg
                className={styles.svgIcon}
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
            <KanbanHeading className={styles.dangerZoneTitleText}>
              Danger Zone
            </KanbanHeading>
          </div>

          <div className={styles.dangerZoneContent}>
            {/* Archive Project */}
            <div className={styles.dangerZoneItem}>
              <div className={styles.dangerZoneItemContent}>
                <KanbanText size="14" className={styles.dangerZoneItemTitle}>
                  Archive this project
                </KanbanText>
                <KanbanText
                  size="14"
                  className={styles.dangerZoneItemDescription}
                >
                  Make the project read-only and hide it from your dashboard
                </KanbanText>
              </div>
              <button className={styles.dangerZoneButtonArchive} type="button">
                <KanbanText size="14" className={styles.dangerZoneButtonText}>
                  Archive
                </KanbanText>
              </button>
            </div>

            {/* Delete Project */}
            <div
              className={cn(styles.dangerZoneItem, styles.dangerZoneItemDelete)}
            >
              <div className={styles.dangerZoneItemContent}>
                <KanbanText size="14" className={styles.dangerZoneItemTitle}>
                  Delete this project
                </KanbanText>
                <KanbanText
                  size="14"
                  className={styles.dangerZoneItemDescription}
                >
                  Permanently delete this project and all of its data
                </KanbanText>
              </div>
              <button className={styles.dangerZoneButtonDelete} type="button">
                <div className={styles.dangerZoneButtonIcon}>
                  <svg
                    className={styles.svgIcon}
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
                <KanbanText size="14" className={styles.dangerZoneButtonText}>
                  Delete
                </KanbanText>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
