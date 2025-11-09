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
} from "./compounds";

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
    <div className="fixed inset-0 bg-black bg-opacity-60 flex items-center justify-center p-4 z-50">
      <div className="bg-zinc-800 rounded-lg w-full max-w-2xl text-white shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-zinc-700">
          <button
            onClick={() => onOpenChange(false)}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
          <div className="flex items-center gap-2 text-sm text-gray-400">
            <span>New Task</span>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Title */}
          <div>
            <input
              type="text"
              placeholder="Task title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="w-full bg-transparent border-none outline-none text-white text-2xl font-semibold placeholder:text-gray-600 mb-2"
              autoFocus
            />
            <textarea
              placeholder="Add a description for this task..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full bg-transparent border-none outline-none text-sm text-gray-400 placeholder:text-gray-600 resize-none leading-relaxed"
              rows={3}
            />
          </div>

          {/* Metadata Grid */}
          <div className="space-y-3 text-sm">
            {/* Status */}
            <MetadataRow label="Status">
              <div className="relative">
                <StatusBadge
                  status={status}
                  config={taskStatusConfig[status]}
                  onClick={() => setShowStatusMenu(!showStatusMenu)}
                />
                {showStatusMenu && (
                  <div className="absolute top-full left-0 mt-2 bg-white rounded-lg shadow-xl py-2 z-10 min-w-[180px]">
                    {(Object.keys(taskStatusConfig) as TaskStatus[]).map(
                      (key) => (
                        <button
                          key={key}
                          onClick={() => {
                            setStatus(key);
                            setShowStatusMenu(false);
                          }}
                          className={`w-full flex items-center gap-2 px-4 py-2 hover:bg-gray-100 transition-colors ${taskStatusConfig[key].color}`}
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
              <div className="flex items-center gap-2">
                <div className="w-5 h-5 bg-orange-500 rounded-full flex items-center justify-center text-xs font-medium">
                  {assignees ? assignees[0].toUpperCase() : "U"}
                </div>
                <input
                  type="text"
                  placeholder="Add assignees"
                  value={assignees}
                  onChange={(e) => setAssignees(e.target.value)}
                  className="bg-transparent border-none outline-none text-white placeholder:text-gray-600"
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
                className="bg-transparent border-none outline-none text-white placeholder:text-gray-600"
              />
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

            {/* Project */}
            <MetadataRow label="Project">
              <div className="relative">
                <button
                  onClick={() => setShowProjectMenu(!showProjectMenu)}
                  className="flex items-center gap-2 hover:opacity-80 transition-opacity"
                >
                  {project ? (
                    <>
                      <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                      <span>{project}</span>
                    </>
                  ) : (
                    <span className="text-gray-600">Add project</span>
                  )}
                </button>
                {showProjectMenu && (
                  <div className="absolute top-full left-0 mt-2 bg-white rounded-lg shadow-xl py-2 z-10 min-w-[160px]">
                    {/* TODO: Replace hardcoded project list with projects from v3 database (see TaskModal.tsx for detailed requirements) */}
                    {["Spotify", "Netflix", "Amazon", "Google"].map((proj) => (
                      <button
                        key={proj}
                        onClick={() => {
                          setProject(proj);
                          setShowProjectMenu(false);
                        }}
                        className="w-full flex items-center gap-2 px-4 py-2 text-gray-700 hover:bg-gray-100 transition-colors"
                      >
                        <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                        {proj}
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
                    className="bg-transparent border-none outline-none text-white placeholder:text-gray-600 text-xs min-w-[80px]"
                  />
                )}
              </div>
            </MetadataRow>
          </div>

          {/* Sub-tasks */}
          <div>
            <div className="flex items-center justify-between mb-3">
              <button className="flex items-center gap-2 text-sm text-gray-400 hover:text-white transition-colors">
                <ChevronDown className="w-4 h-4" />
                <span>Sub-tasks</span>
              </button>
              <div className="flex gap-1">
                <button
                  onClick={() => setIsAddingSubtask(true)}
                  className="p-1 hover:bg-zinc-700 rounded transition-colors"
                >
                  <Plus className="w-4 h-4 text-gray-400" />
                </button>
                <button className="p-1 hover:bg-zinc-700 rounded transition-colors">
                  <MoreHorizontal className="w-4 h-4 text-gray-400" />
                </button>
              </div>
            </div>

            <div className="space-y-2">
              {subtasks.map((task) => (
                <div
                  key={task.id}
                  className="flex items-center gap-3 p-2 hover:bg-zinc-700 rounded transition-colors group"
                >
                  <button
                    onClick={() => toggleSubtask(task.id)}
                    className="flex-shrink-0"
                  >
                    {task.completed ? (
                      <CheckCircle2 className="w-5 h-5 text-green-500" />
                    ) : (
                      <Circle className="w-5 h-5 text-gray-600" />
                    )}
                  </button>
                  <span
                    className={
                      task.completed ? "text-gray-500 line-through" : ""
                    }
                  >
                    {task.title}
                  </span>
                  <div className="ml-auto flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => handleRemoveSubtask(task.id)}
                      className="text-gray-400 hover:text-white"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              ))}

              {isAddingSubtask && (
                <div className="flex items-center gap-2 p-2">
                  <Circle className="w-5 h-5 text-gray-600 flex-shrink-0" />
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
                    className="flex-1 bg-transparent border-none outline-none text-white placeholder:text-gray-600"
                    autoFocus
                  />
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-zinc-700">
          <button
            onClick={() => onOpenChange(false)}
            className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleCreate}
            disabled={!title.trim()}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Create Task
          </button>
        </div>
      </div>
    </div>
  );
}
