"use client";

import { useState, useEffect } from "react";
import { useProjectContext } from "../../ProjectContext";
import { cn } from "../../primitives/utils";
import { Input } from "../../primitives/input";
import { Label } from "../../primitives/label";
import { Switch } from "../../primitives/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../primitives/select";
import { Separator } from "../../primitives/separator";
import { Slider } from "../../primitives/slider";
import {
  getProjectTaskSettings,
  updateProjectTaskSettings,
  type ProjectSettings,
} from "../../../lib/api/projects";
import styles from "./TaskSettingsTab.module.scss";

export function TaskSettingsTabContent() {
  const { currentProjectId } = useProjectContext();
  const [settings, setSettings] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // Form state
  const [defaultStatus, setDefaultStatus] = useState("todo");
  const [autoArchive, setAutoArchive] = useState(true);
  const [autoArchiveDays, setAutoArchiveDays] = useState(30);
  const [enableDependencies, setEnableDependencies] = useState(false);
  const [requireDescription, setRequireDescription] = useState(false);
  const [priorityLevels, setPriorityLevels] = useState("4");
  const [autoAssignPriority, setAutoAssignPriority] = useState(true);
  const [maxTags, setMaxTags] = useState(5);
  const [enableTimeTracking, setEnableTimeTracking] = useState(true);
  const [timeAlertThreshold, setTimeAlertThreshold] = useState(50);
  const [workHours, setWorkHours] = useState("8");
  const [autoMoveStale, setAutoMoveStale] = useState(true);
  const [smartDistribution, setSmartDistribution] = useState(true);
  const [deadlineReminders, setDeadlineReminders] = useState(true);

  useEffect(() => {
    async function fetchSettings() {
      if (!currentProjectId) {
        setIsLoading(false);
        return;
      }

      setIsLoading(true);
      setError(null);

      try {
        const settingsData = await getProjectTaskSettings(currentProjectId);
        setSettings(settingsData);

        // Populate form from settings
        if (settingsData.default_status) setDefaultStatus(settingsData.default_status as string);
        if (settingsData.auto_archive !== undefined) setAutoArchive(settingsData.auto_archive as boolean);
        if (settingsData.auto_archive_days) setAutoArchiveDays(settingsData.auto_archive_days as number);
        if ((settingsData as any).enable_dependencies !== undefined) setEnableDependencies((settingsData as any).enable_dependencies as boolean);
        if ((settingsData as any).require_description !== undefined) setRequireDescription((settingsData as any).require_description as boolean);
        if ((settingsData as any).priority_levels) setPriorityLevels(String((settingsData as any).priority_levels));
        if ((settingsData as any).auto_assign_priority !== undefined) setAutoAssignPriority((settingsData as any).auto_assign_priority as boolean);
        if ((settingsData as any).max_tags) setMaxTags((settingsData as any).max_tags as number);
        if ((settingsData as any).enable_time_tracking !== undefined) setEnableTimeTracking((settingsData as any).enable_time_tracking as boolean);
        if ((settingsData as any).time_alert_threshold) setTimeAlertThreshold((settingsData as any).time_alert_threshold as number);
        if ((settingsData as any).work_hours) setWorkHours(String((settingsData as any).work_hours));
        if ((settingsData as any).auto_move_stale !== undefined) setAutoMoveStale((settingsData as any).auto_move_stale as boolean);
        if ((settingsData as any).smart_distribution !== undefined) setSmartDistribution((settingsData as any).smart_distribution as boolean);
        if ((settingsData as any).deadline_reminders !== undefined) setDeadlineReminders((settingsData as any).deadline_reminders as boolean);
      } catch (err) {
        setError(err instanceof Error ? err : new Error("Failed to load task settings"));
      } finally {
        setIsLoading(false);
      }
    }

    fetchSettings();
  }, [currentProjectId]);

  const handleSave = async () => {
    if (!currentProjectId) return;

    setIsSaving(true);
    setError(null);

    try {
      await updateProjectTaskSettings(currentProjectId, {
        default_status: defaultStatus,
        auto_archive: autoArchive,
        auto_archive_days: autoArchiveDays,
        enable_dependencies: enableDependencies,
        require_description: requireDescription,
        priority_levels: parseInt(priorityLevels) || 5,
        auto_assign_priority: autoAssignPriority || false,
        max_tags: maxTags,
        enable_time_tracking: enableTimeTracking,
        time_alert_threshold: timeAlertThreshold,
        work_hours: parseInt(workHours),
        auto_move_stale: autoMoveStale,
        smart_distribution: smartDistribution,
        deadline_reminders: deadlineReminders,
      } as any);

      alert("Task settings saved successfully");
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Failed to save task settings"));
      alert(`Failed to save settings: ${err instanceof Error ? err.message : "Unknown error"}`);
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading) {
    return (
      <div className={styles.taskSettingsTab}>
        <div className={styles.loadingState}>Loading task settings...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.taskSettingsTab}>
        <div className={styles.errorState}>Error: {error.message}</div>
      </div>
    );
  }

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
            <Select value={defaultStatus} onValueChange={setDefaultStatus}>
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
            <Switch checked={autoArchive} onCheckedChange={setAutoArchive} />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Enable task dependencies</Label>
              <p className={styles.settingDescription}>
                Tasks can block other tasks from starting
              </p>
            </div>
            <Switch checked={enableDependencies} onCheckedChange={setEnableDependencies} />
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
            <Switch checked={requireDescription} onCheckedChange={setRequireDescription} />
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
            <Select value={priorityLevels} onValueChange={setPriorityLevels}>
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
            <Switch checked={autoAssignPriority} onCheckedChange={setAutoAssignPriority} />
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
                value={maxTags}
                onChange={(e) => setMaxTags(parseInt(e.target.value) || 5)}
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
            <Switch checked={enableTimeTracking} onCheckedChange={setEnableTimeTracking} />
          </div>

          <Separator className={styles.separator} />

          <div>
            <Label className={styles.labelWithMargin}>
              Estimated time alerts
            </Label>
            <p className={cn(styles.settingDescription, styles.settingDescriptionWithMargin)}>
              Alert when task exceeds estimated time by:
            </p>
            <div className={styles.sliderContainer}>
              <Slider
                value={[timeAlertThreshold]}
                onValueChange={(vals) => setTimeAlertThreshold(vals[0])}
                max={100}
                step={10}
                className={styles.slider}
              />
              <span className={styles.sliderValue}>{timeAlertThreshold}%</span>
            </div>
          </div>

          <Separator className={styles.separator} />

          <div>
            <Label htmlFor="work-hours" className={styles.label}>
              Standard Work Hours
            </Label>
            <Select value={workHours} onValueChange={setWorkHours}>
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
            <Switch checked={autoMoveStale} onCheckedChange={setAutoMoveStale} />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Smart task distribution</Label>
              <p className={styles.settingDescription}>
                AI distributes tasks based on team capacity
              </p>
            </div>
            <Switch checked={smartDistribution} onCheckedChange={setSmartDistribution} />
          </div>

          <Separator className={styles.separator} />

          <div className={styles.settingRow}>
            <div className={styles.settingInfo}>
              <Label className={styles.label}>Deadline reminders</Label>
              <p className={styles.settingDescription}>
                Send reminders 24h before deadline
              </p>
            </div>
            <Switch checked={deadlineReminders} onCheckedChange={setDeadlineReminders} />
          </div>
        </div>
      </div>

      {/* Save Button */}
      <div className={styles.saveSection}>
        <button
          className={styles.saveButton}
          onClick={handleSave}
          disabled={isSaving || !currentProjectId}
        >
          {isSaving ? 'Saving...' : 'Save All Settings'}
        </button>
      </div>
    </div>
  );
}
