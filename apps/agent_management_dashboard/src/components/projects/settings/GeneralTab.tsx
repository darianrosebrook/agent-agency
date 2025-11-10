'use client';

import { useState, useEffect } from 'react';
import svgPaths from '../../../imports/svg-pj3tus7kw0';
import { cn } from '../../primitives/utils';
import { useProjectContext } from '../../ProjectContext';
import {
  getProjectHandler,
  getProjectSettings,
  updateProjectHandler,
  updateProjectSettings,
  getProjectMembers,
  type ProjectSettings,
  type ProjectApiResponse,
} from '../../../lib/api/projects';
import styles from './GeneralTab.module.scss';

export function GeneralTabContent() {
  const { currentProjectId } = useProjectContext();
  const [project, setProject] = useState<ProjectApiResponse | null>(null);
  const [settings, setSettings] = useState<ProjectSettings | null>(null);
  const [members, setMembers] = useState<Array<{ id: string; name: string; email: string }>>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const [collaboration, setCollaboration] = useState(true);
  const [requireApproval, setRequireApproval] = useState(false);
  const [assignmentNotifs, setAssignmentNotifs] = useState(true);
  const [commentNotifs, setCommentNotifs] = useState(true);
  const [statusNotifs, setStatusNotifs] = useState(false);
  const [projectName, setProjectName] = useState('');
  const [description, setDescription] = useState('');
  const [defaultAssigneeId, setDefaultAssigneeId] = useState<string | null>(null);

  useEffect(() => {
    async function fetchData() {
      if (!currentProjectId) {
        setIsLoading(false);
        return;
      }

      setIsLoading(true);
      setError(null);

      try {
        const [projectData, settingsData, membersData] = await Promise.all([
          getProjectHandler(currentProjectId).catch(() => null),
          getProjectSettings(currentProjectId).catch(() => null),
          getProjectMembers(currentProjectId).catch(() => []),
        ]);

        if (projectData) {
          setProject(projectData);
          setProjectName(projectData.name || '');
          setDescription(projectData.description || '');
        }

        if (settingsData) {
          setSettings(settingsData);
          setDefaultAssigneeId(settingsData.default_assignee_id || null);
          setCollaboration(settingsData.auto_assign_tasks ?? true);
          if (settingsData.notification_preferences) {
            const prefs = settingsData.notification_preferences as Record<string, boolean>;
            setAssignmentNotifs(prefs.assignment ?? true);
            setCommentNotifs(prefs.comment ?? true);
            setStatusNotifs(prefs.status ?? false);
          }
        }

        if (membersData.length > 0) {
          setMembers(
            membersData.map((m) => ({
              id: m.user_id,
              name: m.user_name,
              email: m.user_email,
            }))
          );
        }
      } catch (err) {
        setError(err instanceof Error ? err : new Error('Failed to load project data'));
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
  }, [currentProjectId]);

  const handleSave = async () => {
    if (!currentProjectId) return;

    setIsSaving(true);
    setError(null);

    try {
      // Update project details
      await updateProjectHandler(currentProjectId, {
        name: projectName,
        description: description,
      });

      // Update project settings
      await updateProjectSettings(currentProjectId, {
        default_assignee_id: defaultAssigneeId,
        auto_assign_tasks: collaboration,
        notification_preferences: {
          assignment: assignmentNotifs,
          comment: commentNotifs,
          status: statusNotifs,
        },
      });

      // Refresh data
      const [projectData, settingsData] = await Promise.all([
        getProjectHandler(currentProjectId),
        getProjectSettings(currentProjectId),
      ]);

      if (projectData) setProject(projectData);
      if (settingsData) setSettings(settingsData);

      alert('Settings saved successfully');
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to save settings'));
      alert(`Failed to save settings: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading) {
    return (
      <div className={styles.generalTab}>
        <div className={styles.loadingState}>Loading project settings...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.generalTab}>
        <div className={styles.errorState}>
          Error loading project settings: {error.message}
        </div>
      </div>
    );
  }

  const formatDate = (dateStr: string | undefined): string => {
    if (!dateStr) return 'N/A';
    try {
      return new Date(dateStr).toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      });
    } catch {
      return dateStr;
    }
  };

  return (
    <div className={styles.generalTab} data-name="ProjectSettings">
      <div className={styles.generalTabInner}>
        {/* Project Details Section */}
        <div className={styles.projectDetailsSection} data-name="Container">
          <div
            aria-hidden="true"
            className={styles.projectDetailsBorder}
          />
          <div className={styles.projectDetailsHeading} data-name="Heading 2">
            <p className={styles.projectDetailsHeadingText}>
              Project Details
            </p>
          </div>

          <div className={styles.projectDetailsContent}>
            {/* Project Name */}
            <div className={styles.formField}>
              <div className={styles.formFieldDescription}>
                <p className={styles.formFieldLabel}>Project Name</p>
              </div>
              <input
                type="text"
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
                className={styles.formInput}
                disabled={!currentProjectId}
              />
            </div>

            {/* Description */}
            <div className={styles.formField}>
              <div className={styles.formFieldDescription}>
                <p className={styles.formFieldLabel}>Description</p>
              </div>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className={styles.formTextarea}
                disabled={!currentProjectId}
              />
            </div>

            {/* Project ID and Created */}
            <div className={styles.formFieldRow}>
              <div className={styles.formFieldHalf}>
                <div className={styles.formFieldDescription}>
                  <p className={styles.formFieldLabel}>Project ID</p>
                </div>
                <div className={styles.readOnlyField}>
                  <p className={styles.readOnlyFieldText}>
                    {project?.id || currentProjectId || 'N/A'}
                  </p>
                </div>
              </div>

              <div className={cn(styles.formFieldHalf, styles.formFieldHalfRight)}>
                <div className={styles.formFieldDescription}>
                  <p className={styles.formFieldLabel}>Created</p>
                </div>
                <div className={styles.readOnlyField}>
                  <p className={styles.readOnlyFieldText}>
                    {formatDate(project?.created_at)}
                  </p>
                </div>
              </div>
            </div>
          </div>

          <button
            className={styles.saveButton}
            onClick={handleSave}
            disabled={isSaving || !currentProjectId}
          >
            <p className={styles.saveButtonText}>
              {isSaving ? 'Saving...' : 'Save Changes'}
            </p>
          </button>
        </div>

        {/* Team Settings Section */}
        <div className={styles.teamSettingsSection}>
          <div
            aria-hidden="true"
            className={styles.teamSettingsBorder}
          />
          <div className={styles.teamSettingsHeading}>
            <p className={styles.teamSettingsHeadingText}>
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
            <div className={styles.defaultAssigneeField}>
              <p className={styles.formFieldLabel}>
                Default Assignee
              </p>
              <button className={styles.defaultAssigneeButton}>
                <p className={styles.defaultAssigneeButtonText}>
                  Auto-assign
                </p>
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
                    styles.toggleThumb,
                    collaboration ? styles.toggleThumbActive : styles.toggleThumbInactive
                  )}
                />
              </button>
            </div>

            {/* Require Approval Toggle */}
            <div className={styles.toggleRow}>
              <div className={styles.toggleRowContent}>
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
                    styles.toggleThumb,
                    requireApproval ? styles.toggleThumbActive : styles.toggleThumbInactive
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
          <div className={styles.notificationsHeading}>
            <p className={styles.notificationsHeadingText}>
              Notifications
            </p>
          </div>

          <div className={styles.notificationsContent}>
            {/* Task Assignments */}
            <div className={cn(styles.notificationItem, styles.notificationItemFirst)}>
              <div className={styles.toggleRowContent}>
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
                    styles.toggleThumb,
                    assignmentNotifs ? styles.toggleThumbActive : styles.toggleThumbInactive
                  )}
                />
              </button>
            </div>

            <div className={cn(styles.notificationDivider, styles.notificationDividerFirst)} />

            {/* Task Comments */}
            <div className={cn(styles.notificationItem, styles.notificationItemSecond)}>
              <div className={styles.toggleRowContent}>
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
                    styles.toggleThumb,
                    commentNotifs ? styles.toggleThumbActive : styles.toggleThumbInactive
                  )}
                />
              </button>
            </div>

            <div className={cn(styles.notificationDivider, styles.notificationDividerSecond)} />

            {/* Status Changes */}
            <div className={cn(styles.notificationItem, styles.notificationItemThird)}>
              <div className={styles.toggleRowContent}>
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
                    styles.toggleThumb,
                    statusNotifs ? styles.toggleThumbActive : styles.toggleThumbInactive
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
          <div className={styles.dangerZoneHeading}>
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
            <p className={styles.dangerZoneHeadingText}>
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
              <button className={styles.dangerZoneButtonArchive}>
                <p className={styles.dangerZoneButtonText}>
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
              <button className={styles.dangerZoneButtonDelete}>
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

