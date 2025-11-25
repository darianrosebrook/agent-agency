"use client";

import { useCallback, useEffect, useState } from "react";
import {
  createProjectMilestone,
  createProjectTask,
  deleteProjectTask,
  getProjectMilestones,
  getProjectTasks,
  updateProjectMilestone,
  updateProjectTask,
  type ProjectMilestone,
  type ProjectTask,
} from "../../../lib/api/projects";
import { PhaseHeader } from "./PhaseHeader";
import { PhaseItem } from "./PhaseItem";
import styles from "./PhaseManager.module.scss";
import { initialPhases } from "./initialPhases";
import type {
  ContextChip,
  Phase,
  PhaseManagerProps,
  Subtask,
  Task,
} from "./types";

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
        description: task.description || "",
        subtasks: [], // TODO: Extract from task.metadata or separate endpoint
        contextChips: [], // TODO: Extract from task.context or separate endpoint
      }));

      // API milestone uses 'objective' for title, 'description' for description
      // PhaseManager uses 'title' for phase title, 'description' for phase description
      return {
        id:
          milestone.milestone_id ||
          milestone.id ||
          `milestone-${milestoneIndex}`,
        number: milestoneIndex + 1,
        title: milestone.title || milestone.objective || "Untitled Phase",
        description: milestone.description || "",
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
            id: "default-phase",
            number: 1,
            title: "Default Phase",
            description: "",
            tasks: tasks.map((task, taskIndex) => ({
              id: task.task_id || task.id || `task-${taskIndex}`,
              title: task.title,
              description: task.description || "",
              subtasks: [],
              contextChips: [],
            })),
          };
          setPhases([defaultPhase]);
        } else {
          setPhases(mappedPhases.length > 0 ? mappedPhases : initialData);
        }
      } catch (err) {
        console.error("Failed to fetch project phases and tasks:", err);
        setError(
          err instanceof Error ? err.message : "Failed to load project data"
        );
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
        console.error("Failed to update task title:", err);
        setError(err instanceof Error ? err.message : "Failed to update task");
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
        await updateProjectTask(projectId, taskId, {
          description: newDescription,
        });
      } catch (err) {
        console.error("Failed to update task description:", err);
        setError(err instanceof Error ? err.message : "Failed to update task");
        // TODO: Revert optimistic update on error
      }
    }
  };

  const updatePhaseTitle = async (phaseId: string, newTitle: string) => {
    // Optimistic update
    setPhases(
      phases.map((phase) =>
        phase.id === phaseId ? { ...phase, title: newTitle } : phase
      )
    );

    // Persist to API if projectId is provided
    if (projectId) {
      try {
        await updateProjectMilestone(projectId, phaseId, { title: newTitle });
      } catch (err) {
        console.error("Failed to update phase title:", err);
        setError(err instanceof Error ? err.message : "Failed to update phase");
      }
    }
  };

  const updatePhaseDescription = async (
    phaseId: string,
    newDescription: string
  ) => {
    // Optimistic update
    setPhases(
      phases.map((phase) =>
        phase.id === phaseId ? { ...phase, description: newDescription } : phase
      )
    );

    // Persist to API if projectId is provided
    if (projectId) {
      try {
        await updateProjectMilestone(projectId, phaseId, {
          description: newDescription,
        });
      } catch (err) {
        console.error("Failed to update phase description:", err);
        setError(err instanceof Error ? err.message : "Failed to update phase");
      }
    }
  };

  const toggleTask = async (phaseId: string, taskId: string) => {
    const phase = phases.find((p) => p.id === phaseId);
    const task = phase?.tasks.find((t) => t.id === taskId);
    if (!task) return;

    const newCompleted = !task.completed;

    // Optimistic update
    setPhases(
      phases.map((phase) => {
        if (phase.id === phaseId) {
          return {
            ...phase,
            tasks: phase.tasks.map((t) =>
              t.id === taskId ? { ...t, completed: newCompleted } : t
            ),
          };
        }
        return phase;
      })
    );

    // Persist to API if projectId is provided
    if (projectId) {
      try {
        await updateProjectTask(projectId, taskId, {
          status: newCompleted ? "completed" : "in_progress",
          completed_at: newCompleted ? new Date().toISOString() : null,
        });
      } catch (err) {
        console.error("Failed to toggle task completion:", err);
        setError(err instanceof Error ? err.message : "Failed to update task");
      }
    }
  };

  const updateSubtaskText = (
    phaseId: string,
    taskId: string,
    subtaskId: string,
    newText: string
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
                    s.id === subtaskId ? { ...s, text: newText } : s
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
    // Note: Subtasks are stored in task metadata, so we'd need to update the task
    // For now, this is local-only until we have subtask persistence in the API
  };

  const addPhase = async (
    title: string = "New Phase",
    description: string = ""
  ) => {
    const newPhase: Phase = {
      id: `phase-${Date.now()}`,
      number: phases.length + 1,
      title,
      description,
      tasks: [],
    };

    // Optimistic update
    setPhases([...phases, newPhase]);

    // Persist to API if projectId is provided
    if (projectId) {
      try {
        const milestone = await createProjectMilestone(projectId, {
          title,
          description,
        });
        // Update phase ID with API response
        setPhases(
          phases.map((phase) =>
            phase.id === newPhase.id
              ? {
                  ...phase,
                  id: milestone.milestone_id || milestone.id || newPhase.id,
                }
              : phase
          )
        );
      } catch (err) {
        console.error("Failed to create phase:", err);
        setError(err instanceof Error ? err.message : "Failed to create phase");
        // Revert optimistic update
        setPhases(phases);
      }
    }
  };

  const deletePhase = async (phaseId: string) => {
    // Optimistic update
    const phaseToDelete = phases.find((p) => p.id === phaseId);
    setPhases(phases.filter((phase) => phase.id !== phaseId));

    // Persist to API if projectId is provided
    if (projectId) {
      try {
        // Note: API doesn't have delete milestone endpoint yet, so this is local-only
        // await deleteProjectMilestone(projectId, phaseId);
      } catch (err) {
        console.error("Failed to delete phase:", err);
        setError(err instanceof Error ? err.message : "Failed to delete phase");
        // Revert optimistic update
        if (phaseToDelete) {
          setPhases([...phases, phaseToDelete]);
        }
      }
    }
  };

  const addTask = async (
    phaseId: string,
    title: string = "New Task",
    description: string = ""
  ) => {
    const newTask: Task = {
      id: `task-${Date.now()}`,
      title,
      description,
      completed: false,
      subtasks: [],
      contextChips: [],
    };

    // Optimistic update
    setPhases(
      phases.map((phase) =>
        phase.id === phaseId
          ? { ...phase, tasks: [...phase.tasks, newTask] }
          : phase
      )
    );

    // Persist to API if projectId is provided
    if (projectId) {
      try {
        const task = await createProjectTask(projectId, {
          title,
          description,
        });
        // Update task ID with API response
        setPhases(
          phases.map((phase) => {
            if (phase.id === phaseId) {
              return {
                ...phase,
                tasks: phase.tasks.map((t) =>
                  t.id === newTask.id
                    ? { ...t, id: task.task_id || task.id || newTask.id }
                    : t
                ),
              };
            }
            return phase;
          })
        );
      } catch (err) {
        console.error("Failed to create task:", err);
        setError(err instanceof Error ? err.message : "Failed to create task");
        // Revert optimistic update
        setPhases(
          phases.map((phase) =>
            phase.id === phaseId
              ? {
                  ...phase,
                  tasks: phase.tasks.filter((t) => t.id !== newTask.id),
                }
              : phase
          )
        );
      }
    }
  };

  const deleteTask = async (phaseId: string, taskId: string) => {
    // Optimistic update
    const phase = phases.find((p) => p.id === phaseId);
    const taskToDelete = phase?.tasks.find((t) => t.id === taskId);
    setPhases(
      phases.map((phase) =>
        phase.id === phaseId
          ? { ...phase, tasks: phase.tasks.filter((t) => t.id !== taskId) }
          : phase
      )
    );

    // Persist to API if projectId is provided
    if (projectId) {
      try {
        await deleteProjectTask(projectId, taskId);
      } catch (err) {
        console.error("Failed to delete task:", err);
        setError(err instanceof Error ? err.message : "Failed to delete task");
        // Revert optimistic update
        if (taskToDelete) {
          setPhases(
            phases.map((phase) =>
              phase.id === phaseId
                ? { ...phase, tasks: [...phase.tasks, taskToDelete] }
                : phase
            )
          );
        }
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
                  text: "New subtask",
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
    type: "file" | "reference" | "tool",
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

  // Expose API for AI agents to programmatically edit phases, tasks, and subtasks
  useEffect(() => {
    if (typeof window !== "undefined") {
      (window as any).phaseManagerAPI = {
        // Phase operations
        addPhase: (title?: string, description?: string) =>
          addPhase(title, description),
        updatePhaseTitle: (phaseId: string, title: string) =>
          updatePhaseTitle(phaseId, title),
        updatePhaseDescription: (phaseId: string, description: string) =>
          updatePhaseDescription(phaseId, description),
        deletePhase: (phaseId: string) => deletePhase(phaseId),
        getPhases: () => phases,

        // Task operations
        addTask: (phaseId: string, title?: string, description?: string) =>
          addTask(phaseId, title, description),
        updateTaskTitle: (phaseId: string, taskId: string, title: string) =>
          updateTaskTitle(phaseId, taskId, title),
        updateTaskDescription: (
          phaseId: string,
          taskId: string,
          description: string
        ) => updateTaskDescription(phaseId, taskId, description),
        toggleTask: (phaseId: string, taskId: string) =>
          toggleTask(phaseId, taskId),
        deleteTask: (phaseId: string, taskId: string) =>
          deleteTask(phaseId, taskId),
        getTasks: (phaseId: string) =>
          phases.find((p) => p.id === phaseId)?.tasks || [],

        // Subtask operations
        addSubtask: (phaseId: string, taskId: string) =>
          addSubtask(phaseId, taskId),
        updateSubtaskText: (
          phaseId: string,
          taskId: string,
          subtaskId: string,
          text: string
        ) => updateSubtaskText(phaseId, taskId, subtaskId, text),
        toggleSubtask: (phaseId: string, taskId: string, subtaskId: string) =>
          toggleSubtask(phaseId, taskId, subtaskId),
        deleteSubtask: (phaseId: string, taskId: string, subtaskId: string) =>
          deleteSubtask(phaseId, taskId, subtaskId),
      };
    }

    return () => {
      if (typeof window !== "undefined") {
        delete (window as any).phaseManagerAPI;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [phases]);

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
        <div style={{ color: "red" }}>Error: {error}</div>
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
          onUpdatePhaseTitle={(newTitle) =>
            updatePhaseTitle(phase.id, newTitle)
          }
          onUpdatePhaseDescription={(newDescription) =>
            updatePhaseDescription(phase.id, newDescription)
          }
          onUpdateTaskTitle={(taskId, newTitle) =>
            updateTaskTitle(phase.id, taskId, newTitle)
          }
          onUpdateTaskDescription={(taskId, newDescription) =>
            updateTaskDescription(phase.id, taskId, newDescription)
          }
          onToggleTask={(taskId) => toggleTask(phase.id, taskId)}
          onAddTask={() => addTask(phase.id)}
          onAddSubtask={(taskId) => addSubtask(phase.id, taskId)}
          onUpdateSubtaskText={(taskId, subtaskId, newText) =>
            updateSubtaskText(phase.id, taskId, subtaskId, newText)
          }
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

      <div className={styles.addPhaseContainer}>
        <button
          onClick={() => addPhase()}
          className={styles.addPhaseButton}
          type="button"
        >
          + Add Phase
        </button>
      </div>
    </div>
  );
}
