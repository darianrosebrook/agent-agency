"use client";

import { useState } from "react";
import {
  X,
  ChevronDown,
  Plus,
  MoreHorizontal,
  Circle,
  CheckCircle2,
} from "lucide-react";
import {
  StatusBadge,
  PriorityIndicator,
  MetadataRow,
  TagChip,
  taskStatusConfig,
  priorityConfig,
  type TaskStatus,
  type Priority,
} from "../compounds";
import { cn } from "../primitives/utils";
import styles from "./TaskModal.module.scss";

interface NewTaskModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreateTask: (data: {
    title: string;
    description?: string;
    status: "backlog" | "todo" | "in-progress" | "done";
    priority?: string;
  }) => void;
  defaultStatus?: "backlog" | "todo" | "in-progress" | "done";
}

type Status = TaskStatus;

export function NewTaskModal({
  open,
  onOpenChange,
  onCreateTask,
  defaultStatus = "backlog",
}: NewTaskModalProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState<Status>(defaultStatus);
  const [priority, setPriority] = useState<Priority>("medium");
  const [assignees, setAssignees] = useState("");
  const [dueDate, setDueDate] = useState("");
  const [project, setProject] = useState("");
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState("");
  const [subtasks, setSubtasks] = useState<
    { id: number; title: string; completed: boolean }[]
  >([]);
  const [newSubtask, setNewSubtask] = useState("");
  const [isAddingSubtask, setIsAddingSubtask] = useState(false);

  const [showStatusMenu, setShowStatusMenu] = useState(false);
  const [showPriorityMenu, setShowPriorityMenu] = useState(false);
  const [showProjectMenu, setShowProjectMenu] = useState(false);

  const handleCreate = () => {
    if (title.trim()) {
      onCreateTask({
        title: title.trim(),
        description: description.trim() || undefined,
        status,
        priority,
      });

      // Reset form
      setTitle("");
      setDescription("");
      setStatus(defaultStatus);
      setPriority("medium");
      setAssignees("");
      setDueDate("");
      setProject("");
      setTags([]);
      setTagInput("");
      setSubtasks([]);
      setNewSubtask("");
      setIsAddingSubtask(false);
      onOpenChange(false);
    }
  };

  const handleAddTag = () => {
    if (tagInput.trim() && !tags.includes(tagInput.trim())) {
      setTags([...tags, tagInput.trim()]);
      setTagInput("");
    }
  };

  const handleRemoveTag = (tagToRemove: string) => {
    setTags(tags.filter((tag) => tag !== tagToRemove));
  };

  const handleAddSubtask = () => {
    if (newSubtask.trim()) {
      setSubtasks([
        ...subtasks,
        {
          id: Date.now(),
          title: newSubtask.trim(),
          completed: false,
        },
      ]);
      setNewSubtask("");
      setIsAddingSubtask(false);
    }
  };

  const toggleSubtask = (id: number) => {
    setSubtasks(
      subtasks.map((task) =>
        task.id === id ? { ...task, completed: !task.completed } : task
      )
    );
  };

  const handleRemoveSubtask = (id: number) => {
    setSubtasks(subtasks.filter((task) => task.id !== id));
  };

  if (!open) return null;

  return (
    <div className={styles.modalOverlay}>
      <div className={styles.modal}>
        {/* Header */}
        <div className={styles.modalHeader}>
          <button
            onClick={() => onOpenChange(false)}
            className={styles.closeButton}
          >
            <X className={styles.closeButtonIcon} />
          </button>
          <div className={styles.modalHeaderTitle}>
            <span>New Task</span>
          </div>
        </div>

        {/* Content */}
        <div className={styles.modalContent}>
          {/* Title */}
          <div>
            <input
              type="text"
              placeholder="Task title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className={styles.titleInput}
              autoFocus
            />
            <textarea
              placeholder="Add a description for this task..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className={styles.descriptionTextarea}
              rows={3}
            />
          </div>

          {/* Metadata Grid */}
          <div className={styles.metadataGrid}>
            {/* Status */}
            <MetadataRow label="Status">
              <div className={styles.metadataDropdown}>
                <StatusBadge
                  status={status}
                  config={taskStatusConfig[status]}
                  onClick={() => setShowStatusMenu(!showStatusMenu)}
                />
                {showStatusMenu && (
                  <div className={styles.dropdownMenu}>
                    {(Object.keys(taskStatusConfig) as TaskStatus[]).map(
                      (key) => (
                        <button
                          key={key}
                          onClick={() => {
                            setStatus(key);
                            setShowStatusMenu(false);
                          }}
                          className={styles.dropdownMenuItem}
                        >
                          <StatusBadge
                            status={key}
                            config={taskStatusConfig[key]}
                          />
                        </button>
                      )
                    )}
                  </div>
                )}
              </div>
            </MetadataRow>

            {/* Assignees */}
            {/* TODO: Replace text input with user selection dropdown from v3 database with the following requirements:
            // 1. User list fetching: Load project team members or available users from database
            //    - Data source: GET /api/projects/:projectId/members or GET /api/users endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
            //    - Database table: PostgreSQL `project_members` or `users` table
            //    - Include user names, IDs, emails, and avatars
            // 2. Multi-select support: Allow assigning multiple users to a task
            //    - Support selecting multiple users from dropdown
            //    - Display selected users as chips/badges with avatars
            //    - Store array of user IDs when task is created/updated
            // 3. User display: Show user avatars and names
            //    - Display user avatar (or initials) and name
            //    - Show "Unassigned" when no users are selected
            //    - Handle user lookup and display gracefully */}
            <MetadataRow label="Assignees">
              <div className={styles.assigneesContainer}>
                <div className={styles.assigneeAvatar}>
                  {assignees ? assignees[0].toUpperCase() : "U"}
                </div>
                <input
                  type="text"
                  placeholder="Add assignees"
                  value={assignees}
                  onChange={(e) => setAssignees(e.target.value)}
                  className={styles.assigneeInput}
                />
              </div>
            </MetadataRow>

            {/* Due date */}
            <MetadataRow label="Due date">
              <input
                type="text"
                placeholder="Set due date"
                value={dueDate}
                onChange={(e) => setDueDate(e.target.value)}
                className={styles.dateInput}
              />
            </MetadataRow>

            {/* Priority */}
            <MetadataRow label="Priority">
              <div className={styles.metadataDropdown}>
                <PriorityIndicator
                  priority={priority}
                  config={priorityConfig[priority]}
                  onClick={() => setShowPriorityMenu(!showPriorityMenu)}
                />
                {showPriorityMenu && (
                  <div className={cn(styles.dropdownMenu, styles.dropdownMenuPriority)}>
                    {(Object.keys(priorityConfig) as Priority[]).map((key) => (
                      <button
                        key={key}
                        onClick={() => {
                          setPriority(key);
                          setShowPriorityMenu(false);
                        }}
                        className={cn(styles.dropdownMenuItem, styles.dropdownMenuItemPriority)}
                      >
                        <PriorityIndicator
                          priority={key}
                          config={priorityConfig[key]}
                        />
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </MetadataRow>

            {/* Project */}
            <MetadataRow label="Project">
              <div className={styles.metadataDropdown}>
                <button
                  onClick={() => setShowProjectMenu(!showProjectMenu)}
                  className={styles.projectButton}
                >
                  {project ? (
                    <>
                      <span className={styles.projectIndicator}></span>
                      <span>{project}</span>
                    </>
                  ) : (
                    <span className={styles.projectButtonText}>Add project</span>
                  )}
                </button>
                {showProjectMenu && (
                  <div className={cn(styles.dropdownMenu, styles.dropdownMenuProject)}>
                    {/* TODO: Replace hardcoded project list with projects from v3 database with the following requirements:
                    // 1. Project list fetching: Load available projects from database
                    //    - Data source: GET /api/projects endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
                    //    - Database table: PostgreSQL `projects` table
                    //    - Include project names, IDs, and status for display
                    // 2. Project selection: Allow selecting project for task assignment
                    //    - Store selected project ID when task is created
                    //    - Update task.project_id when project is selected
                    //    - Support filtering projects by current user's access
                    // 3. Project display: Show project name and status indicator
                    //    - Display project name in dropdown
                    //    - Show status indicator (green dot for active projects)
                    //    - Handle empty project list gracefully */}
                    {["Spotify", "Netflix", "Amazon", "Google"].map((proj) => (
                      <button
                        key={proj}
                        onClick={() => {
                          setProject(proj);
                          setShowProjectMenu(false);
                        }}
                        className={styles.dropdownMenuItem}
                      >
                        <span className={styles.projectIndicator}></span>
                        {proj}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </MetadataRow>

            {/* Tags */}
            <MetadataRow label="Tags">
              <div className={styles.tagsContainer}>
                {tags.map((tag) => (
                  <TagChip key={tag} tag={tag} onRemove={handleRemoveTag} />
                ))}
                {tags.length < 5 && (
                  <input
                    type="text"
                    placeholder={tags.length === 0 ? "Add tags..." : ""}
                    value={tagInput}
                    onChange={(e) => setTagInput(e.target.value)}
                    onKeyPress={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        handleAddTag();
                      }
                    }}
                    className={styles.tagInput}
                  />
                )}
              </div>
            </MetadataRow>
          </div>

          {/* Sub-tasks */}
          <div className={styles.subtasksSection}>
            <div className={styles.subtasksHeader}>
              <button className={styles.subtasksHeaderButton}>
                <ChevronDown className={styles.subtasksHeaderIcon} />
                <span>Sub-tasks</span>
              </button>
              <div className={styles.subtasksActions}>
                <button
                  onClick={() => setIsAddingSubtask(true)}
                  className={styles.subtasksActionButton}
                >
                  <Plus className={styles.subtasksActionIcon} />
                </button>
                <button className={styles.subtasksActionButton}>
                  <MoreHorizontal className={styles.subtasksActionIcon} />
                </button>
              </div>
            </div>

            <div className={styles.subtasksList}>
              {subtasks.map((task) => (
                <div
                  key={task.id}
                  className={styles.subtaskItemContainer}
                >
                  <button
                    onClick={() => toggleSubtask(task.id)}
                    className={styles.subtaskCheckboxButton}
                  >
                    {task.completed ? (
                      <CheckCircle2 className={cn(styles.subtaskCheckboxIcon, styles.subtaskCheckboxIconCompleted)} />
                    ) : (
                      <Circle className={cn(styles.subtaskCheckboxIcon, styles.subtaskCheckboxIconIncomplete)} />
                    )}
                  </button>
                  <span
                    className={cn(
                      styles.subtaskTitle,
                      task.completed && styles.subtaskTitleCompleted
                    )}
                  >
                    {task.title}
                  </span>
                  <div className={styles.subtaskActions}>
                    <button
                      onClick={() => handleRemoveSubtask(task.id)}
                      className={styles.subtaskRemoveButton}
                    >
                      <X className={styles.subtaskRemoveIcon} />
                    </button>
                  </div>
                </div>
              ))}

              {isAddingSubtask && (
                <div className={styles.subtaskAddContainer}>
                  <Circle className={styles.subtaskAddIcon} />
                  <input
                    type="text"
                    placeholder="Subtask title"
                    value={newSubtask}
                    onChange={(e) => setNewSubtask(e.target.value)}
                    onKeyPress={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        handleAddSubtask();
                      } else if (e.key === "Escape") {
                        setIsAddingSubtask(false);
                        setNewSubtask("");
                      }
                    }}
                    onBlur={() => {
                      if (!newSubtask.trim()) {
                        setIsAddingSubtask(false);
                      }
                    }}
                    className={styles.subtaskAddInput}
                    autoFocus
                  />
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className={styles.modalFooter}>
          <button
            onClick={() => onOpenChange(false)}
            className={styles.cancelButton}
          >
            Cancel
          </button>
          <button
            onClick={handleCreate}
            disabled={!title.trim()}
            className={styles.createButton}
          >
            Create Task
          </button>
        </div>
      </div>
    </div>
  );
}
