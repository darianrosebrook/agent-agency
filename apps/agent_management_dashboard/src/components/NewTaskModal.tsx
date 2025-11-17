"use client";

import { useState, useEffect } from "react";
import { listProjects, type ProjectListItem } from "../lib/api/projects";
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
} from "./compounds";
import { cn } from "./primitives/utils";
import styles from "./NewTaskModal.module.scss";

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
  const [projects, setProjects] = useState<ProjectListItem[]>([]);
  const [isLoadingProjects, setIsLoadingProjects] = useState(false);
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

  useEffect(() => {
    async function fetchProjects() {
      if (showProjectMenu && projects.length === 0) {
        setIsLoadingProjects(true);
        try {
          const response = await listProjects();
          setProjects(response.projects);
        } catch (error) {
          console.error("Failed to fetch projects:", error);
          setProjects([]);
        } finally {
          setIsLoadingProjects(false);
        }
      }
    }

    fetchProjects();
  }, [showProjectMenu, projects.length]);

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
            {/* TODO: Replace text input with user selection dropdown from v3 database (see TaskModal.tsx for detailed requirements) */}
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
                  <div className={styles.dropdownMenu} style={{ minWidth: '8.75rem' }}>
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
                  <div className={styles.dropdownMenu} style={{ minWidth: '10rem' }}>
                    {isLoadingProjects ? (
                      <div className={cn(styles.dropdownMenuItem, styles.dropdownMenuItemPriority)}>
                        <span>Loading projects...</span>
                      </div>
                    ) : projects.length === 0 ? (
                      <div className={cn(styles.dropdownMenuItem, styles.dropdownMenuItemPriority)}>
                        <span>No projects available</span>
                      </div>
                    ) : (
                      projects.map((proj) => (
                        <button
                          key={proj.project_id}
                          onClick={() => {
                            setProject(proj.name);
                            setShowProjectMenu(false);
                          }}
                          className={cn(styles.dropdownMenuItem, styles.dropdownMenuItemPriority)}
                        >
                          <span className={styles.projectIndicator}></span>
                          {proj.name}
                        </button>
                      ))
                    )}
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
          <div>
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
                    className={task.completed ? styles.subtaskTitleCompleted : styles.subtaskTitle}
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
