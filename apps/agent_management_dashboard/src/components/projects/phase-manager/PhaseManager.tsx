'use client';

import { useState } from 'react';
import { PhaseHeader } from './PhaseHeader';
import { PhaseItem } from './PhaseItem';
import { initialPhases } from './initialPhases';
import type { Phase, PhaseManagerProps, Subtask, ContextChip } from './types';
import styles from './PhaseManager.module.scss';

export function PhaseManager({
  initialData = initialPhases,
  onSaveToProject,
}: PhaseManagerProps) {
  // TODO: Replace hardcoded initial phases with project phases from v3 database with the following requirements:
  // 1. Phase data fetching: Load project phases and tasks from database
  //    - Data source: GET /api/projects/:projectId/phases endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database tables: PostgreSQL `milestones` (phases) and `tasks` tables
  //    - Include phase numbers, titles, descriptions, and associated tasks
  // 2. Task data fetching: Load tasks with subtasks and context chips
  //    - Data source: GET /api/projects/:projectId/tasks endpoint returning tasks with subtasks
  //    - Database table: PostgreSQL `tasks` table with subtask relationships
  //    - Include task titles, descriptions, subtasks, and context chips
  // 3. Phase persistence: Save phase and task updates to database
  //    - Data source: PATCH /api/projects/:projectId/phases/:phaseId and PATCH /api/projects/:projectId/tasks/:taskId endpoints
  //    - Update phase titles, descriptions, and task details
  //    - Persist subtask additions, deletions, and completion status
  // 4. Context chip persistence: Save context chip additions and removals
  //    - Data source: POST /api/projects/:projectId/tasks/:taskId/context-chips and DELETE /api/projects/:projectId/tasks/:taskId/context-chips/:chipId endpoints
  //    - Store file references, tool references, and other context data
  const [phases, setPhases] = useState<Phase[]>(initialData);

  const updateTaskTitle = (
    phaseId: string,
    taskId: string,
    newTitle: string
  ) => {
    setPhases(
      phases.map((phase) => {
        if (phase.id === phaseId) {
          return {
            ...phase,
            tasks: phase.tasks.map((task) =>
              task.id === taskId ? { ...task, title: newTitle } : task
            ),
          };
        }
        return phase;
      })
    );
  };

  const updateTaskDescription = (
    phaseId: string,
    taskId: string,
    newDescription: string
  ) => {
    setPhases(
      phases.map((phase) => {
        if (phase.id === phaseId) {
          return {
            ...phase,
            tasks: phase.tasks.map((task) =>
              task.id === taskId
                ? { ...task, description: newDescription }
                : task
            ),
          };
        }
        return phase;
      })
    );
  };

  const addSubtask = (phaseId: string, taskId: string) => {
    setPhases(
      phases.map((phase) => {
        if (phase.id === phaseId) {
          return {
            ...phase,
            tasks: phase.tasks.map((task) => {
              if (task.id === taskId) {
                const newSubtask: Subtask = {
                  id: `subtask-${Date.now()}`,
                  text: 'New subtask',
                  completed: false,
                };
                return {
                  ...task,
                  subtasks: [...task.subtasks, newSubtask],
                };
              }
              return task;
            }),
          };
        }
        return phase;
      })
    );
  };

  const deleteSubtask = (
    phaseId: string,
    taskId: string,
    subtaskId: string
  ) => {
    setPhases(
      phases.map((phase) => {
        if (phase.id === phaseId) {
          return {
            ...phase,
            tasks: phase.tasks.map((task) => {
              if (task.id === taskId) {
                return {
                  ...task,
                  subtasks: task.subtasks.filter((s) => s.id !== subtaskId),
                };
              }
              return task;
            }),
          };
        }
        return phase;
      })
    );
  };

  const toggleSubtask = (
    phaseId: string,
    taskId: string,
    subtaskId: string
  ) => {
    setPhases(
      phases.map((phase) => {
        if (phase.id === phaseId) {
          return {
            ...phase,
            tasks: phase.tasks.map((task) => {
              if (task.id === taskId) {
                return {
                  ...task,
                  subtasks: task.subtasks.map((s) =>
                    s.id === subtaskId ? { ...s, completed: !s.completed } : s
                  ),
                };
              }
              return task;
            }),
          };
        }
        return phase;
      })
    );
  };

  const addContextChip = (
    phaseId: string,
    taskId: string,
    type: 'file' | 'reference' | 'tool',
    label: string
  ) => {
    setPhases(
      phases.map((phase) => {
        if (phase.id === phaseId) {
          return {
            ...phase,
            tasks: phase.tasks.map((task) => {
              if (task.id === taskId) {
                const newChip: ContextChip = {
                  id: `chip-${Date.now()}`,
                  type,
                  label,
                };
                return {
                  ...task,
                  contextChips: [...task.contextChips, newChip],
                };
              }
              return task;
            }),
          };
        }
        return phase;
      })
    );
  };

  const removeContextChip = (
    phaseId: string,
    taskId: string,
    chipId: string
  ) => {
    setPhases(
      phases.map((phase) => {
        if (phase.id === phaseId) {
          return {
            ...phase,
            tasks: phase.tasks.map((task) => {
              if (task.id === taskId) {
                return {
                  ...task,
                  contextChips: task.contextChips.filter(
                    (c) => c.id !== chipId
                  ),
                };
              }
              return task;
            }),
          };
        }
        return phase;
      })
    );
  };

  return (
    <div className={styles.phaseManager}>
      <PhaseHeader onSaveToProject={onSaveToProject} />

      {phases.map((phase) => (
        <PhaseItem
          key={phase.id}
          phase={phase}
          onUpdateTaskTitle={(taskId, newTitle) =>
            updateTaskTitle(phase.id, taskId, newTitle)
          }
          onUpdateTaskDescription={(taskId, newDescription) =>
            updateTaskDescription(phase.id, taskId, newDescription)
          }
          onAddSubtask={(taskId) => addSubtask(phase.id, taskId)}
          onToggleSubtask={(taskId, subtaskId) =>
            toggleSubtask(phase.id, taskId, subtaskId)
          }
          onDeleteSubtask={(taskId, subtaskId) =>
            deleteSubtask(phase.id, taskId, subtaskId)
          }
          onAddContextChip={(taskId, type, label) =>
            addContextChip(phase.id, taskId, type, label)
          }
          onRemoveContextChip={(taskId, chipId) =>
            removeContextChip(phase.id, taskId, chipId)
          }
        />
      ))}
    </div>
  );
}
