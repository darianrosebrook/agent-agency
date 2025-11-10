import type { PriorityConfig } from './PriorityIndicator';

export type Priority = 'low' | 'medium' | 'high';

export const priorityConfig: Record<Priority, PriorityConfig> = {
  low: { label: 'Low', color: 'text-gray-400', icon: '▼' },
  medium: { label: 'Medium', color: 'text-green-500', icon: '▲' },
  high: { label: 'High', color: 'text-red-500', icon: '▲▲' },
};




