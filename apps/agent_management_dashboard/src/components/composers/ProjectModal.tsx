"use client";

import React, { useState } from "react";
import { X, ChevronUp, ChevronDown } from "lucide-react";
import {
  StatusBadge,
  PriorityIndicator,
  MetadataRow,
  TagChip,
  projectStatusConfig,
  priorityConfig,
  type ProjectStatus,
  type Priority,
} from "../compounds";
import { cn } from "../ui/utils";
import styles from "./ProjectModal.module.scss";

interface NewProjectModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreateProject: (data: {
    name: string;
    summary?: string;
    description?: string;
    milestones?: string[];
  }) => void;
}

export function NewProjectModal({
  open,
  onOpenChange,
  onCreateProject,
}: NewProjectModalProps) {
  const [projectName, setProjectName] = useState("");
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState<ProjectStatus>("planning");
  const [priority, setPriority] = useState<Priority>("medium");
  const [assignees, setAssignees] = useState("");
  const [dueDate, setDueDate] = useState("");
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState("");

  const [showStatusMenu, setShowStatusMenu] = useState(false);
  const [showPriorityMenu, setShowPriorityMenu] = useState(false);
  const [isEditingTitle, setIsEditingTitle] = useState(true);
  const [isEditingDescription, setIsEditingDescription] = useState(false);

  const handleCreate = React.useCallback(() => {
    if (projectName.trim()) {
      onCreateProject({
        name: projectName.trim(),
        description: description.trim() || undefined,
      });
      // Reset form
      setProjectName("");
      setDescription("");
      setStatus("planning");
      setPriority("medium");
      setAssignees("");
      setDueDate("");
      setTags([]);
      setTagInput("");
      onOpenChange(false);
    }
  }, [projectName, description, onCreateProject, onOpenChange]);

  const handleAddTag = () => {
    if (tagInput.trim() && !tags.includes(tagInput.trim())) {
      setTags([...tags, tagInput.trim()]);
      setTagInput("");
    }
  };

  const handleRemoveTag = (tagToRemove: string) => {
    setTags(tags.filter((tag) => tag !== tagToRemove));
  };

  const handleSave = React.useCallback(() => {
    if (projectName.trim()) {
      handleCreate();
    } else {
      onOpenChange(false);
    }
  }, [projectName, handleCreate, onOpenChange]);

  // Handle ESC key to save and close
  React.useEffect(() => {
    if (!open) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        handleSave();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, handleSave]);

  if (!open) return null;

  return (
    <div
      className={styles.modalOverlay}
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          handleSave();
        }
      }}
    >
      <div
        className={styles.modalContent}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className={styles.modalHeader}>
          <button
            onClick={handleSave}
            className={styles.closeButton}
          >
            <X className={styles.closeButtonIcon} />
          </button>
          <div className={styles.modalHeaderTitle}>
            <span>New Project</span>
            <div className={styles.headerActions}>
              <button className={styles.headerActionButton}>
                <ChevronUp className={styles.headerActionIcon} />
              </button>
              <button className={styles.headerActionButton}>
                <ChevronDown className={styles.headerActionIcon} />
              </button>
            </div>
          </div>
        </div>

        {/* Content */}
        <div className={styles.modalBody}>
          {/* Title */}
          <div>
            {isEditingTitle ? (
              <input
                type="text"
                placeholder="Project name"
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
                onBlur={() => {
                  setIsEditingTitle(false);
                  if (projectName.trim()) {
                    handleSave();
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    setIsEditingTitle(false);
                    setIsEditingDescription(true);
                  }
                }}
                className={styles.titleInput}
                autoFocus
              />
            ) : (
              <h2
                className={styles.titleDisplay}
                onClick={() => setIsEditingTitle(true)}
              >
                {projectName || "New Project"}
              </h2>
            )}
            {isEditingDescription ? (
              <textarea
                placeholder="Add a description for this project..."
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                onBlur={() => {
                  setIsEditingDescription(false);
                  handleSave();
                }}
                className={styles.descriptionTextarea}
                rows={3}
                autoFocus
              />
            ) : (
              <p
                className={styles.descriptionDisplay}
                onClick={() => setIsEditingDescription(true)}
              >
                {description || "Add a description for this project..."}
              </p>
            )}
          </div>

          {/* Metadata Grid */}
          <div className={styles.metadataGrid}>
            {/* Status */}
            <MetadataRow label="Status">
              <div className="relative">
                <StatusBadge
                  status={status}
                  config={projectStatusConfig[status]}
                  onClick={() => setShowStatusMenu(!showStatusMenu)}
                />
                {showStatusMenu && (
                  <div className={styles.menuDropdown}>
                    {(Object.keys(projectStatusConfig) as ProjectStatus[]).map(
                      (key) => (
                        <button
                          key={key}
                          onClick={() => {
                            setStatus(key);
                            setShowStatusMenu(false);
                          }}
                          className={cn(styles.menuItem, projectStatusConfig[key].color)}
                        >
                          <StatusBadge
                            status={key}
                            config={projectStatusConfig[key]}
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
                {assignees ? (
                  <>
                    <div className={styles.assigneeAvatar}>
                      {assignees[0].toUpperCase()}
                    </div>
                    <span>{assignees}</span>
                  </>
                ) : (
                  <input
                    type="text"
                    placeholder="Add assignees"
                    value={assignees}
                    onChange={(e) => setAssignees(e.target.value)}
                    className={styles.assigneeInput}
                  />
                )}
              </div>
            </MetadataRow>

            {/* Due date */}
            <MetadataRow label="Due date">
              {dueDate ? (
                <span>{dueDate}</span>
              ) : (
                <input
                  type="text"
                  placeholder="Set due date"
                  value={dueDate}
                  onChange={(e) => setDueDate(e.target.value)}
                  onBlur={handleSave}
                  className={styles.metadataInput}
                />
              )}
            </MetadataRow>

            {/* Priority */}
            <MetadataRow label="Priority">
              <div className="relative">
                <PriorityIndicator
                  priority={priority}
                  config={priorityConfig[priority]}
                  onClick={() => setShowPriorityMenu(!showPriorityMenu)}
                />
                {showPriorityMenu && (
                  <div className={cn(styles.menuDropdown, styles.priorityMenu)}>
                    {(Object.keys(priorityConfig) as Priority[]).map((key) => (
                      <button
                        key={key}
                        onClick={() => {
                          setPriority(key);
                          setShowPriorityMenu(false);
                        }}
                        className={styles.menuItem}
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
                    onBlur={() => {
                      if (tagInput.trim()) {
                        handleAddTag();
                      }
                    }}
                    className={styles.tagInput}
                  />
                )}
              </div>
            </MetadataRow>
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
            disabled={!projectName.trim()}
            className={styles.confirmButton}
          >
            Confirm
          </button>
        </div>
      </div>
    </div>
  );
}
