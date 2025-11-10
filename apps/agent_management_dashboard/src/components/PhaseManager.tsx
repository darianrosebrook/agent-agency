"use client";

import { useState } from "react";
import {
  X,
  ChevronDown,
  Upload,
  Link as LinkIcon,
  Wrench,
  CheckCircle2,
  Circle,
  CircleDashed,
  Trash2,
} from "lucide-react";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "./primitives/accordion";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "./primitives/dropdown-menu";
import { Button } from "./primitives/button";
import { cn } from "./primitives/utils";
import styles from "./PhaseManager.module.scss";

interface Subtask {
  id: string;
  text: string;
  completed: boolean;
}

interface ContextChip {
  id: string;
  type: "file" | "reference" | "tool";
  label: string;
  icon?: string;
}

interface Task {
  id: string;
  title: string;
  description: string;
  subtasks: Subtask[];
  contextChips: ContextChip[];
}

interface Phase {
  id: string;
  number: number;
  title: string;
  description: string;
  tasks: Task[];
}

const initialPhases: Phase[] = [
  {
    id: "phase-1",
    number: 1,
    title: "Research & Planning",
    description:
      "Understand the requirements and plan the architecture for a multi-modal RAG search UI tool.",
    tasks: [
      {
        id: "task-1",
        title: "Define core features",
        description:
          "Identify the key features needed for multi-modal RAG (Retrieval Augmented Generation) including text, image, and vector search capabilities.",
        subtasks: [],
        contextChips: [],
      },
      {
        id: "task-2",
        title: "Research vector databases",
        description:
          "Evaluate options like Pinecone, Weaviate, and Qdrant for storing and querying embeddings.",
        subtasks: [],
        contextChips: [],
      },
      {
        id: "task-3",
        title: "Design UI/UX wireframes",
        description:
          "Create mockups for the search interface, results display, and filter controls.",
        subtasks: [],
        contextChips: [],
      },
    ],
  },
  {
    id: "phase-2",
    number: 2,
    title: "Foundation Setup",
    description: "Set up the development environment and core infrastructure.",
    tasks: [
      {
        id: "task-4",
        title: "Set up integrations",
        description:
          "Configure API connections for vector database, embedding models, and LLM providers.",
        subtasks: [],
        contextChips: [],
      },
      {
        id: "task-5",
        title: "Initialize project structure",
        description:
          "Set up the repository with TypeScript, React, and necessary build tools.",
        subtasks: [],
        contextChips: [],
      },
    ],
  },
];

interface PhaseManagerProps {
  initialData?: Phase[];
  onSaveToProject?: () => void;
}

export function PhaseManager({
  initialData = initialPhases,
  onSaveToProject,
}: PhaseManagerProps) {
  const [phases, setPhases] = useState<Phase[]>(initialData);

  const calculateTaskProgress = (task: Task): number => {
    if (task.subtasks.length === 0) return 0;
    const completed = task.subtasks.filter((s) => s.completed).length;
    return Math.round((completed / task.subtasks.length) * 100);
  };

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

  const getChipIcon = (type: string) => {
    switch (type) {
      case "file":
        return <Upload className={styles.iconSmall} />;
      case "reference":
        return <LinkIcon className={styles.iconSmall} />;
      case "tool":
        return <Wrench className={styles.iconSmall} />;
      default:
        return null;
    }
  };

  return (
    <div className={styles.phaseManager}>
      {/* Header */}
      <div className={styles.header}>
        <h2 className={styles.headerTitle}>Project Plan</h2>
        <p className={styles.headerDescription}>
          Here&apos;s a comprehensive plan for building your multi-modal RAG
          search UI tool
        </p>

        {/* Action buttons */}
        <div className={styles.actionButtons}>
          <Button
            onClick={onSaveToProject}
            className={styles.addToProjectButton}
          >
            Add to Project
          </Button>
          <Button
            variant="outline"
            className={styles.startNewProjectButton}
          >
            Start New Project
          </Button>
        </div>
      </div>

      {/* Phases */}
      {phases.map((phase) => (
        <div key={phase.id} className={styles.phaseCard}>
          {/* Phase Header */}
          <div className={styles.phaseHeader}>
            <div className={styles.phaseHeaderTop}>
              <h3 className={styles.phaseTitle}>{phase.title}</h3>
              <span className={styles.phaseBadge}>
                Phase {phase.number}
              </span>
            </div>
            <p className={styles.phaseDescription}>{phase.description}</p>
          </div>

          {/* Tasks Accordion */}
          <Accordion type="multiple" className={styles.accordion}>
            {phase.tasks.map((task) => {
              const progress = calculateTaskProgress(task);

              return (
                <AccordionItem
                  key={task.id}
                  value={task.id}
                  className={styles.accordionItem}
                >
                  <AccordionTrigger className={styles.accordionTrigger}>
                    <div className={styles.taskHeader}>
                      {task.subtasks.length > 0 ? (
                        <div className={styles.taskProgress}>
                          <Circle className={styles.taskProgressIcon} />
                          <span className={styles.taskProgressText}>
                            {progress}%
                          </span>
                        </div>
                      ) : (
                        <CircleDashed className={styles.taskProgressIcon} />
                      )}
                      {/* Task Title (editable) */}
                      <input
                        type="text"
                        value={task.title}
                        onChange={(e) =>
                          updateTaskTitle(phase.id, task.id, e.target.value)
                        }
                        onClick={(e) => e.stopPropagation()}
                        className={styles.taskTitleInput}
                      />
                    </div>
                  </AccordionTrigger>
                  <AccordionContent className={styles.accordionContent}>
                    {/* Task Description (editable) */}
                    {task.description && (
                      <div className={styles.taskDescriptionContainer}>
                        <textarea
                          value={task.description}
                          onChange={(e) =>
                            updateTaskDescription(
                              phase.id,
                              task.id,
                              e.target.value
                            )
                          }
                          placeholder="Add a description..."
                          className={styles.taskDescription}
                        />
                      </div>
                    )}

                    {/* Context Chips */}
                    {task.contextChips.length > 0 && (
                      <div className={styles.contextChips}>
                        {task.contextChips.map((chip) => (
                          <div key={chip.id} className={styles.contextChip}>
                            {getChipIcon(chip.type)}
                            <span>{chip.label}</span>
                            <button
                              onClick={() =>
                                removeContextChip(phase.id, task.id, chip.id)
                              }
                              className={styles.contextChipRemoveButton}
                            >
                              <X className={styles.iconSmall} />
                            </button>
                          </div>
                        ))}
                      </div>
                    )}

                    {/* Subtasks */}
                    {task.subtasks.length > 0 && (
                      <div className={styles.subtasks}>
                        {task.subtasks.map((subtask) => (
                          <div key={subtask.id} className={styles.subtask}>
                            <button
                              onClick={() =>
                                toggleSubtask(phase.id, task.id, subtask.id)
                              }
                              className={styles.subtaskToggle}
                            >
                              {subtask.completed ? (
                                <CheckCircle2 className={styles.iconGreen} />
                              ) : (
                                <Circle className={styles.iconZinc} />
                              )}
                            </button>
                            <span
                              className={cn(
                                styles.subtaskText,
                                subtask.completed
                                  ? styles.subtaskTextCompleted
                                  : styles.subtaskTextIncomplete
                              )}
                            >
                              {subtask.text}
                            </span>
                            <button
                              onClick={() =>
                                deleteSubtask(phase.id, task.id, subtask.id)
                              }
                              className={styles.subtaskDelete}
                            >
                              <Trash2 className={styles.icon} />
                            </button>
                          </div>
                        ))}
                      </div>
                    )}

                    {/* Action Buttons */}
                    <div className={styles.actionButtonsContainer}>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => addSubtask(phase.id, task.id)}
                        className={styles.addSubtaskButton}
                      >
                        Add subtask
                      </Button>

                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            variant="outline"
                            size="sm"
                            className={styles.addContextButton}
                          >
                            Add context
                            <ChevronDown className={styles.iconWithMargin} />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent className={styles.dropdownContent}>
                          <DropdownMenuItem
                            className={styles.dropdownMenuItem}
                            onClick={() =>
                              addContextChip(
                                phase.id,
                                task.id,
                                "file",
                                "Uploaded file"
                              )
                            }
                          >
                            <Upload className={styles.iconWithMargin} />
                            Upload a file
                          </DropdownMenuItem>

                          <DropdownMenuSub>
                            <DropdownMenuSubTrigger className={styles.dropdownMenuItem}>
                              <LinkIcon className={styles.iconWithMargin} />
                              Reference a task
                            </DropdownMenuSubTrigger>
                            <DropdownMenuSubContent className={styles.dropdownSubContent}>
                              <DropdownMenuSub>
                                <DropdownMenuSubTrigger className={styles.dropdownMenuItem}>
                                  Previous projects
                                </DropdownMenuSubTrigger>
                                <DropdownMenuSubContent className={styles.dropdownSubContent}>
                                  <DropdownMenuItem
                                    className={styles.dropdownMenuItem}
                                    onClick={() =>
                                      addContextChip(
                                        phase.id,
                                        task.id,
                                        "reference",
                                        "Chats"
                                      )
                                    }
                                  >
                                    Chats
                                  </DropdownMenuItem>
                                  <DropdownMenuItem
                                    className={styles.dropdownMenuItem}
                                    onClick={() =>
                                      addContextChip(
                                        phase.id,
                                        task.id,
                                        "reference",
                                        "Artifacts"
                                      )
                                    }
                                  >
                                    Artifacts
                                  </DropdownMenuItem>
                                  <DropdownMenuItem
                                    className={styles.dropdownMenuItem}
                                    onClick={() =>
                                      addContextChip(
                                        phase.id,
                                        task.id,
                                        "reference",
                                        "Tasks"
                                      )
                                    }
                                  >
                                    Tasks
                                  </DropdownMenuItem>
                                </DropdownMenuSubContent>
                              </DropdownMenuSub>
                            </DropdownMenuSubContent>
                          </DropdownMenuSub>

                          <DropdownMenuSub>
                            <DropdownMenuSubTrigger className={styles.dropdownMenuItem}>
                              <Wrench className={styles.iconWithMargin} />
                              Tool selection
                            </DropdownMenuSubTrigger>
                            <DropdownMenuSubContent className={styles.dropdownSubContent}>
                              <DropdownMenuItem
                                className={styles.dropdownMenuItem}
                                onClick={() =>
                                  addContextChip(
                                    phase.id,
                                    task.id,
                                    "tool",
                                    "Research"
                                  )
                                }
                              >
                                Research
                              </DropdownMenuItem>
                              <DropdownMenuItem
                                className={styles.dropdownMenuItem}
                                onClick={() =>
                                  addContextChip(
                                    phase.id,
                                    task.id,
                                    "tool",
                                    "Plan mode"
                                  )
                                }
                              >
                                Plan mode
                              </DropdownMenuItem>
                              <DropdownMenuItem
                                className={styles.dropdownMenuItem}
                                onClick={() =>
                                  addContextChip(
                                    phase.id,
                                    task.id,
                                    "tool",
                                    "Scaffold"
                                  )
                                }
                              >
                                Scaffold
                              </DropdownMenuItem>
                              <DropdownMenuItem
                                className={styles.dropdownMenuItem}
                                onClick={() =>
                                  addContextChip(
                                    phase.id,
                                    task.id,
                                    "tool",
                                    "Audit"
                                  )
                                }
                              >
                                Audit
                              </DropdownMenuItem>
                            </DropdownMenuSubContent>
                          </DropdownMenuSub>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </div>
                  </AccordionContent>
                </AccordionItem>
              );
            })}
          </Accordion>
        </div>
      ))}
    </div>
  );
}
