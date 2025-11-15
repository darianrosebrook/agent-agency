'use client';

import { useState, useEffect, useCallback } from 'react';
import { PhaseHeader } from './PhaseHeader';
import { PhaseItem } from './PhaseItem';
import { initialPhases } from './initialPhases';
import type { Phase, PhaseManagerProps, Subtask, ContextChip, Task } from './types';
import styles from './PhaseManager.module.scss';
import {
  getProjectMilestones,
  getProjectTasks,
  updateProjectMilestone,
  updateProjectTask,
  type ProjectMilestone,
  type ProjectTask,
} from '../../../lib/api/projects';

export function PhaseManager({
  projectId,
  initialData = initialPhases,
  onSaveToProject,
}: PhaseManagerProps) {
  const [phases, setPhases] = useState<Phase[]>(initialData);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Map API milestone to Phase format
  const mapMilestoneToPhase = useCallback(
    (
      milestone: ProjectMilestone,
      allTasks: ProjectTask[],
      milestoneIndex: number
    ): Phase => {
      // Map tasks that might be associated with this milestone
      // Note: Tasks don't have milestone_id in current schema, so we'll group by order
      // This is a simplified mapping - in production, you'd want milestone_id on tasks
      const milestoneTasks: Task[] = allTasks.map((task, taskIndex) => ({
        id: task.task_id || task.id || `task-${taskIndex}`,
        title: task.title,
        description: task.description || '',
        subtasks: [], // TODO: Extract from task.metadata or separate endpoint
        contextChips: [], // TODO: Extract from task.context or separate endpoint
      }));

      // API milestone uses 'objective' for title, 'description' for description
      // PhaseManager uses 'title' for phase title, 'description' for phase description
      return {
        id: milestone.milestone_id || milestone.id || `milestone-${milestoneIndex}`,
        number: milestoneIndex + 1,
        title: milestone.title || milestone.objective || 'Untitled Phase',
        description: milestone.description || '',
        tasks: milestoneTasks,
      };
    },
    []
  );

  // Fetch phases and tasks from API when projectId is provided
  useEffect(() => {
    if (!projectId) {
      // Use initialData if no projectId provided
      setPhases(initialData);
      return;
    }

    const fetchData = async () => {
      setIsLoading(true);
      setError(null);

      try {
        // Fetch milestones and tasks in parallel
        const [milestones, tasksResponse] = await Promise.all([
          getProjectMilestones(projectId),
          getProjectTasks(projectId),
        ]);

        // Extract tasks array from response (API returns { tasks: [...] })
        const tasks = tasksResponse.tasks || [];

        // Map milestones to phases
        const mappedPhases: Phase[] = milestones.map((milestone, index) =>
          mapMilestoneToPhase(milestone, tasks, index)
        );

        // If no milestones, create default phases with tasks
        if (mappedPhases.length === 0 && tasks.length > 0) {
          const defaultPhase: Phase = {
            id: 'default-phase',
            number: 1,
            title: 'Default Phase',
            description: '',
            tasks: tasks.map((task, taskIndex) => ({
              id: task.task_id || task.id || `task-${taskIndex}`,
              title: task.title,
              description: task.description || '',
              subtasks: [],
              contextChips: [],
            })),
          };
          setPhases([defaultPhase]);
        } else {
          setPhases(mappedPhases.length > 0 ? mappedPhases : initialData);
        }
      } catch (err) {
        console.error('Failed to fetch project phases and tasks:', err);
        setError(err instanceof Error ? err.message : 'Failed to load project data');
        // Fallback to initialData on error
        setPhases(initialData);
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [projectId, initialData, mapMilestoneToPhase]);

  const updateTaskTitle = async (
    phaseId: string,
    taskId: string,
    newTitle: string
  ) => {
    // Optimistic update
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

    // Persist to API if projectId is provided
    if (projectId) {
      try {
        await updateProjectTask(projectId, taskId, { title: newTitle });
      } catch (err) {
        console.error('Failed to update task title:', err);
        setError(err instanceof Error ? err.message : 'Failed to update task');
        // TODO: Revert optimistic update on error
      }
    }
  };

  const updateTaskDescription = async (
    phaseId: string,
    taskId: string,
    newDescription: string
  ) => {
    // Optimistic update
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

    // Persist to API if projectId is provided
    if (projectId) {
      try {
        await updateProjectTask(projectId, taskId, { description: newDescription });
      } catch (err) {
        console.error('Failed to update task description:', err);
        setError(err instanceof Error ? err.message : 'Failed to update task');
        // TODO: Revert optimistic update on error
      }
    }
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

  if (isLoading) {
    return (
      <div className={styles.phaseManager}>
        <div>Loading project phases and tasks...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.phaseManager}>
        <div style={{ color: 'red' }}>Error: {error}</div>
      </div>
    );
  }

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
