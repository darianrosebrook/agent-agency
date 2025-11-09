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
} from "./ui/accordion";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { Button } from "./ui/button";

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
        return <Upload className="w-3 h-3" />;
      case "reference":
        return <LinkIcon className="w-3 h-3" />;
      case "tool":
        return <Wrench className="w-3 h-3" />;
      default:
        return null;
    }
  };

  return (
    <div className="w-full">
      {/* Header */}
      <div className="mb-6">
        <h2 className="text-2xl text-white mb-2">Project Plan</h2>
        <p className="text-zinc-400 mb-4">
          Here&apos;s a comprehensive plan for building your multi-modal RAG search
          UI tool
        </p>

        {/* Action buttons */}
        <div className="flex items-center gap-2">
          <Button
            onClick={onSaveToProject}
            className="bg-blue-600 text-white hover:bg-blue-700"
          >
            Add to Project
          </Button>
          <Button
            variant="outline"
            className="bg-[#1a1a1a] border-zinc-700 text-zinc-300 hover:bg-zinc-800"
          >
            Start New Project
          </Button>
        </div>
      </div>

      {/* Phases */}
      {phases.map((phase) => (
        <div
          key={phase.id}
          className="mb-6 bg-[#1a1a1a] rounded-xl border border-zinc-800 overflow-hidden"
        >
          {/* Phase Header */}
          <div className="px-6 py-5 border-b border-zinc-800">
            <div className="flex items-center gap-3 mb-2">
              <h3 className="text-xl text-white">{phase.title}</h3>
              <span className="px-3 py-1 bg-zinc-800 text-zinc-300 text-sm rounded-full">
                Phase {phase.number}
              </span>
            </div>
            <p className="text-zinc-400 text-sm">{phase.description}</p>
          </div>

          {/* Tasks Accordion */}
          <Accordion type="multiple" className="w-full">
            {phase.tasks.map((task) => {
              const progress = calculateTaskProgress(task);

              return (
                <AccordionItem
                  key={task.id}
                  value={task.id}
                  className="border-b border-zinc-800 last:border-0"
                >
                  <AccordionTrigger className="px-6 py-4 hover:bg-[#1f1f1f] hover:no-underline">
                    <div className="flex items-center gap-3 flex-1 text-left">
                      {task.subtasks.length > 0 ? (
                        <div className="flex items-center gap-2">
                          <Circle className="w-5 h-5 text-zinc-500" />
                          <span className="text-sm text-zinc-400">
                            {progress}%
                          </span>
                        </div>
                      ) : (
                        <CircleDashed className="w-5 h-5 text-zinc-500" />
                      )}
                      {/* Task Title (editable) */}
                      <input
                        type="text"
                        value={task.title}
                        onChange={(e) =>
                          updateTaskTitle(phase.id, task.id, e.target.value)
                        }
                        onClick={(e) => e.stopPropagation()}
                        className="w-full bg-transparent border-none outline-none text-zinc-100 focus:ring-2 focus:ring-blue-500 rounded px-2 py-1 -ml-2"
                      />
                    </div>
                  </AccordionTrigger>
                  <AccordionContent className="px-6 pb-6 bg-[#0f0f0f]">
                    {/* Task Description (editable) */}
                    {task.description && (
                      <div className="mb-4 ml-8">
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
                          className="w-full bg-transparent border-none outline-none text-zinc-400 text-sm resize-none focus:ring-2 focus:ring-blue-500 rounded px-2 py-1 mt-2 min-h-[60px]"
                        />
                      </div>
                    )}

                    {/* Context Chips */}
                    {task.contextChips.length > 0 && (
                      <div className="flex flex-wrap gap-2 mb-4 ml-8">
                        {task.contextChips.map((chip) => (
                          <div
                            key={chip.id}
                            className="inline-flex items-center gap-2 px-3 py-1.5 bg-blue-500/10 text-blue-400 rounded-full text-sm group border border-blue-500/20"
                          >
                            {getChipIcon(chip.type)}
                            <span>{chip.label}</span>
                            <button
                              onClick={() =>
                                removeContextChip(phase.id, task.id, chip.id)
                              }
                              className="opacity-0 group-hover:opacity-100 transition-opacity hover:text-blue-300"
                            >
                              <X className="w-3 h-3" />
                            </button>
                          </div>
                        ))}
                      </div>
                    )}

                    {/* Subtasks */}
                    {task.subtasks.length > 0 && (
                      <div className="space-y-2 mb-4 ml-8">
                        {task.subtasks.map((subtask) => (
                          <div
                            key={subtask.id}
                            className="flex items-center gap-3 group py-1"
                          >
                            <button
                              onClick={() =>
                                toggleSubtask(phase.id, task.id, subtask.id)
                              }
                              className="flex-shrink-0"
                            >
                              {subtask.completed ? (
                                <CheckCircle2 className="w-4 h-4 text-green-500" />
                              ) : (
                                <Circle className="w-4 h-4 text-zinc-500" />
                              )}
                            </button>
                            <span
                              className={`flex-1 text-sm ${
                                subtask.completed
                                  ? "text-zinc-500 line-through"
                                  : "text-zinc-300"
                              }`}
                            >
                              {subtask.text}
                            </span>
                            <button
                              onClick={() =>
                                deleteSubtask(phase.id, task.id, subtask.id)
                              }
                              className="opacity-0 group-hover:opacity-100 transition-opacity text-zinc-500 hover:text-red-500"
                            >
                              <Trash2 className="w-4 h-4" />
                            </button>
                          </div>
                        ))}
                      </div>
                    )}

                    {/* Action Buttons */}
                    <div className="flex items-center gap-2 ml-8">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => addSubtask(phase.id, task.id)}
                        className="text-zinc-300 border-zinc-700 hover:bg-zinc-800 hover:text-zinc-100 bg-zinc-950"
                      >
                        Add subtask
                      </Button>

                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            variant="outline"
                            size="sm"
                            className="bg-zinc-950 text-zinc-300 border-zinc-700 hover:bg-zinc-800 hover:text-zinc-100"
                          >
                            Add context
                            <ChevronDown className="w-4 h-4 ml-2" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent className="w-56 bg-[#1a1a1a] border-zinc-700">
                          <DropdownMenuItem
                            className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
                            onClick={() =>
                              addContextChip(
                                phase.id,
                                task.id,
                                "file",
                                "Uploaded file"
                              )
                            }
                          >
                            <Upload className="w-4 h-4 mr-2" />
                            Upload a file
                          </DropdownMenuItem>

                          <DropdownMenuSub>
                            <DropdownMenuSubTrigger className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100">
                              <LinkIcon className="w-4 h-4 mr-2" />
                              Reference a task
                            </DropdownMenuSubTrigger>
                            <DropdownMenuSubContent className="bg-[#1a1a1a] border-zinc-700">
                              <DropdownMenuSub>
                                <DropdownMenuSubTrigger className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100">
                                  Previous projects
                                </DropdownMenuSubTrigger>
                                <DropdownMenuSubContent className="bg-[#1a1a1a] border-zinc-700">
                                  <DropdownMenuItem
                                    className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
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
                                    className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
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
                                    className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
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
                            <DropdownMenuSubTrigger className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100">
                              <Wrench className="w-4 h-4 mr-2" />
                              Tool selection
                            </DropdownMenuSubTrigger>
                            <DropdownMenuSubContent className="bg-[#1a1a1a] border-zinc-700">
                              <DropdownMenuItem
                                className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
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
                                className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
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
                                className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
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
                                className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
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
