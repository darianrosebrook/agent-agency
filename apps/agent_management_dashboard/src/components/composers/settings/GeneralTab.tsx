'use client';

import { useState } from 'react';
import svgPaths from '../../../imports/svg-pj3tus7kw0';
import { cn } from '../../ui/utils';
import styles from './GeneralTab.module.scss';

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
      className={styles.generalTab}
      data-name="ProjectSettings"
    >
      <div className={styles.generalTabInner}>
        {/* Project Details Section */}
        <div
          className={styles.projectDetailsSection}
          data-name="Container"
        >
          <div
            aria-hidden="true"
            className={styles.projectDetailsBorder}
          />
          <div className={styles.sectionHeading}>
            <p className={styles.sectionHeadingText}>
              Project Details
            </p>
          </div>

          <div className={styles.projectDetailsContent}>
            {/* Project Name */}
            <div className={styles.formField}>
              <div className={styles.formFieldLabel}>
                <p className={styles.formFieldLabelText}>
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
            <div className={styles.formField}>
              <div className={styles.formFieldLabel}>
                <p className={styles.formFieldLabelText}>
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
            <div className={styles.projectIdCreatedContainer}>
              <div className={styles.projectIdField}>
                <div className={styles.formFieldLabel}>
                  <p className={styles.formFieldLabelText}>
                    Project ID
                  </p>
                </div>
                <div className={styles.readOnlyField}>
                  {/* TODO: Replace hardcoded project ID with project.id from v3 database */}
                  <p className={styles.readOnlyFieldText}>
                    proj_8k2m9n4p
                  </p>
                </div>
              </div>

              <div className={styles.createdField}>
                <div className={styles.formFieldLabel}>
                  <p className={styles.formFieldLabelText}>
                    Created
                  </p>
                </div>
                <div className={styles.readOnlyField}>
                  {/* TODO: Replace hardcoded created date with project.created_at from v3 database, formatted as readable date */}
                  <p className={styles.readOnlyFieldText}>
                    November 1, 2024
                  </p>
                </div>
              </div>
            </div>
          </div>

          <button className={styles.saveChangesButton}>
            <p className={styles.saveChangesButtonText}>
              Save Changes
            </p>
          </button>
        </div>

        {/* Team Settings Section */}
        <div className={styles.teamSettingsSection}>
          <div
            aria-hidden="true"
            className={styles.teamSettingsBorder}
          />
          <div className={styles.sectionHeading}>
            <p className={styles.sectionHeadingText}>
              Team Settings
            </p>
          </div>

          <div className={styles.teamSettingsContent}>
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
            <div className={styles.formField}>
              <p className={styles.formFieldLabelText}>
                Default Assignee
              </p>
              <button className={styles.defaultAssigneeButton}>
                <p className={styles.defaultAssigneeButtonText}>
                  Auto-assign
                </p>
                <div className={styles.defaultAssigneeButtonIcon}>
                  <svg
                    className={styles.defaultAssigneeButtonIconSvg}
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
              <div className={styles.toggleRowLabel}>
                <p className={styles.toggleRowLabelText}>
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
              <div className={styles.toggleRowLabel}>
                <p className={styles.toggleRowLabelText}>
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
        <div className={styles.notificationsSection}>
          <div
            aria-hidden="true"
            className={styles.notificationsBorder}
          />
          <div className={styles.sectionHeading}>
            <p className={styles.sectionHeadingText}>
              Notifications
            </p>
          </div>

          <div className={styles.notificationsContent}>
            {/* Task Assignments */}
            <div className={cn(styles.notificationRow, styles.notificationRowFirst)}>
              <div className={styles.toggleRowLabel}>
                <p className={styles.toggleRowLabelText}>
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

            <div className={cn(styles.notificationDivider, styles.notificationDividerFirst)} />

            {/* Task Comments */}
            <div className={cn(styles.notificationRow, styles.notificationRowSecond)}>
              <div className={styles.toggleRowLabel}>
                <p className={styles.toggleRowLabelText}>
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

            <div className={cn(styles.notificationDivider, styles.notificationDividerSecond)} />

            {/* Status Changes */}
            <div className={cn(styles.notificationRow, styles.notificationRowThird)}>
              <div className={styles.toggleRowLabel}>
                <p className={styles.toggleRowLabelText}>
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
        <div className={styles.dangerZoneSection}>
          <div
            aria-hidden="true"
            className={styles.dangerZoneBorder}
          />
          <div className={styles.sectionHeading}>
            <div className={styles.dangerZoneIcon}>
              <svg
                className={styles.dangerZoneIconSvg}
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
            <p className={styles.dangerZoneTitle}>
              Danger Zone
            </p>
          </div>

          <div className={styles.dangerZoneContent}>
            {/* Archive Project */}
            <div className={styles.dangerZoneItem}>
              <div className={styles.dangerZoneItemContent}>
                <p className={styles.dangerZoneItemTitle}>
                  Archive this project
                </p>
                <p className={styles.dangerZoneItemDescription}>
                  Make the project read-only and hide it from your dashboard
                </p>
              </div>
              <button className={styles.archiveButton}>
                <p className={styles.archiveButtonText}>
                  Archive
                </p>
              </button>
            </div>

            {/* Delete Project */}
            <div className={cn(styles.dangerZoneItem, styles.dangerZoneItemDelete)}>
              <div className={styles.dangerZoneItemContent}>
                <p className={styles.dangerZoneItemTitle}>
                  Delete this project
                </p>
                <p className={styles.dangerZoneItemDescription}>
                  Permanently delete this project and all of its data
                </p>
              </div>
              <button className={styles.deleteButton}>
                <div className={styles.deleteButtonIcon}>
                  <svg
                    className={styles.deleteButtonIconSvg}
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
                <p className={styles.deleteButtonText}>
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

