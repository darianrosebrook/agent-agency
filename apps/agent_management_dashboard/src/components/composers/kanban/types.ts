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
  title: string;
  description?: string;
  statusTags?: KanbanStatusTag[];
  metadata?: KanbanMetadata[];
  priority?: KanbanPriority;
  height?: number;
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
  className?: string;
}

export interface KanbanColumnProps {
  status: KanbanStatus;
  title: string;
  cardCount: number;
  cards: KanbanCardData[];
  onAddTask?: () => void;
  left?: number;
  className?: string;
}

export interface KanbanCardProps {
  title: string;
  description?: string;
  statusTags?: KanbanStatusTag[];
  metadata?: KanbanMetadata[];
  priority?: KanbanPriority;
  height?: number;
  className?: string;
}

export interface KanbanColumnHeaderProps {
  title: string;
  cardCount: number;
  onAddTask?: () => void;
  className?: string;
}

