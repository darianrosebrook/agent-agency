"use client";

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
import styles from "./TaskSettingsTab.module.scss";

export function TaskSettingsTabContent() {
  // TODO: Replace hardcoded task settings with project task settings from v3 database with the following requirements:
  // 1. Task settings fetching: Load project task workflow settings from database
  //    - Data source: GET /api/projects/:projectId/settings/tasks endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `project_settings` or `task_settings` table
  //    - Include default status, auto-archive settings, dependency settings, etc.
  // 2. Settings persistence: Save task workflow settings to database
  //    - Data source: PATCH /api/projects/:projectId/settings/tasks endpoint
  //    - Update default status, auto-archive days, dependency settings
  //    - Persist priority level configuration and label settings
  // 3. Settings validation: Validate settings before saving
  //    - Ensure valid status values
  //    - Validate numeric values (auto-archive days, priority levels)
  //    - Handle validation errors gracefully
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
            <Label className={styles.labelWithMargin}>
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
