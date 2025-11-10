export interface Subtask {
  id: string;
  text: string;
  completed: boolean;
}

export interface ContextChip {
  id: string;
  type: 'file' | 'reference' | 'tool';
  label: string;
  icon?: string;
}

export interface Task {
  id: string;
  title: string;
  description: string;
  subtasks: Subtask[];
  contextChips: ContextChip[];
}

export interface Phase {
  id: string;
  number: number;
  title: string;
  description: string;
  tasks: Task[];
}

export interface PhaseManagerProps {
  initialData?: Phase[];
  onSaveToProject?: () => void;
}






