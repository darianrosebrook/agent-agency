"use client";

import { useState, useRef, useEffect } from "react";
import { Accordion } from "../../primitives/accordion";
import { TaskItem } from "./TaskItem";
import type { Phase } from "./types";
import styles from "./PhaseItem.module.scss";

interface PhaseItemProps {
  phase: Phase;
  onUpdatePhaseTitle: (phaseId: string, newTitle: string) => void;
  onUpdatePhaseDescription: (phaseId: string, newDescription: string) => void;
  onUpdateTaskTitle: (taskId: string, newTitle: string) => void;
  onUpdateTaskDescription: (taskId: string, newDescription: string) => void;
  onToggleTask: (taskId: string) => void;
  onAddTask: () => void;
  onAddSubtask: (taskId: string) => void;
  onUpdateSubtaskText: (taskId: string, subtaskId: string, newText: string) => void;
  onToggleSubtask: (taskId: string, subtaskId: string) => void;
  onDeleteSubtask: (taskId: string, subtaskId: string) => void;
  onAddContextChip: (
    taskId: string,
    type: "file" | "reference" | "tool",
    label: string
  ) => void;
  onRemoveContextChip: (taskId: string, chipId: string) => void;
}

export function PhaseItem({
  phase,
  onUpdatePhaseTitle,
  onUpdatePhaseDescription,
  onUpdateTaskTitle,
  onUpdateTaskDescription,
  onToggleTask,
  onAddTask,
  onAddSubtask,
  onUpdateSubtaskText,
  onToggleSubtask,
  onDeleteSubtask,
  onAddContextChip,
  onRemoveContextChip,
}: PhaseItemProps) {
  const [isEditingTitle, setIsEditingTitle] = useState(false);
  const [isEditingDescription, setIsEditingDescription] = useState(false);
  const [title, setTitle] = useState(phase.title);
  const [description, setDescription] = useState(phase.description);
  const titleInputRef = useRef<HTMLInputElement>(null);
  const descriptionInputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (isEditingTitle && titleInputRef.current) {
      titleInputRef.current.focus();
      titleInputRef.current.select();
    }
  }, [isEditingTitle]);

  useEffect(() => {
    if (isEditingDescription && descriptionInputRef.current) {
      descriptionInputRef.current.focus();
    }
  }, [isEditingDescription]);

  useEffect(() => {
    setTitle(phase.title);
    setDescription(phase.description);
  }, [phase.title, phase.description]);

  const handleTitleBlur = () => {
    setIsEditingTitle(false);
    if (title.trim() !== phase.title) {
      onUpdatePhaseTitle(phase.id, title.trim() || phase.title);
    }
  };

  const handleTitleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleTitleBlur();
    } else if (e.key === "Escape") {
      setTitle(phase.title);
      setIsEditingTitle(false);
    }
  };

  const handleDescriptionBlur = () => {
    setIsEditingDescription(false);
    if (description !== phase.description) {
      onUpdatePhaseDescription(phase.id, description);
    }
  };

  const handleDescriptionKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Escape") {
      setDescription(phase.description);
      setIsEditingDescription(false);
    }
  };

  return (
    <div className={styles.phaseItem}>
      <div className={styles.phaseHeader}>
        <div className={styles.phaseHeaderTop}>
          {isEditingTitle ? (
            <input
              ref={titleInputRef}
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onBlur={handleTitleBlur}
              onKeyDown={handleTitleKeyDown}
              className={styles.phaseTitleInput}
            />
          ) : (
            <h3
              onClick={() => setIsEditingTitle(true)}
              className={styles.phaseTitleEditable}
            >
              {phase.title}
            </h3>
          )}
          <span className={styles.phaseBadge}>Phase {phase.number}</span>
        </div>
        {isEditingDescription ? (
          <textarea
            ref={descriptionInputRef}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            onBlur={handleDescriptionBlur}
            onKeyDown={handleDescriptionKeyDown}
            className={styles.phaseDescriptionInput}
            placeholder="Add a description..."
          />
        ) : (
          <p
            onClick={() => setIsEditingDescription(true)}
            className={styles.phaseDescriptionEditable}
          >
            {phase.description || "Click to add a description..."}
          </p>
        )}
      </div>

      <Accordion type="multiple" className={styles.phaseAccordion}>
        {phase.tasks.map((task) => (
          <TaskItem
            key={task.id}
            task={task}
            phaseId={phase.id}
            onUpdateTitle={(newTitle) => onUpdateTaskTitle(task.id, newTitle)}
            onUpdateDescription={(newDescription) =>
              onUpdateTaskDescription(task.id, newDescription)
            }
            onToggleTask={() => onToggleTask(task.id)}
            onAddSubtask={() => onAddSubtask(task.id)}
            onUpdateSubtaskText={(subtaskId, newText) =>
              onUpdateSubtaskText(task.id, subtaskId, newText)
            }
            onToggleSubtask={(subtaskId) => onToggleSubtask(task.id, subtaskId)}
            onDeleteSubtask={(subtaskId) => onDeleteSubtask(task.id, subtaskId)}
            onAddContextChip={(type, label) =>
              onAddContextChip(task.id, type, label)
            }
            onRemoveContextChip={(chipId) =>
              onRemoveContextChip(task.id, chipId)
            }
          />
        ))}
        <div className={styles.addTaskContainer}>
          <button
            onClick={onAddTask}
            className={styles.addTaskButton}
            type="button"
          >
            + Add Task
          </button>
        </div>
      </Accordion>
    </div>
  );
}
