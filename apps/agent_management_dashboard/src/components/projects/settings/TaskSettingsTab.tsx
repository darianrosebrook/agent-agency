'use client';

import { Input } from '../../ui/input';
import { Label } from '../../ui/label';
import { Switch } from '../../ui/switch';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../../ui/select';
import { Separator } from '../../ui/separator';
import { Slider } from '../../ui/slider';

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
    <div className="space-y-6">
      {/* Task Workflow */}
      <div className="bg-[#1a1a1a] rounded-lg border border-[#262626] p-6">
        <h2 className="text-white text-xl mb-4">Task Workflow</h2>

        <div className="space-y-4">
          <div>
            <Label htmlFor="default-status" className="text-[#d1d5dc]">
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
              <Label className="text-[#d1d5dc]">Enable task dependencies</Label>
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
        <h2 className="text-white text-xl mb-4">Priority & Labels</h2>

        <div className="space-y-4">
          <div>
            <Label htmlFor="priority-levels" className="text-[#d1d5dc]">
              Priority Levels
            </Label>
            <Select defaultValue="4">
              <SelectTrigger className="mt-1.5 bg-[#0d0d0d] border-[#262626] text-white">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="bg-[#1a1a1a] border-[#262626]">
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

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">Auto-assign priority</Label>
              <p className="text-sm text-[#888888]">
                AI suggests priority based on task content
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className="bg-[#262626]" />

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">Limit tags per task</Label>
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
        <h2 className="text-white text-xl mb-4">Time Tracking</h2>

        <div className="space-y-4">
          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">Enable time tracking</Label>
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
              <span className="text-white w-12 text-right">50%</span>
            </div>
          </div>

          <Separator className="bg-[#262626]" />

          <div>
            <Label htmlFor="work-hours" className="text-[#d1d5dc]">
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
              <Label className="text-[#d1d5dc]">Auto-move stale tasks</Label>
              <p className="text-sm text-[#888888]">
                Move tasks stuck in &quot;In Progress&quot; for 7+ days
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className="bg-[#262626]" />

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">Smart task distribution</Label>
              <p className="text-sm text-[#888888]">
                AI distributes tasks based on team capacity
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <Separator className="bg-[#262626]" />

          <div className="flex items-center justify-between py-3">
            <div className="space-y-0.5">
              <Label className="text-[#d1d5dc]">Deadline reminders</Label>
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

