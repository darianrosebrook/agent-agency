import { useState } from 'react';
import { X, Check } from 'lucide-react';

interface NewProjectModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreateProject: (data: { name: string; summary?: string; description?: string; milestones?: string[] }) => void;
}

type Status = 'planning' | 'in-progress' | 'on-hold' | 'completed';
type Priority = 'low' | 'medium' | 'high';

const statusConfig = {
  'planning': { label: 'Planning', color: 'bg-gray-100 text-gray-700', icon: 'dashed-circle' },
  'in-progress': { label: 'In-progress', color: 'bg-orange-100 text-orange-700', icon: 'half-circle' },
  'on-hold': { label: 'On-hold', color: 'bg-blue-100 text-blue-700', icon: 'circle-arrow' },
  'completed': { label: 'Completed', color: 'bg-green-100 text-green-700', icon: 'check' }
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
  if (type === 'half-circle') {
    return (
      <svg className="w-4 h-4" viewBox="0 0 16 16" fill="none">
        <path d="M8 2 A6 6 0 0 1 8 14 Z" fill="currentColor" />
        <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="2" fill="none" />
      </svg>
    );
  }
  if (type === 'circle-arrow') {
    return (
      <svg className="w-4 h-4" viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="2" />
        <path d="M8 5 L8 11 M8 5 L6 7 M8 5 L10 7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      </svg>
    );
  }
  if (type === 'check') {
    return <Check className="w-4 h-4" />;
  }
  return null;
};

export function NewProjectModal({ open, onOpenChange, onCreateProject }: NewProjectModalProps) {
  const [projectName, setProjectName] = useState('');
  const [description, setDescription] = useState('');
  const [status, setStatus] = useState<Status>('planning');
  const [priority, setPriority] = useState<Priority>('medium');
  const [assignees, setAssignees] = useState('');
  const [dueDate, setDueDate] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState('');
  
  const [showStatusMenu, setShowStatusMenu] = useState(false);
  const [showPriorityMenu, setShowPriorityMenu] = useState(false);

  const handleCreate = () => {
    if (projectName.trim()) {
      onCreateProject({
        name: projectName.trim(),
        description: description.trim() || undefined,
      });
      // Reset form
      setProjectName('');
      setDescription('');
      setStatus('planning');
      setPriority('medium');
      setAssignees('');
      setDueDate('');
      setTags([]);
      setTagInput('');
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

  if (!open) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-60 flex items-center justify-center p-4">
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
            <span>New Project</span>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Title */}
          <div>
            <input
              type="text"
              placeholder="Project name"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value)}
              className="w-full bg-transparent border-none outline-none text-white text-2xl font-semibold placeholder:text-gray-600 mb-2"
              autoFocus
            />
            <textarea
              placeholder="Add a description for this project..."
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
            disabled={!projectName.trim()}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Create Project
          </button>
        </div>
      </div>
    </div>
  );
}
