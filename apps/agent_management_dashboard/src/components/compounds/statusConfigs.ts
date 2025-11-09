import type { StatusIconType } from './StatusIcon';
import type { StatusConfig } from './StatusBadge';

// Project status configuration
export type ProjectStatus = 'planning' | 'in-progress' | 'on-hold' | 'completed';

export const projectStatusConfig: Record<ProjectStatus, StatusConfig> = {
  planning: {
    label: 'Planning',
    color: 'bg-gray-100 text-gray-700',
    icon: 'dashed-circle',
  },
  'in-progress': {
    label: 'In-progress',
    color: 'bg-orange-100 text-orange-700',
    icon: 'half-circle',
  },
  'on-hold': {
    label: 'On-hold',
    color: 'bg-blue-100 text-blue-700',
    icon: 'circle-arrow',
  },
  completed: {
    label: 'Completed',
    color: 'bg-green-100 text-green-700',
    icon: 'check',
  },
};

// Task status configuration
export type TaskStatus = 'backlog' | 'todo' | 'in-progress' | 'done';

export const taskStatusConfig: Record<TaskStatus, StatusConfig> = {
  backlog: {
    label: 'Backlog',
    color: 'bg-gray-100 text-gray-700',
    icon: 'dashed-circle',
  },
  todo: {
    label: 'Todo',
    color: 'bg-blue-100 text-blue-700',
    icon: 'circle',
  },
  'in-progress': {
    label: 'In-progress',
    color: 'bg-orange-100 text-orange-700',
    icon: 'half-circle',
  },
  done: {
    label: 'Done',
    color: 'bg-green-100 text-green-700',
    icon: 'check',
  },
};

