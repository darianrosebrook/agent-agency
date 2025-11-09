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
} from "./compounds";

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
      className="fixed inset-0 bg-black bg-opacity-60 flex items-center justify-center p-4 z-50"
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          handleSave();
        }
      }}
    >
      <div
        className="bg-zinc-800 rounded-lg w-full max-w-2xl text-white shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-zinc-700">
          <button
            onClick={handleSave}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
          <div className="flex items-center gap-2 text-sm text-gray-400">
            <span>New Project</span>
            <div className="flex gap-1">
              <button className="p-1 hover:bg-zinc-700 rounded">
                <ChevronUp className="w-4 h-4" />
              </button>
              <button className="p-1 hover:bg-zinc-700 rounded">
                <ChevronDown className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
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
                className="w-full bg-transparent border-none outline-none text-white text-2xl font-semibold placeholder:text-gray-600 mb-2"
                autoFocus
              />
            ) : (
              <h2
                className="text-2xl font-semibold mb-2 cursor-text"
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
                className="w-full bg-transparent border-none outline-none text-sm text-gray-400 placeholder:text-gray-600 resize-none leading-relaxed"
                rows={3}
                autoFocus
              />
            ) : (
              <p
                className="text-sm text-gray-400 leading-relaxed cursor-text"
                onClick={() => setIsEditingDescription(true)}
              >
                {description || "Add a description for this project..."}
              </p>
            )}
          </div>

          {/* Metadata Grid */}
          <div className="space-y-3 text-sm">
            {/* Status */}
            <MetadataRow label="Status">
              <div className="relative">
                <StatusBadge
                  status={status}
                  config={projectStatusConfig[status]}
                  onClick={() => setShowStatusMenu(!showStatusMenu)}
                />
                {showStatusMenu && (
                  <div className="absolute top-full left-0 mt-2 bg-white rounded-lg shadow-xl py-2 z-10 min-w-[180px]">
                    {(Object.keys(projectStatusConfig) as ProjectStatus[]).map(
                      (key) => (
                        <button
                          key={key}
                          onClick={() => {
                            setStatus(key);
                            setShowStatusMenu(false);
                          }}
                          className={`w-full flex items-center gap-2 px-4 py-2 hover:bg-gray-100 transition-colors ${projectStatusConfig[key].color}`}
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
            <MetadataRow label="Assignees">
              <div className="flex items-center gap-2">
                {assignees ? (
                  <>
                    <div className="w-5 h-5 bg-orange-500 rounded-full flex items-center justify-center text-xs font-medium">
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
                    className="bg-transparent border-none outline-none text-white placeholder:text-gray-600"
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
                  className="bg-transparent border-none outline-none text-white placeholder:text-gray-600"
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
                  <div className="absolute top-full left-0 mt-2 bg-white rounded-lg shadow-xl py-2 z-10 min-w-[140px]">
                    {(Object.keys(priorityConfig) as Priority[]).map((key) => (
                      <button
                        key={key}
                        onClick={() => {
                          setPriority(key);
                          setShowPriorityMenu(false);
                        }}
                        className="w-full flex items-center gap-2 px-4 py-2 text-gray-700 hover:bg-gray-100 transition-colors"
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
              <div className="flex gap-2 flex-wrap">
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
                    className="bg-transparent border-none outline-none text-white placeholder:text-gray-600 text-xs min-w-[80px]"
                  />
                )}
              </div>
            </MetadataRow>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-zinc-700">
          <button
            onClick={() => onOpenChange(false)}
            className="px-4 py-2 text-gray-400 hover:text-white transition-colors rounded hover:bg-zinc-700"
          >
            Cancel
          </button>
          <button
            onClick={handleCreate}
            disabled={!projectName.trim()}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
          >
            Confirm
          </button>
        </div>
      </div>
    </div>
  );
}
