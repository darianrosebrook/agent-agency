import { useState } from "react";
import { X, Check, ChevronDown, Plus, MoreHorizontal, Circle, CheckCircle2 } from "lucide-react";

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

type Status = 'backlog' | 'todo' | 'in-progress' | 'done';
type Priority = 'low' | 'medium' | 'high';

const statusConfig = {
  'backlog': { label: 'Backlog', color: 'bg-gray-100 text-gray-700', icon: 'dashed-circle' },
  'todo': { label: 'Todo', color: 'bg-blue-100 text-blue-700', icon: 'circle' },
  'in-progress': { label: 'In-progress', color: 'bg-orange-100 text-orange-700', icon: 'half-circle' },
  'done': { label: 'Done', color: 'bg-green-100 text-green-700', icon: 'check' }
};

const priorityConfig = {
  'low': { label: 'Low', color: 'text-gray-400', icon: '▼' },
  'medium': { label: 'Medium', color: 'text-green-500', icon: '▲' },
  'high': { label: 'High', color: 'text-red-500', icon: '▲▲' }
};

const StatusIcon = ({ type }: { type: string }) => {
  if (type === 'dashed-circle') {
    return (
      <svg className="w-4 h-4" viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="2" strokeDasharray="2 2" />
      </svg>
    );
  }
  if (type === 'circle') {
    return (
      <svg className="w-4 h-4" viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="2" />
      </svg>
    );
  }
  if (type === 'half-circle') {
    return (
      <svg className="w-4 h-4" viewBox="0 0 16 16" fill="none">
        <path d="M8 2 A6 6 0 0 1 8 14 Z" fill="currentColor" />
        <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="2" fill="none" />
      </svg>
    );
  }
  if (type === 'check') {
    return <Check className="w-4 h-4" />;
  }
  return null;
};

export function NewTaskModal({
  open,
  onOpenChange,
  onCreateTask,
  defaultStatus = "backlog",
}: NewTaskModalProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState<Status>(defaultStatus);
  const [priority, setPriority] = useState<Priority>('medium');
  const [assignees, setAssignees] = useState('');
  const [dueDate, setDueDate] = useState('');
  const [project, setProject] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState('');
  const [subtasks, setSubtasks] = useState<{ id: number; title: string; completed: boolean }[]>([]);
  const [newSubtask, setNewSubtask] = useState('');
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
      setPriority('medium');
      setAssignees('');
      setDueDate('');
      setProject('');
      setTags([]);
      setTagInput('');
      setSubtasks([]);
      setNewSubtask('');
      setIsAddingSubtask(false);
      onOpenChange(false);
    }
  };

  const handleAddTag = () => {
    if (tagInput.trim() && !tags.includes(tagInput.trim())) {
      setTags([...tags, tagInput.trim()]);
      setTagInput('');
    }
  };

  const handleRemoveTag = (tagToRemove: string) => {
    setTags(tags.filter(tag => tag !== tagToRemove));
  };

  const handleAddSubtask = () => {
    if (newSubtask.trim()) {
      setSubtasks([...subtasks, { 
        id: Date.now(), 
        title: newSubtask.trim(), 
        completed: false 
      }]);
      setNewSubtask('');
      setIsAddingSubtask(false);
    }
  };

  const toggleSubtask = (id: number) => {
    setSubtasks(subtasks.map(task => 
      task.id === id ? { ...task, completed: !task.completed } : task
    ));
  };

  const handleRemoveSubtask = (id: number) => {
    setSubtasks(subtasks.filter(task => task.id !== id));
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
            <div className="grid grid-cols-[120px_1fr] items-center">
              <div className="text-gray-400">Status</div>
              <div className="relative">
                <button
                  onClick={() => setShowStatusMenu(!showStatusMenu)}
                  className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-full ${statusConfig[status].color} font-medium hover:opacity-80 transition-opacity`}
                >
                  <StatusIcon type={statusConfig[status].icon} />
                  {statusConfig[status].label}
                </button>
                {showStatusMenu && (
                  <div className="absolute top-full left-0 mt-2 bg-white rounded-lg shadow-xl py-2 z-10 min-w-[180px]">
                    {(Object.keys(statusConfig) as Status[]).map((key) => (
                      <button
                        key={key}
                        onClick={() => {
                          setStatus(key);
                          setShowStatusMenu(false);
                        }}
                        className={`w-full flex items-center gap-2 px-4 py-2 hover:bg-gray-100 transition-colors ${statusConfig[key].color}`}
                      >
                        <StatusIcon type={statusConfig[key].icon} />
                        {statusConfig[key].label}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>

            {/* Assignees */}
            <div className="grid grid-cols-[120px_1fr] items-center">
              <div className="text-gray-400">Assignees</div>
              <div className="flex items-center gap-2">
                <div className="w-5 h-5 bg-orange-500 rounded-full flex items-center justify-center text-xs font-medium">
                  {assignees ? assignees[0].toUpperCase() : 'U'}
                </div>
                <input
                  type="text"
                  placeholder="Add assignees"
                  value={assignees}
                  onChange={(e) => setAssignees(e.target.value)}
                  className="bg-transparent border-none outline-none text-white placeholder:text-gray-600"
                />
              </div>
            </div>

            {/* Due date */}
            <div className="grid grid-cols-[120px_1fr] items-center">
              <div className="text-gray-400">Due date</div>
              <input
                type="text"
                placeholder="Set due date"
                value={dueDate}
                onChange={(e) => setDueDate(e.target.value)}
                className="bg-transparent border-none outline-none text-white placeholder:text-gray-600"
              />
            </div>

            {/* Priority */}
            <div className="grid grid-cols-[120px_1fr] items-center">
              <div className="text-gray-400">Priority</div>
              <div className="relative">
                <button
                  onClick={() => setShowPriorityMenu(!showPriorityMenu)}
                  className="flex items-center gap-2 hover:opacity-80 transition-opacity"
                >
                  <span className={priorityConfig[priority].color}>{priorityConfig[priority].icon}</span>
                  <span>{priorityConfig[priority].label}</span>
                </button>
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
                        <span className={priorityConfig[key].color}>{priorityConfig[key].icon}</span>
                        {priorityConfig[key].label}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>

            {/* Project */}
            <div className="grid grid-cols-[120px_1fr] items-center">
              <div className="text-gray-400">Project</div>
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
                    {['Spotify', 'Netflix', 'Amazon', 'Google'].map((proj) => (
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
            </div>

            {/* Tags */}
            <div className="grid grid-cols-[120px_1fr] items-center">
              <div className="text-gray-400">Tags</div>
              <div className="flex gap-2">
                {tags.map((tag) => (
                  <span 
                    key={tag}
                    onClick={() => handleRemoveTag(tag)}
                    className="px-2 py-1 bg-zinc-700 rounded text-xs cursor-pointer hover:bg-zinc-600 transition-colors"
                  >
                    {tag}
                  </span>
                ))}
                {tags.length < 5 && (
                  <input
                    type="text"
                    placeholder={tags.length === 0 ? "Add tags..." : ""}
                    value={tagInput}
                    onChange={(e) => setTagInput(e.target.value)}
                    onKeyPress={(e) => {
                      if (e.key === 'Enter') {
                        e.preventDefault();
                        handleAddTag();
                      }
                    }}
                    className="bg-transparent border-none outline-none text-white placeholder:text-gray-600 text-xs min-w-[80px]"
                  />
                )}
              </div>
            </div>
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
              {subtasks.map(task => (
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
                  <span className={task.completed ? 'text-gray-500 line-through' : ''}>
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
                      if (e.key === 'Enter') {
                        e.preventDefault();
                        handleAddSubtask();
                      } else if (e.key === 'Escape') {
                        setIsAddingSubtask(false);
                        setNewSubtask('');
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
