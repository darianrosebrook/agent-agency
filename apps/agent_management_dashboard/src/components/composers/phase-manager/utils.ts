import type { Task } from './types';

export function calculateTaskProgress(task: Task): number {
  if (task.subtasks.length === 0) return 0;
  const completed = task.subtasks.filter((s) => s.completed).length;
  return Math.round((completed / task.subtasks.length) * 100);
}













