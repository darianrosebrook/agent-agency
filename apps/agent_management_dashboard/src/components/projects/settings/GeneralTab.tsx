"use client";

import { useState, useEffect } from "react";
import svgPaths from "../../../imports/svg-pj3tus7kw0";
import { cn } from "../../primitives/utils";
import { useProjectContext } from "../../ProjectContext";
import {
  getProjectHandler,
  getProjectSettings,
  updateProjectHandler,
  updateProjectSettings,
  getProjectMembers,
  type ProjectSettings,
  type ProjectApiResponse,
} from "../../../lib/api/projects";
import { KanbanHeading } from "../../primitives/kanban/KanbanHeading";
import { KanbanText } from "../../primitives/kanban/KanbanText";
import { Separator } from "../../primitives/separator";
import styles from "./GeneralTab.module.scss";

export function GeneralTabContent() {
  const { currentProjectId, deleteProject, clearCurrentProject } =
    useProjectContext();
  const router = useRouter();
  const [project, setProject] = useState<ProjectApiResponse | null>(null);
  const [settings, setSettings] = useState<ProjectSettings | null>(null);
  const [members, setMembers] = useState<
    Array<{ id: string; name: string; email: string }>
  >([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const [collaboration, setCollaboration] = useState(true);
  const [requireApproval, setRequireApproval] = useState(false);
  const [assignmentNotifs, setAssignmentNotifs] = useState(true);
  const [commentNotifs, setCommentNotifs] = useState(true);
  const [statusNotifs, setStatusNotifs] = useState(false);
  const [projectName, setProjectName] = useState("");
  const [description, setDescription] = useState("");
  const [defaultAssigneeId, setDefaultAssigneeId] = useState<string | null>(
    null
  );

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
          setProjectName(projectData.name || "");
          setDescription(projectData.description || "");
        }

        if (settingsData) {
          setSettings(settingsData);
          setDefaultAssigneeId(settingsData.default_assignee_id || null);
          setCollaboration(settingsData.auto_assign_tasks ?? true);
          if (settingsData.notification_preferences) {
            const prefs = settingsData.notification_preferences as Record<
              string,
              boolean
            >;
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
        setError(
          err instanceof Error ? err : new Error("Failed to load project data")
        );
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
      await updateProjectHandler(currentProjectId, {
        name: projectName,
        description: description,
      });

      await updateProjectSettings(currentProjectId, {
        default_assignee_id: defaultAssigneeId,
        auto_assign_tasks: collaboration,
        notification_preferences: {
          assignment: assignmentNotifs,
          comment: commentNotifs,
          status: statusNotifs,
        },
      });

      const [projectData, settingsData] = await Promise.all([
        getProjectHandler(currentProjectId),
        getProjectSettings(currentProjectId),
      ]);

      if (projectData) setProject(projectData);
      if (settingsData) setSettings(settingsData);

      alert("Settings saved successfully");
    } catch (err) {
      setError(
        err instanceof Error ? err : new Error("Failed to save settings")
      );
      alert(
        `Failed to save settings: ${
          err instanceof Error ? err.message : "Unknown error"
        }`
      );
    } finally {
      setIsSaving(false);
    }
  };

  const formatDate = (dateStr: string | undefined): string => {
    if (!dateStr) return "N/A";
    try {
      return new Date(dateStr).toLocaleDateString("en-US", {
        year: "numeric",
        month: "long",
        day: "numeric",
      });
    } catch {
      return dateStr;
    }
  };

  if (isLoading) {
    return (
      <div className={styles.generalTab}>
        <div className={styles.loadingState}>
          <KanbanText>Loading project settings...</KanbanText>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.generalTab}>
        <div className={styles.errorState}>
          <KanbanText color="error">
            Error loading project settings: {error.message}
          </KanbanText>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.generalTab}>
      <div className={styles.generalTabInner}>
        {/* Project Details Section */}
        <div className={styles.settingsSection}>
          <div aria-hidden="true" className={styles.settingsSectionBorder} />
          <KanbanHeading size="lg" className={styles.sectionTitle}>
            Project Details
          </KanbanHeading>

          <div className={styles.sectionContent}>
            {/* Project Name */}
            <div className={styles.formField}>
              <label className={styles.formLabel}>
                <KanbanText size="sm" className={styles.formLabelText}>
                  Project Name
                </KanbanText>
              </label>
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
              <label className={styles.formLabel}>
                <KanbanText size="sm" className={styles.formLabelText}>
                  Description
                </KanbanText>
              </label>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className={styles.formTextarea}
                disabled={!currentProjectId}
              />
            </div>

            {/* Project ID and Created */}
            <div className={styles.formRow}>
              <div className={styles.formField}>
                <label className={styles.formLabel}>
                  <KanbanText size="sm" className={styles.formLabelText}>
                    Project ID
                  </KanbanText>
                </label>
                <div className={styles.readOnlyField}>
                  <KanbanText size="sm" className={styles.readOnlyFieldText}>
                    {project?.id || currentProjectId || "N/A"}
                  </KanbanText>
                </div>
              </div>

              <div className={styles.formField}>
                <label className={styles.formLabel}>
                  <KanbanText size="sm" className={styles.formLabelText}>
                    Created
                  </KanbanText>
                </label>
                <div className={styles.readOnlyField}>
                  <KanbanText size="sm" className={styles.readOnlyFieldText}>
                    {formatDate(project?.created_at)}
                  </KanbanText>
                </div>
              </div>
            </div>
          </div>

          <button
            className={styles.saveButton}
            onClick={handleSave}
            disabled={isSaving || !currentProjectId}
            type="button"
          >
            <KanbanText size="sm" className={styles.saveButtonText}>
              {isSaving ? "Saving..." : "Save Changes"}
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
            <div className={styles.formField}>
              <KanbanText size="sm" className={styles.formLabelText}>
                Default Assignee
              </KanbanText>
              <button className={styles.defaultAssigneeButton} type="button">
                <KanbanText
                  size="sm"
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
                <KanbanText size="sm" className={styles.toggleRowTitle}>
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
                <KanbanText size="sm" className={styles.toggleRowTitle}>
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
          <KanbanHeading size="lg" className={styles.sectionTitle}>
            Notifications
          </KanbanHeading>

          <div className={styles.sectionContent}>
            {/* Task Assignments */}
            <div className={styles.toggleRow}>
              <div className={styles.toggleRowContent}>
                <KanbanText size="sm" className={styles.toggleRowTitle}>
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
                <KanbanText size="sm" className={styles.toggleRowTitle}>
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
                <KanbanText size="sm" className={styles.toggleRowTitle}>
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
            <KanbanHeading size="lg" className={styles.dangerZoneTitleText}>
              Danger Zone
            </KanbanHeading>
          </div>

          <div className={styles.dangerZoneContent}>
            {/* Archive Project */}
            <div className={styles.dangerZoneItem}>
              <div className={styles.dangerZoneItemContent}>
                <KanbanText size="sm" className={styles.dangerZoneItemTitle}>
                  Archive this project
                </KanbanText>
                <KanbanText
                  size="xs"
                  className={styles.dangerZoneItemDescription}
                >
                  Make the project read-only and hide it from your dashboard
                </KanbanText>
              </div>
              <button className={styles.dangerZoneButtonArchive} type="button">
                <KanbanText size="sm" className={styles.dangerZoneButtonText}>
                  Archive
                </KanbanText>
              </button>
            </div>

            {/* Delete Project */}
            <div
              className={cn(styles.dangerZoneItem, styles.dangerZoneItemDelete)}
            >
              <div className={styles.dangerZoneItemContent}>
                <KanbanText size="sm" className={styles.dangerZoneItemTitle}>
                  Delete this project
                </KanbanText>
                <KanbanText
                  size="xs"
                  className={styles.dangerZoneItemDescription}
                >
                  Permanently delete this project and all of its data
                </KanbanText>
              </div>
              {showDeleteConfirm ? (
                <div className={styles.deleteConfirmContainer}>
                  <KanbanText size="sm" className={styles.deleteConfirmText}>
                    Are you sure? This cannot be undone.
                  </KanbanText>
                  <div className={styles.deleteConfirmButtons}>
                    <button
                      className={styles.deleteConfirmCancel}
                      type="button"
                      onClick={() => setShowDeleteConfirm(false)}
                      disabled={isDeleting}
                    >
                      Cancel
                    </button>
                    <button
                      className={styles.dangerZoneButtonDelete}
                      type="button"
                      onClick={async () => {
                        if (!currentProjectId) return;
                        setIsDeleting(true);
                        setError(null);
                        try {
                          await deleteProject(currentProjectId);
                          clearCurrentProject();
                          router.push("/projects");
                        } catch (err) {
                          console.error("Failed to delete project:", err);
                          setError(
                            err instanceof Error
                              ? err
                              : new Error("Failed to delete project")
                          );
                          setIsDeleting(false);
                        }
                      }}
                      disabled={isDeleting}
                    >
                      {isDeleting ? (
                        <KanbanText
                          size="sm"
                          className={styles.dangerZoneButtonText}
                        >
                          Deleting...
                        </KanbanText>
                      ) : (
                        <>
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
                          <KanbanText
                            size="sm"
                            className={styles.dangerZoneButtonText}
                          >
                            Confirm Delete
                          </KanbanText>
                        </>
                      )}
                    </button>
                  </div>
                </div>
              ) : (
                <button
                  className={styles.dangerZoneButtonDelete}
                  type="button"
                  onClick={() => setShowDeleteConfirm(true)}
                  disabled={isDeleting}
                >
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
                  <KanbanText size="sm" className={styles.dangerZoneButtonText}>
                    Delete
                  </KanbanText>
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
