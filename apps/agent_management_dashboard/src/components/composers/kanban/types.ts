// Kanban Board Types
// Consolidated type definitions for all Kanban components

export type KanbanStatus = "backlog" | "todo" | "in-progress" | "done";
export type KanbanPriority = "low" | "medium" | "high";

export interface KanbanStatusTag {
  label: string;
  icon?: React.ReactNode;
  bgColor?: string;
  textColor?: string;
}

export interface KanbanMetadata {
  icon: React.ReactNode | { path: string | string[]; size?: number };
  text: string;
}

export interface KanbanCardData {
  id: string; // Task ID for CRUD operations
  title: string;
  description?: string;
  statusTags?: KanbanStatusTag[];
  metadata?: KanbanMetadata[];
  priority?: KanbanPriority;
  height?: number;
  commentCount?: number; // Number of comments on this task
}

export interface KanbanColumnConfig {
  status: KanbanStatus;
  title: string;
  cardCount: number;
  cards: KanbanCardData[];
  onAddTask?: () => void;
}

export interface KanbanBoardProps {
  columns: KanbanColumnConfig[];
  onAddTask?: (status: KanbanStatus) => void;
  onTaskMove?: (taskId: string, newStatus: KanbanStatus) => void;
  onTaskEdit?: (taskId: string) => void;
  onTaskDelete?: (taskId: string) => void;
  onTaskViewComments?: (taskId: string) => void;
  className?: string;
}

export interface KanbanColumnProps {
  status: KanbanStatus;
  title: string;
  cardCount: number;
  cards: KanbanCardData[];
  onAddTask?: () => void;
  onTaskMove?: (taskId: string, newStatus: KanbanStatus) => void;
  onTaskEdit?: (taskId: string) => void;
  onTaskDelete?: (taskId: string) => void;
  onTaskViewComments?: (taskId: string) => void;
  left?: number;
  className?: string;
}

export interface KanbanCardProps {
  id: string; // Task ID
  title: string;
  description?: string;
  statusTags?: KanbanStatusTag[];
  metadata?: KanbanMetadata[];
  priority?: KanbanPriority;
  height?: number;
  commentCount?: number;
  className?: string;
  onEdit?: (taskId: string) => void;
  onDelete?: (taskId: string) => void;
  onViewComments?: (taskId: string) => void;
}

export interface KanbanColumnHeaderProps {
  title: string;
  cardCount: number;
  onAddTask?: () => void;
  className?: string;
}
